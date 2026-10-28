#!/usr/bin/env python3
import sys,json,subprocess,shutil,hashlib
from pathlib import Path
a=sys.argv[1:];p=Path(a[1]).resolve();r=p.parent
root=Path('output/pr-review/semanticist21-20261005')
source=root/'font-contract-wasm-export'/r.name
if a[0] in ('export-svg','export-render-tree'):
 assert r.name in ('registered-hcr','registered-arial','registered-hcr-fontspace')
 m=json.loads((source/'manifest.json').read_text())
 assert hashlib.sha256(p.read_bytes()).hexdigest()==m['inputSha256']
 dest=Path(a[a.index('-o')+1]);dest.mkdir(parents=True,exist_ok=True)
 if a[0]=='export-svg':
  f=dest/(p.stem+'_001.svg');shutil.copyfile(source/'raw_svg/wasm_001.svg',f)
  print(json.dumps({'pageCount':1,'renderedCount':1,'pages':[{'page':0,'path':str(f)}]}))
 else:shutil.copyfile(source/'render_tree/render_tree_001.json',dest/'render_tree_001.json')
else:sys.exit(subprocess.run(['target/pr-review/release-test/rhwp',*a]).returncode)
