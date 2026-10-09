import {test} from 'node:test';
import assert from 'node:assert/strict';
import {assignRotation} from '../../ui/src/assignments.ts';
import type {Config,Profile} from '../../ui/src/types.ts';
function fixture():Config {
 const p:Profile={id:'first',name:'First',groups:[],preferences:{input:null,output:null},mappings:[
  {control:{device:'primary',kind:'analog',index:0},action:{type:'volume',target:{type:'default_input'}}},
  {control:{device:'primary',kind:'button',index:0},action:{type:'next_profile'}},
  {control:{device:'primary',kind:'analog',index:1},action:{type:'volume',target:{type:'default_input'}}},
 ]};
 return {schema_version:2,active_profile:'first',profiles:[p,{...structuredClone(p),id:'second'}],hardware:{mode:'disabled',model:'mini',address:''},settings:{theme:'system',close_to_tray:false,start_minimized:false}};
}
test('quick assignment changes only the selected analog control and never mutates the source',()=>{
 const original=fixture(),before=structuredClone(original);
 const updated=assignRotation(original,0,{type:'default_output'});
 assert.deepEqual(original,before);
 assert.deepEqual(updated.profiles[1],before.profiles[1]);
 assert.deepEqual(updated.profiles[0].mappings.slice(0,2),before.profiles[0].mappings.slice(1));
 assert.deepEqual(updated.profiles[0].mappings[2].action,{type:'volume',target:{type:'default_output'}});
});
test('invalid control indexes are rejected and Pro sliders are accepted',()=>{
 for(const index of [-1,4,1.5,NaN])assert.throws(()=>assignRotation(fixture(),index,{type:'default_output'}));
 const pro=fixture();pro.hardware.model='pro';
 assert.equal(assignRotation(pro,8,{type:'default_output'}).profiles[0].mappings.at(-1)?.control.index,8);
});
