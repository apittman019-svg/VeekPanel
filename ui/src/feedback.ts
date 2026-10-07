import type {ControlFeedback} from './types';

export function feedbackText(info:ControlFeedback|undefined):string {
 if(!info)return 'Unassigned';
 switch(info.phase){
  case 'awaiting_input':return 'Move control to begin pickup';
  case 'pickup':{
   const level=Math.round((info.target_volume??0)*100);
   const direction=info.position===null?'Move':info.position<(info.target_volume??0)?'Turn up':info.position>(info.target_volume??0)?'Turn down':'Move';
   return `${direction} through ${level}% to take control`;
  }
  case 'controlling':return 'Controlling volume';
  case 'waiting_release':return 'Release button before pressing';
  case 'ready':return 'Ready to press';
  case 'audio_offline':return 'Audio offline';
  case 'target_unavailable':return 'Waiting for target';
  case 'blocked':return info.message??'Action unavailable';
 }
}
