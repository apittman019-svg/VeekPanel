"""Native WebKit visual scenarios using real IPC and private synthetic audio only."""
import base64
import copy
import json
import os
import subprocess
import time
import wave


def run(js, http, session, click, until, idle, saved_config, artifacts, env, stop_server):
    results = []
    stream = None

    def capture(name):
        time.sleep(.35)
        metrics = js('''return {width:innerWidth,height:innerHeight,dpr:devicePixelRatio,
          scrollWidth:document.documentElement.scrollWidth,overflow:document.documentElement.scrollWidth>innerWidth+1,
          reduced:matchMedia('(prefers-reduced-motion: reduce)').matches,
          systemDark:matchMedia('(prefers-color-scheme: dark)').matches};''')
        if metrics['overflow']:
            metrics['outside']=js('''return [...document.querySelectorAll('body *')].map(e=>({tag:e.tagName,cls:e.className?.baseVal??e.className,right:e.getBoundingClientRect().right,width:e.scrollWidth})).filter(e=>e.right>innerWidth+1).slice(0,12);''')
            metrics['spilling']=js('''return [...document.querySelectorAll('body *')].filter(e=>e.scrollWidth>e.clientWidth+2).map(e=>({tag:e.tagName,cls:e.className?.baseVal??e.className,client:e.clientWidth,scroll:e.scrollWidth,overflow:getComputedStyle(e).overflowX})).slice(0,20);''')
        results.append(dict(name=name, **metrics))
        (artifacts / (name + '.png')).write_bytes(base64.b64decode(http('GET', f'/session/{session}/screenshot')))
        assert not metrics['overflow'], (name, metrics)
        if os.environ.get('VEEK_GUI_REDUCED_MOTION') == '1':
            assert metrics['reduced'], 'GTK reduced-motion preference did not reach WebKit'
            assert js('''return [...document.querySelectorAll('.page-content,.dial-pointer,.fader-cap')].every(e=>{
                const s=getComputedStyle(e);return s.animationName==='none'&&s.transitionDuration==='0s';});''')
        return metrics

    def apply_config(config):
        click('Settings')
        until(lambda: js('return document.querySelector("h1").textContent==="Settings"'))
        until(idle)
        js('''const e=document.querySelector('textarea');e.value=arguments[0];
              e.dispatchEvent(new Event('input',{bubbles:true}));''', json.dumps(config))
        click('Import and apply')
        until(lambda: saved_config() == config and idle())
        click('Dashboard')
        until(lambda: js('return document.querySelector("h1").textContent==="Dashboard"'))
        js('window.scrollTo(0,0)')

    baseline = copy.deepcopy(saved_config())
    profile = next(p for p in baseline['profiles'] if p['id'] == baseline['active_profile'])
    profile['name'] = 'Gaming / streaming / recording — a deliberately long profile name'
    profile['mappings'] = [
        {'control': {'device': 'primary', 'kind': 'analog', 'index': 0},
         'action': {'type': 'volume', 'target': {'type': 'default_output'}}},
        {'control': {'device': 'primary', 'kind': 'analog', 'index': 1},
         'action': {'type': 'volume', 'target': {'type': 'match', 'kind': 'playback',
                    'identities': {'application.id': 'org.veekpanel.absent-visual-fixture'}}}}
    ]
    try:
        wav = artifacts / 'visual-silence.wav'
        with wave.open(str(wav), 'wb') as out:
            out.setnchannels(2); out.setsampwidth(2); out.setframerate(48000)
            out.writeframes(bytes(192000))
        stream = subprocess.Popen(['pw-cat', '--playback', '--target', '0', '--properties',
            '{ application.id = org.veekpanel.visual-fixture application.name = "Visual fixture — a deliberately very long application name for layout validation" }', str(wav)],
            env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for model in ['mini', 'pro']:
            for theme in ['light', 'dark', 'system']:
                config = copy.deepcopy(baseline)
                config['hardware'].update(model=model, mode='mock', address='')
                config['settings']['theme'] = theme
                config['hardware']['mode']='disabled'
                apply_config(config)
                config['hardware']['mode']='mock'
                apply_config(config)
                count = 9 if model == 'pro' else 4
                until(lambda: js('return document.querySelectorAll(".knob-select").length===arguments[0]', count))
                assert js('return document.querySelectorAll(".fader").length===arguments[0]', 4 if model == 'pro' else 0)
                assert js('return [...document.querySelectorAll(".slider-card")].every(e=>!e.querySelector(".press"))')
                until(lambda: js('return document.querySelectorAll(".dial.unknown,.fader.unknown").length===arguments[0]', count))
                assert js('return document.querySelector("#control-status-1").textContent.includes("Waiting")')
                assert js('return document.querySelectorAll(".readout")[2].textContent.includes("—")')
                for width, height in [(1200, 850), (850, 650)]:
                    try:http('POST', f'/session/{session}/window/rect', {'width': width, 'height': height})
                    except AssertionError as e:
                        if 'unsupported operation' not in str(e):raise
                        from private_x11 import perform
                        scale=js('return devicePixelRatio')
                        perform(env['DISPLAY'],'resize',round(width*scale),round(height*scale))
                    capture(f'{model}-{theme}-{width}x{height}-unknown')
                # Native simulated position changes: physical 100%, audio remains
                # unchanged on the first input because pickup is still unarmed.
                until(idle)
                js('''const e=document.querySelector('.knob-card input[type=range]');e.value=255;
                      e.dispatchEvent(new Event('change',{bubbles:true}));''')
                until(lambda: js('return document.querySelector("#control-status-0").textContent.includes("100% knob position")'))
                if model == 'pro':
                    js('''const e=document.querySelector('.slider-card input[type=range]');e.value=255;
                          e.dispatchEvent(new Event('change',{bubbles:true}));''')
                    until(lambda: js('return document.querySelector("#control-status-5").textContent.includes("100% slider position")'))
                    js(r'document.querySelector("[data-control-index=\"5\"]").click()')
                    until(lambda: js('return document.querySelector("#assignment-title").textContent==="Configure slider 1"'))
                    assert js(r'return !document.querySelector("select[aria-label=\"Button action\"]")')
                capture(f'{model}-{theme}-positions')
            # Minimum-size secondary pages with long names and native app streams.
            for page in ['Profiles', 'Groups', 'Settings', 'Diagnostics']:
                click(page)
                until(lambda: js('return document.querySelector("h1").textContent===arguments[0]', page))
                js('window.scrollTo(0,0)')
                capture(f'{model}-{page.lower()}-minimum')
            click('Dashboard')
            until(lambda: js('return document.querySelector("h1").textContent==="Dashboard"'))
            until(lambda: js('return [...document.querySelectorAll(".audio-name strong")].some(e=>e.textContent.includes("deliberately very long"))'))
            js('document.querySelector(".audio-list").scrollIntoView()')
            capture(f'{model}-mixer-long-name')
        # Hardware disabled is an offline/unassigned state, not synthetic HID discovery.
        offline = copy.deepcopy(baseline)
        offline['hardware'].update(model='mini', mode='disabled', address='')
        offline['settings']['theme'] = 'dark'
        apply_config(offline)
        until(lambda: js('return document.body.textContent.includes("Connect your PCPanel")'))
        capture('mini-disconnected')
        stop_server()
        until(lambda: js('return document.querySelector("#control-status-0").textContent.includes("Audio offline")'))
        capture('mini-audio-offline')
        print('PASS: native Mini/Pro visual matrix, themes, window sizes, unknown/100% positions, absent targets, distinct faders, long names and offline feedback')
    finally:
        (artifacts / 'visual-metrics.json').write_text(json.dumps(results, indent=2))
        if stream and stream.poll() is None:
            stream.terminate()
            try: stream.wait(timeout=5)
            except subprocess.TimeoutExpired: stream.kill(); stream.wait()
