#!/usr/bin/env python3
"""Audit a Nobara AppDir against RPM build IDs and exact resource bytes.

Usage: native_inventory.py AppDir fresh-output-directory
Records unresolved origins/license texts explicitly and exits nonzero for gaps.
Source provision and AppImage runtime/launcher still require separate review.
"""
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys


def output(*args):
    return subprocess.check_output(args, text=True, stderr=subprocess.DEVNULL)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def build_id(path):
    match = re.search(r'Build ID: ([a-f0-9]+)', output('readelf', '-n', str(path)))
    return match[1] if match else None


def audit(root, out):
    root = root.resolve()
    out.mkdir(parents=True, exist_ok=False)
    index = {}
    for directory in ['/usr/lib64', '/usr/libexec', '/usr/bin']:
        for path in Path(directory).rglob('*'):
            if path.is_file():
                index.setdefault(path.name, []).append(path)
    by_source = {}
    for line in output('rpm', '-qa', '--qf', '%{SOURCERPM}\t%{NEVRA}\n').splitlines():
        source, package = line.split('\t')
        by_source.setdefault(source, []).append(package)
    packages, files, unknown, generated, external, symlinks = {}, [], [], [], [], []
    tool_origins = json.loads(Path(__file__).with_name('appimage-tools.json').read_text())
    # Explicit generated/project-owned items, not a blanket exclusion by type.
    generated_paths = {
        'usr/bin/veekpanel', 'AppRun', 'VeekPanel.png',
        'usr/share/applications/VeekPanel.desktop',
        'usr/share/icons/hicolor/128x128/apps/veekpanel.png',
        'usr/lib/VeekPanel/LICENSE.txt', 'usr/lib/VeekPanel/README.txt',
        'usr/lib/VeekPanel/70-veekpanel.rules',
        'usr/share/glib-2.0/schemas/gschemas.compiled',
        'usr/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache',
        'usr/lib/gtk-3.0/3.0.0/immodules.cache',
        'usr/lib/gio/modules/giomodule.cache',
    }
    for path in sorted(root.rglob('*')):
        if path.is_symlink():
            record = {'file': str(path.relative_to(root)), 'target': str(path.readlink())}
            if not path.exists() or not path.resolve().is_relative_to(root):
                unknown.append(record)
            else:
                symlinks.append(record)
            continue
        if not path.is_file():
            continue
        rel = str(path.relative_to(root))
        if rel.startswith('usr/lib/VeekPanel/THIRD_PARTY/'):
            continue  # Notice output is not an additional bundled dependency.
        with path.open('rb') as stream:
            elf = stream.read(4) == b'\x7fELF'
        record = {'file': rel, 'sha256': digest(path), 'kind': 'elf' if elf else 'data'}
        if rel in tool_origins and record['sha256'] == tool_origins[rel]['sha256']:
            record.update(tool_origins[rel])
            external.append(record)
            continue
        if rel in generated_paths:
            generated.append(record)
            continue
        if elf:
            bid = build_id(path)
            record['build_id'] = bid
            matches = [p for p in index.get(path.name, []) if bid and build_id(p) == bid]
        else:
            candidates = [Path('/') / rel]
            if rel.startswith('usr/lib/'):
                candidates.append(Path('/usr/lib64') / path.relative_to(root / 'usr/lib'))
            matches = [p for p in candidates if p.is_file() and digest(p) == record['sha256']]
        if not matches:
            unknown.append(record)
            continue
        src = matches[0].resolve()
        owners = output('rpm', '-qf', '--qf', '%{NEVRA}\t%{LICENSE}\t%{SOURCERPM}\t%{URL}\n', str(src)).splitlines()
        # Identical architecture-independent files may have multiple installed owners.
        nevra, license, srpm, url = sorted(owners, key=lambda s: '.x86_64\t' not in s)[0].split('\t')
        if nevra not in packages:
            texts = []
            for candidate in [nevra] + [p for p in by_source.get(srpm, []) if p != nevra]:
                listed = output('rpm', '-ql', candidate).splitlines()
                flagged = output('rpm', '-q', '--licensefiles', candidate).splitlines()
                texts = [Path(p) for p in flagged if Path(p).is_file()]
                texts += [Path(p) for p in listed if Path(p).is_file() and (
                    Path(p).name.lower().startswith(('license', 'licence', 'copying', 'copyright', 'notice'))
                    or Path(p).name.upper() in ('LGPL', 'GPL', 'MPL', 'BSD', 'MIT'))]
                if texts:
                    break
            copied = []
            for text in sorted(set(texts)):
                dest = out / nevra / text.relative_to('/')
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(text, dest)
                copied.append(str(dest.relative_to(out)))
            # Exact source identity only: never use an unrelated version's text.
            if not copied and srpm == 'hyphen-2.8.8-28.fc44.src.rpm':
                supplement = Path(__file__).resolve().parents[1] / 'third-party/hyphen-2.8.8'
                for text in sorted(supplement.iterdir()):
                    dest = out / nevra / text.name
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(text, dest)
                    copied.append(str(dest.relative_to(out)))
            packages[nevra] = {'license': license, 'source_rpm': srpm,
                               'upstream': url, 'license_files': copied}
        record.update(host_file=str(src), package=nevra)
        files.append(record)
    tool_notices = Path(__file__).resolve().parents[1] / 'third-party/appimage-tools'
    shutil.copytree(tool_notices, out / 'appimage-tools')
    missing = [p for p, v in packages.items() if not v['license_files']]
    result = {'scope': 'RPM ELF build-ID and exact-byte data inventory; external tools and source provision require separate review',
              'files': files, 'packages': packages, 'generated_or_project': generated,
              'external_tools': external,
              'symlinks': symlinks,
              'unmatched': unknown, 'missing_license_text': missing}
    (out / 'inventory.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'matched_elf': sum(f['kind'] == 'elf' for f in files),
                      'matched_data': sum(f['kind'] == 'data' for f in files),
                      'packages': len(packages), 'external_tools': len(external), 'unmatched': unknown,
                      'missing_license_text': missing}, indent=2))
    return bool(unknown or missing)


if __name__ == '__main__':
    sys.exit(audit(Path(sys.argv[1]), Path(sys.argv[2])))
