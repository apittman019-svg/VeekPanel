<script lang="ts">
 type Name='dashboard'|'profiles'|'groups'|'settings'|'diagnostics'|'speaker'|'mic'|'app'|'check'|'chevron'|'close'|'panel';
 let {name,size=20}:{name:Name,size?:number}=$props();
</script>

<!-- Original VeekPanel line icons. Decorative; controls supply their own labels. -->
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
 {#if name==='dashboard'}
  <rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>
 {:else if name==='profiles'}
  <rect x="5" y="7" width="14" height="14" rx="3"/><path d="M8 3h8M7 11h10M10 15h4"/>
 {:else if name==='groups'}
  <rect x="3" y="8" width="6" height="8" rx="2"/><rect x="15" y="3" width="6" height="6" rx="2"/><rect x="15" y="15" width="6" height="6" rx="2"/><path d="M9 12h3V6h3M12 12v6h3"/>
 {:else if name==='settings'}
  <path d="M4 6h16M4 12h16M4 18h16"/><circle cx="8" cy="6" r="2" fill="var(--icon-surface)"/><circle cx="16" cy="12" r="2" fill="var(--icon-surface)"/><circle cx="10" cy="18" r="2" fill="var(--icon-surface)"/>
 {:else if name==='diagnostics'}
  <path d="M3 12h4l3-7 4 14 3-7h4"/>
 {:else if name==='speaker'}
  <path d="M4 9h4l5-4v14l-5-4H4zM17 8a6 6 0 0 1 0 8M19 5a10 10 0 0 1 0 14"/>
 {:else if name==='mic'}
  <rect x="9" y="3" width="6" height="12" rx="3"/><path d="M6 10v2a6 6 0 0 0 12 0v-2M12 18v3M9 21h6"/>
 {:else if name==='app'}
  <rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 9h18M7 6.5h.01M10 6.5h.01M10 12l5 3-5 3z"/>
 {:else if name==='check'}
  <path d="m5 12 4 4L19 6"/>
 {:else if name==='chevron'}
  <path d="m9 5 7 7-7 7"/>
 {:else if name==='close'}
  <path d="m6 6 12 12M18 6 6 18"/>
 {:else if name==='panel'}
  <rect x="3" y="6" width="18" height="12" rx="3"/><circle cx="8" cy="12" r="2"/><circle cx="16" cy="12" r="2"/>
 {/if}
</svg>
