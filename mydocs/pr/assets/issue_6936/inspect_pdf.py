"""Reproduce #6936's PDF operator/text/visual evidence. Requires pypdf, pymupdf, Pillow.
Run from repository root: python mydocs/pr/assets/issue_6936/inspect_pdf.py
"""
from collections import Counter
import hashlib
import json
from pathlib import Path
import pymupdf
from PIL import Image, ImageDraw
from pypdf import PdfReader
from pypdf.generic import ContentStream

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
PDFS = {
    'hancom': 'pdf/issue6936-bold-faces-2020.pdf',
    'before': 'pdf/issue6936-before.pdf',
    'stroke_only': 'pdf/issue6936-stroke-only.pdf',
    'after': 'pdf/issue6936-after.pdf',
}

def inspect(path):
    reader = PdfReader(path)
    modes = Counter()
    shows = 0
    widths = set()
    fonts = set()
    def walk(stream, resources):
        nonlocal shows
        for f in resources.get('/Font', {}).values():
            fonts.add(str(f.get_object().get('/BaseFont')))
        for operands, op in ContentStream(stream, reader).operations:
            if op == b'Tr': modes[int(operands[0])] += 1
            if op in (b'Tj', b'TJ', b"'", b'"'): shows += 1
            if op == b'w': widths.add(float(operands[0]))
            if op == b'Do':
                child = resources['/XObject'][operands[0]].get_object()
                if child.get('/Subtype') == '/Form':
                    walk(child, child.get('/Resources', resources))
    text = '\n'.join(page.extract_text() for page in reader.pages)
    for page in reader.pages: walk(page.get_contents(), page['/Resources'])
    return {'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'pages': len(reader.pages), 'text': text, 'nonWhitespaceCharacters': len(''.join(text.split())),
            'Tr': dict(modes), 'textShowOps': shows, 'strokeWidths': sorted(widths), 'fonts': sorted(fonts)}

results = {name: inspect(ROOT / path) for name, path in PDFS.items()}
expected = ''.join(results['hancom']['text'].split())
for name in ['before', 'after']:
    assert ''.join(results[name]['text'].split()) == expected, name
assert results['stroke_only']['nonWhitespaceCharacters'] > len(expected)
assert results['after']['textShowOps'] == len(expected)
assert results['after']['Tr'].get(2) == 49
assert all(r['pages'] == 1 for r in results.values())

# Raw glyph origins, without alignment or geometric tolerances. New Gulim is the
# sole intentional font-name change; other rows must preserve before/after origins.
def chars(path):
    with pymupdf.open(path) as doc:
        return [c for b in doc[0].get_text('rawdict')['blocks'] if 'lines' in b
                for line in b['lines'] for span in line['spans'] for c in span['chars']]
old, new = chars(ROOT / PDFS['before']), chars(ROOT / PDFS['after'])
assert ''.join(c['c'] for c in old) == ''.join(c['c'] for c in new)
assert len(old) == len(new)
geometry = {'glyphsIncludingSpaces': len(old), 'changedOrigins': [], 'maxOriginDeltaPt': 0}
for i, (a, b) in enumerate(zip(old, new)):
    delta = max(abs(x-y) for x,y in zip(a['origin'],b['origin']))
    geometry['maxOriginDeltaPt'] = max(geometry['maxOriginDeltaPt'], delta)
    if delta > 0:
        geometry['changedOrigins'].append({'index': i, 'text': a['c'], 'before': a['origin'], 'after': b['origin']})
# Saved fixture baselines identify the two New Gulim rows (regular/bold).
with pymupdf.open(ROOT/PDFS['before']) as doc:
    lines = [line for block in doc[0].get_text('dict')['blocks'] if 'lines' in block for line in block['lines']]
new_gulim_baselines = {lines[i]['spans'][0]['origin'][1] for i in (6, 7)}
assert all(item['before'][1] in new_gulim_baselines for item in geometry['changedOrigins'])
geometry['otherFontRowsChangedOrigins'] = 0
results['beforeAfterGeometry'] = geometry
(OUT / 'pdf-analysis.json').write_text(json.dumps(results, ensure_ascii=False, indent=2)+'\n')

# Full text region at 144 DPI, no reflow or coordinate alignment. All twelve rows.
images = {}
for name, path in PDFS.items():
    with pymupdf.open(ROOT/path) as doc:
        pix = doc[0].get_pixmap(matrix=pymupdf.Matrix(2, 2), alpha=False)
        im = Image.frombytes('RGB', [pix.width,pix.height], pix.samples)
        im.save(OUT / (name+'.png'))
        images[name] = im.crop((140, 195, 750, 1020))
canvas = Image.new('RGB', (1830, 865), 'white')
draw = ImageDraw.Draw(canvas)
for i, name in enumerate(['hancom', 'before', 'after']):
    canvas.paste(images[name], (i*610, 40))
    draw.text((i*610+12, 12), {'hancom':'Hancom 12.0.0.4605', 'before':'devel 70bf40af2 (before)', 'after':'#6936 candidate (after)'}[name], fill='black')
canvas.save(OUT / 'comparison.png')
print(json.dumps({k:{field:v[field] for field in ['pages','nonWhitespaceCharacters','Tr','textShowOps']} for k,v in results.items() if k != 'beforeAfterGeometry'},ensure_ascii=False,indent=2))
print('Before/after glyph origin max delta (pt):',geometry['maxOriginDeltaPt'])
