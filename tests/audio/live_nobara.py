#!/usr/bin/env python3
"""Explicit opt-in live PipeWire verification. Never run in ordinary CI.

Records/restores selected endpoint state; uses a named unlinked pw-cat test stream.
This tests the real desktop server, not PCPanel hardware or audible signal quality.
"""
import argparse,json,pathlib,subprocess,tempfile,time,wave,os
p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('--allow-live',action='store_true');args=p.parse_args()
if not args.allow_live:p.error('live audio mutation requires --allow-live')
binary=str(pathlib.Path(args.binary).resolve())
def cli(*args):
 r=subprocess.run([binary,*args],text=True,capture_output=True,timeout=8)
 if r.returncode:raise RuntimeError((args,r.stderr,r.stdout))
 return json.loads(r.stdout)
def state():return cli('list')['targets']
def change(id,key,value):
 r=cli('set','--target',id,'--'+key,str(value))
 assert r['confirmed'],r
 return r
initial=state()
master=next(t for t in initial if t['kind']=='output' and t['default'])
secondary=next((t for t in initial if t['kind']=='output' and not t['default']),None)
source=next((t for t in initial if t['kind']=='input' and not t['default']),None)
original={t['id']:t for t in [master,secondary,source] if t}
folder=pathlib.Path(tempfile.mkdtemp(prefix='veek-live-audio-'))
(folder/'before.json').write_text(json.dumps(list(original.values()),indent=2))
changed=set();app=None
try:
 # Never amplify: lower the current master by one percentage point, then restore.
 assert master['volume'] is not None and 0<=master['volume']<=1
 changed.add((master['id'],'volume'))
 change(master['id'],'volume',max(0,master['volume']*100-1))
 change(master['id'],'volume',master['volume']*100)
 if secondary:
  changed.add((secondary['id'],'mute'));change(secondary['id'],'mute','on');change(secondary['id'],'mute','on' if secondary['muted'] else 'off')
 if source:
  changed.add((source['id'],'mute'));change(source['id'],'mute','on');change(source['id'],'mute','on' if source['muted'] else 'off')
 wav=folder/'silence.wav'
 with wave.open(str(wav),'wb') as w:
  w.setnchannels(2);w.setsampwidth(2);w.setframerate(48000);w.writeframes(bytes(192000))
 log=open(folder/'client.log','w')
 app=subprocess.Popen(['pw-cat','--playback','--target','0','--properties','{ application.id = org.veekpanel.live-test application.name = "VeekPanel live integration" }',str(wav)],stdout=log,stderr=log)
 for _ in range(50):
  streams=[t for t in state() if t['identity'].get('application.id')=='org.veekpanel.live-test']
  if streams:break
  time.sleep(.1)
 assert streams,'test client did not appear'
 stream=streams[0];change(stream['id'],'volume',35);change(stream['id'],'mute','on');change(stream['id'],'mute','off')
 assert stream['identity'].get('application.process.binary')
 app.terminate();app.wait(timeout=4);app=None
 for _ in range(30):
  if all(t['id']!=stream['id'] for t in state()):break
  time.sleep(.1)
 else:raise AssertionError('closed test stream still present')
 print('PASS: live Nobara default-output volume, secondary output/input mute, native app discovery/identity/volume/mute and disappearance')
finally:
 if app:app.terminate();app.wait(timeout=4)
 errors=[]
 for id,key in changed:
  try:change(id,key,original[id]['volume']*100 if key=='volume' else ('on' if original[id]['muted'] else 'off'))
  except Exception as e:errors.append(str(e))
 if errors:raise RuntimeError(f'Restore failed. Original state retained at {folder}/before.json: {errors}')
 restored={t['id']:t for t in state()}
 for id,key in changed:
  field='muted' if key=='mute' else 'volume'
  assert abs(float(restored[id][field])-float(original[id][field]))<.005,(id,field)
 print('PASS: all changed endpoints restored and verified; private evidence at',folder)
