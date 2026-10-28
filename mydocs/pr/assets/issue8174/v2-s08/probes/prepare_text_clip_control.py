"""Independent PDF syntax and embedded DejaVu TTF; no RHWP rendering."""
from pathlib import Path
import hashlib
import json
import pymupdf as fitz

ROOT = Path(__file__).resolve().parents[6]
FONT = Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf')
DEST = ROOT / 'pdf/issue8174/clip/text-consecutive-moves.pdf'


def main():
    assert FONT.is_file(), FONT
    doc = fitz.open()
    rectangle = '20 20 260 100 re W n'
    prefixes = [
        rectangle,
        '0 0 m q Q 500 500 m 20 20 m 280 20 l 280 120 l 20 120 l h W n',
        '0 0 m 500 500 m ' + rectangle,
    ]
    for prefix in prefixes:
        page = doc.new_page(width=300, height=140)
        page.insert_font(fontname='ClipSans', fontfile=str(FONT))
        page.insert_text((30, 70), 'RECTANGLE ABCDEF', fontname='ClipSans', fontsize=14)
        content = b''.join(doc.xref_stream(x) for x in page.get_contents())
        xref = doc.get_new_xref()
        doc.update_object(xref, '<< >>')
        doc.update_stream(xref, ('q ' + prefix + '\n').encode() + content + b'\nQ\n')
        page.set_contents(xref)
    doc.subset_fonts()
    DEST.parent.mkdir(parents=True, exist_ok=True)
    doc.save(DEST, garbage=4, deflate=True, no_new_id=True)
    print(json.dumps({'path': str(DEST.relative_to(ROOT)), 'pages': 3,
                      'sha256': hashlib.sha256(DEST.read_bytes()).hexdigest(),
                      'font': {'path': str(FONT),
                               'sha256': hashlib.sha256(FONT.read_bytes()).hexdigest()}}, indent=2))


if __name__ == '__main__':
    main()
