<script lang="ts">
 import {onMount, tick} from 'svelte';
 import Icon from './Icon.svelte';

 // Remote player only: no emulator scripts execute in VeekPanel's IPC origin.
 const playerUrl='https://archive.org/embed/DoomsharewareEpisode';
 let playing=$state(false),online=$state(true),frame=$state<HTMLIFrameElement>();
 onMount(()=>{
  online=navigator.onLine;
  const hide=()=>{if(document.hidden)playing=false;};
  document.addEventListener('visibilitychange',hide);
  return()=>document.removeEventListener('visibilitychange',hide);
 });
 async function play(){playing=true;await tick();frame?.focus();}
</script>

<svelte:window ononline={()=>online=true} onoffline={()=>{online=false;playing=false;}}/>

<section class="card doom-card" aria-labelledby="doom-title">
 <div class="section-heading">
  <div class="doom-title"><span class="section-icon"><Icon name="gamepad" size={24}/></span><div><p class="eyebrow">COMPLETELY NECESSARY FEATURE</p><h2 id="doom-title">DOOM</h2></div></div>
  {#if playing}<button onclick={()=>playing=false}>Stop game</button>{/if}
 </div>
 {#if playing}
  <p class="helper" id="doom-help">Click inside the player to start and use the keyboard. Arrow keys move, Ctrl fires, Space opens doors, Esc opens the game menu.</p>
  <iframe bind:this={frame} src={playerUrl} title="DOOM game player" aria-describedby="doom-help" sandbox="allow-scripts allow-same-origin allow-pointer-lock" allow="fullscreen; autoplay; gamepad" allowfullscreen referrerpolicy="no-referrer"></iframe>
  <p class="helper">Player not loading? Check your connection, stop the game and try again.</p>
 {:else}
  <div class="doom-start"><Icon name="gamepad" size={56}/><h3>Your audio panel has a demon problem.</h3><p>The original Doom shareware episode, running in your very serious audio application.</p><button class="primary" onclick={play} disabled={!online}>Play Doom</button></div>
 {/if}
 {#if !online}<p role="status">You’re offline. Connect to the internet to play Doom.</p>{/if}
 <p class="helper doom-note">Online player provided by Internet Archive. Loads only when you press Play. Leaving this tab ends the game session.</p>
</section>

<style>
 .doom-title {display:flex;align-items:center;gap:13px;min-width:0;}
 .doom-title h2 {margin:0;letter-spacing:2px;}
 .doom-start {display:flex;flex-direction:column;align-items:center;text-align:center;gap:12px;padding:48px 16px;}
 .doom-start :global(svg) {color:var(--accent);}
 .doom-start h3 {font-size:22px;line-height:1.35;margin:8px 0 0;}
 .doom-start p {max-width:420px;margin:0 0 8px;}
 iframe {display:block;width:100%;height:clamp(320px,65vh,720px);border:1px solid var(--line);border-radius:14px;background:#000;}
 .doom-note {margin-bottom:0;}
</style>
