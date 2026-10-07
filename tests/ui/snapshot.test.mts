import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mergeSnapshot} from '../../ui/src/snapshot.ts';
import type {Payload,SnapshotResponse} from '../../ui/src/types.ts';

function fixture():Payload {
 return {version:2,state:{config:{schema_version:2,active_profile:'first',profiles:[],hardware:{mode:'mock',model:'mini',address:''},settings:{theme:'system',close_to_tray:false,start_minimized:false}},revision:1,audio:null,audio_status:'Unavailable',hardware_status:'Development',controls:{},feedback:[],diagnostics:[]},mappings:[],desktop:{tray_ready:true,tray_error:null,startup_supported:true,startup_registered:false,startup_error:null}};
}
function unchanged(current:Payload):SnapshotResponse {
 return {version:current.version,state:null,mappings:null,desktop:{...current.desktop}};
}
test('idle replies retain the current object and edited state references',()=>{
 const current=fixture();
 assert.equal(mergeSnapshot(current,unchanged(current)),current);
});
test('desktop status changes remain visible without replacing runtime data',()=>{
 const current=fixture();const response=unchanged(current);
 response.desktop.startup_registered=true;
 response.desktop.tray_ready=false;
 response.desktop.tray_error='Tray unavailable';
 const merged=mergeSnapshot(current,response);
 assert.notEqual(merged,current);
 assert.equal(merged.state,current.state);
 assert.equal(merged.mappings,current.mappings);
 assert.equal(merged.desktop.startup_registered,true);
 assert.equal(merged.desktop.tray_error,'Tray unavailable');
 assert.equal(current.desktop.startup_registered,false);
});
test('fresh publications replace runtime and mapping data, including profile switches',()=>{
 const current=fixture();const next=fixture();
 next.version=3;next.state.config.active_profile='second';
 next.mappings=[{control:{device:'primary',kind:'analog',index:0},targets:['output'],messages:[]}];
 assert.deepEqual(mergeSnapshot(current,next),next);
 assert.deepEqual(mergeSnapshot(null,next),next);
});
test('missing or mismatched cached versions fail visibly',()=>{
 const current=fixture();
 assert.throws(()=>mergeSnapshot(null,unchanged(current)),/version/);
 assert.throws(()=>mergeSnapshot(current,{...unchanged(current),version:3}),/version/);
 assert.throws(()=>mergeSnapshot(current,{...current,mappings:null}),/mapping/);
});
