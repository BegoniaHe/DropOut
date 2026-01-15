//! Forge Loader support module.
//!
//! This module provides functionality to:
//! - Fetch available Forge versions from the Forge promotions API
//! - Install Forge loader for a specific Minecraft version
//!
//! Note: Forge installation is more complex than Fabric, especially for versions 1.13+.
//! This implementation focuses on the basic JSON generation approach.
//! For full Forge 1.13+ support, processor execution would need to be implemented.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

const FORGE_PROMOTIONS_URL: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
const FORGE_MAVEN_URL: &str = "https://maven.minecraftforge.net/";

/// Represents a Forge version entry.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ForgeVersion {
    pub version: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub latest: bool,
}

/// Forge promotions response from the API.
#[derive(Debug, Deserialize)]
struct ForgePromotions {
    promos: std::collections::HashMap<String, String>,
}

/// Information about an installed Forge version.
#[derive(Debug, Serialize, Clone)]
pub struct InstalledForgeVersion {
    pub id: String,
    pub minecraft_version: String,
    pub forge_version: String,
    pub path: PathBuf,
}

/// Fetch all Minecraft versions supported by Forge.
///
/// # Returns
/// A list of Minecraft version strings that have Forge available.
pub async fn fetch_supported_game_versions() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let promos = fetch_promotions().await?;

    let mut versions: Vec<String> = promos
        .promos
        .keys()
        .filter_map(|key| {
            // Keys are like "1.20.4-latest", "1.20.4-recommended"
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() >= 2 {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();

    // Deduplicate and sort
    versions.sort();
    versions.dedup();
    versions.reverse(); // Newest first

    Ok(versions)
}

/// Fetch Forge promotions data.
async fn fetch_promotions() -> Result<ForgePromotions, Box<dyn Error + Send + Sync>> {
    let resp = reqwest::get(FORGE_PROMOTIONS_URL)
        .await?
        .json::<ForgePromotions>()
        .await?;
    Ok(resp)
}

/// Fetch available Forge versions for a specific Minecraft version.
///
/// # Arguments
/// * `game_version` - The Minecraft version (e.g., "1.20.4")
///
/// # Returns
/// A list of Forge versions available for the specified game version.
pub async fn fetch_forge_versions(
    game_version: &str,
) -> Result<Vec<ForgeVersion>, Box<dyn Error + Send + Sync>> {
    let promos = fetch_promotions().await?;
    let mut versions = Vec::new();

    // Look for both latest and recommended
    let latest_key = format!("{}-latest", game_version);
    let recommended_key = format!("{}-recommended", game_version);

    if let Some(latest) = promos.promos.get(&latest_key) {
        versions.push(ForgeVersion {
            version: latest.clone(),
            minecraft_version: game_version.to_string(),
            recommended: false,
            latest: true,
        });
    }

    if let Some(recommended) = promos.promos.get(&recommended_key) {
        // Don't duplicate if recommended == latest
        if !versions.iter().any(|v| v.version == *recommended) {
            versions.push(ForgeVersion {
                version: recommended.clone(),
                minecraft_version: game_version.to_string(),
                recommended: true,
                latest: false,
            });
        } else {
            // Mark the existing one as both
            if let Some(v) = versions.iter_mut().find(|v| v.version == *recommended) {
                v.recommended = true;
            }
        }
    }

    Ok(versions)
}

/// Generate the version ID for a Forge installation.
///
/// # Arguments
/// * `game_version` - The Minecraft version
/// * `forge_version` - The Forge version
///
/// # Returns
/// The version ID string (e.g., "1.20.4-forge-49.0.38")
pub fn generate_version_id(game_version: &str, forge_version: &str) -> String {
    format!("{}-forge-{}", game_version, forge_version)
}

/// Install Forge for a specific Minecraft version.
///
/// Note: This creates a basic version JSON. For Forge 1.13+, the full installation
/// requires running the Forge installer processors, which is not yet implemented.
/// This basic implementation works for legacy Forge versions (<1.13) and creates
/// the structure needed for modern Forge (libraries will need to be downloaded
/// separately).
///
/// # Arguments
/// * `game_dir` - The .minecraft directory path
/// * `game_version` - The Minecraft version (e.g., "1.20.4")
/// * `forge_version` - The Forge version (e.g., "49.0.38")
///
/// # Returns
/// Information about the installed version.
pub async fn install_forge(
    game_dir: &PathBuf,
    game_version: &str,
    forge_version: &str,
) -> Result<InstalledForgeVersion, Box<dyn Error + Send + Sync>> {
    let version_id = generate_version_id(game_version, forge_version);

    // Create basic version JSON structure
    // Note: This is a simplified version. Full Forge installation requires
    // downloading the installer and running processors.
    let version_json = create_forge_version_json(game_version, forge_version)?;

    // Create the version directory
    let version_dir = game_dir.join("versions").join(&version_id);
    tokio::fs::create_dir_all(&version_dir).await?;

    // Write the version JSON
    let json_path = version_dir.join(format!("{}.json", version_id));
    let json_content = serde_json::to_string_pretty(&version_json)?;
    tokio::fs::write(&json_path, json_content).await?;

    Ok(InstalledForgeVersion {
        id: version_id,
        minecraft_version: game_version.to_string(),
        forge_version: forge_version.to_string(),
        path: json_path,
    })
}

/// Create a basic Forge version JSON.
///
/// This creates a minimal version JSON that inherits from vanilla and adds
/// the Forge libraries. For full functionality with Forge 1.13+, the installer
/// would need to be run to patch the game.
fn create_forge_version_json(
    game_version: &str,
    forge_version: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    let version_id = generate_version_id(game_version, forge_version);
    let forge_maven_coord = format!(
        "net.minecraftforge:forge:{}-{}",
        game_version, forge_version
    );

    // Determine main class based on version
    // Forge 1.13+ uses different launchers
    let (main_class, libraries) = if is_modern_forge(game_version) {
        // Modern Forge (1.13+) uses cpw.mods.bootstraplauncher
        (
            "cpw.mods.bootstraplauncher.BootstrapLauncher".to_string(),
            vec![
                create_library_entry(&forge_maven_coord, Some(FORGE_MAVEN_URL)),
                create_library_entry(
                    &format!(
                        "net.minecraftforge:forge:{}-{}:universal",
                        game_version, forge_version
                    ),
                    Some(FORGE_MAVEN_URL),
                ),
            ],
        )
    } else {
        // Legacy Forge uses LaunchWrapper
        (
            "net.minecraft.launchwrapper.Launch".to_string(),
            vec![
                create_library_entry(&forge_maven_coord, Some(FORGE_MAVEN_URL)),
                create_library_entry("net.minecraft:launchwrapper:1.12", None),
            ],
        )
    };

    let json = serde_json::json!({
        "id": version_id,
        "inheritsFrom": game_version,
        "type": "release",
        "mainClass": main_class,
        "libraries": libraries,
        "arguments": {
            "game": [],
            "jvm": []
        }
    });

    Ok(json)
}

/// Create a library entry for the version JSON.
fn create_library_entry(name: &str, maven_url: Option<&str>) -> serde_json::Value {
    let mut entry = serde_json::json!({
        "name": name
    });

    if let Some(url) = maven_url {
        entry["url"] = serde_json::Value::String(url.to_string());
    }

    entry
}

/// Check if the Minecraft version uses modern Forge (1.13+).
fn is_modern_forge(game_version: &str) -> bool {
    let parts: Vec<&str> = game_version.split('.').collect();
    if parts.len() >= 2 {
        if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return major > 1 || (major == 1 && minor >= 13);
        }
    }
    false
}

/// Check if Forge is installed for a specific version combination.
///
/// # Arguments
/// * `game_dir` - The .minecraft directory path
/// * `game_version` - The Minecraft version
/// * `forge_version` - The Forge version
///
/// # Returns
/// `true` if the version JSON exists, `false` otherwise.
pub fn is_forge_installed(game_dir: &PathBuf, game_version: &str, forge_version: &str) -> bool {
    let version_id = generate_version_id(game_version, forge_version);
    let json_path = game_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{}.json", version_id));
    json_path.exists()
}

/// List all installed Forge versions in the game directory.
///
/// # Arguments
/// * `game_dir` - The .minecraft directory path
///
/// # Returns
/// A list of installed Forge version IDs.
pub async fn list_installed_forge_versions(
    game_dir: &PathBuf,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let versions_dir = game_dir.join("versions");
    let mut installed = Vec::new();

    if !versions_dir.exists() {
        return Ok(installed);
    }

    let mut entries = tokio::fs::read_dir(&versions_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains("-forge-") {
            // Verify the JSON file exists
            let json_path = entry.path().join(format!("{}.json", name));
            if json_path.exists() {
                installed.push(name);
            }
        }
    }

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_version_id() {
        assert_eq!(
            generate_version_id("1.20.4", "49.0.38"),
            "1.20.4-forge-49.0.38"
        );
    }

    #[test]
    fn test_is_modern_forge() {
        assert!(!is_modern_forge("1.12.2"));
        assert!(is_modern_forge("1.13"));
        assert!(is_modern_forge("1.20.4"));
        assert!(is_modern_forge("1.21"));
    }
}
