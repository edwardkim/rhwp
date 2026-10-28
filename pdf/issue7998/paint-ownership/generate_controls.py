"""Recreate explicit PDF paint controls from the published normal Hancom Print.
Usage: python3 pdf/issue7998/paint-ownership/generate_controls.py --output /tmp/new-controls
PDF container IDs can vary; origin.json pins the actual validated input bytes.
"""
import argparse
import json
from pathlib import Path
import pikepdf

args = argparse.ArgumentParser()
args.add_argument('--output', type=Path, required=True)
output = args.parse_args().output
output.mkdir(parents=True, exist_ok=True)
base = Path(__file__).resolve().parent
original = base.parent / 'local-regions/local-outer-only.pdf'
for item in json.loads((base / 'origin.json').read_text()):
    with pikepdf.open(original) as pdf:
        page = pdf.pages[0]
        content = (page.Contents.read_bytes() if isinstance(page.Contents, pikepdf.Stream)
                   else b'\n'.join(stream.read_bytes() for stream in page.Contents))
        rect = ' '.join(f'{n:.6f}' for n in item['rectanglePdfPt'])
        page.Contents = pikepdf.Stream(pdf, content + f'\nq 1 0 0 rg {rect} re f Q\n'.encode())
        pdf.save(output / (item['key'] + '.pdf'))
