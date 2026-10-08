#!/usr/bin/env python3
"""Audit a Nobara-built AppDir against RPM build IDs; not a redistribution clearance.

Usage: native_inventory.py AppDir fresh-output-directory
Records unresolved ELF origins/license texts explicitly and exits nonzero for gaps.
Native source provision, non-ELF data and AppImage runtime/launcher need review too.
"""
import sys
from pathlib import Path
import subprocess,json,hashlib,shutil,re
root=Path(sys.argv[1]).resolve()
out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
index={}
for d in ['/usr/lib64','/usr/libexec','/usr/bin']:
 for p in Path(d).rglob('*'):
  if p.is_file():index.setdefault(p.name,[]).append(p)
by_source={}
for line in subprocess.check_output(['rpm','-qa','--qf','%{SOURCERPM}\t%{NEVRA}\n'],text=True).splitlines():
 source,pkg=line.split('\t');by_source.setdefault(source,[]).append(pkg)
packages={};files=[];unknown=[]
def build_id(p):
 s=subprocess.check_output(['readelf','-n',str(p)],text=True,stderr=subprocess.DEVNULL)
 m=re.search('Build ID: ([a-f0-9]+)',s);return m[1] if m else None
for p in root.rglob('*'):
 if not p.is_file() or p.is_symlink():continue
 with p.open('rb') as f:
  if f.read(4)!=b'\x7fELF':continue
 rel=str(p.relative_to(root)); bid=build_id(p)
 if rel in ['usr/bin/veekpanel','AppRun']:continue
 matches=[x for x in index.get(p.name,[]) if bid and build_id(x)==bid]
 if not matches:unknown.append({'file':rel,'build_id':bid});continue
 src=matches[0].resolve()
 info=subprocess.check_output(['rpm','-qf','--qf','%{NAME}\n%{NEVRA}\n%{LICENSE}\n%{SOURCERPM}\n%{URL}\n',str(src)],text=True).splitlines()
 name,nevra,license,srpm,url=info[:5]
 if nevra not in packages:
  texts=[]
  for candidate in [nevra]+[x for x in by_source.get(srpm,[]) if x!=nevra]:
   listed=subprocess.check_output(['rpm','-ql',candidate],text=True).splitlines()
   flagged=subprocess.check_output(['rpm','-q','--licensefiles',candidate],text=True).splitlines()
   texts=[f for f in flagged if Path(f).is_file()]+[f for f in listed if Path(f).is_file() and (
    Path(f).name.lower().startswith(('license','licence','copying','copyright','notice')) or
    Path(f).name.upper() in ('LGPL','GPL','MPL','BSD','MIT'))]
   if texts:break
  texts=sorted(set(texts))
  copied=[]
  for t in texts:
   t=Path(t)
   if t.is_file():
    dest=out/nevra/t.relative_to('/');dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(t,dest);copied.append(str(dest.relative_to(out)))
  packages[nevra]={'license':license,'source_rpm':srpm,'upstream':url,'license_files':copied}
 files.append({'file':rel,'build_id':bid,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'host_file':str(src),'package':nevra})
result={'scope':'ELF library/helper inventory by preserved GNU build ID; source acquisition and non-ELF bundled data/tool licensing require separate review','files':files,'packages':packages,'unmatched':unknown}
(out/'inventory.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'matched_elf':len(files),'packages':len(packages),'unmatched':unknown,'missing_license_text':[k for k,v in packages.items() if not v['license_files']]},indent=2))

if unknown or any(not p['license_files'] for p in packages.values()):sys.exit(1)
