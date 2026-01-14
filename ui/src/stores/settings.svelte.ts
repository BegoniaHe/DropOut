import { invoke } from "@tauri-apps/api/core";
import type { LauncherConfig, JavaInstallation, JavaDownloadInfo } from "../types";
import { uiState } from "./ui.svelte";

export class SettingsState {
  settings = $state<LauncherConfig>({
    min_memory: 1024,
    max_memory: 2048,
    java_path: "java",
    width: 854,
    height: 480,
    download_threads: 32,
    enable_gpu_acceleration: false,
    enable_visual_effects: true,
    active_effect: "constellation",
    theme: "dark",
  });
  javaInstallations = $state<JavaInstallation[]>([]);
  isDetectingJava = $state(false);
  
  // Java download state
  showJavaDownloadModal = $state(false);
  availableJavaVersions = $state<number[]>([]);
  selectedJavaVersion = $state(21);
  selectedImageType = $state<"jre" | "jdk">("jre");
  isDownloadingJava = $state(false);
  javaDownloadStatus = $state("");

  async loadSettings() {
    try {
      const result = await invoke<LauncherConfig>("get_settings");
      this.settings = result;
      // Force dark mode
      if (this.settings.theme !== "dark") {
          this.settings.theme = "dark";
          this.saveSettings();
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async saveSettings() {
    try {
      await invoke("save_settings", { config: this.settings });
      uiState.setStatus("Settings saved!");
    } catch (e) {
      console.error("Failed to save settings:", e);
      uiState.setStatus("Error saving settings: " + e);
    }
  }

  async detectJava() {
    this.isDetectingJava = true;
    try {
      this.javaInstallations = await invoke("detect_java");
      if (this.javaInstallations.length === 0) {
        uiState.setStatus("No Java installations found");
      } else {
        uiState.setStatus(`Found ${this.javaInstallations.length} Java installation(s)`);
      }
    } catch (e) {
      console.error("Failed to detect Java:", e);
      uiState.setStatus("Error detecting Java: " + e);
    } finally {
      this.isDetectingJava = false;
    }
  }

  selectJava(path: string) {
    this.settings.java_path = path;
  }

  async openJavaDownloadModal() {
    this.showJavaDownloadModal = true;
    this.javaDownloadStatus = "";
    try {
      this.availableJavaVersions = await invoke("fetch_available_java_versions");
      // Default selection logic
      if (this.availableJavaVersions.includes(21)) {
        this.selectedJavaVersion = 21;
      } else if (this.availableJavaVersions.includes(17)) {
        this.selectedJavaVersion = 17;
      } else if (this.availableJavaVersions.length > 0) {
        this.selectedJavaVersion = this.availableJavaVersions[this.availableJavaVersions.length - 1];
      }
    } catch (e) {
      console.error("Failed to fetch available Java versions:", e);
      this.javaDownloadStatus = "Error fetching Java versions: " + e;
    }
  }

  closeJavaDownloadModal() {
    if (!this.isDownloadingJava) {
      this.showJavaDownloadModal = false;
    }
  }

  async downloadJava() {
    this.isDownloadingJava = true;
    this.javaDownloadStatus = `Downloading Java ${this.selectedJavaVersion} ${this.selectedImageType.toUpperCase()}...`;
    
    try {
      const result: JavaInstallation = await invoke("download_adoptium_java", {
        majorVersion: this.selectedJavaVersion,
        imageType: this.selectedImageType,
        customPath: null,
      });
      
      this.javaDownloadStatus = `Java ${this.selectedJavaVersion} installed at ${result.path}`;
      this.settings.java_path = result.path;
      
      await this.detectJava();
      
      setTimeout(() => {
        this.showJavaDownloadModal = false;
        uiState.setStatus(`Java ${this.selectedJavaVersion} is ready to use!`);
      }, 1500);
    } catch (e) {
      console.error("Failed to download Java:", e);
      this.javaDownloadStatus = "Download failed: " + e;
    } finally {
      this.isDownloadingJava = false;
    }
  }
}

export const settingsState = new SettingsState();
