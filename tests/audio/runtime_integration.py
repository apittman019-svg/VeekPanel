#!/usr/bin/env python3
"""Persistent mapping worker against a PRIVATE PipeWire server; synthetic panel only."""
import copy,json,os,pathlib,queue,subprocess,sys,tempfile,threading,time,wave
binary=pathlib.Path(sys.argv[1]).resolve()
config=pathlib.Path(__file__).with_name('pipewire.conf').resolve()
def until(fn,timeout=8):
 end=time.monotonic()+timeout
 while time.monotonic()<end:
  result=fn()
  if result:return result
  time.sleep(.05)
 raise AssertionError('condition timed out')
def stop(p):
 if p and p.poll() is None:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait()
with tempfile.TemporaryDirectory(prefix='veek-runtime-private-') as folder:
 root=pathlib.Path(folder);env=dict(os.environ,XDG_RUNTIME_DIR=folder,PIPEWIRE_RUNTIME_DIR=folder,PIPEWIRE_REMOTE='veek-test')
 env.pop('PIPEWIRE_CONFIG_DIR',None);env.pop('PIPEWIRE_CONFIG_NAME',None)
 # A second private output makes profile device preferences observably different.
 private_config=root/'pipewire.conf'
 private_config.write_text(config.read_text().rsplit(']',1)[0]+'''    { factory = adapter args = {
        factory.name = support.null-audio-sink node.name = veek.secondary
        node.description = "VeekPanel second isolated output" media.class = Audio/Sink
        audio.position = [ FL FR ] object.linger = true
    } }
]
''')
 server=worker=app=None
 with open(root/'log','w+') as log:
  def start_server():
   p=subprocess.Popen(['pipewire','-c',str(private_config)],env=env,stdout=log,stderr=log)
   until(lambda:(root/'veek-test').exists())
   for key,name in [('sink','output'),('source','input')]:subprocess.run(['pw-metadata','-n','default','0','default.audio.'+key,json.dumps({'name':'veek.'+name}),'Spa:String:JSON'],env=env,stdout=log,stderr=log,check=True)
   return p
  def start_worker():
   p=subprocess.Popen([str(binary),'--config',str(root/'config.json')],env=env,text=True,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log)
   q=queue.Queue()
   def read():
    for line in p.stdout:q.put(json.loads(line))
   threading.Thread(target=read,daemon=True).start()
   return p,q
  def request(command,ok=True):
   worker.stdin.write(json.dumps(command)+'\n');worker.stdin.flush();r=responses.get(timeout=12)
   assert r['ok']==ok,r
   return r.get('state') if ok else r
  def state():return request({'type':'state'})
  def output(s):return next(t for t in s['audio']['targets'] if t['identity'].get('node.name')=='veek.output')
  def secondary():return next(t for t in state()['audio']['targets'] if t['identity'].get('node.name')=='veek.secondary')
  def volume(v):return until(lambda:abs(output(state())['volume']-v)<.01)
  try:
   server=start_server();worker,responses=start_worker();s=until(lambda:(x if (x:=state())['audio'] else None));t=output(s)
   request({'type':'mock_analog','index':0,'raw':0},ok=False)
   request({'type':'volume','selection':{'id':t['id'],'generation':s['audio']['generation']},'value':.5});volume(.5)
   c=s['config'];c['hardware']['mode']='mock';c['hardware']['model']='mini'
   c['profiles'][0]['mappings']=[{'control':{'device':'primary','kind':'analog','index':0},'action':{'type':'volume','target':{'type':'default_output'}}},{'control':{'device':'primary','kind':'button','index':0},'action':{'type':'toggle_mute','target':{'type':'default_input'}}}]
   request({'type':'save','revision':s['revision'],'config':c});until(lambda:state()['hardware_status'].startswith('Development panel'))
   request({'type':'save','revision':s['revision'],'config':c},ok=False)
   request({'type':'mock_analog','index':0,'raw':0});volume(.5)
   request({'type':'mock_analog','index':0,'raw':200});volume(200/255)
   source=lambda:next(t for t in state()['audio']['targets'] if t['kind']=='input')
   before=source()['muted'];request({'type':'mock_button','index':0,'pressed':True});assert source()['muted']==before
   request({'type':'mock_button','index':0,'pressed':False});request({'type':'mock_button','index':0,'pressed':True});until(lambda:source()['muted']!=before)
   request({'type':'mock_button','index':0,'pressed':True});assert source()['muted']!=before
   assert (root/'config.json.bak').exists()
   # Persist application identity and group mappings; verify against a real native client.
   wav=root/'silence.wav'
   with wave.open(str(wav),'wb') as w:
    w.setnchannels(2);w.setsampwidth(2);w.setframerate(48000);w.writeframes(bytes(192000))
   def launch_app():
    return subprocess.Popen(['pw-cat','--playback','--target','0','--properties','{ application.id = org.veekpanel.mapping-test application.name = "Mapping integration" }',str(wav)],env=env,stdout=log,stderr=log)
   def application():
    return next((t for t in state()['audio']['targets'] if t['identity'].get('application.id')=='org.veekpanel.mapping-test'),None)
   app=launch_app();stream=until(application)
   selector={'type':'match','kind':'playback','identities':{'application.id':'org.veekpanel.mapping-test'}}
   s=state();c=s['config'];primary=c['profiles'][0]
   primary['preferences']['output']={'type':'match','kind':'output','identities':{'node.name':'veek.output'}}
   primary['groups']=[{'id':'mix','name':'Output + app','members':[{'type':'preferred_output'},selector],'relative':True}]
   for index,target in [(1,selector),(2,{'type':'group','id':'mix'})]:
    c['profiles'][0]['mappings'].append({'control':{'device':'primary','kind':'analog','index':index},'action':{'type':'volume','target':target}})
   request({'type':'save','revision':s['revision'],'config':c})
   def set_level(t,value):
    request({'type':'volume','selection':{'id':t['id'],'generation':state()['audio']['generation']},'value':value})
   set_level(stream,.5)
   request({'type':'mock_analog','index':1,'raw':0});request({'type':'mock_analog','index':1,'raw':180})
   until(lambda:abs(application()['volume']-180/255)<.01)
   old_id=stream['id'];stop(app);until(lambda:application() is None);app=launch_app();stream=until(application)
   assert stream['id']!=old_id
   set_level(stream,.5)
   request({'type':'mock_analog','index':1,'raw':10});assert abs(application()['volume']-.5)<.01
   request({'type':'mock_analog','index':1,'raw':200});until(lambda:abs(application()['volume']-200/255)<.01)
   set_level(output(state()),.4);set_level(application(),.2)
   request({'type':'mock_analog','index':2,'raw':0});request({'type':'mock_analog','index':2,'raw':204})
   volume(.8);until(lambda:abs(application()['volume']-.4)<.01)
   # The same group ID is local to each profile, as is its preferred device.
   s=state();c=s['config'];primary=c['profiles'][0]
   alternate=copy.deepcopy(primary);alternate.update(id='alternate',name='Second output')
   alternate['preferences']['output']={'type':'match','kind':'output','identities':{'node.name':'veek.secondary'}}
   alternate['groups']=[{'id':'mix','name':'Only second output','members':[{'type':'preferred_output'}],'relative':False}]
   c['profiles'].append(alternate)
   request({'type':'save','revision':s['revision'],'config':c})
   set_level(secondary(),.3)
   request({'type':'activate','id':'alternate'})
   request({'type':'mock_analog','index':2,'raw':255})
   assert abs(secondary()['volume']-.3)<.01 # First input after switching cannot jump volume.
   request({'type':'mock_analog','index':2,'raw':0})
   until(lambda:secondary()['volume']<.01)
   request({'type':'mock_analog','index':2,'raw':153})
   until(lambda:abs(secondary()['volume']-.6)<.01)
   volume(.8);assert abs(application()['volume']-.4)<.01
   request({'type':'activate','id':primary['id']})
   request({'type':'mock_analog','index':2,'raw':0});volume(.8)
   request({'type':'mock_analog','index':2,'raw':255})
   volume(1);until(lambda:abs(application()['volume']-.5)<.01)
   assert abs(secondary()['volume']-.6)<.01
   request({'type':'activate','id':'alternate'});c['active_profile']='alternate'
   stop(app);app=None;until(lambda:application() is None)

   old=state()['audio'];stop(server);until(lambda:state()['audio'] is None);until(lambda:not(root/'veek-test').exists());server=start_server()
   s=until(lambda:x if (x:=state())['audio'] and x['audio']['generation']!=old['generation'] else None)
   request({'type':'mute','selection':{'id':old['targets'][0]['id'],'generation':old['generation']},'muted':True},ok=False)
   before=output(s)['volume'];request({'type':'mock_analog','index':0,'raw':10});volume(before)
   request({'type':'quit'});assert worker.wait(timeout=5)==0
   worker,responses=start_worker();s=until(lambda:x if (x:=state())['audio'] else None)
   assert s['config']==c
   until(lambda:state()['hardware_status'].startswith('Development panel'))
   before=output(s)['volume'];request({'type':'mock_analog','index':0,'raw':220});volume(before)
   set_level(secondary(),.5)
   request({'type':'mock_analog','index':2,'raw':0});assert abs(secondary()['volume']-.5)<.01
   request({'type':'mock_analog','index':2,'raw':200});until(lambda:abs(secondary()['volume']-200/255)<.01)
   volume(before) # Relaunch still routes the local group to the second output.
   request({'type':'quit'});assert worker.wait(timeout=5)==0
   print('PASS: private native PipeWire persistent mappings, simulated Mini pickup/buttons, stale UI rejection, durable app relaunch/group volume, profile-local groups/preferences and switch pickup, backup/relaunch, service recovery and no stale writes')
  except BaseException:
   log.flush();log.seek(0);print(log.read()[-12000:],file=sys.stderr);raise
  finally:stop(app);stop(worker);stop(server)
