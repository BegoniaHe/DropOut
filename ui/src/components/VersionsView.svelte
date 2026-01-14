<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { gameState } from "../stores/game.svelte";
  import ModLoaderSelector from "./ModLoaderSelector.svelte";

  let searchQuery = $state("");
  let normalizedQuery = $derived(
    searchQuery.trim().toLowerCase().replace(/。/g, ".")
  );

  // Filter by version type
  let typeFilter = $state<"all" | "release" | "snapshot" | "modded">("all");

  // Installed modded versions
  let installedFabricVersions = $state<string[]>([]);
  let isLoadingModded = $state(false);

  // Load installed modded versions
  async function loadInstalledModdedVersions() {
    isLoadingModded = true;
    try {
      installedFabricVersions = await invoke<string[]>(
        "list_installed_fabric_versions"
      );
    } catch (e) {
      console.error("Failed to load installed fabric versions:", e);
    } finally {
      isLoadingModded = false;
    }
  }

  // Load on mount
  $effect(() => {
    loadInstalledModdedVersions();
  });

  // Combined versions list (vanilla + modded)
  let allVersions = $derived(() => {
    const moddedVersions = installedFabricVersions.map((id) => ({
      id,
      type: "fabric",
      url: "",
      time: "",
      releaseTime: new Date().toISOString(),
    }));
    return [...moddedVersions, ...gameState.versions];
  });

  let filteredVersions = $derived(() => {
    let versions = allVersions();

    // Apply type filter
    if (typeFilter === "release") {
      versions = versions.filter((v) => v.type === "release");
    } else if (typeFilter === "snapshot") {
      versions = versions.filter((v) => v.type === "snapshot");
    } else if (typeFilter === "modded") {
      versions = versions.filter(
        (v) => v.type === "fabric" || v.type === "forge"
      );
    }

    // Apply search filter
    if (normalizedQuery.length > 0) {
      versions = versions.filter((v) =>
        v.id.toLowerCase().includes(normalizedQuery)
      );
    }

    return versions;
  });

  function getVersionBadge(type: string) {
    switch (type) {
      case "release":
        return { text: "Release", class: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30" };
      case "snapshot":
        return { text: "Snapshot", class: "bg-amber-500/20 text-amber-300 border-amber-500/30" };
      case "fabric":
        return { text: "Fabric", class: "bg-indigo-500/20 text-indigo-300 border-indigo-500/30" };
      case "forge":
        return { text: "Forge", class: "bg-orange-500/20 text-orange-300 border-orange-500/30" };
      default:
        return { text: type, class: "bg-zinc-500/20 text-zinc-300 border-zinc-500/30" };
    }
  }

  function handleModLoaderInstall(versionId: string) {
    // Refresh the installed versions list
    loadInstalledModdedVersions();
    // Select the newly installed version
    gameState.selectedVersion = versionId;
  }

  // Get the base Minecraft version from selected version (for mod loader selector)
  let selectedBaseVersion = $derived(() => {
    const selected = gameState.selectedVersion;
    if (!selected) return "";

    // If it's a modded version, extract the base version
    if (selected.startsWith("fabric-loader-")) {
      // Format: fabric-loader-X.X.X-1.20.4
      const parts = selected.split("-");
      return parts[parts.length - 1];
    }
    if (selected.includes("-forge-")) {
      // Format: 1.20.4-forge-49.0.38
      return selected.split("-forge-")[0];
    }

    // Check if it's a valid vanilla version
    const version = gameState.versions.find((v) => v.id === selected);
    return version ? selected : "";
  });
</script>

<div class="h-full flex flex-col p-6 overflow-hidden">
  <div class="flex items-center justify-between mb-6">
     <h2 class="text-3xl font-black bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">Version Manager</h2>
     <div class="text-sm text-white/40">Select a version to play or modify</div>
  </div>

  <div class="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden">
    <!-- Left: Version List -->
    <div class="lg:col-span-2 flex flex-col gap-4 overflow-hidden">
      <!-- Search and Filters (Glass Bar) -->
      <div class="flex gap-3">
        <div class="relative flex-1">
            <span class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30">🔍</span>
            <input
              type="text"
              placeholder="Search versions..."
              class="w-full pl-9 pr-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-white/30 focus:outline-none focus:border-indigo-500/50 focus:bg-black/40 transition-all backdrop-blur-sm"
              bind:value={searchQuery}
            />
        </div>
      </div>

      <!-- Type Filter Tabs (Glass Caps) -->
      <div class="flex p-1 bg-black/20 rounded-xl border border-white/5">
        {#each ['all', 'release', 'snapshot', 'modded'] as filter}
            <button
            class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200 capitalize
            {typeFilter === filter
                ? 'bg-white/10 text-white shadow-lg border border-white/10'
                : 'text-white/40 hover:text-white hover:bg-white/5'}"
            onclick={() => (typeFilter = filter as any)}
            >
            {filter}
            </button>
        {/each}
      </div>

      <!-- Version List SCROLL -->
      <div class="flex-1 overflow-y-auto pr-2 space-y-2 custom-scrollbar">
        {#if gameState.versions.length === 0}
          <div class="flex items-center justify-center h-40 text-white/30 italic animate-pulse">
             Fetching manifest...
          </div>
        {:else if filteredVersions().length === 0}
          <div class="flex flex-col items-center justify-center -40 text-white/30 gap-2">
             <span class="text-2xl">👻</span>
             <span>No matching versions found</span>
          </div>
        {:else}
          {#each filteredVersions() as version}
            {@const badge = getVersionBadge(version.type)}
            {@const isSelected = gameState.selectedVersion === version.id}
            <button
              class="w-full group flex items-center justify-between p-4 rounded-xl text-left border transition-all duration-200 relative overflow-hidden
              {isSelected
                ? 'bg-indigo-600/20 border-indigo-500/50 shadow-[0_0_20px_rgba(99,102,241,0.2)]'
                : 'bg-white/5 border-white/5 hover:bg-white/10 hover:border-white/10 hover:translate-x-1'}"
              onclick={() => (gameState.selectedVersion = version.id)}
            >
              <!-- Selection Glow -->
              {#if isSelected}
                 <div class="absolute inset-0 bg-gradient-to-r from-indigo-500/10 to-transparent pointer-events-none"></div>
              {/if}

              <div class="relative z-10 flex items-center gap-4">
                <span
                  class="px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide border {badge.class}"
                >
                  {badge.text}
                </span>
                <div>
                  <div class="font-bold font-mono text-lg tracking-tight {isSelected ? 'text-white' : 'text-zinc-300 group-hover:text-white'}">
                    {version.id}
                  </div>
                  {#if version.releaseTime && version.type !== "fabric" && version.type !== "forge"}
                    <div class="text-xs text-white/30">
                      {new Date(version.releaseTime).toLocaleDateString()}
                    </div>
                  {/if}
                </div>
              </div>
              
              {#if isSelected}
                <div class="relative z-10 text-indigo-400">
                   <span class="text-lg">Selected</span>
                </div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
    </div>

    <!-- Right: Mod Loader Panel -->
    <div class="flex flex-col gap-4">
      <!-- Selected Version Info Card -->
      <div class="bg-gradient-to-br from-white/10 to-white/5 p-6 rounded-2xl border border-white/10 backdrop-blur-md relative overflow-hidden group">
          <div class="absolute top-0 right-0 p-8 bg-indigo-500/20 blur-[60px] rounded-full group-hover:bg-indigo-500/30 transition-colors"></div>
          
          <h3 class="text-xs font-bold uppercase tracking-widest text-white/40 mb-2 relative z-10">Current Selection</h3>
          {#if gameState.selectedVersion}
            <p class="font-mono text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-white to-white/70 relative z-10 truncate">
                {gameState.selectedVersion}
            </p>
          {:else}
            <p class="text-white/20 italic relative z-10">None selected</p>
          {/if}
      </div>

      <!-- Mod Loader Selector Card -->
      <div class="bg-black/20 p-4 rounded-2xl border border-white/5 backdrop-blur-sm flex-1 flex flex-col">
          <ModLoaderSelector
            selectedGameVersion={selectedBaseVersion()}
            onInstall={handleModLoaderInstall}
          />
      </div>

    </div>
  </div>
</div>

