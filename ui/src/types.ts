export type Kind='output'|'input'|'playback'|'recording';
export type Selector={type:'default_output'|'default_input'|'preferred_output'|'preferred_input'}|{type:'match',kind:Kind,identities:Record<string,string>}|{type:'group',id:string};
export type Action={type:'volume'|'toggle_mute',target:Selector}|{type:'set_mute',target:Selector,muted:boolean}|{type:'switch_profile',id:string}|{type:'next_profile'};
export type Control={device:string,kind:'analog'|'button',index:number};
export type Profile={id:string,name:string,mappings:{control:Control,action:Action}[]};
export type Config={schema_version:number,active_profile:string,profiles:Profile[],groups:{id:string,name:string,members:Selector[],relative:boolean}[],hardware:{mode:'disabled'|'mock'|'serial'|'hid'|'auto_hid',model:'original'|'rgb'|'mini'|'pro',address:string},preferences:{output:Selector|null,input:Selector|null},settings:{theme:'system'|'light'|'dark',close_to_tray:boolean}};
export type Target={id:string,name:string,kind:Kind,default:boolean,volume:number|null,muted:boolean|null,identity:Record<string,string>};
export type Audio={backend:string,generation:number,targets:Target[]};
export type State={config:Config,revision:number,audio:Audio|null,audio_status:string,hardware_status:string,controls:Record<string,number>,diagnostics:string[]};
export type Payload={state:State,mappings:{control:Control,targets:string[],messages:string[]}[]};
export function durable(t:Target):Selector|null {
 const keys=t.kind==='output'||t.kind==='input'?['endpoint.id','node.name']:['application.id','application.path','application.process.binary'];
 const key=keys.find(k=>t.identity[k]);return key?{type:'match',kind:t.kind,identities:{[key]:t.identity[key]}}:null;
}
// IPC maps may arrive with alphabetic keys; selector identity must not depend on
// JavaScript insertion order (especially nested identity metadata).
export function key(value:unknown,_replacer:unknown=null,space?:number):string {
 const normalize=(v:unknown):unknown=>Array.isArray(v)?v.map(normalize):v!==null&&typeof v==='object'?Object.fromEntries(Object.entries(v).sort(([a],[b])=>a<b?-1:a>b?1:0).map(([k,x])=>[k,normalize(x)])):v;
 return JSON.stringify(normalize(value),null,space);
}
