use serde::{Deserialize, Serialize};

/// Represents a Minecraft version JSON, supporting both vanilla and modded (Fabric/Forge) formats.
/// Modded versions use `inheritsFrom` to reference a parent vanilla version.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameVersion {
    pub id: String,
    /// Optional for mod loaders that inherit from vanilla
    pub downloads: Option<Downloads>,
    /// Optional for mod loaders that inherit from vanilla
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<Arguments>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    /// For mod loaders: the vanilla version this inherits from
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    /// Fabric/Forge may specify a custom assets version
    pub assets: Option<String>,
    /// Release type (release, snapshot, old_beta, etc.)
    #[serde(rename = "type")]
    pub version_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Downloads {
    pub client: DownloadArtifact,
    pub server: Option<DownloadArtifact>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DownloadArtifact {
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Library {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<serde_json::Value>,
    /// Maven repository URL for mod loader libraries
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub action: String, // "allow" or "disallow"
    pub os: Option<OsRule>,
    pub features: Option<serde_json::Value>, // Feature-based rules (e.g., is_demo_user, has_quick_plays_support)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OsRule {
    pub name: Option<String>,    // "linux", "osx", "windows"
    pub version: Option<String>, // Regex
    pub arch: Option<String>,    // "x86"
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadArtifact>,
    pub classifiers: Option<serde_json::Value>, // Complex, simplifying for now
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Arguments {
    pub game: Option<serde_json::Value>,
    pub jvm: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u64,
}
