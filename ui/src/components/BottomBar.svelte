<script lang="ts">
  import { authState } from "../stores/auth.svelte";
  import { gameState } from "../stores/game.svelte";
  import { uiState } from "../stores/ui.svelte";
</script>

<div
  class="h-24 bg-gradient-to-t from-black/50 to-transparent border-t border-white/5 flex items-center px-8 justify-between z-20 backdrop-blur-md"
>
  <!-- Account Area -->
  <div class="flex items-center gap-6">
    <div
      class="group flex items-center gap-4 cursor-pointer transition-all duration-300 hover:scale-105"
      onclick={() => authState.openLoginModal()}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === "Enter" && authState.openLoginModal()}
    >
      <div
        class="w-12 h-12 rounded-xl bg-gradient-to-tr from-indigo-500 to-purple-500 shadow-lg shadow-indigo-500/20 flex items-center justify-center text-white font-bold text-xl overflow-hidden ring-2 ring-transparent group-hover:ring-white/20 transition-all"
      >
        {#if authState.currentAccount}
          <img
            src={`https://minotar.net/avatar/${authState.currentAccount.username}/48`}
            alt={authState.currentAccount.username}
            class="w-full h-full"
          />
        {:else}
          <span class="text-white/50 text-2xl">?</span>
        {/if}
      </div>
      <div>
        <div class="font-bold text-white text-lg group-hover:text-indigo-300 transition-colors">
          {authState.currentAccount ? authState.currentAccount.username : "Login"}
        </div>
        <div class="text-xs text-zinc-400 flex items-center gap-1.5">
          <span
            class="w-2 h-2 rounded-full {authState.currentAccount
              ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]'
              : 'bg-zinc-600'}"
          ></span>
          {authState.currentAccount ? "Ready to play" : "Guest Mode"}
        </div>
      </div>
    </div>
    
    <div class="h-8 w-px bg-white/10"></div>
    
    <!-- Console Toggle -->
    <button
      class="text-xs font-mono text-zinc-500 hover:text-white transition-colors flex items-center gap-2"
      onclick={() => uiState.toggleConsole()}
    >
      <span class="text-lg">_</span>
      {uiState.showConsole ? "Hide Logs" : "Show Logs"}
    </button>
  </div>

  <!-- Action Area -->
  <div class="flex items-center gap-6">
    <div class="flex flex-col items-end mr-2">
      <label
        for="version-select"
        class="text-[10px] text-white/40 mb-1 uppercase font-bold tracking-wider"
        >Selected Version</label
      >
      <div class="relative group">
        <select
          id="version-select"
          bind:value={gameState.selectedVersion}
          class="appearance-none bg-black/40 text-white border border-white/10 rounded-xl pl-4 pr-10 py-2.5 hover:border-white/30 transition-all cursor-pointer outline-none focus:ring-2 focus:ring-indigo-500/50 w-56 text-sm font-mono backdrop-blur-sm shadow-inner"
        >
          {#if gameState.versions.length === 0}
            <option>Loading...</option>
          {:else}
            {#each gameState.versions as version}
              <option value={version.id}>{version.id} {version.type !== 'release' ? `(${version.type})` : ''}</option>
            {/each}
          {/if}
        </select>
        <div class="absolute right-3 top-1/2 -translate-y-1/2 text-white/20 pointer-events-none group-hover:text-white/50 transition-colors">▼</div>
      </div>
    </div>

    <button
      onclick={() => gameState.startGame()}
      class="bg-gradient-to-r from-emerald-600 to-green-600 hover:from-emerald-500 hover:to-green-500 text-white font-bold h-14 px-10 rounded-xl transition-all duration-300 transform hover:scale-105 active:scale-95 shadow-[0_0_20px_rgba(16,185,129,0.3)] hover:shadow-[0_0_40px_rgba(16,185,129,0.5)] flex flex-col items-center justify-center uppercase tracking-widest text-xl relative overflow-hidden group"
    >
      <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 skew-y-12"></div>
      <span class="relative z-10 flex items-center gap-2">
        PLAY
      </span>
      <span
        class="relative z-10 text-[9px] font-normal opacity-70 normal-case tracking-wide -mt-1"
        >Launch Game</span
      >
    </button>
  </div>
</div>
