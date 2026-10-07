<script lang="ts">
 import {onMount} from 'svelte';
 import {invoke} from '@tauri-apps/api/core';
 import {durable,key} from './types';
 import {feedbackText} from './feedback';
 import {mergeSnapshot} from './snapshot';
 import type {Payload,SnapshotResponse,Config,Selector,Target,Action} from './types';
 let data=$state<Payload|null>(null),page=$state('Dashboard'),error=$state(''),notice=$state(''),busy=$state(false);
 let selected=$state(0),rotation=$state(''),press=$state(''),name=$state(''),groupMembers=$state<string[]>([]),relative=$state(true),importText=$state('');
 let editingGroup=$state<string|null>(null);let hardwareAddress=$state('');let deviceFilter=$state('all');let refreshing:Promise<void>|null=null;
 const pages=['Dashboard','Profiles','Groups','Settings','Diagnostics'];
 const observed=$derived(data?.state),config=$derived(observed?.config),audio=$derived(observed?.audio);
 const profile=$derived(config?.profiles.find(p=>p.id===config.active_profile));
 const mock=$derived(config?.hardware.mode==='mock');
 const analogCount=$derived(config?.hardware.model==='pro'?9:4),buttonCount=$derived(config?.hardware.model==='pro'?5:4);
 const maximum=$derived(config?.hardware.model==='mini'||config?.hardware.model==='pro'?255:100);
 const targets=$derived(audio?.targets??[]);
 const options=$derived([
  {label:'System output',value:key({type:'default_output'})},
  {label:'System microphone',value:key({type:'default_input'})},
  {label:'Preferred output',value:key({type:'preferred_output'})},
  {label:'Preferred microphone',value:key({type:'preferred_input'})},
  ...(profile?.groups??[]).map(g=>({label:'Group · '+g.name,value:key({type:'group',id:g.id})})),
  ...targets.flatMap(t=>{const s=durable(t);return s?[{label:t.name+' · '+t.kind,value:key(s)}]:[]})
 ]);
 const theme=$derived(config?.settings.theme??'system');
 function clone():Config {if(!config)throw Error('Configuration unavailable');return structuredClone($state.snapshot(config));}
 function clearProfileDrafts(){name='';editingGroup=null;groupMembers=[];relative=true;}
 function editableProfile(c:Config){return c.profiles.find(p=>p.id===c.active_profile)!;}
 function refresh():Promise<void>{
  if(refreshing)return refreshing;
  refreshing=(async()=>{
   try{const previous=data?.state.config.active_profile;const first=!data;const response=await invoke<SnapshotResponse>('snapshot',{knownVersion:data?.version??null});data=mergeSnapshot(data,response);
    if(first||previous!==data.state.config.active_profile){clearProfileDrafts();selectKnob(0);}
   }catch(e){error=String(e);}
  })().finally(()=>{refreshing=null;});
  return refreshing;
 }
 onMount(()=>{void refresh();const timer=setInterval(()=>{if(!document.hidden&&!busy)void refresh();},400);return()=>clearInterval(timer);});
 async function command(request:unknown){
  if(busy)return false;busy=true;error='';
  try{await invoke('action',{request});if(refreshing)await refreshing;await refresh();return true;}
  catch(e){error=String(e);if(refreshing)await refreshing;await refresh();return false;}
  finally{busy=false;}
 }
 async function save(c:Config){return command({type:'save',revision:observed?.revision,config:c});}
 async function loginStartup(enabled:boolean){
  if(busy)return;busy=true;error='';
  try{await invoke('set_login_startup',{enabled});notice=enabled?'Login startup registered for this account on this machine.':'Login startup registration removed.';}
  catch(e){error=String(e);}
  finally{if(refreshing)await refreshing;await refresh();busy=false;}
 }
 function selectKnob(index:number){selected=index;const mappings=profile?.mappings??[];const a=mappings.find(m=>m.control.index===index&&m.control.kind==='analog')?.action;const b=mappings.find(m=>m.control.index===index&&m.control.kind==='button')?.action;rotation=a&&'target'in a?key(a.target):'';press=b?(b.type==='set_mute'?'action:'+key(b):b.type==='next_profile'?'next_profile':b.type==='switch_profile'?'profile:'+b.id:'target'in b?key(b.target):''):'';}
 function label(action:Action|undefined){if(!action)return 'Unassigned';if(action.type==='next_profile')return 'Next profile';if(action.type==='switch_profile')return config?.profiles.find(p=>p.id===action.id)?.name??'Missing profile';return options.find(o=>o.value===key(action.target))?.label??'Saved target · unavailable';}
 function mapping(index:number,kind:'analog'|'button'){return profile?.mappings.find(m=>m.control.index===index&&m.control.kind===kind);}
 function controlLabel(index:number){return index<buttonCount?`knob ${index+1}`:`slider ${index-buttonCount+1}`;}
 function liveFeedback(index:number,kind:'analog'|'button'){return observed?.feedback.find(f=>f.control.index===index&&f.control.kind===kind);}
 function controlKey(event:KeyboardEvent,index:number){
  if(event.altKey||event.ctrlKey||event.metaKey)return;
  const next=event.key==='ArrowRight'?(index+1)%analogCount:event.key==='ArrowLeft'?(index+analogCount-1)%analogCount:event.key==='Home'?0:event.key==='End'?analogCount-1:null;
  if(next===null)return;event.preventDefault();selectKnob(next);document.querySelector<HTMLButtonElement>(`[data-control-index="${next}"]`)?.focus();
 }
 function feedback(index:number){const m=data?.mappings.find(m=>m.control.index===index&&m.control.kind==='analog');const live=targets.filter(t=>m?.targets.includes(t.id));const levels=live.flatMap(t=>t.volume===null?[]:[Math.round(t.volume*100)]);return {volume:levels.length?`${Math.min(...levels)===Math.max(...levels)?Math.max(...levels):Math.min(...levels)+'–'+Math.max(...levels)}%`:'—',mute:live.length&&live.every(t=>t.muted===true)?'Muted':live.some(t=>t.muted===true)?'Partly muted':'',status:feedbackText(liveFeedback(index,'analog')),warning:m?.messages.join(' · ')??''};}
 async function assign(){const c=clone();const p=c.profiles.find(p=>p.id===c.active_profile)!;p.mappings=p.mappings.filter(m=>m.control.index!==selected);if(rotation)p.mappings.push({control:{device:'primary',kind:'analog',index:selected},action:{type:'volume',target:JSON.parse(rotation)}});if(press&&selected<buttonCount){const action:Action=press.startsWith('action:')?JSON.parse(press.slice(7)):press==='next_profile'?{type:'next_profile'}:press.startsWith('profile:')?{type:'switch_profile',id:press.slice(8)}:{type:'toggle_mute',target:JSON.parse(press)};p.mappings.push({control:{device:'primary',kind:'button',index:selected},action});}if(await save(c))notice='Assignment saved. Move through the current volume to pick up control.';}
 async function mode(value:Config['hardware']['mode']){const c=clone();c.hardware.mode=value;if(value==='auto_hid'&&c.hardware.model==='original')c.hardware.model='mini';await save(c);}
 async function mockButton(index:number){if(busy)return;await command({type:'mock_button',index,pressed:false});await command({type:'mock_button',index,pressed:true});await command({type:'mock_button',index,pressed:false});}
 async function volume(t:Target,value:number){await command({type:'volume',selection:{generation:audio?.generation,id:t.id},value});}
 async function mute(t:Target){await command({type:'mute',selection:{generation:audio?.generation,id:t.id},muted:!t.muted});}
 async function newProfile(duplicate=false){
  if(!name.trim())return;const c=clone();const id=crypto.randomUUID();
  const contents=duplicate?structuredClone(editableProfile(c)):{mappings:[],groups:[],preferences:{output:null,input:null}};
  c.profiles.push({...contents,id,name:name.trim()});c.active_profile=id;
  if(await save(c)){clearProfileDrafts();selectKnob(0);}
 }
 async function renameProfile(){if(!name.trim())return;const c=clone();c.profiles.find(p=>p.id===c.active_profile)!.name=name.trim();if(await save(c))name='';}
 async function deleteProfile(id:string){const c=clone();if(c.profiles.length<=1)return;if(c.profiles.some(p=>p.mappings.some(m=>m.action.type==='switch_profile'&&m.action.id===id))){error='Remove buttons that switch to this profile before deleting it.';return;}c.profiles=c.profiles.filter(p=>p.id!==id);if(c.active_profile===id)c.active_profile=c.profiles[0].id;await save(c);}
 async function addGroup(){
  if(!name.trim()||!groupMembers.length)return;const c=clone();const p=editableProfile(c);
  const group={id:editingGroup??crypto.randomUUID(),name:name.trim(),members:groupMembers.map(v=>JSON.parse(v) as Selector),relative};
  if(editingGroup)p.groups=p.groups.map(g=>g.id===editingGroup?group:g);else p.groups.push(group);
  if(await save(c))clearProfileDrafts();
 }
 async function deleteGroup(id:string){
  const c=clone();const p=editableProfile(c);
  if(p.mappings.some(m=>'target'in m.action&&m.action.target.type==='group'&&m.action.target.id===id)){error='Remove this group from this profile’s knob/button assignments before deleting it.';return;}
  p.groups=p.groups.filter(g=>g.id!==id);if(await save(c)){if(editingGroup===id)clearProfileDrafts();}
 }
 async function preference(kind:'output'|'input',value:string){const c=clone();editableProfile(c).preferences[kind]=value?JSON.parse(value):null;await save(c);}
 function exportConfig(){if(!config)return;const url=URL.createObjectURL(new Blob([key($state.snapshot(config),null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download='veekpanel-config.json';a.click();URL.revokeObjectURL(url);notice='Configuration exported. It can include local application and hardware paths.';}
 async function report(){const value={app:'VeekPanel development',hardwareMode:config?.hardware.mode,model:config?.hardware.model,backend:audio?.backend,audioConnected:observed?.audio_status==='Connected',targetCounts:Object.fromEntries(['output','input','playback','recording'].map(k=>[k,targets.filter(t=>t.kind===k).length])),profileCount:config?.profiles.length,mappingCount:profile?.mappings.length,diagnosticCount:observed?.diagnostics.length};try{await navigator.clipboard.writeText(key(value,null,2));notice='Redacted report copied. Names, paths, serials and raw errors omitted.';}catch{error='Clipboard unavailable. The local diagnostics below remain available.';}}
</script>

<div class="shell" data-theme={theme}>
 <aside>
  <div class="brand"><span class="brand-mark" aria-hidden="true">◉</span><div>VeekPanel</div></div>
  <nav aria-label="Main navigation">{#each pages as item,i}<button class:active={page===item} onclick={()=>{page=item;clearProfileDrafts();}}><span aria-hidden="true">{['◈','▤','◫','⚙','⌁'][i]}</span>{item}</button>{/each}</nav>
  <div class="sidebar-bottom"><span class="status-dot" class:online={observed?.audio_status==='Connected'}></span>{audio?.backend??'Connecting audio'}<small>Development preview · 0.1</small></div>
 </aside>
 <main>
  <header><div><p class="eyebrow">A LITTLE MORE CONTROL</p><h1>{page}</h1></div>{#if config}<label class="profile-picker">Active profile<select aria-label="Active profile" disabled={busy} value={config.active_profile} onchange={async e=>{await command({type:'activate',id:e.currentTarget.value});selectKnob(0);}}>{#each config.profiles as p}<option value={p.id}>{p.name}</option>{/each}</select></label>{/if}</header>
  {#if error}<div class="banner error" role="alert">{error}<button onclick={()=>error=''} aria-label="Dismiss error">×</button></div>{/if}
  {#if notice}<div class="banner" role="status">{notice}<button onclick={()=>notice=''} aria-label="Dismiss message">×</button></div>{/if}
  {#if data?.desktop.tray_error}<div class="banner error" role="status">{data.desktop.tray_error}</div>{/if}
  {#if !observed}<section class="card empty"><h2>Connecting to VeekPanel</h2><p>The desktop backend supplies your audio devices and saved configuration.</p><button onclick={refresh}>Try again</button></section>
  {:else if page==='Dashboard'}
   <section class="card panel-card">
    <div class="section-heading"><div><p class="eyebrow">{mock?'DEVELOPMENT PANEL':'YOUR HARDWARE'}</p><h2>PCPanel {config?.hardware.model==='mini'?'Mini':config?.hardware.model==='pro'?'Pro':config?.hardware.model==='rgb'?'RGB':'Original'}</h2></div><span class="pill" class:mock>{observed.hardware_status}</span></div>
    {#if mock}<p class="helper">Simulated knobs and buttons · <strong>real system audio</strong>. Move a control through the current volume to pick it up.</p>{:else if !observed.hardware_status.startsWith('Connected:')}<div class="connect"><p>Connect your PCPanel to bring your controls to life.</p><button class="primary" disabled={busy} onclick={()=>mode('auto_hid')}>Detect my {config?.hardware.model==='original'?'Mini':config?.hardware.model==='pro'?'Pro':config?.hardware.model==='rgb'?'RGB':'Mini'}</button><button disabled={busy} onclick={()=>mode('mock')}>Use development panel</button></div>{/if}
    <div class="knobs" class:pro={analogCount===9}>{#each Array(analogCount) as _,i}
     {@const raw=observed.controls['analog_'+i]}
     {@const info=feedback(i)}
     <div class="knob-card" class:selected={selected===i}>
      <button class="knob-select" data-control-index={i} aria-label={`Configure ${controlLabel(i)}`} aria-pressed={selected===i} aria-controls="assignment" aria-describedby={`control-status-${i}`} onkeydown={e=>controlKey(e,i)} onclick={()=>selectKnob(i)}><div class="dial" style={`--turn:${(raw??0)/maximum*270-135}deg;--amount:${(raw??0)/maximum*100}%`}><span></span></div><strong>{i<buttonCount?'Knob':'Slider'} {i<buttonCount?i+1:i-buttonCount+1}</strong><small>{label(mapping(i,'analog')?.action)}</small></button>
      <div class="readout">{info.volume}<small>Target volume{info.mute?' · '+info.mute:''}</small></div>
      <div class="control-status" id={`control-status-${i}`} data-phase={liveFeedback(i,'analog')?.phase??'unassigned'}>{info.status}<small>{raw===undefined?'No hardware input':Math.round(raw/maximum*100)+'% knob position'}</small>{#if info.warning}<small class="mapping-warning">{info.warning}</small>{/if}</div>
      {#if mock}<input aria-label={`Simulated ${controlLabel(i)}`} aria-describedby={`control-status-${i}`} aria-valuetext={`${Math.round((raw??0)/maximum*100)}% position`} type="range" min="0" max={maximum} value={raw??0} disabled={busy} onchange={e=>command({type:'mock_analog',index:i,raw:Number(e.currentTarget.value)})}/>{/if}
      {#if i<buttonCount}<button class="press" aria-label={`Simulate press of knob ${i+1}: ${label(mapping(i,'button')?.action)}`} aria-describedby={`button-status-${i}`} class:pressed={observed.controls['button_'+i]===1} disabled={!mock||busy} onclick={()=>mockButton(i)}>{mock?'Press · ':''}{label(mapping(i,'button')?.action)}</button><small class="button-status" id={`button-status-${i}`}>{feedbackText(liveFeedback(i,'button'))}</small>{/if}
     </div>
    {/each}</div>
    <form class="assignment" id="assignment" aria-labelledby="assignment-title" onsubmit={e=>{e.preventDefault();void assign();}}><div><h3 id="assignment-title">Configure {controlLabel(selected)}</h3><p id="assignment-help">Rotation and press work independently. Use left/right arrows, Home or End to select a control.</p></div><label>Turn controls<select aria-label="Rotation target" aria-describedby="assignment-help" disabled={busy} bind:value={rotation}><option value="">Unassigned</option>{#if rotation&&!options.some(o=>o.value===rotation)}<option value={rotation}>Saved target · unavailable</option>{/if}{#each options as o}<option value={o.value}>{o.label}</option>{/each}</select></label>{#if selected<buttonCount}<label>Press action<select aria-label="Button action" disabled={busy} bind:value={press}><option value="">Unassigned</option><option value="next_profile">Next profile</option>{#each config?.profiles??[] as p}<option value={'profile:'+p.id}>Profile · {p.name}</option>{/each}{#if press&&!press.startsWith('profile:')&&press!=='next_profile'&&!options.some(o=>o.value===press)}<option value={press}>{press.startsWith('action:')?'Keep saved mute action':'Saved target · unavailable'}</option>{/if}{#each options as o}<option value={o.value}>Mute · {o.label}</option>{/each}</select></label>{/if}<button class="primary" type="submit" disabled={busy}>Save assignment</button></form>
    <p class="selected-feedback" role="status" aria-live="polite" aria-atomic="true">{controlLabel(selected)}: {feedbackText(liveFeedback(selected,'analog'))}{selected<buttonCount?' · Button: '+feedbackText(liveFeedback(selected,'button')):''}</p>
    {#each data?.mappings.filter(m=>m.control.index===selected)??[] as m}{#each m.messages as message}<p class="mapping-note">{message}</p>{/each}{/each}
   </section>
   <section class="card"><div class="section-heading"><div><p class="eyebrow">LIVE FROM YOUR SYSTEM</p><h2>Audio mixer</h2></div><select aria-label="Filter audio targets" bind:value={deviceFilter}><option value="all">All audio</option><option value="output">Outputs</option><option value="input">Microphones</option><option value="playback">Applications</option><option value="recording">Recording apps</option></select></div>
   {#if observed.audio_status!=='Connected'}<p class="helper">{observed.audio_status}</p>{/if}
   <div class="audio-list">{#each targets.filter(t=>deviceFilter==='all'||t.kind===deviceFilter) as t (t.id)}<div class="audio-row"><div class="audio-icon" aria-hidden="true">{t.kind==='input'||t.kind==='recording'?'◉':'♫'}</div><div class="audio-name"><strong>{t.name}</strong><small>{t.kind}{t.default?' · System default':''}</small></div><input aria-label={`${t.name} volume`} type="range" min="0" max="100" value={Math.min(100,(t.volume??0)*100)} disabled={busy||t.volume===null} onchange={e=>volume(t,Number(e.currentTarget.value)/100)}/><span class="volume">{t.volume===null?'—':Math.round(t.volume*100)+'%'}</span><button class:muted={t.muted} disabled={busy||t.muted===null} onclick={()=>mute(t)} aria-label={`${t.muted?'Unmute':'Mute'} ${t.name}`}>{t.muted?'Unmute':'Mute'}</button></div>{:else}<p class="empty">No matching audio targets. Applications appear when they create an audio stream.</p>{/each}</div></section>
  {:else if page==='Profiles'}
   <section class="card"><h2>A setup for every moment</h2><p class="helper">Profiles save knob and button assignments, audio groups, and preferred devices. Switching rearms controls to avoid sudden volume changes.</p><div class="profile-grid">{#each config?.profiles??[] as p}<article class="profile-tile" class:chosen={p.id===config?.active_profile}><span class="tile-icon">◈</span><h3>{p.name}</h3><p>{p.mappings.length} assignments · {p.groups.length} groups</p><button class="primary" disabled={busy||p.id===config?.active_profile} onclick={()=>command({type:'activate',id:p.id})}>{p.id===config?.active_profile?'Active':'Use profile'}</button><button disabled={busy||(config?.profiles.length??0)<=1} onclick={()=>deleteProfile(p.id)}>Delete</button></article>{/each}</div><div class="form-row"><label>Profile name<input bind:value={name} placeholder="Gaming, work, or your own" maxlength="160"/></label><button disabled={busy||!name.trim()} onclick={()=>newProfile()}>Create</button><button disabled={busy||!name.trim()} onclick={()=>newProfile(true)}>Duplicate active</button><button disabled={busy||!name.trim()} onclick={renameProfile}>Rename active</button></div></section>
  {:else if page==='Groups'}
   <section class="card"><h2>Bring your audio together</h2><p class="helper">Groups in {profile?.name} control several applications or devices together. Unavailable members wait for their next appearance.</p>{#each profile?.groups??[] as g}<div class="group-row"><div><h3>{g.name}</h3><p>{g.members.length} targets · {g.relative?'Preserve relative volume':'Set equal volume'}</p></div><label>Relative balance<input type="checkbox" checked={g.relative} disabled={busy} onchange={async e=>{const c=clone();editableProfile(c).groups.find(x=>x.id===g.id)!.relative=e.currentTarget.checked;await save(c);}}/></label><button disabled={busy} onclick={()=>{editingGroup=g.id;name=g.name;groupMembers=g.members.map(m=>key(m));relative=g.relative;}}>Edit</button><button disabled={busy} onclick={()=>deleteGroup(g.id)}>Delete</button></div>{/each}<div class="group-form"><label>Group name<input bind:value={name} placeholder="Game + chat" maxlength="160"/></label><label>Audio members<select multiple bind:value={groupMembers} size="6">{#each groupMembers.filter(v=>!options.some(o=>o.value===v)) as v}<option value={v}>Saved member · unavailable</option>{/each}{#each options.filter(o=>!o.label.startsWith('Group ·')) as o}<option value={o.value}>{o.label}</option>{/each}</select></label><label class="check"><input type="checkbox" bind:checked={relative}/>Keep relative volume between members</label><button class="primary" disabled={busy||!name.trim()||!groupMembers.length} onclick={addGroup}>{editingGroup?'Save group':'Create group'}</button>{#if editingGroup}<button onclick={()=>{editingGroup=null;name='';groupMembers=[];}}>Cancel edit</button>{/if}</div></section>
  {:else if page==='Settings'&&config}
   <section class="card settings"><h2>Startup and background</h2>
    <label class="check"><input data-setting="login-startup" type="checkbox" checked={data?.desktop.startup_registered===true} disabled={busy||!data?.desktop.startup_supported} aria-describedby="startup-help" onchange={e=>{const enabled=e.currentTarget.checked;e.currentTarget.checked=data?.desktop.startup_registered===true;void loginStartup(enabled);}}/>Register launch at login</label>
    <p id="startup-help" class="helper">{data?.desktop.startup_registered===null?'Registration state unavailable.':'Registration read from this machine.'} This applies only to your account here, not exported profiles. Your OS startup manager or policy can override it.</p>
    {#if data?.desktop.startup_error}<p class="mapping-note" role="status">{data.desktop.startup_error}</p><button disabled={busy||!data.desktop.startup_supported} onclick={()=>loginStartup(false)}>Remove VeekPanel login entry</button>{/if}
    <label class="check"><input data-setting="start-minimized" type="checkbox" checked={config.settings.start_minimized} disabled={busy} onchange={async e=>{const c=clone();c.settings.start_minimized=e.currentTarget.checked;e.currentTarget.checked=config.settings.start_minimized;await save(c);}}/>Start in the tray on the next launch</label>
    <p class="helper">The window stays visible if initialization or tray creation fails. Launch VeekPanel again to reopen it; use the tray’s Quit to stop background control.</p>
   </section>
   <section class="card settings"><h2>Make it yours</h2><label>Appearance<select value={theme} disabled={busy} onchange={async e=>{const c=clone();c.settings.theme=e.currentTarget.value as Config['settings']['theme'];await save(c);}}><option value="system">Follow system</option><option value="light">Light</option><option value="dark">Dark</option></select></label><label class="check"><input type="checkbox" checked={config.settings.close_to_tray} disabled={busy} onchange={async e=>{const c=clone();c.settings.close_to_tray=e.currentTarget.checked;await save(c);}}/>Keep running in the tray when the window closes</label><hr/><h3>Hardware</h3><label>Panel model<select value={config.hardware.model} disabled={busy} onchange={async e=>{const c=clone();c.hardware.model=e.currentTarget.value as Config['hardware']['model'];c.hardware.mode='disabled';await save(c);selectKnob(0);}}>{#each ['mini','pro','rgb','original'] as m}<option value={m}>{m}</option>{/each}</select></label><label>Connection mode<select value={config.hardware.mode} disabled={busy} onchange={e=>mode(e.currentTarget.value as Config['hardware']['mode'])}>{#if config.hardware.mode==='hid'||config.hardware.mode==='serial'}<option value={config.hardware.mode}>Explicit {config.hardware.mode} address</option>{/if}<option value="disabled">Disabled</option><option value="mock">Development panel · real audio</option>{#if config.hardware.model!=='original'}<option value="auto_hid">Automatic matching HID</option>{/if}</select></label><details><summary>Explicit connection address</summary><p>Use only the panel's known HID path or Original serial port. Automatic HID is recommended for Mini.</p><input aria-label="Explicit hardware address" bind:value={hardwareAddress} placeholder={config.hardware.address||'Exact HID path / serial port'}/><button disabled={busy||!hardwareAddress.trim()} onclick={async()=>{const c=clone();c.hardware.address=hardwareAddress;c.hardware.mode=c.hardware.model==='original'?'serial':'hid';await save(c);}}>Connect to this address</button></details><hr/><h3>Preferred devices · {profile?.name}</h3><p>Preferred output and Preferred microphone assignments use these devices for this profile.</p>{#each ['output','input'] as k}{@const kind=k as 'output'|'input'}<label>{kind==='output'?'Preferred output':'Preferred microphone'}<select value={profile?.preferences[kind]?key(profile?.preferences[kind]):''} disabled={busy} onchange={e=>preference(kind,e.currentTarget.value)}><option value="">Follow system default</option>{#if profile?.preferences[kind]&&!targets.some(t=>key(durable(t))===key(profile?.preferences[kind]))}<option value={key(profile?.preferences[kind])}>Saved device · unavailable</option>{/if}{#each targets.filter(t=>t.kind===kind&&durable(t)) as t}<option value={key(durable(t))}>{t.name}</option>{/each}</select></label>{/each}<hr/><h3>Configuration transfer</h3><p>Import validates the entire configuration and backs up the previous version. Review hardware addresses and mappings before importing; imported settings become active.</p><button onclick={exportConfig}>Export configuration</button><label>Configuration JSON<textarea bind:value={importText} rows="5" placeholder="Paste a VeekPanel configuration" maxlength="1048576"></textarea></label><button disabled={busy||!importText.trim()} onclick={async()=>{if(await command({type:'import',revision:observed?.revision,json:importText})){clearProfileDrafts();selectKnob(0);importText='';notice='Configuration imported and previous version backed up.';}}}>Import and apply</button></section>
  {:else if page==='Diagnostics'}
   <section class="card"><div class="section-heading"><div><p class="eyebrow">OBSERVED, NOT ASSUMED</p><h2>Connection details</h2></div><button onclick={report}>Copy redacted report</button></div><dl><dt>Hardware</dt><dd>{observed.hardware_status}</dd><dt>Audio</dt><dd>{observed.audio_status}</dd><dt>Backend</dt><dd>{audio?.backend??'Unavailable'}</dd><dt>Configuration</dt><dd>Schema {config?.schema_version} · revision {observed.revision}</dd></dl><p class="helper">Mini basic controls verified on one Windows 11 unit. Sleep/resume and Nobara physical validation remain pending.</p><h3>Recent diagnostics · local only</h3><pre>{observed.diagnostics.join('\n')||'No diagnostic messages.'}</pre><h3>Audio identities · local only</h3><p>These details can include application paths. They are omitted from the copied report.</p>{#each targets as t}<details><summary>{t.name} · {t.kind}</summary><pre>{key(t.identity,null,2)}</pre></details>{/each}</section>
  {/if}
  <footer>VeekPanel · Native audio, personal control <span>Development preview</span></footer>
 </main>
</div>
