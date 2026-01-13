<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import DownloadMonitor from "./lib/DownloadMonitor.svelte";
  import GameConsole from "./lib/GameConsole.svelte";

  let status = "Ready";
  let showConsole = false;
  let currentView = 'home';
  let statusTimeout: any;

  // Watch for status changes to auto-dismiss
  $: if (status !== "Ready") {
    if (statusTimeout) clearTimeout(statusTimeout);
    statusTimeout = setTimeout(() => {
        status = "Ready";
    }, 5000);
  }

  interface Version {
      id: string;
      type: string;
      url: string;
      time: string;
      releaseTime: string;
  }

  interface OfflineAccount {
      username: string;
      uuid: string;
  }

  let versions: Version[] = [];
  let selectedVersion = "";
  let currentAccount: OfflineAccount | null = null;

  onMount(async () => {
      checkAccount();
      try {
          versions = await invoke("get_versions");
          if (versions.length > 0) {
              // Find latest release or default to first
              const latest = versions.find(v => v.type === 'release');
              selectedVersion = latest ? latest.id : versions[0].id;
          }
      } catch (e) {
          console.error("Failed to fetch versions:", e);
          status = "Error fetching versions: " + e;
      }
  });

  async function checkAccount() {
      try {
          const acc = await invoke("get_active_account");
          currentAccount = acc as OfflineAccount | null;
      } catch (e) {
          console.error("Failed to check account:", e);
      }
  }

  async function login() {
      if (currentAccount) {
         if (confirm("Logout " + currentAccount.username + "?")) {
             try {
                await invoke("logout");
                currentAccount = null;
             } catch(e) {
                console.error("Logout failed:", e);
             }
         }
         return;
      }
      const username = prompt("Enter username for offline login:");
      if (username) {
          try {
              currentAccount = await invoke("login_offline", { username });
          } catch(e) {
              alert("Login failed: " + e);
          }
      }
  }

  async function startGame() {
    if (!currentAccount) {
        alert("Please login first!");
        login();
        return;
    }

    if (!selectedVersion) {
        alert("Please select a version!");
        return;
    }

    status = "Preparing to launch " + selectedVersion + "...";
    console.log("Invoking start_game for version:", selectedVersion);
    try {
        const msg = await invoke("start_game", { versionId: selectedVersion });
        console.log("Response:", msg);
        status = msg as string;
    } catch (e) {
        console.error(e);
        status = "Error: " + e;
    }
  }
</script>

<div class="flex h-screen bg-zinc-900 text-white font-sans overflow-hidden select-none">
  <!-- Sidebar -->
  <aside class="w-20 lg:w-64 bg-zinc-950 flex flex-col items-center lg:items-start transition-all duration-300 border-r border-zinc-800 shrink-0">
    <div class="h-20 w-full flex items-center justify-center lg:justify-start lg:px-6 border-b border-zinc-800/50">
        <!-- Icon Logo (Visible on small) -->
        <div class="lg:hidden text-2xl font-black bg-clip-text text-transparent bg-gradient-to-tr from-indigo-400 to-purple-400">D</div>
        <!-- Full Logo (Visible on large) -->
        <div class="hidden lg:block font-bold text-xl tracking-wider text-indigo-400">DROP<span class="text-white">OUT</span></div>
    </div>
    
    <nav class="flex-1 w-full flex flex-col gap-2 p-3">
      <button 
        class="group flex items-center lg:gap-4 justify-center lg:justify-start w-full px-0 lg:px-4 py-3 rounded-lg hover:bg-zinc-800 {currentView === 'home' ? 'bg-zinc-800/80 text-white' : 'text-zinc-400'} transition-all relative"
        onclick={() => currentView = 'home'}
      >
        <span class="text-xl relative z-10">🏠</span>
        <span class="hidden lg:block font-medium relative z-10 transition-opacity">Home</span>
      </button>
      <button 
        class="group flex items-center lg:gap-4 justify-center lg:justify-start w-full px-0 lg:px-4 py-3 rounded-lg hover:bg-zinc-800 {currentView === 'versions' ? 'bg-zinc-800/80 text-white' : 'text-zinc-400'} transition-all"
        onclick={() => currentView = 'versions'}
      >
        <span class="text-xl">📦</span>
        <span class="hidden lg:block font-medium">Versions</span>
      </button>
      <button 
        class="group flex items-center lg:gap-4 justify-center lg:justify-start w-full px-0 lg:px-4 py-3 rounded-lg hover:bg-zinc-800 {currentView === 'settings' ? 'bg-zinc-800/80 text-white' : 'text-zinc-400'} transition-all"
        onclick={() => currentView = 'settings'}
      >
        <span class="text-xl">⚙️</span>
        <span class="hidden lg:block font-medium">Settings</span>
      </button>
    </nav>
    
    <div class="p-4 w-full border-t border-zinc-800 flex justify-center lg:justify-start">
      <div class="text-xs text-zinc-600 font-mono">v0.1.3</div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col relative min-w-0">
    <DownloadMonitor />
    <!-- Top Bar (Window Controls Placeholder) -->
    <div class="h-8 w-full bg-zinc-900/50 absolute top-0 left-0 z-50 drag-region" data-tauri-drag-region>
        <!-- Windows/macOS controls would go here or be handled by OS -->
    </div>

    <!-- Background / Poster area -->
    <div class="flex-1 relative overflow-hidden group">
      {#if currentView === 'home'}
        <!-- Background Image -->
        <div class="absolute inset-0 z-0 opacity-40 bg-[url('https://www.minecraft.net/content/dam/games/minecraft/key-art/Minecraft-KeyArt-02_800x450.jpg')] bg-cover bg-center transition-transform duration-[10s] ease-linear group-hover:scale-105"></div>
        <div class="absolute inset-0 z-0 bg-gradient-to-t from-zinc-900 via-transparent to-black/50"></div>

        <div class="absolute bottom-24 left-8 z-10 p-4">
            <h1 class="text-6xl font-black mb-2 tracking-tight text-white drop-shadow-lg">MINECRAFT</h1>
            <div class="flex items-center gap-2 text-zinc-300">
                <span class="bg-zinc-800 text-xs px-2 py-1 rounded border border-zinc-600">JAVA EDITION</span>
                <span class="text-lg">Release 1.20.4</span>
            </div>
        </div>
      {:else if currentView === 'versions'}
        <div class="p-8 h-full overflow-y-auto bg-zinc-900">
            <h2 class="text-3xl font-bold mb-6">Versions</h2>
            <div class="grid gap-2">
                {#if versions.length === 0}
                    <div class="text-zinc-500">Loading versions...</div>
                {:else}
                    {#each versions as version}
                        <button 
                            class="flex items-center justify-between p-4 bg-zinc-800 rounded hover:bg-zinc-700 transition text-left border border-zinc-700 {selectedVersion === version.id ? 'border-green-500 bg-zinc-800/80 ring-1 ring-green-500' : ''}"
                            onclick={() => selectedVersion = version.id}
                        >
                            <div>
                                <div class="font-bold font-mono text-lg">{version.id}</div>
                                <div class="text-xs text-zinc-400 capitalize">{version.type} • {new Date(version.releaseTime).toLocaleDateString()}</div>
                            </div>
                            {#if selectedVersion === version.id}
                                <div class="text-green-500 font-bold text-sm">SELECTED</div>
                            {/if}
                        </button>
                    {/each}
                {/if}
            </div>
        </div>
      {:else if currentView === 'settings'}
        <div class="p-8 bg-zinc-900 h-full">
            <h2 class="text-3xl font-bold mb-6">Settings</h2>
            <div class="text-zinc-400">Settings page coming soon...</div>
        </div>
      {/if}
    </div>

    <!-- Bottom Bar -->
    <div class="h-24 bg-zinc-900 border-t border-zinc-800 flex items-center px-8 justify-between z-20 shadow-2xl">
        <div class="flex items-center gap-4">
             <div class="flex items-center gap-4 cursor-pointer hover:opacity-80 transition-opacity" onclick={login} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && login()}>
                <div class="w-12 h-12 rounded bg-gradient-to-tr from-indigo-500 to-purple-500 shadow-lg flex items-center justify-center text-white font-bold text-xl overflow-hidden">
                    {#if currentAccount}
                        <img src={`https://minotar.net/avatar/${currentAccount.username}/48`} alt={currentAccount.username} class="w-full h-full">
                    {:else}
                        ?
                    {/if}
                </div>
                <div>
                    <div class="font-bold text-white text-lg">{currentAccount ? currentAccount.username : "Click to Login"}</div>
                    <div class="text-xs text-zinc-400 flex items-center gap-1">
                        <span class="w-1.5 h-1.5 rounded-full {currentAccount ? 'bg-green-500' : 'bg-zinc-500'}"></span> {currentAccount ? 'Ready' : 'Guest'}
                    </div>
                </div>
            </div>
            <!-- Console Toggle -->
            <button class="ml-4 text-xs text-zinc-500 hover:text-zinc-300 transition" onclick={() => showConsole = !showConsole}>
                {showConsole ? 'Hide Logs' : 'Show Logs'}
            </button>
        </div>

        <div class="flex items-center gap-4">
            <div class="flex flex-col items-end mr-2">
                 <label class="text-xs text-zinc-500 mb-1 uppercase font-bold tracking-wider">Version</label>
                 <select bind:value={selectedVersion} class="bg-zinc-950 text-zinc-200 border border-zinc-700 rounded px-4 py-2 hover:border-zinc-500 transition-colors cursor-pointer outline-none focus:ring-1 focus:ring-indigo-500 w-48">
                    {#if versions.length === 0}
                        <option>Loading...</option>
                    {:else}
                        {#each versions as version}
                            <option value={version.id}>{version.id} ({version.type})</option>
                        {/each}
                    {/if}
                </select>
            </div>
           
            <button
                onclick={startGame}
                class="bg-green-600 hover:bg-green-500 text-white font-bold h-14 px-12 rounded transition-all transform active:scale-95 shadow-[0_0_15px_rgba(22,163,74,0.4)] hover:shadow-[0_0_25px_rgba(22,163,74,0.6)] flex flex-col items-center justify-center uppercase tracking-wider text-lg"
            >
                Play
                <span class="text-[10px] font-normal opacity-80 normal-case tracking-normal">Click to launch</span>
            </button>
        </div>
    </div>
  </main>

  <!-- Overlay Status (Toast) -->
  {#if status !== "Ready"}
  <div class="absolute top-12 right-12 bg-zinc-800/90 backdrop-blur border border-zinc-600 p-4 rounded-lg shadow-2xl max-w-sm animate-in fade-in slide-in-from-top-4 duration-300 z-50 group">
      <div class="flex justify-between items-start mb-1">
          <div class="text-xs text-zinc-400 uppercase font-bold">Status</div>
          <button 
            onclick={() => status = "Ready"}
            class="text-zinc-500 hover:text-white transition -mt-1 -mr-1 p-1"
          >
            ✕
          </button>
      </div>
      <div class="font-mono text-sm whitespace-pre-wrap mb-2">{status}</div>
      <div class="w-full bg-zinc-700/50 h-1 rounded-full overflow-hidden">
          <div class="h-full bg-indigo-500 animate-[progress_5s_linear_forwards] origin-left w-full"></div>
      </div>
  </div>
  {/if}

  <style>
      @keyframes progress {
          from { transform: scaleX(1); }
          to { transform: scaleX(0); }
      }
  </style>

  <GameConsole visible={showConsole} />
</div>
