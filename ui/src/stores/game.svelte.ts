import { invoke } from "@tauri-apps/api/core";
import type { Version } from "../types";
import { uiState } from "./ui.svelte";
import { authState } from "./auth.svelte";

export class GameState {
  versions = $state<Version[]>([]);
  installedVersionIds = $state<string[]>([]);
  selectedVersion = $state("");

  async loadVersions() {
    try {
      // Fetch both full version list and installed versions
      const [allVersions, installedIds] = await Promise.all([
        invoke<Version[]>("get_versions"),
        invoke<string[]>("get_installed_versions")
      ]);
      
      this.versions = allVersions;
      this.installedVersionIds = installedIds;

      if (this.installedVersionIds.length > 0) {
        // Find the first installed version that appears in our manifest (preserving order)
        // Usually we want the latest release that is installed
        const installedVersions = this.versions.filter(v => this.installedVersionIds.includes(v.id));
        
        // Try to find latest release among installed
        const latestInstalledRelease = installedVersions.find(v => v.type === "release");
        
        if (latestInstalledRelease) {
          this.selectedVersion = latestInstalledRelease.id;
        } else if (installedVersions.length > 0) {
          this.selectedVersion = installedVersions[0].id;
        } else {
            // Fallback to just the first ID if not in manifest
            this.selectedVersion = this.installedVersionIds[0];
        }
      }
    } catch (e) {
      console.error("Failed to fetch versions:", e);
      uiState.setStatus("Error fetching versions: " + e);
    }
  }

  async startGame() {
    if (!authState.currentAccount) {
      alert("Please login first!");
      authState.openLoginModal();
      return;
    }

    if (!this.selectedVersion) {
      alert("Please select a version!");
      return;
    }

    uiState.setStatus("Preparing to launch " + this.selectedVersion + "...");
    console.log("Invoking start_game for version:", this.selectedVersion);
    try {
      const msg = await invoke<string>("start_game", { versionId: this.selectedVersion });
      console.log("Response:", msg);
      uiState.setStatus(msg);
    } catch (e) {
      console.error(e);
      uiState.setStatus("Error: " + e);
    }
  }
}

export const gameState = new GameState();
