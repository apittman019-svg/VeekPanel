#!/usr/bin/env python3
"""Collect exact Fedora SRPMs for an AppDir inventory, without installing anything.

Usage: fetch_native_sources.py inventory.json cache-directory
The companion directory must accompany any distributed binary once the separate
launcher/runtime review is complete. This does not certify license compliance.
Downloads come only from Fedora Koji; unavailable/custom packages fail closed.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess
import urllib.parse
import urllib.request


def source_url(filename):
    if not re.fullmatch(r'[A-Za-z0-9_+.~^%-]+\.src\.rpm', filename):
        raise ValueError(f'Invalid source package name: {filename}')
    name, version, release = filename[:-8].rsplit('-', 2)
    return 'https://kojipkgs.fedoraproject.org/packages/' + '/'.join(
        urllib.parse.quote(s, safe='') for s in (name, version, release, 'src', filename))


def verify(path, expected):
    # SRPM ARCH can be the build architecture. SOURCEPACKAGE distinguishes source.
    identity = subprocess.check_output([
        'rpm', '-qp', '--qf', '%{NAME}-%{VERSION}-%{RELEASE}.src.rpm\t%{SOURCEPACKAGE}',
        str(path)], text=True, stderr=subprocess.DEVNULL)
    if identity != expected + '\t1':
        raise ValueError(f'Wrong source package: {identity}')
    subprocess.run(['rpm', '-K', '--nosignature', str(path)], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def collect(filename, destination):
    record = {'source_rpm': filename}
    try:
        record['url'] = source_url(filename)
        target = destination / filename
        if not target.exists():
            temporary = destination / (filename + '.download')
            # Never accept partial downloads as an existing verified source RPM.
            with urllib.request.urlopen(record['url'], timeout=60) as response, temporary.open('wb') as stream:
                while chunk := response.read(1024 * 1024):
                    stream.write(chunk)
            verify(temporary, filename)
            temporary.rename(target)
        verify(target, filename)
        with target.open('rb') as stream:
            record['sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
        record['bytes'] = target.stat().st_size
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        record['error'] = str(error)
    print(filename, record.get('error', 'verified'), flush=True)
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('inventory', type=Path)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    packages = json.loads(args.inventory.read_text())['packages']
    sources = sorted({p['source_rpm'] for p in packages.values()})
    args.destination.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=4) as executor:
        records = list(executor.map(lambda s: collect(s, args.destination), sources))
    (args.destination / 'sources.json').write_text(json.dumps(records, indent=2) + '\n')
    failures = [r for r in records if 'error' in r]
    tools = args.destination / 'tools'
    tools.mkdir(exist_ok=True)
    runtime_sources = json.loads(Path(__file__).with_name('runtime-sources.json').read_text())
    for record in runtime_sources:
        target = tools / record['file']
        if not target.exists():
            with urllib.request.urlopen(record['url'], timeout=60) as response:
                content = response.read()
            if hashlib.sha256(content).hexdigest() != record['sha256']:
                raise ValueError(f"Runtime source hash mismatch: {record['file']}")
            target.write_bytes(content)
        with target.open('rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != record['sha256']:
                raise ValueError(f"Cached runtime source hash mismatch: {record['file']}")
    (tools / 'sources.json').write_text(json.dumps(runtime_sources, indent=2) + '\n')
    print(f'{len(records) - len(failures)}/{len(records)} exact source packages verified')
    return bool(failures)


if __name__ == '__main__':
    raise SystemExit(main())
