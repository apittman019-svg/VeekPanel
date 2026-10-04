#!/usr/bin/env python3
"""Native PipeWire integration against a private daemon, never the desktop server.

Real OS API tests with synthetic endpoints/streams and PTY hardware, not physical
PCPanel or audible endpoint acceptance. Requires pipewire, pw-cat, pw-cli, pw-metadata.
"""
import json
import os
import pathlib
import pty
import queue
import signal
import subprocess
import sys
import tempfile
import threading
import time
import wave

binary = pathlib.Path(sys.argv[1]).resolve()
config = pathlib.Path(__file__).with_name('pipewire.conf').resolve()

def until(fn, timeout=6):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        result = fn()
        if result:
            return result
        time.sleep(.05)
    raise AssertionError('condition not met before deadline')

def stop(process):
    if process and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=4)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

with tempfile.TemporaryDirectory(prefix='veek-audio-test-') as folder:
    runtime = pathlib.Path(folder)
    env = dict(os.environ, XDG_RUNTIME_DIR=folder, PIPEWIRE_RUNTIME_DIR=folder,
               PIPEWIRE_REMOTE='veek-test')
    env.pop('PIPEWIRE_CONFIG_DIR', None)
    env.pop('PIPEWIRE_CONFIG_NAME', None)
    # Every child receives the private runtime AND explicit remote name.
    def cli(*args, ok=True):
        result = subprocess.run([str(binary), *args], env=env, text=True, capture_output=True, timeout=8)
        if ok:
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            return json.loads(result.stdout)
        assert result.returncode != 0, (args, result.stdout)
        return result

    def native(*args):
        result = subprocess.run(args, env=env, text=True, capture_output=True, timeout=6)
        assert result.returncode == 0, (args, result.stderr)
        return result.stdout

    def daemon():
        p = subprocess.Popen(['pipewire', '-c', str(config)], env=env, stdout=log, stderr=log)
        until(lambda: (runtime/'veek-test').exists())
        return p

    def targets():
        return cli('list')['targets']

    def target(kind):
        return next(t for t in targets() if t['kind'] == kind)

    def global_id(name):
        objects = json.loads(native('pw-dump'))
        return next(o['id'] for o in objects if o.get('info', {}).get('props', {}).get('node.name') == name)

    def observations(process):
        q = queue.Queue()
        def read():
            for line in process.stdout:
                q.put(json.loads(line))
        threading.Thread(target=read, daemon=True).start()
        return q

    def await_event(q, predicate, timeout=8):
        end = time.monotonic() + timeout
        seen = []
        while time.monotonic() < end:
            try:
                event = q.get(timeout=.2)
            except queue.Empty:
                continue
            seen.append(event)
            if predicate(event):
                return event
        raise AssertionError(('expected event absent', seen))

    server = app = watcher = binding = None
    master = slave = None
    with open(runtime/'daemon.log', 'w+') as log:
        try:
            server = daemon()
            initial = targets()
            assert {t['kind'] for t in initial} == {'output', 'input'}, initial
            output = target('output')['id']
            source = target('input')['id']
            native('pw-metadata', '-n', 'default', '0', 'default.audio.sink', '{"name":"veek.output"}', 'Spa:String:JSON')
            native('pw-metadata', '-n', 'default', '0', 'default.audio.source', '{"name":"veek.input"}', 'Spa:String:JSON')
            assert target('output')['default'] and target('input')['default']
            for selected in [output, source, 'default-output', 'default-input']:
                assert cli('set', '--target', selected, '--volume', '50')['confirmed']
                assert cli('set', '--target', selected, '--mute', 'on')['confirmed']
                assert cli('set', '--target', selected, '--mute', 'off')['confirmed']
            cli('set', '--target', output, '--volume', 'nan', ok=False)
            cli('set', '--target', output, '--volume', '101', ok=False)
            cli('set', '--target', 'missing', '--mute', 'on', ok=False)

            node = str(global_id('veek.output'))
            native('pw-cli', 'set-param', node, 'Props', '{ channelVolumes: [ 0.125 1.0 ] }')
            assert cli('set', '--target', output, '--volume', '50')['confirmed']
            data = json.loads(native('pw-dump'))
            props = next(o for o in data if str(o['id']) == node)['info']['params']['Props'][0]
            assert abs(props['channelVolumes'][0] - .015625) < .0001, props
            assert abs(props['channelVolumes'][1] - .125) < .0001, props

            watcher = subprocess.Popen([str(binary), 'watch', '--duration', '45'], env=env, text=True, stdout=subprocess.PIPE, stderr=log)
            events = observations(watcher)
            first = await_event(events, lambda e: 'targets' in e)
            native('pw-cli', 'set-param', node, 'Props', '{ mute: true }')
            await_event(events, lambda e: any(t['id'] == output and t['muted'] for t in e.get('targets', [])))
            # A WAV fixture works on both Ubuntu's older pw-cat and current Nobara.
            # Newer --raw/standard-input behavior differs across those versions.
            wav = runtime/'silence.wav'
            with wave.open(str(wav), 'wb') as fixture:
                fixture.setnchannels(2)
                fixture.setsampwidth(2)
                fixture.setframerate(48000)
                fixture.writeframes(bytes(48000 * 2 * 2))
            app = subprocess.Popen(['pw-cat', '--playback', '--target', '0', str(wav)], env=env, stdout=log, stderr=log)
            appearance = await_event(events, lambda e: any(t['kind'] == 'playback' for t in e.get('targets', [])))
            playback = next(t for t in appearance['targets'] if t['kind'] == 'playback')
            assert playback['identity'].get('application.name'), playback
            assert cli('set', '--target', playback['id'], '--volume', '35')['confirmed']
            assert cli('set', '--target', playback['id'], '--mute', 'on')['confirmed']
            stop(app)
            await_event(events, lambda e: 'targets' in e and all(t['id'] != playback['id'] for t in e['targets']))
            cli('set', '--target', playback['id'], '--volume', '25', ok=False)

            # A real capture client is a separate recording target, not an output.
            app = subprocess.Popen(['pw-cat', '--record', '--rate', '48000', '--channels', '2', '--format', 's16', '--target', '0', str(runtime/'capture.wav')], env=env, stdout=log, stderr=log)
            recording_event = await_event(events, lambda e: any(t['kind'] == 'recording' for t in e.get('targets', [])))
            recording = next(t for t in recording_event['targets'] if t['kind'] == 'recording')
            assert cli('set', '--target', recording['id'], '--volume', '40')['confirmed']
            assert cli('set', '--target', recording['id'], '--mute', 'on')['confirmed']
            stop(app)
            await_event(events, lambda e: 'targets' in e and all(t['id'] != recording['id'] for t in e['targets']))

            # Independent synthetic Original PTY input -> actual isolated OS volume/mute.
            cli('set', '--target', output, '--mute', 'off')
            cli('set', '--target', output, '--volume', '50')
            master, slave = pty.openpty()
            binding = subprocess.Popen([str(binary), 'bind', '--serial', os.ttyname(slave), '--target', output, '--button-target', source, '--duration', '15'], env=env, text=True, stdout=subprocess.PIPE, stderr=log)
            binding_events = observations(binding)
            await_event(binding_events, lambda e: e.get('status') == 'binding')
            os.write(master, b'v0x90\r\nb0 0\r\nv0x80\r\n')
            time.sleep(.4)
            assert abs(target('output')['volume'] - .5) < .005
            assert not target('input')['muted']
            os.write(master, b'v0x45\r\nb0 1\r\nb0 0\r\nb0 0\r\n')
            await_event(binding_events, lambda e: e.get('observed', {}).get('id') == output and e.get('confirmed'))
            await_event(binding_events, lambda e: e.get('observed', {}).get('id') == source and e.get('confirmed'))
            assert abs(target('output')['volume'] - .45) < .005
            assert target('input')['muted']
            os.close(master); master = None
            assert binding.wait(timeout=5) != 0  # Fail closed on hardware loss.

            # Watch recovers; stale object IDs from before restart cannot mutate replacements.
            stop(server)
            await_event(events, lambda e: e.get('status') == 'unavailable')
            until(lambda: not (runtime/'veek-test').exists())
            server = daemon()
            recovered = await_event(events, lambda e: 'targets' in e and e['generation'] != first['generation'])
            assert output not in [t['id'] for t in recovered['targets']]
            cli('set', '--target', output, '--volume', '20', ok=False)
            watcher.send_signal(signal.SIGINT)
            assert watcher.wait(timeout=4) == 0
            stop(server)
            cli('list', ok=False)
            print('PASS: native isolated PipeWire outputs/inputs/app stream, volume/mute, balance, defaults, external events, app disappearance, PTY binding/pickup, service restart and stale IDs')
        except BaseException:
            log.flush();log.seek(0);print(log.read()[-12000:], file=sys.stderr)
            raise
        finally:
            for p in [binding, watcher, app, server]: stop(p)
            for fd in [master, slave]:
                if fd is not None: os.close(fd)
