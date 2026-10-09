import type {Config,Selector} from './types';

/** Replace only one primary analog mapping; leave buttons and other profiles intact. */
export function assignRotation(config:Config,index:number,target:Selector):Config {
 const count=config.hardware.model==='pro'?9:4;
 if(!Number.isInteger(index)||index<0||index>=count)throw Error('Select an available control.');
 const next=structuredClone(config);
 const profile=next.profiles.find(p=>p.id===next.active_profile);
 if(!profile)throw Error('Active profile unavailable.');
 profile.mappings=profile.mappings.filter(m=>!(m.control.device==='primary'&&m.control.kind==='analog'&&m.control.index===index));
 profile.mappings.push({control:{device:'primary',kind:'analog',index},action:{type:'volume',target:structuredClone(target)}});
 return next;
}
