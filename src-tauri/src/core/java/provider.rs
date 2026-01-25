use crate::core::java::{ImageType, JavaCatalog, JavaDownloadInfo};
use tauri::AppHandle;

/// Trait for Java download providers (Adoptium, Temurin, Corretto, etc.)
/// 
/// Implements SOLID principles:
/// - Single Responsibility: Each provider handles one download source
/// - Open/Closed: New providers can be added without modifying existing code
/// - Liskov Substitution: All providers are interchangeable
/// - Interface Segregation: Minimal required methods
/// - Dependency Inversion: Code depends on trait, not concrete implementations
pub trait JavaProvider: Send + Sync {
    /// Fetch the Java catalog (all available versions for this provider)
    async fn fetch_catalog(
        &self,
        app_handle: &AppHandle,
        force_refresh: bool,
    ) -> Result<JavaCatalog, String>;

    /// Fetch a specific Java release
    async fn fetch_release(
        &self,
        major_version: u32,
        image_type: ImageType,
    ) -> Result<JavaDownloadInfo, String>;

    /// Get list of available major versions
    async fn available_versions(&self) -> Result<Vec<u32>, String>;

    /// Get provider name (e.g., "adoptium", "corretto")
    fn provider_name(&self) -> &'static str;

    /// Get OS name for this provider's API
    fn os_name(&self) -> &'static str;

    /// Get architecture name for this provider's API
    fn arch_name(&self) -> &'static str;

    /// Get installation directory prefix (e.g., "temurin", "corretto")
    fn install_prefix(&self) -> &'static str;
}
