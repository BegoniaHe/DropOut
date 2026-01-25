use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Remove the UNC prefix (\\?\) from Windows paths
        let s = path.to_string_lossy().to_string();
        if s.starts_with(r"\\?\") {
            return PathBuf::from(&s[4..]);
        }
    }
    path
}

pub fn find_sdkman_java() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let sdkman_path = PathBuf::from(&home).join(".sdkman/candidates/java/current/bin/java");
    if sdkman_path.exists() {
        Some(sdkman_path)
    } else {
        None
    }
}

fn run_which_command_with_timeout() -> Option<String> {
    let mut cmd = Command::new(if cfg!(windows) { "where" } else { "which" });
    cmd.arg("java");
    #[cfg(target_os = "windows")]
    // Hide the console window on Windows
    cmd.creation_flags(0x08000000);

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn get_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Only attempt 'which' or 'where' if is not Windows
    // CAUTION: linux 'which' may return symlinks, so we need to canonicalize later
    if let Some(paths_str) = run_which_command_with_timeout() {
        for line in paths_str.lines() {
            let path = PathBuf::from(line.trim());
            if path.exists() {
                let resolved = std::fs::canonicalize(&path).unwrap_or(path);
                let final_path = strip_unc_prefix(resolved);
                candidates.push(final_path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let linux_paths = [
            "/usr/lib/jvm",
            "/usr/java",
            "/opt/java",
            "/opt/jdk",
            "/opt/openjdk",
        ];

        for base in &linux_paths {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let java_path = entry.path().join("bin/java");
                    if java_path.exists() {
                        candidates.push(java_path);
                    }
                }
            }
        }

        let home = std::env::var("HOME").unwrap_or_default();
        // Check common SDKMAN! java candidates
        if let Some(sdkman_java) = find_sdkman_java() {
            candidates.push(sdkman_java);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mac_paths = [
            "/Library/Java/JavaVirtualMachines",
            "/System/Library/Java/JavaVirtualMachines",
            "/usr/local/opt/openjdk/bin/java",
            "/opt/homebrew/opt/openjdk/bin/java",
        ];

        for path in &mac_paths {
            let p = PathBuf::from(path);
            if p.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&p) {
                    for entry in entries.flatten() {
                        let java_path = entry.path().join("Contents/Home/bin/java");
                        if java_path.exists() {
                            candidates.push(java_path);
                        }
                    }
                }
            } else if p.exists() {
                candidates.push(p);
            }
        }

        // Check common Homebrew java candidates for aarch64 macs
        let homebrew_arm = PathBuf::from("/opt/homebrew/Cellar/openjdk");
        if homebrew_arm.exists() {
            if let Ok(entries) = std::fs::read_dir(&homebrew_arm) {
                for entry in entries.flatten() {
                    let java_path = entry
                        .path()
                        .join("libexec/openjdk.jdk/Contents/Home/bin/java");
                    if java_path.exists() {
                        candidates.push(java_path);
                    }
                }
            }
        }

        // Check common SDKMAN! java candidates
        if let Some(sdkman_java) = find_sdkman_java() {
            candidates.push(sdkman_java);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // Common installation paths for various JDK distributions
        let mut win_paths = vec![];
        for base in &[&program_files, &program_files_x86, &local_app_data] {
            win_paths.push(format!("{}\\Java", base));
            win_paths.push(format!("{}\\Eclipse Adoptium", base));
            win_paths.push(format!("{}\\AdoptOpenJDK", base));
            win_paths.push(format!("{}\\Microsoft\\jdk", base));
            win_paths.push(format!("{}\\Zulu", base));
            win_paths.push(format!("{}\\Amazon Corretto", base));
            win_paths.push(format!("{}\\BellSoft\\LibericaJDK", base));
            win_paths.push(format!("{}\\Programs\\Eclipse Adoptium", base));
        }

        for base in &win_paths {
            let base_path = PathBuf::from(base);
            if base_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&base_path) {
                    for entry in entries.flatten() {
                        let java_path = entry.path().join("bin\\java.exe");
                        if java_path.exists() {
                            candidates.push(java_path);
                        }
                    }
                }
            }
        }
    }

    // Check JAVA_HOME java candidate
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let bin_name = if cfg!(windows) { "java.exe" } else { "java" };
        let java_path = PathBuf::from(&java_home).join("bin").join(bin_name);
        if java_path.exists() {
            candidates.push(java_path);
        }
    }

    candidates
}
