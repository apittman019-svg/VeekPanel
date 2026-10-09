"""Opt-in live-provider check; private native test session only."""
import json,time,subprocess,sys
from Xlib import display,X
from PIL import Image
from private_x11 import perform

def run(js,http,session,click,until,artifacts,env,saved_config,binary):
 from doom_audio import PrivateAudio
 with PrivateAudio(env,artifacts):
  exercise(js,http,session,click,until,artifacts,env,saved_config,binary)


def exercise(js,http,session,click,until,artifacts,env,saved_config,binary):
 def capture(name):
  d=display.Display(env['DISPLAY'])
  try:
   windows=[w for w in d.screen().root.query_tree().children if w.get_wm_name()=='VeekPanel']
   assert len(windows)==1
   w=windows[0];g=w.get_geometry();raw=w.get_image(0,0,g.width,g.height,X.ZPixmap,0xffffffff)
   Image.frombytes('RGB',(g.width,g.height),raw.data,'raw','BGRX').save(artifacts/(name+'.png'))
  finally:d.close()
 click('Doom');until(lambda:js('return !!document.querySelector(".doom-start")'))
 assert js('return !document.querySelector("iframe")')
 capture('doom-ready')
 click('Play Doom')
 rect=until(lambda:js('const f=document.querySelector("iframe");if(!f)return null;const r=f.getBoundingClientRect(),b=document.querySelector(".doom-card .section-heading button").getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2,stopX:b.x+b.width/2,stopY:b.y+b.height/2}'))
 time.sleep(8)
 capture('doom-provider')
 perform(env['DISPLAY'],'click',rect['x'],rect['y'])
 for i in range(6):
  time.sleep(5);capture(f'doom-start-{i}');print('Captured',i,flush=True)
 for key in ['Escape','Return','Return','Return']:
  perform(env['DISPLAY'],'key',key);time.sleep(1)
 capture('doom-level')
 perform(env['DISPLAY'],'key_down','Up');time.sleep(1);perform(env['DISPLAY'],'key_up','Up')
 perform(env['DISPLAY'],'key_down','Control_L');time.sleep(.3);perform(env['DISPLAY'],'key_up','Control_L');time.sleep(1);capture('doom-moved')
 perform(env['DISPLAY'],'click',rect['stopX'],rect['stopY']);until(lambda:js('return !document.querySelector("iframe")'))
 click('Play Doom');until(lambda:js('return !!document.querySelector("iframe")'))
 click('Dashboard');until(lambda:js('return !document.querySelector("iframe")'))
 click('Doom');until(lambda:js('return !!document.querySelector(".doom-start")'))
 js('window.dispatchEvent(new Event("offline"))')
 assert js('return document.querySelector(".doom-start button").disabled')
 js('window.dispatchEvent(new Event("online"))')
 assert js('return !document.querySelector(".doom-start button").disabled')
 perform(env['DISPLAY'],'resize',850,650);time.sleep(.3)
 assert js('return document.documentElement.scrollWidth<=innerWidth')
 capture('doom-minimum');perform(env['DISPLAY'],'resize',1200,850)
 baseline=saved_config()
 click('Settings');until(lambda:js('return !!document.querySelector(".settings select")'))
 js('const e=document.querySelector(".settings select");e.value="light";e.dispatchEvent(new Event("change",{bubbles:true}))')
 until(lambda:saved_config()['settings']['theme']=='light')
 click('Doom');time.sleep(.3);capture('doom-light')
 click('Settings')
 js('const e=document.querySelector(".settings select");e.value=arguments[0];e.dispatchEvent(new Event("change",{bubbles:true}))',baseline['settings']['theme'])
 until(lambda:saved_config()==baseline)
 click('Doom')
 with (artifacts/'tray-fixture.log').open('w') as log:
  tray=subprocess.Popen([sys.executable,str(__import__('pathlib').Path(__file__).with_name('doom_tray.py'))],env=dict(env,VEEK_PRIVATE_DBUS_TEST='1'),stdout=log,stderr=log)
  try:
   until(lambda:js('return !document.querySelector(".tray-notice")'),10)
   assert saved_config()['settings']['close_to_tray']
   click('Play Doom');until(lambda:js('return !!document.querySelector("iframe")'))
   perform(env['DISPLAY'],'close');time.sleep(1)
   subprocess.run([str(binary)],env=env,check=True,timeout=10,stdout=log,stderr=log)
   until(lambda:js('return !document.querySelector("iframe")'),10)
   capture('doom-restored')
  finally:
   tray.terminate();tray.wait(timeout=5)
 assert saved_config()==baseline and baseline['hardware']['mode']=='disabled'
 print('PASS: native game/input captured; stop/tab exit/offline events, minimum/light layout, config preservation and fixture-backed hide/restore checked',flush=True)
