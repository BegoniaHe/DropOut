export type ViewType = "home" | "versions" | "settings";

export interface Version {
  id: string;
  type: string;
  url: string;
  time: string;
  releaseTime: string;
}

export interface Account {
  type: "Offline" | "Microsoft";
  username: string;
  uuid: string;
}

export interface DeviceCodeResponse {
  user_code: string;
  device_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
  message?: string;
}

export interface LauncherConfig {
  min_memory: number;
  max_memory: number;
  java_path: string;
  width: number;
  height: number;
  download_threads: number;
  custom_background_path?: string;
  enable_gpu_acceleration: boolean;
  enable_visual_effects: boolean;
  active_effect: string;
  theme: string;
}

export interface JavaInstallation {
  path: string;
  version: string;
  is_64bit: boolean;
}

export interface JavaDownloadInfo {
  version: string;
  release_name: string;
  download_url: string;
  file_name: string;
  file_size: number;
  checksum: string | null;
  image_type: string;
}

// ==================== Fabric Types ====================

export interface FabricGameVersion {
  version: string;
  stable: boolean;
}

export interface FabricLoaderVersion {
  separator: string;
  build: number;
  maven: string;
  version: string;
  stable: boolean;
}

export interface FabricLoaderEntry {
  loader: FabricLoaderVersion;
  intermediary: {
    maven: string;
    version: string;
    stable: boolean;
  };
  launcherMeta: {
    version: number;
    mainClass: {
      client: string;
      server: string;
    };
  };
}

export interface InstalledFabricVersion {
  id: string;
  minecraft_version: string;
  loader_version: string;
  path: string;
}

// ==================== Forge Types ====================

export interface ForgeVersion {
  version: string;
  minecraft_version: string;
  recommended: boolean;
  latest: boolean;
}

export interface InstalledForgeVersion {
  id: string;
  minecraft_version: string;
  forge_version: string;
  path: string;
}

// ==================== Mod Loader Type ====================

export type ModLoaderType = "vanilla" | "fabric" | "forge";

