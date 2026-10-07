#!/usr/bin/env python3
import sys,json,subprocess,shutil,base64,re
from pathlib import Path
r=Path(sys.argv[2]).resolve().parent
a=sys.argv[1:]
if a[0] in ('export-svg','export-render-tree'):
 assert r.name in ('registered-hcr','registered-arial','registered-hcr-fontspace')
 dest=Path(a[a.index('-o')+1]);dest.mkdir(parents=True,exist_ok=True)
 if a[0]=='export-svg':
  f=dest/(Path(a[1]).stem+'_001.svg');source=(r/'native_1.svg').read_text()
  font_file=Path('/usr/local/share/fonts/hwp-convert-mcp-survey')/('525979822591-Arial.ttf' if r.name=='registered-arial' else '8664669bd2d6-HANBatang.ttf')
  encoded=base64.b64encode(font_file.read_bytes()).decode()
  source=re.sub(r'(@font-face\s*\{[^}]*?src:)\s*[^;]+;',lambda m:m.group(1)+' url(data:font/ttf;base64,'+encoded+') format("truetype");',source)
  f.write_text(source)
  print(json.dumps({'pageCount':1,'renderedCount':1,'pages':[{'page':0,'path':str(f)}]}))
 else:
  shutil.copyfile(r/'native-tree_1.json',dest/'render_tree_001.json')
else:
 sys.exit(subprocess.run(['target/pr-review/release-test/rhwp',*a]).returncode)
