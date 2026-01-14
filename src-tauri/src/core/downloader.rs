use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub file: String,
    pub downloaded: u64,
    pub total: u64,
    pub status: String, // "Downloading", "Verifying", "Finished", "Error"
    pub completed_files: usize,
    pub total_files: usize,
    pub total_downloaded_bytes: u64,
}

pub async fn download_files(window: Window, tasks: Vec<DownloadTask>, max_concurrent: usize) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(max_concurrent)
        .build()
        .map_err(|e| e.to_string())?;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let completed_files = Arc::new(AtomicUsize::new(0));
    let total_downloaded_bytes = Arc::new(AtomicU64::new(0));
    let total_files = tasks.len();

    // Notify start (total files)
    let _ = window.emit("download-start", tasks.len());

    let tasks_stream = futures::stream::iter(tasks).map(|task| {
        let client = client.clone();
        let window = window.clone();
        let semaphore = semaphore.clone();
        let completed_files = completed_files.clone();
        let total_downloaded_bytes = total_downloaded_bytes.clone();

        async move {
            let _permit = semaphore.acquire().await.unwrap();
            let file_name = task.path.file_name().unwrap().to_string_lossy().to_string();

            // 1. Check if file exists and verify SHA1
            if task.path.exists() {
                let _ = window.emit(
                    "download-progress",
                    ProgressEvent {
                        file: file_name.clone(),
                        downloaded: 0,
                        total: 0,
                        status: "Verifying".into(),
                        completed_files: completed_files.load(Ordering::Relaxed),
                        total_files,
                        total_downloaded_bytes: total_downloaded_bytes.load(Ordering::Relaxed),
                    },
                );

                if let Some(expected_sha1) = &task.sha1 {
                    if let Ok(data) = tokio::fs::read(&task.path).await {
                        let mut hasher = sha1::Sha1::new();
                        use sha1::Digest;
                        hasher.update(&data);
                        let result = hex::encode(hasher.finalize());
                        if &result == expected_sha1 {
                            // Already valid
                            let completed = completed_files.fetch_add(1, Ordering::Relaxed) + 1;
                            let _ = window.emit(
                                "download-progress",
                                ProgressEvent {
                                    file: file_name.clone(),
                                    downloaded: 0,
                                    total: 0,
                                    status: "Skipped".into(),
                                    completed_files: completed,
                                    total_files,
                                    total_downloaded_bytes: total_downloaded_bytes.load(Ordering::Relaxed),
                                },
                            );
                            return Ok(());
                        }
                    }
                }
            }

            // 2. Download
            if let Some(parent) = task.path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }

            match client.get(&task.url).send().await {
                Ok(mut resp) => {
                    let total_size = resp.content_length().unwrap_or(0);
                    let mut file = match tokio::fs::File::create(&task.path).await {
                        Ok(f) => f,
                        Err(e) => return Err(format!("Create file error: {}", e)),
                    };

                    let mut downloaded: u64 = 0;
                    loop {
                        match resp.chunk().await {
                            Ok(Some(chunk)) => {
                                if let Err(e) = file.write_all(&chunk).await {
                                    return Err(format!("Write error: {}", e));
                                }
                                downloaded += chunk.len() as u64;
                                let total_bytes = total_downloaded_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed) + chunk.len() as u64;
                                let _ = window.emit(
                                    "download-progress",
                                    ProgressEvent {
                                        file: file_name.clone(),
                                        downloaded,
                                        total: total_size,
                                        status: "Downloading".into(),
                                        completed_files: completed_files.load(Ordering::Relaxed),
                                        total_files,
                                        total_downloaded_bytes: total_bytes,
                                    },
                                );
                            }
                            Ok(None) => break,
                            Err(e) => return Err(format!("Download error: {}", e)),
                        }
                    }
                }
                Err(e) => return Err(format!("Request error: {}", e)),
            }

            let completed = completed_files.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = window.emit(
                "download-progress",
                ProgressEvent {
                    file: file_name.clone(),
                    downloaded: 0,
                    total: 0,
                    status: "Finished".into(),
                    completed_files: completed,
                    total_files,
                    total_downloaded_bytes: total_downloaded_bytes.load(Ordering::Relaxed),
                },
            );

            Ok(())
        }
    });

    // Buffer unordered to run concurrently
    tasks_stream
        .buffer_unordered(10)
        .collect::<Vec<Result<(), String>>>()
        .await;

    let _ = window.emit("download-complete", ());
    Ok(())
}
