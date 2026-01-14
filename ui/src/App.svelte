<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import DownloadMonitor from "./lib/DownloadMonitor.svelte";
  import GameConsole from "./lib/GameConsole.svelte";
  
  // Components
  import Sidebar from "./components/Sidebar.svelte";
  import HomeView from "./components/HomeView.svelte";
  import VersionsView from "./components/VersionsView.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import BottomBar from "./components/BottomBar.svelte";
  import LoginModal from "./components/LoginModal.svelte";
  import StatusToast from "./components/StatusToast.svelte";

  // Stores
  import { uiState } from "./stores/ui.svelte";
  import { authState } from "./stores/auth.svelte";
  import { settingsState } from "./stores/settings.svelte";
  import { gameState } from "./stores/game.svelte";

  onMount(async () => {
    authState.checkAccount();
    settingsState.loadSettings();
    gameState.loadVersions();
    getVersion().then((v) => (uiState.appVersion = v));
  });
</script>

<div
  class="flex h-screen bg-zinc-900 text-white font-sans overflow-hidden select-none"
>
  <Sidebar />

  <!-- Main Content -->
  <main class="flex-1 flex flex-col relative min-w-0">
    <DownloadMonitor />
    <!-- Top Bar (Window Controls Placeholder) -->
    <div
      class="h-8 w-full bg-zinc-900/50 absolute top-0 left-0 z-50 drag-region"
      data-tauri-drag-region
    >
      <!-- Windows/macOS controls would go here or be handled by OS -->
    </div>

    <!-- Background / Poster area -->
    <div class="flex-1 relative overflow-hidden group">
      {#if uiState.currentView === "home"}
        <HomeView />
      {:else if uiState.currentView === "versions"}
        <VersionsView />
      {:else if uiState.currentView === "settings"}
        <SettingsView />
      {/if}
    </div>

    <BottomBar />
  </main>

  <LoginModal />
  <StatusToast />

  <GameConsole visible={uiState.showConsole} />
</div>
