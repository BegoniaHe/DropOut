/*
ProviderRegistry - draft implementation

Placed under: src-tauri/src/core/java/providers/registry.rs

Purpose:
- Provide a thread-safe registry of Java providers (Adoptium, Corretto, etc.)
- Allow registration, lookup by name, default selection, removal, and enumeration.
- Keep the surface small and ergonomic for initial integration into the codebase.

Design notes / caveats:
- This is a draft. It uses trait objects `Arc<dyn JavaProvider + Send + Sync>` which
  requires the `JavaProvider` trait to be object-safe. In this repository we rely on
  `async-trait` for async trait methods (see changes to `provider.rs` and `adoptium.rs`).
- The registry uses `std::sync::RwLock` for concurrent reads and exclusive writes,
  suitable for runtime mutation (e.g., UI toggles default provider). If you expect
  high-frequency updates, consider more advanced concurrency primitives.
- Methods perform simple `.unwrap()` on lock acquisition for clarity; in a production
  implementation you may want to handle poisoning explicitly.
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::java::provider::JavaProvider;

/// Thread-safe registry of Java providers.
///
/// Keys are normalized to lowercase. Providers are stored as `Arc<dyn JavaProvider + Send + Sync>`.
/// The registry maintains an optional default provider name.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn JavaProvider + Send + Sync>>>,
    default: RwLock<Option<String>>,
}

impl ProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default: RwLock::new(None),
        }
    }

    /// Register a provider under `name`.
    ///
    /// If `make_default` is true, this provider becomes the default provider.
    pub fn register(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn JavaProvider + Send + Sync>,
        make_default: bool,
    ) {
        let name = name.into().to_lowercase();
        {
            let mut map = self.providers.write().unwrap();
            map.insert(name.clone(), provider);
        }
        if make_default {
            let mut d = self.default.write().unwrap();
            *d = Some(name);
        }
    }

    /// Remove a provider by name. Returns the removed provider if it existed.
    /// If the removed provider was the default, the default is unset.
    pub fn remove(&self, name: &str) -> Option<Arc<dyn JavaProvider + Send + Sync>> {
        let name = name.to_lowercase();
        let removed = {
            let mut map = self.providers.write().unwrap();
            map.remove(&name)
        };
        if removed.is_some() {
            let mut d = self.default.write().unwrap();
            if d.as_ref().map(|s| s == &name).unwrap_or(false) {
                *d = None;
            }
        }
        removed
    }

    /// Set the default provider by name (must be already registered).
    pub fn set_default(&self, name: &str) -> Result<(), String> {
        let name = name.to_lowercase();
        let map = self.providers.read().unwrap();
        if map.contains_key(&name) {
            let mut d = self.default.write().unwrap();
            *d = Some(name);
            Ok(())
        } else {
            Err(format!("Provider '{}' not found", name))
        }
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn JavaProvider + Send + Sync>> {
        let name = name.to_lowercase();
        self.providers.read().unwrap().get(&name).cloned()
    }

    /// Get the default provider (if set).
    pub fn default(&self) -> Option<Arc<dyn JavaProvider + Send + Sync>> {
        let opt = self.default.read().unwrap().clone();
        match opt {
            Some(name) => self.get(&name),
            None => None,
        }
    }

    /// List registered provider names.
    pub fn names(&self) -> Vec<String> {
        self.providers
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    }

    /// Number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.read().unwrap().len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Simple dummy provider used only for registry tests.
    #[derive(Clone)]
    struct DummyProvider {
        name: &'static str,
    }

    #[async_trait::async_trait]
    impl crate::core::java::provider::JavaProvider for DummyProvider {
        async fn fetch_catalog(
            &self,
            _app_handle: &tauri::AppHandle,
            _force_refresh: bool,
        ) -> Result<crate::core::java::JavaCatalog, crate::core::java::error::JavaError> {
            Ok(Default::default())
        }

        async fn fetch_release(
            &self,
            _major_version: u32,
            _image_type: crate::core::java::ImageType,
        ) -> Result<crate::core::java::JavaDownloadInfo, crate::core::java::error::JavaError>
        {
            Err(crate::core::java::error::JavaError::NotFound)
        }

        async fn available_versions(
            &self,
        ) -> Result<Vec<u32>, crate::core::java::error::JavaError> {
            Ok(vec![])
        }

        fn provider_name(&self) -> &'static str {
            self.name
        }

        fn os_name(&self) -> &'static str {
            "linux"
        }

        fn arch_name(&self) -> &'static str {
            "x64"
        }

        fn install_prefix(&self) -> &'static str {
            "dummy"
        }
    }

    #[test]
    fn test_register_get_default_remove() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());

        let provider = Arc::new(DummyProvider { name: "dummy" });
        registry.register("dummy", provider.clone(), true);

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.names(), vec!["dummy".to_string()]);

        let default = registry
            .default()
            .expect("default provider should be present");
        assert_eq!(default.provider_name(), "dummy");

        registry.set_default("dummy").unwrap();
        assert!(registry.get("dummy").is_some());

        let removed = registry.remove("dummy");
        assert!(removed.is_some());
        assert!(registry.get("dummy").is_none());
        assert!(registry.default().is_none());
    }

    #[test]
    fn test_set_default_errors() {
        let registry = ProviderRegistry::new();
        assert!(registry.set_default("nope").is_err());
    }
}
