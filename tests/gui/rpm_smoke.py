#!/usr/bin/env python3
"""Nobara RPM transaction/GUI checks in a disposable root, never a host install.
Requires bwrap user namespaces, RPM, the native GUI harness prerequisites and
installed runtime dependencies. --nodeps is confined to the empty test RPM DB;
host capability availability is checked separately, not a clean-distro solver test.
"""
import hashlib,json,os,pathlib,subprocess,sys,tempfile
repo=pathlib.Path(__file__).resolve().parents[2]
package=pathlib.Path(sys.argv[1]).resolve();artifacts=pathlib.Path(sys.argv[2]).resolve()
artifacts.mkdir(parents=True,exist_ok=False)
def run(args,**kwargs):return subprocess.check_output([str(a) for a in args],text=True,**kwargs)
result={'package':package.name,'sha256':hashlib.sha256(package.read_bytes()).hexdigest(),'status':'running'}
try:
 requirements=run(['rpm','-qp','--requires',package]).splitlines()
 for requirement in requirements:
  if not requirement.startswith('rpmlib('):run(['rpm','-q','--whatprovides',requirement])
 result['host_requirements']='satisfied'
 assert not run(['rpm','-qp','--scripts',package]).strip(),'Unexpected privileged package script'
 with tempfile.TemporaryDirectory(prefix='veek-rpm-root-') as folder:
  root=pathlib.Path(folder)
  # Run RPM inside the disposable filesystem directly. Nesting rpm --root
  # inside a mount namespace produced SQLite WAL errors on reinstall here.
  # Only native RPM/runtime tools are read-only mounts from the host; plugins
  # are disabled. No host package DB, config, device ACL or system bus is exposed.
  (root/'lib64').symlink_to('usr/lib64')
  rpm=['bwrap','--unshare-user','--uid','0','--gid','0','--bind',root,'/','--ro-bind','/usr/lib64','/usr/lib64','--ro-bind','/usr/lib/rpm','/usr/lib/rpm','--ro-bind','/usr/bin/rpm','/usr/bin/rpm','--ro-bind','/usr/bin/rpmdb','/usr/bin/rpmdb','--ro-bind',package,'/package.rpm','--dev','/dev','--proc','/proc','--','/usr/bin/rpm','--dbpath','/var/lib/rpm','--noplugins']
  run(rpm+['--initdb']);run(rpm+['--nodeps','-i','/package.rpm'])
  name=run(['rpm','-qp','--qf','%{NAME}',package]).strip()
  run(rpm+['--nodeps','-V',name])
  binary=root/'usr/bin/veekpanel'
  assert binary.is_file() and os.access(binary,os.X_OK)
  assert (root/'usr/lib/udev/rules.d/70-veekpanel.rules').read_bytes()==(repo/'packaging/linux/70-veekpanel.rules').read_bytes()
  assert (root/'usr/lib/VeekPanel/THIRD_PARTY/rust/README.txt').is_file()
  assert (root/'usr/lib/VeekPanel/THIRD_PARTY/frontend/README.txt').is_file()
  desktops=list((root/'usr/share/applications').glob('*.desktop'));assert len(desktops)==1
  run(['desktop-file-validate',desktops[0]])
  (artifacts/'desktop-entry.txt').write_text(desktops[0].read_text())
  assert 'Exec=veekpanel' in desktops[0].read_text() or 'Exec=/usr/bin/veekpanel' in desktops[0].read_text()
  assert list((root/'usr/share/icons').rglob('*.png'))
  sentinel=root/'home/test/.config/org.veekpanel.desktop/config.json';sentinel.parent.mkdir(parents=True)
  sentinel.write_text('{"user_data":"must survive package operations"}\n');before=sentinel.read_bytes()
  user_rule=root/'etc/udev/rules.d/99-user-panel.rules';user_rule.parent.mkdir(parents=True);user_rule.write_text('# unrelated user rule\n')
  run(rpm+['--nodeps','--replacepkgs','-U','/package.rpm']);run(rpm+['--nodeps','-V',name])
  assert sentinel.read_bytes()==before and user_rule.is_file()
  with (artifacts/'native-harness.log').open('w') as log:
   subprocess.run(['dbus-run-session','--',sys.executable,str(repo/'tests/gui/native_smoke.py'),str(binary),str(artifacts/'native')],env=dict(os.environ,VEEK_PACKAGE_RELAUNCH='1'),stdout=log,stderr=log,check=True)
  result['native_gui']=json.loads((artifacts/'native/result.json').read_text())['status']
  run(rpm+['--nodeps','-e',name])
  assert not binary.exists() and not desktops[0].exists() and not (root/'usr/lib/udev/rules.d/70-veekpanel.rules').exists()
  assert sentinel.read_bytes()==before and user_rule.is_file()
 result['status']='passed'
 result['scope']='isolated RPM install/reinstall/uninstall, host dependency availability, packaged native GUI/audio/config/startup/ownership/relaunch; no host install or physical USB'
except Exception as error:
 result['status']='failed';result['error']=repr(error);raise
finally:(artifacts/'result.json').write_text(json.dumps(result,indent=2)+'\n')
