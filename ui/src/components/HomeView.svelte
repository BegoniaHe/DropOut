<script lang="ts">
  type Props = {
    mouseX: number;
    mouseY: number;
  };
  let { mouseX = 0, mouseY = 0 }: Props = $props();
</script>

<div class="absolute inset-0 z-0 overflow-hidden">
  <!-- Parallax Background Layers -->
  
  <div class="absolute inset-0 bg-gradient-to-t from-[#09090b] via-[#09090b]/40 to-transparent"></div>
</div>

<div class="relative z-10 h-full flex flex-col justify-end p-12 pb-24">
  <!-- 3D Floating Hero Text -->
  <div 
    class="transition-transform duration-200 ease-out origin-bottom-left"
    style:transform={`perspective(1000px) rotateX(${mouseY * -2}deg) rotateY(${mouseX * 2}deg)`}
  >
    <h1
      class="text-8xl font-black tracking-tighter text-white drop-shadow-2xl mb-4"
      style="text-shadow: 0 10px 30px rgba(0,0,0,0.5);"
    >
      MINECRAFT
    </h1>
    
    <div class="flex items-center gap-4">
      <div 
        class="bg-white/10 backdrop-blur-md border border-white/10 px-4 py-1.5 rounded-full text-sm font-bold uppercase tracking-widest text-emerald-400 shadow-xl"
      >
        Java Edition
      </div>
      <div class="text-2xl font-light text-zinc-300">
        Latest Release 1.21
      </div>
    </div>
  </div>

  <!-- Action Area -->
  <div class="mt-8 flex gap-4">
    <!-- Quick Play Button (Visual only here, logic is in BottomBar usually) -->
    <div class="text-zinc-400 text-sm italic">
      Ready to play. Select version below or hit Launch.
    </div>
  </div>
</div>
