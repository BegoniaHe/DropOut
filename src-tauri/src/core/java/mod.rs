use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

pub mod detection;
pub mod error;
pub mod persistence;
pub mod priority;
pub mod provider;
pub mod providers;
pub mod validation;

pub use error::JavaError;
use ts_rs::TS;

/// Remove the UNC prefix (\\?\) from Windows paths
pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy().to_string();
        if s.starts_with(r"\\?\\") {
            return PathBuf::from(&s[4..]);
        }
    }
    path
}

use crate::core::downloader::{DownloadQueue, JavaDownloadProgress, PendingJavaDownload};
use crate::utils::zip;
use provider::JavaProvider;
use providers::{AdoptiumProvider, ProviderRegistry};
use std::sync::Arc;

const CACHE_DURATION_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../packages/ui-new/src/types/bindings/java/index.ts"
)]
pub struct JavaInstallation {
    pub path: String,
    pub version: String,
    pub arch: String,
    pub vendor: String,
    pub source: String,
    pub is_64bit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    Jre,
    Jdk,
}

impl Default for ImageType {
    fn default() -> Self {
        Self::Jre
    }
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Jre => write!(f, "jre"),
            Self::Jdk => write!(f, "jdk"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../packages/ui-new/src/types/bindings/java/index.ts"
)]
pub struct JavaReleaseInfo {
    pub major_version: u32,
    pub image_type: String,
    pub version: String,
    pub release_name: String,
    pub release_date: Option<String>,
    pub file_size: u64,
    pub checksum: Option<String>,
    pub download_url: String,
    pub is_lts: bool,
    pub is_available: bool,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(
    export,
    export_to = "../../packages/ui-new/src/types/bindings/java/index.ts"
)]
pub struct JavaCatalog {
    pub releases: Vec<JavaReleaseInfo>,
    pub available_major_versions: Vec<u32>,
    pub lts_versions: Vec<u32>,
    pub cached_at: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(
    export,
    export_to = "../../packages/ui-new/src/types/bindings/java/index.ts"
)]
pub struct JavaDownloadInfo {
    pub version: String,          // e.g., "17.0.2+8"
    pub release_name: String,     // e.g., "jdk-17.0.2+8"
    pub download_url: String,     // Direct download URL
    pub file_name: String,        // e.g., "OpenJDK17U-jre_x64_linux_hotspot_17.0.2_8.tar.gz"
    pub file_size: u64,           // in bytes
    pub checksum: Option<String>, // SHA256 checksum
    pub image_type: String,       // "jre" or "jdk"
}

pub fn get_java_install_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle.path().app_data_dir().unwrap().join("java")
}

/// Get legacy catalog cache path (keeps backward compatibility).
pub fn get_catalog_cache_path(app_handle: &AppHandle) -> PathBuf {
    get_catalog_cache_path_for_provider(app_handle, None)
}

/// Get the catalog cache path for a given provider name (or legacy default if None)
pub fn get_catalog_cache_path_for_provider(
    app_handle: &AppHandle,
    provider_name: Option<&str>,
) -> PathBuf {
    let base = app_handle.path().app_data_dir().unwrap();
    get_catalog_cache_path_for_base(&base, provider_name)
}

/// Construct a catalog cache path given a base app data directory and optional provider name.
/// Exposed for tests so we don't need an AppHandle in unit tests.
pub fn get_catalog_cache_path_for_base(
    base_dir: &std::path::Path,
    provider_name: Option<&str>,
) -> PathBuf {
    let file_name = if let Some(name) = provider_name {
        format!("java_catalog_cache_{}.json", name)
    } else {
        "java_catalog_cache.json".to_string()
    };
    base_dir.join(file_name)
}

pub fn load_cached_catalog(app_handle: &AppHandle) -> Option<JavaCatalog> {
    load_cached_catalog_for_provider(app_handle, None)
}

/// Load cache for a specific provider (provider_name = None => legacy/default cache)
pub fn load_cached_catalog_for_provider(
    app_handle: &AppHandle,
    provider_name: Option<&str>,
) -> Option<JavaCatalog> {
    let cache_path = get_catalog_cache_path_for_provider(app_handle, provider_name);
    load_cached_catalog_from_path(&cache_path)
}

/// Load cache directly from a base path (testable without AppHandle)
pub fn load_cached_catalog_from_base(
    base_dir: &std::path::Path,
    provider_name: Option<&str>,
) -> Option<JavaCatalog> {
    let cache_path = get_catalog_cache_path_for_base(base_dir, provider_name);
    load_cached_catalog_from_path(&cache_path)
}

/// Internal helper: try to read and validate catalog from a concrete path
fn load_cached_catalog_from_path(cache_path: &PathBuf) -> Option<JavaCatalog> {
    if !cache_path.exists() {
        return None;
    }

    // Read cache file
    let content = std::fs::read_to_string(&cache_path).ok()?;
    let catalog: JavaCatalog = serde_json::from_str(&content).ok()?;

    // Get current time in seconds since UNIX_EPOCH
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check if cache is still valid
    if now - catalog.cached_at < CACHE_DURATION_SECS {
        Some(catalog)
    } else {
        None
    }
}

pub fn save_catalog_cache(app_handle: &AppHandle, catalog: &JavaCatalog) -> Result<(), String> {
    save_catalog_cache_for_provider(app_handle, None, catalog)
}

/// Save catalog cache for a specific provider (provider_name = None => legacy/default cache)
pub fn save_catalog_cache_for_provider(
    app_handle: &AppHandle,
    provider_name: Option<&str>,
    catalog: &JavaCatalog,
) -> Result<(), String> {
    let cache_path = get_catalog_cache_path_for_provider(app_handle, provider_name);
    save_catalog_cache_to_path(&cache_path, catalog)
}

/// Save catalog cache directly to a base path (testable without AppHandle)
pub fn save_catalog_cache_to_base(
    base_dir: &std::path::Path,
    provider_name: Option<&str>,
    catalog: &JavaCatalog,
) -> Result<(), String> {
    let cache_path = get_catalog_cache_path_for_base(base_dir, provider_name);
    save_catalog_cache_to_path(&cache_path, catalog)
}

/// Internal helper: write JSON to a concrete path ensuring parent dir exists
fn save_catalog_cache_to_path(cache_path: &PathBuf, catalog: &JavaCatalog) -> Result<(), String> {
    let content = serde_json::to_string_pretty(catalog).map_err(|e| e.to_string())?;
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(cache_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn clear_catalog_cache(app_handle: &AppHandle) -> Result<(), String> {
    clear_catalog_cache_for_provider(app_handle, None)
}

/// Clear cache for a specific provider
pub fn clear_catalog_cache_for_provider(
    app_handle: &AppHandle,
    provider_name: Option<&str>,
) -> Result<(), String> {
    let cache_path = get_catalog_cache_path_for_provider(app_handle, provider_name);
    if cache_path.exists() {
        std::fs::remove_file(&cache_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Clear cache file in a base dir (testable)
pub fn clear_catalog_cache_from_base(
    base_dir: &std::path::Path,
    provider_name: Option<&str>,
) -> Result<(), String> {
    let cache_path = get_catalog_cache_path_for_base(base_dir, provider_name);
    if cache_path.exists() {
        std::fs::remove_file(&cache_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    // Additional imports used by the archive extraction test
    use ::zip::write::FileOptions;
    use ::zip::write::ZipWriter;
    use std::fs::File;

    #[test]
    fn test_catalog_cache_path_for_provider() {
        let base = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&base).unwrap();

        let p_adoptium = get_catalog_cache_path_for_base(&base, Some("adoptium"));
        assert!(p_adoptium
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("java_catalog_cache_adoptium"));
        assert_eq!(p_adoptium.parent().unwrap(), base.as_path());

        let p_legacy = get_catalog_cache_path_for_base(&base, None);
        assert_eq!(
            p_legacy.file_name().unwrap().to_string_lossy(),
            "java_catalog_cache.json"
        );

        // cleanup
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_save_and_load_catalog_for_provider() {
        let base = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&base).unwrap();

        let catalog = JavaCatalog {
            releases: Vec::new(),
            available_major_versions: vec![17, 21],
            lts_versions: vec![17],
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Save and load for provider name
        save_catalog_cache_to_base(&base, Some("adoptium"), &catalog).unwrap();
        let loaded = load_cached_catalog_from_base(&base, Some("adoptium")).unwrap();
        assert_eq!(loaded.available_major_versions, vec![17, 21]);

        // Clear and ensure absent
        clear_catalog_cache_from_base(&base, Some("adoptium")).unwrap();
        assert!(load_cached_catalog_from_base(&base, Some("adoptium")).is_none());

        // cleanup
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_extract_archive_and_find_java_bin_zip() {
        // prepare a temporary directory with a top-level dir that contains bin/java and bin/java.exe
        let base = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&base.join("topdir").join("bin")).unwrap();

        let bin_dir = base.join("topdir").join("bin");
        // a simple shell script that prints java version to stderr (for unix-like systems)
        fs::write(
            bin_dir.join("java"),
            b"#!/bin/sh\necho 'openjdk version \"17.0.1\"' >&2\n",
        )
        .unwrap();
        // a placeholder for java.exe (exists on Windows)
        fs::write(bin_dir.join("java.exe"), b"dummy").unwrap();

        // create a zip archive containing topdir/bin/java and topdir/bin/java.exe
        let archive_path = base.join("test.zip");
        let file = File::create(&archive_path).unwrap();
        let mut zipw = ZipWriter::new(file);
        let options: ::zip::write::FileOptions<'_, ::zip::write::ExtendedFileOptions> =
            ::zip::write::FileOptions::default();

        let mut f = File::open(bin_dir.join("java")).unwrap();
        zipw.start_file("topdir/bin/java", options.clone()).unwrap();

        let mut f2 = File::open(bin_dir.join("java.exe")).unwrap();
        zipw.start_file("topdir/bin/java.exe", options.clone())
            .unwrap();

        zipw.finish().unwrap();

        let extract_to = base.join("extract");
        fs::create_dir_all(&extract_to).unwrap();

        let (top, java_bin) =
            extract_archive_and_find_java_bin(&archive_path, &extract_to, "test.zip").unwrap();
        assert_eq!(top, "topdir");
        // Ensure the returned java_bin path exists after extraction
        assert!(java_bin.exists());
    }
}

pub async fn fetch_java_catalog(
    app_handle: &AppHandle,
    force_refresh: bool,
) -> Result<JavaCatalog, String> {
    // Backwards-compatible wrapper: no explicit provider -> use default provider (caller can pass provider-aware variant)
    fetch_java_catalog_with_provider(app_handle, force_refresh, None).await
}

/// Provider-aware variant: if `provider` is None, falls back to AdoptiumProvider
pub async fn fetch_java_catalog_with_provider(
    app_handle: &AppHandle,
    force_refresh: bool,
    provider: Option<Arc<dyn JavaProvider + Send + Sync>>,
) -> Result<JavaCatalog, String> {
    if let Some(provider) = provider {
        provider
            .fetch_catalog(app_handle, force_refresh)
            .await
            .map_err(|e| e.to_string())
    } else {
        let provider = AdoptiumProvider::new();
        provider
            .fetch_catalog(app_handle, force_refresh)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Provider-name-aware wrapper: resolve provider by name from app's ProviderRegistry (fallback to Adoptium)
pub async fn fetch_java_catalog_with_provider_name(
    app_handle: &AppHandle,
    force_refresh: bool,
    provider_name: Option<String>,
) -> Result<JavaCatalog, String> {
    let provider_impl = resolve_provider_from_app(app_handle, provider_name.as_deref());
    fetch_java_catalog_with_provider(app_handle, force_refresh, Some(provider_impl)).await
}

pub async fn fetch_java_release(
    major_version: u32,
    image_type: ImageType,
) -> Result<JavaDownloadInfo, String> {
    // Backwards-compatible wrapper
    fetch_java_release_with_provider(None, major_version, image_type).await
}

/// Provider-aware variant: if `provider` is None, falls back to AdoptiumProvider
pub async fn fetch_java_release_with_provider(
    provider: Option<Arc<dyn JavaProvider + Send + Sync>>,
    major_version: u32,
    image_type: ImageType,
) -> Result<JavaDownloadInfo, String> {
    if let Some(provider) = provider {
        provider
            .fetch_release(major_version, image_type)
            .await
            .map_err(|e| e.to_string())
    } else {
        let provider = AdoptiumProvider::new();
        provider
            .fetch_release(major_version, image_type)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Provider-name-aware wrapper: resolve provider by name from app's ProviderRegistry (fallback to Adoptium)
pub async fn fetch_java_release_with_provider_name(
    app_handle: &AppHandle,
    major_version: u32,
    image_type: ImageType,
    provider_name: Option<String>,
) -> Result<JavaDownloadInfo, String> {
    let provider_impl = resolve_provider_from_app(app_handle, provider_name.as_deref());
    fetch_java_release_with_provider(Some(provider_impl), major_version, image_type).await
}

pub async fn fetch_available_versions() -> Result<Vec<u32>, String> {
    // Backwards-compatible wrapper
    fetch_available_versions_with_provider(None).await
}

/// Provider-aware variant: if `provider` is None, falls back to AdoptiumProvider
pub async fn fetch_available_versions_with_provider(
    provider: Option<Arc<dyn JavaProvider + Send + Sync>>,
) -> Result<Vec<u32>, String> {
    if let Some(provider) = provider {
        provider
            .available_versions()
            .await
            .map_err(|e| e.to_string())
    } else {
        let provider = AdoptiumProvider::new();
        provider
            .available_versions()
            .await
            .map_err(|e| e.to_string())
    }
}

/// Provider-name-aware wrapper: resolve provider by name from app's ProviderRegistry (fallback to Adoptium)
pub async fn fetch_available_versions_with_provider_name(
    app_handle: &AppHandle,
    provider_name: Option<String>,
) -> Result<Vec<u32>, String> {
    let provider_impl = resolve_provider_from_app(app_handle, provider_name.as_deref());
    fetch_available_versions_with_provider(Some(provider_impl)).await
}

pub async fn download_and_install_java(
    app_handle: &AppHandle,
    major_version: u32,
    image_type: ImageType,
    custom_path: Option<PathBuf>,
) -> Result<JavaInstallation, String> {
    // Backwards-compatible wrapper which uses default provider
    download_and_install_java_with_provider(
        app_handle,
        major_version,
        image_type,
        custom_path,
        None,
    )
    .await
}

/// Provider-aware variant. If `provider` is None, falls back to AdoptiumProvider.
pub async fn download_and_install_java_with_provider(
    app_handle: &AppHandle,
    major_version: u32,
    image_type: ImageType,
    custom_path: Option<PathBuf>,
    provider: Option<Arc<dyn JavaProvider + Send + Sync>>,
) -> Result<JavaInstallation, String> {
    let provider_impl: Arc<dyn JavaProvider + Send + Sync> = if let Some(p) = provider {
        p
    } else {
        Arc::new(AdoptiumProvider::new())
    };

    // Fetch release info from chosen provider
    let info = provider_impl
        .fetch_release(major_version, image_type)
        .await?;
    let file_name = info.file_name.clone();

    let install_base = custom_path.unwrap_or_else(|| get_java_install_dir(app_handle));

    std::fs::create_dir_all(&install_base)
        .map_err(|e| format!("Failed to create installation directory: {}", e))?;

    let mut queue = DownloadQueue::load(app_handle);
    queue.add(PendingJavaDownload {
        major_version,
        image_type: image_type.to_string(),
        download_url: info.download_url.clone(),
        file_name: info.file_name.clone(),
        file_size: info.file_size,
        checksum: info.checksum.clone(),
        install_path: install_base.to_string_lossy().to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        provider_name: Some(provider_impl.provider_name().to_string()),
    });
    queue.save(app_handle)?;

    let archive_path = install_base.join(&info.file_name);

    /// Provider-name-aware wrapper: resolve provider by name from app's ProviderRegistry (fallback to Adoptium)
    pub async fn download_and_install_java_with_provider_name(
        app_handle: &AppHandle,
        major_version: u32,
        image_type: ImageType,
        custom_path: Option<PathBuf>,
        provider_name: Option<String>,
    ) -> Result<JavaInstallation, String> {
        let provider_impl = resolve_provider_from_app(app_handle, provider_name.as_deref());
        download_and_install_java_with_provider(
            app_handle,
            major_version,
            image_type,
            custom_path,
            Some(provider_impl),
        )
        .await
    }

    let need_download = if archive_path.exists() {
        if let Some(expected_checksum) = &info.checksum {
            let data = std::fs::read(&archive_path)
                .map_err(|e| format!("Failed to read downloaded file: {}", e))?;
            !crate::core::downloader::verify_checksum(&data, Some(expected_checksum), None)
        } else {
            false
        }
    } else {
        true
    };

    if need_download {
        crate::core::downloader::download_with_resume(
            app_handle,
            &info.download_url,
            &archive_path,
            info.checksum.as_deref(),
            info.file_size,
        )
        .await?;
    }

    // Install (either from an existing file or the freshly downloaded one)
    let installation = install_from_archive(
        app_handle,
        &archive_path,
        &info,
        major_version,
        image_type,
        &install_base,
        Some(provider_impl.clone()),
    )
    .await?;

    // After successful install, remove the pending entry specific to this provider and persist
    queue.remove_with_provider(
        major_version,
        &image_type.to_string(),
        Some(provider_impl.provider_name()),
    );
    queue.save(app_handle)?;

    let _ = app_handle.emit(
        "java-download-progress",
        JavaDownloadProgress {
            file_name,
            downloaded_bytes: info.file_size,
            total_bytes: info.file_size,
            speed_bytes_per_sec: 0,
            eta_seconds: 0,
            status: "Completed".to_string(),
            percentage: 100.0,
        },
    );

    Ok(installation)
}

fn resolve_provider_from_app(
    app_handle: &AppHandle,
    provider_name: Option<&str>,
) -> Arc<dyn JavaProvider + Send + Sync> {
    // Try to read ProviderRegistry from app state if available (use try_state to avoid panics if not managed)
    if let Some(reg) = app_handle.try_state::<ProviderRegistry>() {
        if let Some(name) = provider_name {
            if let Some(p) = reg.get(name) {
                return p;
            }
        }
        if let Some(p) = reg.default() {
            return p;
        }
    }

    // Fallback to Adoptium directly
    Arc::new(AdoptiumProvider::new())
}

pub async fn install_from_archive(
    app_handle: &AppHandle,
    archive_path: &PathBuf,
    info: &JavaDownloadInfo,
    major_version: u32,
    image_type: ImageType,
    install_base: &PathBuf,
    provider_impl: Option<Arc<dyn JavaProvider + Send + Sync>>,
) -> Result<JavaInstallation, String> {
    let file_name = info.file_name.clone();
    let _ = app_handle.emit(
        "java-download-progress",
        JavaDownloadProgress {
            file_name: file_name.clone(),
            downloaded_bytes: info.file_size,
            total_bytes: info.file_size,
            speed_bytes_per_sec: 0,
            eta_seconds: 0,
            status: "Extracting".to_string(),
            percentage: 100.0,
        },
    );

    // Determine install prefix (fallback to Adoptium provider prefix)
    let prefix = if let Some(p) = provider_impl.as_ref() {
        p.install_prefix()
    } else {
        AdoptiumProvider::new().install_prefix()
    };

    let version_dir = install_base.join(format!("{}-{}-{}", prefix, major_version, image_type));

    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir)
            .map_err(|e| format!("Failed to remove old version directory: {}", e))?;
    }

    std::fs::create_dir_all(&version_dir)
        .map_err(|e| format!("Failed to create version directory: {}", e))?;

    // Extract archive and locate java executable (delegated to helper)
    let (_top_level_dir, java_bin) =
        extract_archive_and_find_java_bin(&archive_path, &version_dir, &info.file_name)?;
    let _ = std::fs::remove_file(&archive_path);

    if !java_bin.exists() {
        return Err(format!(
            "Installation completed but Java executable not found: {}",
            java_bin.display()
        ));
    }

    let java_bin = std::fs::canonicalize(&java_bin).map_err(|e| e.to_string())?;
    let java_bin = strip_unc_prefix(java_bin);

    let installation = validation::check_java_installation(&java_bin)
        .await
        .ok_or_else(|| "Failed to verify Java installation".to_string())?;

    let _ = app_handle.emit(
        "java-download-progress",
        JavaDownloadProgress {
            file_name,
            downloaded_bytes: info.file_size,
            total_bytes: info.file_size,
            speed_bytes_per_sec: 0,
            eta_seconds: 0,
            status: "Completed".to_string(),
            percentage: 100.0,
        },
    );

    Ok(installation)
}

fn extract_archive_and_find_java_bin(
    archive_path: &PathBuf,
    version_dir: &PathBuf,
    file_name: &str,
) -> Result<(String, PathBuf), String> {
    let top_level_dir = if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        zip::extract_tar_gz(archive_path, version_dir)?
    } else if file_name.ends_with(".zip") {
        zip::extract_zip(archive_path, version_dir)?;
        find_top_level_dir(version_dir)?
    } else {
        return Err(format!("Unsupported archive format: {}", file_name));
    };

    let java_home = version_dir.join(&top_level_dir);
    let java_bin = if cfg!(target_os = "macos") {
        java_home
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    } else if cfg!(windows) {
        java_home.join("bin").join("java.exe")
    } else {
        java_home.join("bin").join("java")
    };

    Ok((top_level_dir, java_bin))
}

fn find_top_level_dir(extract_dir: &PathBuf) -> Result<String, String> {
    let entries: Vec<_> = std::fs::read_dir(extract_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    if entries.len() == 1 {
        Ok(entries[0].file_name().to_string_lossy().to_string())
    } else {
        Ok(String::new())
    }
}

pub async fn detect_java_installations() -> Vec<JavaInstallation> {
    let mut installations = Vec::new();
    let candidates = detection::get_java_candidates();

    for candidate in candidates {
        if let Some(java) = validation::check_java_installation(&candidate).await {
            if !installations
                .iter()
                .any(|j: &JavaInstallation| j.path == java.path)
            {
                installations.push(java);
            }
        }
    }

    installations.sort_by(|a, b| {
        let v_a = validation::parse_java_version(&a.version);
        let v_b = validation::parse_java_version(&b.version);
        v_b.cmp(&v_a)
    });

    installations
}

pub async fn get_recommended_java(required_major_version: Option<u64>) -> Option<JavaInstallation> {
    let installations = detect_java_installations().await;

    if let Some(required) = required_major_version {
        installations.into_iter().find(|java| {
            let major = validation::parse_java_version(&java.version);
            major >= required as u32
        })
    } else {
        installations.into_iter().next()
    }
}

pub async fn get_compatible_java(
    app_handle: &AppHandle,
    required_major_version: Option<u64>,
    max_major_version: Option<u32>,
) -> Option<JavaInstallation> {
    let installations = detect_all_java_installations(app_handle).await;

    installations.into_iter().find(|java| {
        let major = validation::parse_java_version(&java.version);
        validation::is_version_compatible(major, required_major_version, max_major_version)
    })
}

pub async fn is_java_compatible(
    java_path: &str,
    required_major_version: Option<u64>,
    max_major_version: Option<u32>,
) -> bool {
    let java_path_buf = PathBuf::from(java_path);
    if let Some(java) = validation::check_java_installation(&java_path_buf).await {
        let major = validation::parse_java_version(&java.version);
        validation::is_version_compatible(major, required_major_version, max_major_version)
    } else {
        false
    }
}

pub async fn detect_all_java_installations(app_handle: &AppHandle) -> Vec<JavaInstallation> {
    let mut installations: Vec<JavaInstallation> = detect_java_installations().await;

    let dropout_java_dir = get_java_install_dir(app_handle);
    if dropout_java_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dropout_java_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let java_bin = find_java_executable(&path);
                    if let Some(java_path) = java_bin {
                        if let Some(java) = validation::check_java_installation(&java_path).await {
                            if !installations.iter().any(|j| j.path == java.path) {
                                installations.push(java);
                            }
                        }
                    }
                }
            }
        }
    }

    installations.sort_by(|a, b| {
        let v_a = validation::parse_java_version(&a.version);
        let v_b = validation::parse_java_version(&b.version);
        v_b.cmp(&v_a)
    });

    installations
}

fn find_java_executable(dir: &PathBuf) -> Option<PathBuf> {
    let bin_name = if cfg!(windows) { "java.exe" } else { "java" };

    let direct_bin = dir.join("bin").join(bin_name);
    if direct_bin.exists() {
        let resolved = std::fs::canonicalize(&direct_bin).unwrap_or(direct_bin);
        return Some(strip_unc_prefix(resolved));
    }

    #[cfg(target_os = "macos")]
    {
        let macos_bin = dir.join("Contents").join("Home").join("bin").join(bin_name);
        if macos_bin.exists() {
            return Some(macos_bin);
        }
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let nested_bin = path.join("bin").join(bin_name);
                if nested_bin.exists() {
                    let resolved = std::fs::canonicalize(&nested_bin).unwrap_or(nested_bin);
                    return Some(strip_unc_prefix(resolved));
                }

                #[cfg(target_os = "macos")]
                {
                    let macos_nested = path
                        .join("Contents")
                        .join("Home")
                        .join("bin")
                        .join(bin_name);
                    if macos_nested.exists() {
                        return Some(macos_nested);
                    }
                }
            }
        }
    }

    None
}

pub async fn resume_pending_downloads(
    app_handle: &AppHandle,
) -> Result<Vec<JavaInstallation>, String> {
    let mut installed = Vec::new();

    // Work on a snapshot of pending downloads to allow modifying the persisted queue during iteration
    let pendings = {
        let q = DownloadQueue::load(app_handle);
        q.pending_downloads.clone()
    };

    for pending in pendings.iter() {
        let image_type = if pending.image_type == "jdk" {
            ImageType::Jdk
        } else {
            ImageType::Jre
        };

        // Try to resolve provider by recorded provider_name (exact match in registry)
        let provider_opt = if let Some(reg) = app_handle.try_state::<ProviderRegistry>() {
            pending
                .provider_name
                .as_deref()
                .and_then(|name| reg.get(name))
        } else {
            None
        };

        let archive_path = PathBuf::from(&pending.install_path).join(&pending.file_name);

        // If an archive file already exists, attempt to install directly from it
        if archive_path.exists() {
            let info = JavaDownloadInfo {
                version: "".to_string(),
                release_name: "".to_string(),
                download_url: pending.download_url.clone(),
                file_name: pending.file_name.clone(),
                file_size: pending.file_size,
                checksum: pending.checksum.clone(),
                image_type: pending.image_type.clone(),
            };

            match install_from_archive(
                app_handle,
                &archive_path,
                &info,
                pending.major_version,
                image_type,
                &PathBuf::from(&pending.install_path),
                provider_opt.clone(),
            )
            .await
            {
                Ok(inst) => {
                    installed.push(inst);
                    let mut q = DownloadQueue::load(app_handle);
                    q.remove_with_provider(
                        pending.major_version,
                        &pending.image_type,
                        pending.provider_name.as_deref(),
                    );
                    let _ = q.save(app_handle);
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to install Java from existing archive {} {}: {}",
                        pending.major_version, pending.image_type, e
                    );
                    continue;
                }
            }
        }

        // If no archive exists, try to resume download using stored download_url
        if !pending.download_url.is_empty() {
            if let Err(e) = crate::core::downloader::download_with_resume(
                app_handle,
                &pending.download_url,
                &archive_path,
                pending.checksum.as_deref(),
                pending.file_size,
            )
            .await
            {
                eprintln!(
                    "Failed to resume download for Java {} {}: {}",
                    pending.major_version, pending.image_type, e
                );
                continue;
            }

            // Attempt to install after successful (re-)download
            let info = JavaDownloadInfo {
                version: "".to_string(),
                release_name: "".to_string(),
                download_url: pending.download_url.clone(),
                file_name: pending.file_name.clone(),
                file_size: pending.file_size,
                checksum: pending.checksum.clone(),
                image_type: pending.image_type.clone(),
            };

            match install_from_archive(
                app_handle,
                &archive_path,
                &info,
                pending.major_version,
                image_type,
                &PathBuf::from(&pending.install_path),
                provider_opt.clone(),
            )
            .await
            {
                Ok(inst) => {
                    installed.push(inst);
                    let mut q = DownloadQueue::load(app_handle);
                    q.remove_with_provider(
                        pending.major_version,
                        &pending.image_type,
                        pending.provider_name.as_deref(),
                    );
                    let _ = q.save(app_handle);
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to install Java after resumed download {} {}: {}",
                        pending.major_version, pending.image_type, e
                    );
                    continue;
                }
            }
        } else {
            // No download URL available: fallback to default provider installer
            match download_and_install_java_with_provider(
                app_handle,
                pending.major_version,
                image_type,
                Some(PathBuf::from(&pending.install_path)),
                None,
            )
            .await
            {
                Ok(inst) => {
                    installed.push(inst);
                    let mut q = DownloadQueue::load(app_handle);
                    q.remove_with_provider(
                        pending.major_version,
                        &pending.image_type,
                        pending.provider_name.as_deref(),
                    );
                    let _ = q.save(app_handle);
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "No download URL and fallback failed for Java {} {}: {}",
                        pending.major_version, pending.image_type, e
                    );
                    continue;
                }
            }
        }
    }

    Ok(installed)
}

pub fn cancel_current_download() {
    crate::core::downloader::cancel_java_download();
}

pub fn get_pending_downloads(app_handle: &AppHandle) -> Vec<PendingJavaDownload> {
    let queue = DownloadQueue::load(app_handle);
    queue.pending_downloads
}

#[allow(dead_code)]
pub fn clear_pending_download(
    app_handle: &AppHandle,
    major_version: u32,
    image_type: &str,
) -> Result<(), String> {
    let mut queue = DownloadQueue::load(app_handle);
    queue.remove_with_provider(major_version, image_type, None);
    queue.save(app_handle)
}
