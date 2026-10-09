#!/usr/bin/env python3
"""Build a Nobara RPM with exact ELF requirements; never install on the host."""
import hashlib,json,os,pathlib,platform,shutil,subprocess,sys

repo=pathlib.Path(__file__).resolve().parents[2]
output=pathlib.Path(sys.argv[1]).resolve()
release=platform.freedesktop_os_release()
if release.get('ID')!='nobara' or release.get('VERSION_ID')!='44' or platform.machine()!='x86_64':
 raise SystemExit('This packaging recipe is validated only for Nobara 44 x86_64.')
output.mkdir(parents=True,exist_ok=False)
def run(args,cwd=repo):
 return subprocess.check_output([str(a) for a in args],cwd=cwd,text=True)
notices=output/'THIRD_PARTY';notices.mkdir()
commit=run(['git','rev-parse','HEAD']).strip()
(notices/'BUILD_COMMIT.txt').write_text(commit+'\n')
(notices/'BUILD_DIFF.patch').write_text(run(['git','diff','HEAD']))
print(run(['pnpm','--dir','ui','install','--frozen-lockfile']))
print(run(['pnpm','--dir','ui','build']))
print(run(['cargo','build','--manifest-path','app/Cargo.toml','--release','--locked']))
metadata=json.loads(run(['cargo','metadata','--no-deps','--format-version','1','--locked'],repo/'app'))
binary=pathlib.Path(metadata['target_directory'])/'release/veekpanel'
print(run([sys.executable,repo/'tests/manual/license_inventory.py',notices/'rust'],repo/'app'))
print(run([sys.executable,repo/'packaging/windows/frontend_notices.py',notices/'frontend']))
# Tauri's RPM builder does not infer all linked ABI requirements. RPM's own ELF
# scanner records exact symbol versions; explicit packages cover dlopen/plugins.
requires=subprocess.check_output(['/usr/lib/rpm/find-requires'],input=str(binary)+'\n',text=True).splitlines()
if not requires or not any('GLIBC_' in r for r in requires):
 raise SystemExit('No versioned ELF dependencies detected; refusing incomplete package.')
requires=sorted(set(requires+['libayatana-appindicator-gtk3','pipewire-libs','gstreamer1-plugins-base','systemd-udev']))
version=json.loads((repo/'app/tauri.conf.json').read_text())['version']
config={'bundle':{'active':True,'targets':['rpm'],'publisher':'VeekPanel','shortDescription':'PCPanel audio controller for Nobara','license':'MIT','homepage':'https://github.com/apittman019-svg/VeekPanel','resources':{
 str(repo/'LICENSE'):'LICENSE.txt',str(repo/'packaging/linux/RPM-README.txt'):'README.txt',str(notices)+'/':'THIRD_PARTY/'},'linux':{'rpm':{'release':'0.preview.'+commit[:8]+'.nobara44','depends':requires,'files':{'/usr/lib/udev/rules.d/70-veekpanel.rules':str(repo/'packaging/linux/70-veekpanel.rules')}}}}}
config_path=output/'rpm.conf.json';config_path.write_text(json.dumps(config,indent=2)+'\n')
print(run(['node',repo/'ui/node_modules/@tauri-apps/cli/tauri.js','bundle','--config',config_path,'--bundles','rpm'],repo/'app'))
packages=list((binary.parent/'bundle/rpm').glob(f'VeekPanel_{version}*.rpm')) or list((binary.parent/'bundle/rpm').glob(f'VeekPanel-{version}-0.preview.{commit[:8]}.nobara44.x86_64.rpm'))
if len(packages)!=1:raise SystemExit(f'Expected exactly one current RPM, got {packages}')
package=output/packages[0].name;shutil.copy2(packages[0],package)
receipt={'commit':commit,'version':version,'platform':platform.platform(),'os':release,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'package':package.name,'sha256':hashlib.sha256(package.read_bytes()).hexdigest(),'bytes':package.stat().st_size,'requires':requires}
(output/'build.json').write_text(json.dumps(receipt,indent=2)+'\n')
(output/'SHA256SUMS.txt').write_text(receipt['sha256']+'  '+package.name+'\n')
print(package)
