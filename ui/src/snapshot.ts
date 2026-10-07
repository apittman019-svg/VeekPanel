import type {Desktop,Payload,SnapshotResponse} from './types';

function sameDesktop(a:Desktop,b:Desktop):boolean {
 return a.tray_ready===b.tray_ready&&a.tray_error===b.tray_error&&a.startup_supported===b.startup_supported&&a.startup_registered===b.startup_registered&&a.startup_error===b.startup_error;
}

export function mergeSnapshot(current:Payload|null,response:SnapshotResponse):Payload {
 if(response.state!==null){
  if(response.mappings===null)throw Error('Desktop snapshot is missing mapping status');
  return {version:response.version,state:response.state,mappings:response.mappings,desktop:response.desktop};
 }
 if(!current||current.version!==response.version)throw Error('Desktop snapshot version is unavailable; refresh state');
 return sameDesktop(current.desktop,response.desktop)?current:{...current,desktop:response.desktop};
}
