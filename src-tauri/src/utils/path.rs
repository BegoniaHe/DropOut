/// Path utilities for cross-platform compatibility
use std::path::PathBuf;

/// Normalize a Java executable path for the current platform.
///
/// On Windows:
/// - Adds .exe extension if missing
/// - Attempts to locate java.exe in PATH if only "java" is provided
/// - Validates that the path exists
///
/// On Unix:
/// - Returns the path as-is
///
/// # Arguments
/// * `java_path` - The Java executable path to normalize
///
/// # Returns
/// * `Ok(PathBuf)` - Normalized path that exists
/// * `Err(String)` - Error if the path cannot be found or validated
#[cfg(target_os = "windows")]
pub fn normalize_java_path(java_path: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(java_path);

    // If path doesn't exist and doesn't end with .exe, try adding .exe
    if !path.exists() && path.extension().is_none() {
        path.set_extension("exe");
    }

    // If still not found and it's just "java.exe", try to find it in PATH
    if !path.exists() && path.file_name() == Some(std::ffi::OsStr::new("java.exe")) {
        // Try to locate java.exe in PATH
        if let Ok(output) = std::process::Command::new("where").arg("java").output() {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                if let Some(first_path) = paths.lines().next() {
                    path = PathBuf::from(first_path.trim());
                }
            }
        }
    }

    // Verify the path exists
    if !path.exists() {
        return Err(format!(
            "Java executable not found at: {}\nPlease configure a valid Java path in Settings.",
            path.display()
        ));
    }

    Ok(path)
}

#[cfg(not(target_os = "windows"))]
pub fn normalize_java_path(java_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(java_path);

    if !path.exists() && java_path == "java" {
        // Try to find java in PATH
        if let Ok(output) = std::process::Command::new("which").arg("java").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(first_path) = path_str.lines().next() {
                    return Ok(PathBuf::from(first_path.trim()));
                }
            }
        }
    }

    if !path.exists() && java_path != "java" {
        return Err(format!(
            "Java executable not found at: {}\nPlease configure a valid Java path in Settings.",
            path.display()
        ));
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_normalize_adds_exe_extension() {
        // This test assumes java is not in the current directory
        let result = normalize_java_path("nonexistent_java");
        // Should fail since the file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_existing_path() {
        // Test with a path that should exist on most systems
        #[cfg(target_os = "windows")]
        let test_path = "C:\\Windows\\System32\\cmd.exe";
        #[cfg(not(target_os = "windows"))]
        let test_path = "/bin/sh";

        if std::path::Path::new(test_path).exists() {
            let result = normalize_java_path(test_path);
            assert!(result.is_ok());
        }
    }
}
