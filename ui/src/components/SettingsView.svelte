<script lang="ts">
  import { settingsState } from "../stores/settings.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";

  async function selectBackground() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Images",
            extensions: ["png", "jpg", "jpeg", "webp", "gif"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        settingsState.settings.custom_background_path = selected;
        settingsState.saveSettings();
      }
    } catch (e) {
      console.error("Failed to select background:", e);
    }
  }

  function clearBackground() {
    settingsState.settings.custom_background_path = undefined;
    settingsState.saveSettings();
  }
</script>

<div class="h-full flex flex-col p-6 overflow-hidden">
  <div class="flex items-center justify-between mb-6">
     <h2 class="text-3xl font-black bg-clip-text text-transparent bg-gradient-to-r dark:from-white dark:to-white/60 from-gray-900 to-gray-600">Settings</h2>
  </div>

  <div class="flex-1 overflow-y-auto pr-2 space-y-6 custom-scrollbar pb-10">

    <!-- Appearance / Background -->
    <div class="dark:bg-black/20 bg-white/60 p-6 rounded-2xl border dark:border-white/5 border-black/5 shadow-sm backdrop-blur-sm">
      <h3 class="text-xs font-bold uppercase tracking-widest dark:text-white/40 text-black/40 mb-6 flex items-center gap-2">
        Appearance
      </h3>
      
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium dark:text-white/70 text-black/70 mb-3">Custom Background Image</label>
          
          <div class="flex items-center gap-6">
            <!-- Preview -->
            <div class="w-40 h-24 rounded-xl overflow-hidden dark:bg-black/50 bg-gray-200 border dark:border-white/10 border-black/10 relative group shadow-lg">
              {#if settingsState.settings.custom_background_path}
                <img 
                  src={convertFileSrc(settingsState.settings.custom_background_path)} 
                  alt="Background Preview" 
                  class="w-full h-full object-cover"
                />
              {:else}
                <div class="w-full h-full bg-gradient-to-br from-emerald-900 via-zinc-900 to-indigo-950 opacity-100"></div>
                <div class="absolute inset-0 flex items-center justify-center text-[10px] text-white/50 bg-black/20 ">Default Gradient</div>
              {/if}
            </div>

            <!-- Actions -->
            <div class="flex flex-col gap-2">
              <button
                onclick={selectBackground}
                class="dark:bg-white/10 dark:hover:bg-white/20 bg-black/5 hover:bg-black/10 dark:text-white text-black px-4 py-2 rounded-lg text-sm transition-colors border dark:border-white/5 border-black/5"
              >
                Select Image
              </button>
              
              {#if settingsState.settings.custom_background_path}
                <button
                  onclick={clearBackground}
                  class="text-red-400 hover:text-red-300 text-sm px-4 py-1 text-left transition-colors"
                >
                  Reset to Default
                </button>
              {/if}
            </div>
          </div>
          <p class="text-xs dark:text-white/30 text-black/40 mt-3">
            Select an image from your computer to replace the default gradient background.
            Supported formats: PNG, JPG, WEBP, GIF.
          </p>
        </div>

        <!-- Visual Settings -->
        <div class="pt-4 border-t dark:border-white/5 border-black/5 space-y-4">
             <div class="flex items-center justify-between">
                <div>
                   <h4 class="text-sm font-medium dark:text-white/90 text-black/80" id="visual-effects-label">Visual Effects</h4>
                   <p class="text-xs dark:text-white/40 text-black/50 mt-1">Enable particle effects and animated gradients. (Default: On)</p>
                </div>
                <button 
                    aria-labelledby="visual-effects-label"
                    onclick={() => { settingsState.settings.enable_visual_effects = !settingsState.settings.enable_visual_effects; settingsState.saveSettings(); }}
                    class="w-11 h-6 rounded-full transition-colors duration-200 ease-in-out relative focus:outline-none {settingsState.settings.enable_visual_effects ? 'bg-indigo-500' : 'dark:bg-white/10 bg-black/10'}"
                >
                    <div class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full shadow-sm transition-transform duration-200 ease-in-out {settingsState.settings.enable_visual_effects ? 'translate-x-5' : 'translate-x-0'}"></div>
                </button>
             </div>

             {#if settingsState.settings.enable_visual_effects}
                 <div class="flex items-center justify-between pl-2 border-l-2 dark:border-white/5 border-black/5 ml-1">
                    <div>
                       <h4 class="text-sm font-medium dark:text-white/90 text-black/80" id="theme-effect-label">Theme Effect</h4>
                       <p class="text-xs dark:text-white/40 text-black/50 mt-1">Select the active visual theme.</p>
                    </div>
                    <select
                        aria-labelledby="theme-effect-label"
                        bind:value={settingsState.settings.active_effect}
                        onchange={() => settingsState.saveSettings()}
                        class="dark:bg-black/40 bg-white dark:text-white text-black text-xs px-3 py-2 rounded-lg border dark:border-white/10 border-black/10 outline-none focus:border-indigo-500/50 appearance-none cursor-pointer hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
                    >
                        <option value="saturn">Saturn (Saturn)</option>
                        <option value="constellation">Network (Constellation)</option>
                    </select>
                 </div>
             {/if}

             <div class="flex items-center justify-between">
                <div>
                   <h4 class="text-sm font-medium dark:text-white/90 text-black/80" id="gpu-acceleration-label">GPU Acceleration</h4>
                   <p class="text-xs dark:text-white/40 text-black/50 mt-1">Enable GPU acceleration for the interface. (Default: Off, Requires Restart)</p>
                </div>
                <button 
                    aria-labelledby="gpu-acceleration-label"
                    onclick={() => { settingsState.settings.enable_gpu_acceleration = !settingsState.settings.enable_gpu_acceleration; settingsState.saveSettings(); }}
                    class="w-11 h-6 rounded-full transition-colors duration-200 ease-in-out relative focus:outline-none {settingsState.settings.enable_gpu_acceleration ? 'bg-indigo-500' : 'dark:bg-white/10 bg-black/10'}"
                >
                    <div class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full shadow-sm transition-transform duration-200 ease-in-out {settingsState.settings.enable_gpu_acceleration ? 'translate-x-5' : 'translate-x-0'}"></div>
                </button>
             </div>
             
             <!-- Color Theme Switcher -->
             <div class="flex items-center justify-between pt-4 border-t dark:border-white/5 border-black/5 opacity-50 cursor-not-allowed">
                <div>
                   <h4 class="text-sm font-medium dark:text-white/90 text-black/80" id="color-theme-label">Color Theme</h4>
                   <p class="text-xs dark:text-white/40 text-black/50 mt-1">Interface color mode. (Locked to Dark)</p>
                </div>
                <div class="flex items-center bg-black/5 dark:bg-white/5 rounded-lg p-1 pointer-events-none">
                    <button
                        disabled
                        class="px-3 py-1 rounded-md text-xs font-medium transition-all text-gray-500 dark:text-gray-600"
                    >
                        Light
                    </button>
                    <button
                        disabled
                        class="px-3 py-1 rounded-md text-xs font-medium transition-all bg-indigo-500 text-white shadow-sm"
                    >
                        Dark
                    </button>
                </div>
             </div>
        </div>
      </div>
    </div>

    <!-- Java Path -->
    <div class="dark:bg-black/20 bg-white/60 p-6 rounded-2xl border dark:border-white/5 border-black/5 shadow-sm backdrop-blur-sm">
      <h3 class="text-xs font-bold uppercase tracking-widest text-white/40 mb-6 flex items-center gap-2">
        Java Environment
      </h3>
      <div class="space-y-4">
        <div>
             <label for="java-path" class="block text-sm font-medium text-white/70 mb-2">Java Executable Path</label>
             <div class="flex gap-2">
                <input
                id="java-path"
                bind:value={settingsState.settings.java_path}
                type="text"
                class="bg-black/40 text-white flex-1 px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none font-mono text-xs transition-colors"
                placeholder="e.g. java, /usr/bin/java"
                />
                <button
                onclick={() => settingsState.detectJava()}
                disabled={settingsState.isDetectingJava}
                class="bg-white/10 hover:bg-white/20 disabled:opacity-50 text-white px-4 py-2 rounded-xl border border-white/5 transition-colors whitespace-nowrap text-sm font-medium"
                >
                {settingsState.isDetectingJava ? "Detecting..." : "Auto Detect"}
                </button>
                <button
                onclick={() => settingsState.openJavaDownloadModal()}
                class="bg-indigo-600 hover:bg-indigo-500 text-white px-4 py-2 rounded-xl transition-colors whitespace-nowrap text-sm font-medium"
                >
                Download Java
                </button>
            </div>
        </div>
      
      {#if settingsState.javaInstallations.length > 0}
        <div class="mt-4 space-y-2">
          <p class="text-[10px] text-white/30 uppercase font-bold tracking-wider">Detected Installations</p>
          {#each settingsState.javaInstallations as java}
            <button
              onclick={() => settingsState.selectJava(java.path)}
              class="w-full text-left p-3 rounded-lg border transition-all duration-200 group
              {settingsState.settings.java_path === java.path 
                ? 'bg-indigo-500/20 border-indigo-500/30' 
                : 'bg-black/20 border-white/5 hover:bg-white/5 hover:border-white/10'}"
            >
              <div class="flex justify-between items-center">
                <div>
                  <span class="text-white font-mono text-xs font-bold">{java.version}</span>
                  <span class="text-white/40 text-[10px] ml-2">{java.is_64bit ? "64-bit" : "32-bit"}</span>
                </div>
                {#if settingsState.settings.java_path === java.path}
                  <span class="text-indigo-300 text-[10px] font-bold uppercase tracking-wider">Selected</span>
                {/if}
              </div>
              <div class="text-white/30 text-[10px] font-mono truncate mt-1 group-hover:text-white/50">{java.path}</div>
            </button>
          {/each}
        </div>
      {/if}
      </div>
    </div>

    <!-- Memory -->
    <div class="bg-black/20 p-6 rounded-2xl border border-white/5 ">
      <h3 class="text-xs font-bold uppercase tracking-widest text-white/40 mb-6 flex items-center gap-2">
        Memory Allocation (RAM)
      </h3>
      <div class="grid grid-cols-2 gap-6">
        <div>
          <label for="min-memory" class="block text-sm font-medium text-white/70 mb-2">Minimum (MB)</label>
          <input
            id="min-memory"
            bind:value={settingsState.settings.min_memory}
            type="number"
            class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none transition-colors"
          />
        </div>
        <div>
          <label for="max-memory" class="block text-sm font-medium text-white/70 mb-2">Maximum (MB)</label>
          <input
            id="max-memory"
            bind:value={settingsState.settings.max_memory}
            type="number"
            class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none transition-colors"
          />
        </div>
      </div>
    </div>

    <!-- Resolution -->
    <div class="bg-black/20 p-6 rounded-2xl border border-white/5 ">
      <h3 class="text-xs font-bold uppercase tracking-widest text-white/40 mb-6 flex items-center gap-2">
        Game Window Size
      </h3>
      <div class="grid grid-cols-2 gap-6">
        <div>
          <label for="window-width" class="block text-sm font-medium text-white/70 mb-2">Width</label>
          <input
            id="window-width"
            bind:value={settingsState.settings.width}
            type="number"
            class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none transition-colors"
          />
        </div>
        <div>
          <label for="window-height" class="block text-sm font-medium text-white/70 mb-2">Height</label>
          <input
            id="window-height"
            bind:value={settingsState.settings.height}
            type="number"
            class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none transition-colors"
          />
        </div>
      </div>
    </div>

    <!-- Download Settings -->
    <div class="bg-black/20 p-6 rounded-2xl border border-white/5 ">
        <h3 class="text-xs font-bold uppercase tracking-widest text-white/40 mb-6 flex items-center gap-2">
            Network
        </h3>
        <div>
            <label for="download-threads" class="block text-sm font-medium text-white/70 mb-2">Concurrent Download Threads</label>
            <input
              id="download-threads"
              bind:value={settingsState.settings.download_threads}
              type="number"
              min="1"
              max="128"
              class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none transition-colors"
            />
            <p class="text-xs text-white/30 mt-2">Higher values usually mean faster downloads but use more CPU/Network.</p>
        </div>
    </div>

    <div class="pt-4 flex justify-end">
      <button
        onclick={() => settingsState.saveSettings()}
        class="bg-gradient-to-r from-emerald-600 to-green-600 hover:from-emerald-500 hover:to-green-500 text-white font-bold py-3 px-8 rounded-xl shadow-lg shadow-emerald-500/20 transition-all hover:scale-105 active:scale-95"
      >
        Save Settings
      </button>
    </div>
  </div>
</div>

<!-- Java Download Modal -->
{#if settingsState.showJavaDownloadModal}
  <div class="fixed inset-0 z-[100] flex items-center justify-center backdrop-blur-sm bg-black/70">
    <div class="bg-zinc-900 rounded-2xl border border-white/10 p-8 max-w-md w-full mx-4 shadow-2xl">
      <h3 class="text-2xl font-bold mb-6 text-white">Download Java</h3>
      
      <div class="space-y-6">
        <!-- Java Version Selection -->
        <div>
          <label class="block text-sm font-medium text-white/70 mb-2">Java Version</label>
          <select
            bind:value={settingsState.selectedJavaVersion}
            class="bg-black/40 text-white w-full px-4 py-3 rounded-xl border border-white/10 focus:border-indigo-500/50 outline-none"
          >
            {#each settingsState.availableJavaVersions as version}
              <option value={version}>Java {version}</option>
            {/each}
          </select>
        </div>

        <!-- Image Type Selection -->
        <div>
          <label class="block text-sm font-medium text-white/70 mb-2">Image Type</label>
          <div class="grid grid-cols-2 gap-3">
            <button
              onclick={() => settingsState.selectedImageType = "jre"}
              class="px-4 py-3 rounded-xl border transition-all {settingsState.selectedImageType === 'jre' ? 'bg-indigo-500/20 border-indigo-500/50 text-white' : 'bg-black/20 border-white/10 text-white/60 hover:bg-white/5'}"
            >
              JRE
            </button>
            <button
              onclick={() => settingsState.selectedImageType = "jdk"}
              class="px-4 py-3 rounded-xl border transition-all {settingsState.selectedImageType === 'jdk' ? 'bg-indigo-500/20 border-indigo-500/50 text-white' : 'bg-black/20 border-white/10 text-white/60 hover:bg-white/5'}"
            >
              JDK
            </button>
          </div>
          <p class="text-xs text-white/40 mt-2">
            JRE: Runtime only (smaller). JDK: Includes development tools.
          </p>
        </div>

        <!-- Status -->
        {#if settingsState.javaDownloadStatus}
          <div class="bg-black/40 border border-white/10 rounded-xl p-4 text-sm text-white/80">
            {settingsState.javaDownloadStatus}
          </div>
        {/if}

        <!-- Actions -->
        <div class="flex gap-3 pt-2">
          <button
            onclick={() => settingsState.closeJavaDownloadModal()}
            disabled={settingsState.isDownloadingJava}
            class="flex-1 bg-white/10 hover:bg-white/20 disabled:opacity-50 text-white px-4 py-3 rounded-xl border border-white/5 transition-colors font-medium"
          >
            Cancel
          </button>
          <button
            onclick={() => settingsState.downloadJava()}
            disabled={settingsState.isDownloadingJava}
            class="flex-1 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white px-4 py-3 rounded-xl transition-colors font-medium"
          >
            {settingsState.isDownloadingJava ? "Downloading..." : "Download & Install"}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
