#!/usr/bin/env python3
"""Native Tauri/WebKit smoke against a private PipeWire daemon and config folder.
Requires Xvfb, tauri-driver, WebKitWebDriver. No desktop audio is modified.
"""
import base64,json,os,pathlib,subprocess,sys,tempfile,time,urllib.request,urllib.error,shutil
binary=pathlib.Path(sys.argv[1]).resolve();root_repo=pathlib.Path(__file__).resolve().parents[2]
artifacts=pathlib.Path(sys.argv[2]).resolve() if len(sys.argv)>2 else pathlib.Path(tempfile.mkdtemp(prefix='veek-gui-artifacts-'))
artifacts.mkdir(parents=True,exist_ok=True)
# tauri-driver treats --native-driver as a filesystem path, not a PATH command.
native_driver=shutil.which(os.environ.get('VEEK_WEBKIT_DRIVER','WebKitWebDriver'))
if not native_driver:raise SystemExit('WebKitWebDriver not found; install it or set VEEK_WEBKIT_DRIVER to its full path')
def until(fn,timeout=15):
 end=time.monotonic()+timeout
 while time.monotonic()<end:
  try:
   result=fn()
   if result:return result
  except (OSError,AssertionError):pass
  time.sleep(.15)
 raise AssertionError('condition timed out')
def stop(p):
 if p and p.poll() is None:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait()
with tempfile.TemporaryDirectory(prefix='veek-native-gui-') as folder:
 root=pathlib.Path(folder);env=dict(os.environ,XDG_RUNTIME_DIR=folder,PIPEWIRE_RUNTIME_DIR=folder,PIPEWIRE_REMOTE='veek-test',XDG_CONFIG_HOME=str(root/'config'),GDK_BACKEND='x11')
 env.pop('PIPEWIRE_CONFIG_DIR',None);env.pop('PIPEWIRE_CONFIG_NAME',None)
 # Private display; never manipulate windows on the user's desktop.
 display=next(n for n in range(110,180) if not pathlib.Path(f'/tmp/.X11-unix/X{n}').exists());env['DISPLAY']=f':{display}'
 xvfb=server=driver=None;session=None
 port=4454
 def http(method,path,body=None):
  request=urllib.request.Request(f'http://127.0.0.1:{port}'+path,data=None if body is None else json.dumps(body).encode(),method=method,headers={'Content-Type':'application/json'})
  try:
   with urllib.request.urlopen(request,timeout=25) as r:result=json.load(r)
  except urllib.error.HTTPError as e:raise AssertionError(e.read().decode()) from e
  value=result.get('value');assert not(isinstance(value,dict) and 'error'in value),result
  return value
 def js(script,*args):return http('POST',f'/session/{session}/execute/sync',{'script':script,'args':list(args)})
 def click(text):
  assert js('const b=[...document.querySelectorAll("button")].find(b=>b.textContent.trim().endsWith(arguments[0]));if(!b||b.disabled)return false;b.click();return true;',text),text
 def ready():return js('return document.querySelector("h1")?.textContent==="Dashboard" && document.body.textContent.includes("PipeWire (native)")')
 def saved_config():return json.loads((root/'config/org.veekpanel.desktop/config.json').read_text())
 def active_profile():
  saved=saved_config();return next(p for p in saved['profiles'] if p['id']==saved['active_profile'])
 def idle():return js('return !document.querySelector("header select").disabled')
 def choose_profile(profile_id):
  until(idle)
  js('const s=document.querySelector("header select");s.value=arguments[0];s.dispatchEvent(new Event("change",{bubbles:true}));',profile_id)
  until(lambda:saved_config()['active_profile']==profile_id and idle())
 def group_members(labels):
  js('const s=document.querySelector(".group-form select");for(const o of s.options)o.selected=arguments[0].includes(o.textContent);s.dispatchEvent(new Event("change",{bubbles:true}));',labels)
 def group_name(value):
  js('const i=document.querySelector(".group-form input:not([type=checkbox])");i.value=arguments[0];i.dispatchEvent(new Event("input",{bubbles:true}));',value)
 with open(artifacts/'native.log','w+') as log:
  try:
   xvfb=subprocess.Popen(['Xvfb',env['DISPLAY'],'-screen','0','1280x960x24','-nolisten','tcp'],env=env,stdout=log,stderr=log)
   until(lambda:pathlib.Path(f'/tmp/.X11-unix/X{display}').exists())
   server=subprocess.Popen(['pipewire','-c',str(root_repo/'tests/audio/pipewire.conf')],env=env,stdout=log,stderr=log);until(lambda:(root/'veek-test').exists())
   for key,name in [('sink','output'),('source','input')]:subprocess.run(['pw-metadata','-n','default','0','default.audio.'+key,json.dumps({'name':'veek.'+name}),'Spa:String:JSON'],env=env,stdout=log,stderr=log,check=True)
   driver=subprocess.Popen(['tauri-driver','--port',str(port),'--native-port','4455','--native-driver',native_driver],env=env,stdout=log,stderr=log)
   until(lambda:http('GET','/status'))
   session=http('POST','/session',{'capabilities':{'alwaysMatch':{'tauri:options':{'application':str(binary)}}}})['sessionId']
   until(ready)
   click('Use development panel');until(lambda:js('return document.body.textContent.includes("Simulated knobs and buttons")'))
   # Configure actual GUI controls: synthetic hardware -> real private PipeWire output.
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Turn controls")).querySelector("select");s.value=JSON.stringify({type:"default_output"});s.dispatchEvent(new Event("change",{bubbles:true}));')
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Press action")).querySelector("select");s.value=JSON.stringify({type:"default_input"});s.dispatchEvent(new Event("change",{bubbles:true}));')
   click('Save assignment');until(lambda:js('return document.body.textContent.includes("Assignment saved")'))
   js('const r=[...document.querySelectorAll(".audio-row")].find(r=>r.querySelector("small").textContent.startsWith("output"));const i=r.querySelector("input");i.value=50;i.dispatchEvent(new Event("change",{bubbles:true}));')
   until(lambda:js('return [...document.querySelectorAll(".audio-row")].find(r=>r.querySelector("small").textContent.startsWith("output")).querySelector(".volume").textContent==="50%"'))
   for raw in [0,200]:
    until(lambda:js('return !document.querySelector(".knob-card input").disabled'))
    js('const i=document.querySelector(".knob-card input");i.value=arguments[0];i.dispatchEvent(new Event("change",{bubbles:true}));',raw)
    time.sleep(.4)
   until(lambda:js('return [...document.querySelectorAll(".audio-row")].find(r=>r.querySelector("small").textContent.startsWith("output")).querySelector(".volume").textContent==="78%"'))
   js('document.querySelector(".knob-card .press").click();')
   until(lambda:js('return [...document.querySelectorAll(".audio-row")].find(r=>r.querySelector("small").textContent.startsWith("input")).querySelector("button").textContent==="Unmute"'))
   dump=json.loads(subprocess.check_output(['pw-dump'],env=env,text=True))
   output=next(o for o in dump if o.get('info',{}).get('props',{}).get('node.name')=='veek.output')
   props=next(p for p in output['info']['params']['Props'] if 'channelVolumes'in p)
   assert abs(max(props['channelVolumes'])-(200/255)**3)<.001,props

   time.sleep(.3) # Let the native compositor present the updated page before capture.
   (artifacts/'dashboard-system.png').write_bytes(base64.b64decode(http('GET',f'/session/{session}/screenshot')))
   # Groups and preferred devices belong to a profile; duplication must deep-copy them.
   original_id=saved_config()['active_profile']
   click('Groups');until(lambda:js('return document.querySelector("h1").textContent==="Groups"'))
   group_name('Integration mix');group_members(['System output','System microphone'])
   click('Create group');until(lambda:len(active_profile()['groups'])==1 and idle())
   click('Settings');until(lambda:js('return document.querySelector("h1").textContent==="Settings"'))
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Preferred output")).querySelector("select");s.value=[...s.options].find(o=>o.value&&JSON.parse(o.value).identities?.["node.name"]==="veek.output").value;s.dispatchEvent(new Event("change",{bubbles:true}));')
   until(lambda:active_profile()['preferences']['output'] is not None and idle())
   original=active_profile()
   click('Profiles');until(lambda:js('return document.querySelector("h1").textContent==="Profiles"'))
   js('const i=document.querySelector("input[placeholder]");i.value="Integration profile";i.dispatchEvent(new Event("input",{bubbles:true}));')
   click('Duplicate active');until(lambda:js('return document.querySelector("header select").selectedOptions[0].textContent==="Integration profile"') and idle())
   duplicate_id=saved_config()['active_profile'];duplicate=active_profile()
   assert duplicate['groups']==original['groups'] and duplicate['preferences']==original['preferences']
   click('Groups');until(lambda:js('return document.querySelector("h1").textContent==="Groups"'))
   click('Edit');group_name('Alternate mix');group_members(['System microphone'])
   js('const c=document.querySelector(".group-form input[type=checkbox]");if(c.checked)c.click();')
   click('Save group');until(lambda:active_profile()['groups'][0]['name']=='Alternate mix' and idle())
   edited=active_profile()['groups'][0]
   assert len(edited['members'])==1 and not edited['relative']
   click('Settings');until(lambda:js('return document.querySelector("h1").textContent==="Settings"'))
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Preferred output")).querySelector("select");s.value="";s.dispatchEvent(new Event("change",{bubbles:true}));')
   until(lambda:active_profile()['preferences']['output'] is None and idle())
   click('Groups');until(lambda:js('return document.querySelector("h1").textContent==="Groups"'))
   click('Edit');group_name('Unsaved duplicate edit');group_members(['System output'])
   choose_profile(original_id)
   until(lambda:js('return document.querySelector(".group-row h3")?.textContent==="Integration mix"'))
   assert js('return document.querySelector(".group-form input:not([type=checkbox])").value==="" && document.querySelector(".group-form select").selectedOptions.length===0 && ![...document.querySelectorAll("button")].some(b=>b.textContent==="Cancel edit")')
   assert active_profile()==original
   click('Settings');until(lambda:js('return document.querySelector("h1").textContent==="Settings"'))
   assert js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Preferred output")).querySelector("select");return JSON.parse(s.value).identities["node.name"]==="veek.output";')
   choose_profile(duplicate_id)
   assert js('return [...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Preferred output")).querySelector("select").value===""')
   assert active_profile()['groups']==[edited]
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Appearance")).querySelector("select");s.value="dark";s.dispatchEvent(new Event("change",{bubbles:true}));')
   until(lambda:js('return document.querySelector(".shell").dataset.theme==="dark"'))
   click('Dashboard');until(ready)
   time.sleep(.3) # Let the native compositor present the updated page before capture.
   (artifacts/'dashboard-dark.png').write_bytes(base64.b64decode(http('GET',f'/session/{session}/screenshot')))
   click('Settings');until(lambda:js('return document.querySelector("h1").textContent==="Settings"'))
   js('const s=[...document.querySelectorAll("label")].find(l=>l.textContent.startsWith("Appearance")).querySelector("select");s.value="light";s.dispatchEvent(new Event("change",{bubbles:true}));')
   until(lambda:js('return document.querySelector(".shell").dataset.theme==="light"'))
   click('Dashboard');until(ready)
   time.sleep(.3) # Let the native compositor present the updated page before capture.
   (artifacts/'dashboard-light.png').write_bytes(base64.b64decode(http('GET',f'/session/{session}/screenshot')))
   # No horizontal overflow in the normal desktop size.
   assert js('return document.documentElement.scrollWidth<=window.innerWidth')
   saved=saved_config()
   assert saved['hardware']['mode']=='mock' and saved['settings']['theme']=='light'
   active=next(p for p in saved['profiles'] if p['id']==saved['active_profile']);assert active['name']=='Integration profile' and len(active['mappings'])==2
   assert saved['schema_version']==2 and 'groups' not in saved and 'preferences' not in saved
   assert next(p for p in saved['profiles'] if p['id']==original_id)==original
   assert active['groups']==[edited] and active['preferences']['output'] is None
   print('PASS: native Tauri IPC, real private PipeWire discovery, GUI simulated Mini -> native volume/mic mute, assignment persistence, profile-owned groups/preferences with independent duplicates and cleared switch drafts, dark/light rendering; artifacts:',artifacts)
  except BaseException:
   log.flush();log.seek(0);print(log.read()[-10000:],file=sys.stderr);raise
  finally:
   if session:
    try:http('DELETE',f'/session/{session}')
    except Exception:pass
   stop(driver);stop(server);stop(xvfb)
