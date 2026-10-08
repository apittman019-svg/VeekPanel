<script lang="ts">
 let {raw,maximum,slider=false}:{raw:number|undefined,maximum:number,slider?:boolean}=$props();
 const position=$derived(Math.min(1,Math.max(0,(raw??0)/maximum)));
</script>

<!-- Shows the observed physical position, never the assigned audio level. -->
{#if slider}
 <div class="fader" class:unknown={raw===undefined} style={`--position:${position*100}%`} aria-hidden="true">
  <div class="fader-ticks"></div><div class="fader-track"><div class="fader-fill"></div><div class="fader-cap"><span></span></div></div>
 </div>
{:else}
 <div class="dial" class:unknown={raw===undefined} style={`--turn:${position*270-135}deg`} aria-hidden="true">
  <svg class="dial-ring" viewBox="0 0 100 100" fill="none" focusable="false">
   <circle class="dial-track" cx="50" cy="50" r="43" pathLength="100" stroke-dasharray="75 100" transform="rotate(135 50 50)"/>
   {#if raw!==undefined}<circle class="dial-progress" cx="50" cy="50" r="43" pathLength="100" stroke-dasharray={`${position*75} 100`} transform="rotate(135 50 50)"/>{/if}
  </svg>
  <div class="dial-face"><span class="dial-pointer"></span></div>
 </div>
{/if}
