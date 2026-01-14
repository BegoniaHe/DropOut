<script lang="ts">
  import { uiState } from '../stores/ui.svelte';
</script>

<aside
  class="w-20 lg:w-64 bg-black flex flex-col items-center lg:items-start transition-all duration-300 shrink-0 py-6"
>
  <!-- Logo Area -->
  <div
    class="h-16 w-full flex items-center justify-center lg:justify-start lg:px-8 mb-6"
  >
    <!-- Icon Logo (Small) -->
    <div
      class="lg:hidden text-3xl font-black bg-clip-text text-transparent bg-gradient-to-tr from-indigo-400 to-fuchsia-400 drop-shadow-lg"
    >
      D
    </div>
    <!-- Full Logo (Large) -->
    <div
      class="hidden lg:block font-bold text-2xl tracking-wider text-white"
    >
      <span class="bg-clip-text text-transparent bg-gradient-to-r from-indigo-400 to-fuchsia-400">DROP</span>OUT
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 w-full flex flex-col gap-3 px-3">
    <!-- Nav Item Helper -->
    {#snippet navItem(view, icon, label)}
      <button
        class="group flex items-center lg:gap-4 justify-center lg:justify-start w-full px-0 lg:px-5 py-3.5 rounded-xl transition-all duration-200 relative overflow-hidden
        {uiState.currentView === view
          ? 'bg-gradient-to-r from-indigo-500/20 to-purple-500/20 text-white shadow-lg shadow-indigo-500/10 border border-white/10'
          : 'text-zinc-400 hover:text-white hover:bg-white/5'}"
        onclick={() => uiState.setView(view)}
      >
        <span class="text-xl relative z-10 transition-transform group-hover:scale-110 duration-200">{icon}</span>
        <span class="hidden lg:block font-medium relative z-10">{label}</span>
        
        <!-- Active Indicator Line -->
        {#if uiState.currentView === view}
          <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-indigo-500 rounded-r-full lg:hidden"></div>
        {/if}
      </button>
    {/snippet}

    {@render navItem('home', '🏠', 'Home')}
    {@render navItem('versions', '📦', 'Versions')}
    {@render navItem('settings', '⚙️', 'Settings')}
  </nav>

  <!-- Footer Info -->
  <div
    class="p-4 w-full flex justify-center lg:justify-start lg:px-8 opacity-50 hover:opacity-100 transition-opacity"
  >
    <div class="text-xs font-mono tracking-widest text-zinc-500">v{uiState.appVersion}</div>
  </div>
</aside>
