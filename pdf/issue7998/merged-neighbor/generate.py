"""Explicit perimeter contract from public Hancom text/clip scaffolds.

The source Print extends horizontal pen endpoints for joins. These controls
deliberately terminate at the declared clip axes; they are not the original
Print and do not claim support for its overshooting decorations.
"""
from pathlib import Path
from decimal import Decimal
import argparse, hashlib, json, pikepdf

parser = argparse.ArgumentParser()
parser.add_argument('--source-dir', type=Path, required=True)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--top-print', type=Path, required=True)
args = parser.parse_args()
args.out.mkdir(parents=True, exist_ok=False)
records = []
for name in ['colspan', 'rowspan', 'rowspan-top']:
    source = args.top_print if name == 'rowspan-top' else args.source_dir / (name + '.pdf')
    with pikepdf.open(source) as pdf:
        ops = list(pikepdf.parse_content_stream(pdf.pages[0]))
        strokes = [i for i, op in enumerate(ops) if str(op.operator) == 'S']
        assert len(strokes) == 10
        for i, op in enumerate(ops):
            if str(op.operator) == 'w':
                assert op.operands[0] == Decimal('0.359000')
                ops[i] = pikepdf.ContentStreamInstruction([Decimal('0.5')], op.operator)
            if str(op.operator) in ['m', 'l']:
                operands = list(op.operands)
                endpoints = {Decimal('2.753000'): Decimal('2.873'),
                             Decimal('356.502000'): Decimal('356.263')}
                if name.startswith('rowspan'): endpoints[Decimal('179.448000')] = Decimal('179.568')
                if operands[0] in endpoints:
                    operands[0] = endpoints[operands[0]]
                    ops[i] = pikepdf.ContentStreamInstruction(operands, op.operator)
        ops = [op for i, op in enumerate(ops) if i not in strokes[:-1]]
        pdf.pages[0].Contents = pikepdf.Stream(pdf, pikepdf.unparse_content_stream(ops))
        dest = args.out / (name + '-retained.pdf')
        pdf.save(dest, deterministic_id=True)
        records.append({'name': dest.name, 'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                        'penPt': 0.5, 'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()})

with pikepdf.open(args.out / 'colspan-retained.pdf') as pdf:
    ops = list(pikepdf.parse_content_stream(pdf.pages[0]))
    at = max(i for i, op in enumerate(ops) if str(op.operator) == 'S') + 1
    ops[at:at] = pikepdf.parse_content_stream(pikepdf.Stream(
        pdf, b'0.9 w 179.568 357.12 m 356.263 357.12 l S'))
    pdf.pages[0].Contents = pikepdf.Stream(pdf, pikepdf.unparse_content_stream(ops))
    dest = args.out / 'colspan-varying-side.pdf'
    pdf.save(dest, deterministic_id=True)
    records.append({'name': dest.name, 'role': 'partial wider pen on a merged side; reject structural edit',
                    'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()})

# Deliberately widen only the paper. Inserting an original-width column then
# remains on the page; cell dimensions/text/line metrics are unchanged.
for dest in args.out.glob('*.pdf'):
    with pikepdf.open(dest) as pdf:
        pdf.pages[0].MediaBox = [0, 0, 600, 360]
        if '/CropBox' in pdf.pages[0].obj: pdf.pages[0].CropBox = [0, 0, 600, 360]
        temporary = dest.with_suffix('.wide.pdf')
        pdf.save(temporary, deterministic_id=True)
    temporary.replace(dest)
for record in records: record['sha256'] = hashlib.sha256((args.out/record['name']).read_bytes()).hexdigest()
(args.out / 'inputs.json').write_text(json.dumps({'pikepdfVersion': pikepdf.__version__, 'paperPt': [600, 360], 'inputs': records}, indent=2) + '\n')
