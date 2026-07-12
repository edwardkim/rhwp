# -*- coding: utf-8 -*-
# #2019 진단 하네스: nogo_sample.txt 각 문서의 rhwp dump-pages 페이지수 캡처.
# before/after 회귀 대조용. usage: python capture_pages.py <out.tsv>
import subprocess, os, sys
EXE = r'C:\Users\planet\t\rhwp\target\release\rhwp.exe'
BASE = os.path.dirname(os.path.abspath(__file__))
out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(BASE, 'pages.tsv')
files = [l.strip() for l in open(os.path.join(BASE, 'nogo_sample.txt'), encoding='utf-8') if l.strip()]

def pages(f):
    try:
        r = subprocess.run([EXE, 'dump-pages', f], capture_output=True, text=True,
                           encoding='utf-8', errors='replace', timeout=180)
        return sum(1 for ln in r.stdout.splitlines() if '=== 페이지' in ln)
    except Exception as e:
        return f'ERR:{type(e).__name__}'

with open(out, 'w', encoding='utf-8') as o:
    o.write('file\tpages\n')
    for f in files:
        o.write(f'{os.path.basename(f)}\t{pages(f)}\n')
print('done', len(files), '->', out)
