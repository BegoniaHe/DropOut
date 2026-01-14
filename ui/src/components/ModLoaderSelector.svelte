<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    FabricGameVersion,
    FabricLoaderVersion,
    ForgeVersion,
    ModLoaderType,
  } from "../types";

  interface Props {
    selectedGameVersion: string;
    onInstall: (versionId: string) => void;
  }

  let { selectedGameVersion, onInstall }: Props = $props();

  // State
  let selectedLoader = $state<ModLoaderType>("vanilla");
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // Fabric state
  let fabricLoaders = $state<FabricLoaderVersion[]>([]);
  let selectedFabricLoader = $state("");

  // Forge state
  let forgeVersions = $state<ForgeVersion[]>([]);
  let selectedForgeVersion = $state("");

  // Load mod loader versions when game version changes
  $effect(() => {
    if (selectedGameVersion && selectedLoader !== "vanilla") {
      loadModLoaderVersions();
    }
  });

  async function loadModLoaderVersions() {
    isLoading = true;
    error = null;

    try {
      if (selectedLoader === "fabric") {
        const loaders = await invoke<any[]>("get_fabric_loaders_for_version", {
          gameVersion: selectedGameVersion,
        });
        fabricLoaders = loaders.map((l) => l.loader);
        if (fabricLoaders.length > 0) {
          // Select first stable version or first available
          const stable = fabricLoaders.find((l) => l.stable);
          selectedFabricLoader = stable?.version || fabricLoaders[0].version;
        }
      } else if (selectedLoader === "forge") {
        forgeVersions = await invoke<ForgeVersion[]>(
          "get_forge_versions_for_game",
          {
            gameVersion: selectedGameVersion,
          }
        );
        if (forgeVersions.length > 0) {
          // Select recommended version first, then latest
          const recommended = forgeVersions.find((v) => v.recommended);
          const latest = forgeVersions.find((v) => v.latest);
          selectedForgeVersion =
            recommended?.version || latest?.version || forgeVersions[0].version;
        }
      }
    } catch (e) {
      error = `Failed to load ${selectedLoader} versions: ${e}`;
      console.error(e);
    } finally {
      isLoading = false;
    }
  }

  async function installModLoader() {
    if (!selectedGameVersion) {
      error = "Please select a Minecraft version first";
      return;
    }

    isLoading = true;
    error = null;

    try {
      if (selectedLoader === "fabric" && selectedFabricLoader) {
        const result = await invoke<any>("install_fabric", {
          gameVersion: selectedGameVersion,
          loaderVersion: selectedFabricLoader,
        });
        onInstall(result.id);
      } else if (selectedLoader === "forge" && selectedForgeVersion) {
        const result = await invoke<any>("install_forge", {
          gameVersion: selectedGameVersion,
          forgeVersion: selectedForgeVersion,
        });
        onInstall(result.id);
      }
    } catch (e) {
      error = `Failed to install ${selectedLoader}: ${e}`;
      console.error(e);
    } finally {
      isLoading = false;
    }
  }

  function onLoaderChange(loader: ModLoaderType) {
    selectedLoader = loader;
    error = null;
    if (loader !== "vanilla" && selectedGameVersion) {
      loadModLoaderVersions();
    }
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
      <h3 class="text-xs font-bold uppercase tracking-widest text-white/40">Select Mod Loader</h3>
  </div>

  <!-- Loader Type Tabs - Segmented Control -->
  <div class="flex p-1 bg-black/40 rounded-xl border border-white/5 backdrop-blur-sm">
    <button
      class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200
      {selectedLoader === 'vanilla'
        ? 'bg-white/10 text-white shadow-lg border border-white/10'
        : 'text-zinc-500 hover:text-white hover:bg-white/5'}"
      onclick={() => onLoaderChange("vanilla")}
    >
      Vanilla
    </button>
    <button
      class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200
      {selectedLoader === 'fabric'
        ? 'bg-indigo-500/20 text-indigo-300 shadow-lg border border-indigo-500/20'
        : 'text-zinc-500 hover:text-white hover:bg-white/5'}"
      onclick={() => onLoaderChange("fabric")}
    >
      Fabric
    </button>
    <button
      class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200
      {selectedLoader === 'forge'
        ? 'bg-orange-500/20 text-orange-300 shadow-lg border border-orange-500/20'
        : 'text-zinc-500 hover:text-white hover:bg-white/5'}"
      onclick={() => onLoaderChange("forge")}
    >
      Forge
    </button>
  </div>

  <!-- Content Area -->
  <div class="min-h-[100px] flex flex-col justify-center">
    {#if selectedLoader === "vanilla"}
        <div class="text-center p-4 rounded-xl bg-white/5 border border-dashed border-white/10 text-white/40 text-sm">
           No mod loader selected. <br> Pure vanilla experience.
        </div>
        
    {:else if !selectedGameVersion}
        <div class="text-center p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-red-300 text-sm">
           ⚠️ Please select a base Minecraft version first.
        </div>
        
    {:else if isLoading}
        <div class="flex flex-col items-center gap-2 text-sm text-white/50 py-4">
            <div class="w-6 h-6 border-2 border-white/20 border-t-white rounded-full animate-spin"></div>
            Loading {selectedLoader} versions...
        </div>
        
    {:else if error}
        <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-300 text-sm break-words">
            {error}
        </div>
        
    {:else if selectedLoader === "fabric"}
        <div class="space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div>
            <label for="fabric-loader-select" class="block text-xs text-white/40 mb-2 pl-1"
            >Loader Version</label
            >
            <div class="relative">
                <select
                id="fabric-loader-select"
                class="w-full appearance-none bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-indigo-500/50 text-white transition-colors"
                bind:value={selectedFabricLoader}
                >
                {#each fabricLoaders as loader}
                    <option value={loader.version}>
                    {loader.version} {loader.stable ? "(stable)" : ""}
                    </option>
                {/each}
                </select>
                <div class="absolute right-4 top-1/2 -translate-y-1/2 text-white/20 pointer-events-none">▼</div>
            </div>
        </div>
        
        <button
            class="w-full bg-indigo-600 hover:bg-indigo-500 text-white py-3 px-4 rounded-xl font-bold text-sm transition-all shadow-lg shadow-indigo-500/20 disabled:opacity-50 disabled:shadow-none hover:scale-[1.02] active:scale-[0.98]"
            onclick={installModLoader}
            disabled={isLoading || !selectedFabricLoader}
        >
            Install Fabric {selectedFabricLoader}
        </button>
        </div>
        
    {:else if selectedLoader === "forge"}
        <div class="space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
        {#if forgeVersions.length === 0}
            <div class="text-center p-4 text-sm text-white/40 italic">
            No Forge versions available for {selectedGameVersion}
            </div>
        {:else}
            <div>
            <label for="forge-version-select" class="block text-xs text-white/40 mb-2 pl-1"
                >Forge Version</label
            >
            <div class="relative">
                <select
                    id="forge-version-select"
                    class="w-full appearance-none bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-orange-500/50 text-white transition-colors"
                    bind:value={selectedForgeVersion}
                >
                    {#each forgeVersions as version}
                    <option value={version.version}>
                        {version.version}
                        {version.recommended ? "⭐ recommended" : ""}
                        {version.latest ? "(latest)" : ""}
                    </option>
                    {/each}
                </select>
                <div class="absolute right-4 top-1/2 -translate-y-1/2 text-white/20 pointer-events-none">▼</div>
            </div>
            </div>
            
            <button
            class="w-full bg-orange-600 hover:bg-orange-500 text-white py-3 px-4 rounded-xl font-bold text-sm transition-all shadow-lg shadow-orange-500/20 disabled:opacity-50 disabled:shadow-none hover:scale-[1.02] active:scale-[0.98]"
            onclick={installModLoader}
            disabled={isLoading || !selectedForgeVersion}
            >
            Install Forge {selectedForgeVersion}
            </button>
        {/if}
        </div>
    {/if}
  </div>
</div>
