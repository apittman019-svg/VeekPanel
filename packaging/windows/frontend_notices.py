"""Collect the pinned frontend production dependency tree's distributed licenses."""
import json,pathlib,shutil,sys
root=pathlib.Path(__file__).resolve().parents[2]/'ui'
dest=pathlib.Path(sys.argv[1]);dest.mkdir(parents=True,exist_ok=False)
seen=set();inventory=[]
for filename in json.loads((root/'bundled-modules.json').read_text()):
 if '/node_modules/' not in filename.replace(chr(92),'/'):continue
 source=pathlib.Path(filename.split('?')[0]).parent
 while not (source/'package.json').is_file():
  if source==source.parent:raise RuntimeError('Package metadata missing: '+filename)
  source=source.parent
 if source in seen:continue
 seen.add(source);data=json.loads((source/'package.json').read_text())
 name=data['name']+'@'+data['version'];target=dest/name.replace('/','_');target.mkdir()
 notices=[p for p in source.iterdir() if p.is_file() and p.name.lower().startswith(('license','copying','notice'))]
 if not notices:raise RuntimeError('Missing distributed license: '+name)
 for p in notices:shutil.copyfile(p,target/p.name)
 inventory.append(name+': '+str(data.get('license','see included license')))
(dest/'README.txt').write_text('Dependencies included in the production frontend bundle.\n\n'+'\n'.join(sorted(inventory))+'\n')
print('Collected frontend notices:',len(seen))
