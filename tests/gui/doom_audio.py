"""Private WebAudio routing/recording; never connect to the desktop audio daemon."""
import json
from pathlib import Path
import subprocess
import time
import wave


class PrivateAudio:
    def __init__(self, env, artifacts):
        runtime = Path(env['XDG_RUNTIME_DIR'])
        if env.get('PIPEWIRE_REMOTE') != 'veek-test' or not runtime.name.startswith('veek-native-gui-'):
            raise RuntimeError('Refusing to route desktop audio')
        self.env = dict(env, XDG_STATE_HOME=str(runtime/'state'), XDG_CACHE_HOME=str(runtime/'cache'))
        self.artifacts = artifacts

    def __enter__(self):
        self.log=(self.artifacts/'audio-routing.log').open('w')
        # The installed policy-only profile contains no hardware monitors.
        self.policy=subprocess.Popen(['wireplumber','--profile','policy'],env=self.env,stdout=self.log,stderr=self.log)
        time.sleep(1)
        if self.policy.poll() is not None:raise RuntimeError('Private WirePlumber policy failed')
        self.record = subprocess.Popen(['pw-record','--target','veek.output','--rate','44100','--channels','2',
            '--format','s16','--properties','{ stream.capture.sink = true node.name = veek.doom.capture }',
            str(self.artifacts/'game-audio.wav')],env=self.env,stdout=self.log,stderr=self.log)
        return self

    def __exit__(self, *exception):
        for process in [self.record,self.policy]:
            process.terminate()
            try:process.wait(timeout=5)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        self.log.close()
        with wave.open(str(self.artifacts/'game-audio.wav'),'rb') as wav:
            data=wav.readframes(wav.getnframes())
            result={'frames':wav.getnframes(),'nonzero_bytes':sum(b!=0 for b in data)}
        (self.artifacts/'audio-result.json').write_text(json.dumps(result,indent=2))
