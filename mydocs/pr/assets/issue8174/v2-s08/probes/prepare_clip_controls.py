"""Independent PDF syntax controls; no RHWP rendering or private input content."""
from pathlib import Path
import hashlib
import json
import zlib

ROOT = Path(__file__).resolve().parents[6]
DEST = ROOT / 'pdf/issue8174/clip'


def stream(data, extra=''):
    return f'<< /Length {len(data)} {extra} >>\nstream\n'.encode() + data + b'\nendstream'


def document(contents):
    objects = [b'<< /Type /Catalog /Pages 2 0 R >>', b'',
               stream(zlib.compress(bytes([255, 0, 0]) * 10000),
                      '/Type /XObject /Subtype /Image /Width 100 /Height 100 '
                      '/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode')]
    kids = []
    for i, content in enumerate(contents):
        page = len(objects) + 1
        kids.append(f'{page} 0 R')
        encoded = content.encode()
        content_stream = (stream(zlib.compress(encoded), '/Filter /FlateDecode')
                          if len(encoded) > 1024 else stream(encoded))
        objects.extend([b'', content_stream])
        if i == 4:
            form = len(objects) + 1
            objects.append(stream(
                b'q -200 -200 m 400 400 m 20 20 80 80 re W n '
                b'100 0 0 100 10 10 cm /Im Do Q',
                '/Type /XObject /Subtype /Form /BBox [0 0 120 120] '
                '/Matrix [.75 .25 -.25 .75 30 0] '
                '/Resources << /XObject << /Im 3 0 R >> >>'))
            resources = f'/XObject << /Fm {form} 0 R >>'
        else:
            resources = '/XObject << /Im 3 0 R >>'
        objects[page-1] = (f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 120 120] '
                           f'/Resources << {resources} >> /Contents {page+1} 0 R >>').encode()
    objects[1] = f'<< /Type /Pages /Count {len(kids)} /Kids [{" ".join(kids)}] >>'.encode()
    output = bytearray(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(output))
        output.extend(f'{index} 0 obj\n'.encode() + obj + b'\nendobj\n')
    xref = len(output)
    output.extend(f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode())
    for offset in offsets[1:]:
        output.extend(f'{offset:010} 00000 n \n'.encode())
    output.extend(f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
    return bytes(output)


def main():
    DEST.mkdir(parents=True, exist_ok=True)
    rectangle = '20 20 m 100 20 l 100 100 l 20 100 l h W n '
    paint = '100 0 0 100 10 10 cm /Im Do Q'
    controls = {
        'consecutive-moves.pdf': [
            'q ' + rectangle + paint,
            'q -500 -500 m q Q 500 500 m ' + rectangle + paint,
            'q -500 -500 m 500 500 m 20 20 80 80 re W* n ' + paint,
            'q -500 -500 m 500 500 m 20 20 m 100 20 l 20 100 l h W n ' + paint,
            'q /Fm Do Q',
        ],
        'compound-rejected.pdf': ['q 20 20 30 30 re 70 70 30 30 re W n ' + paint],
        'curve-rejected.pdf': ['q 20 20 m 100 20 100 100 20 100 c h W n ' + paint],
        'batch-operator-control.pdf': ['q Q\n' * 80000 + 'q ' + rectangle + paint] * 2,
    }
    rows = []
    for name, contents in controls.items():
        data = document(contents)
        (DEST / name).write_bytes(data)
        rows.append({'path': str((DEST/name).relative_to(ROOT)), 'pages': len(contents),
                     'sha256': hashlib.sha256(data).hexdigest(),
                     'expected': 'unsupported' if 'rejected' in name else 'success'})
    print(json.dumps(rows, indent=2))


if __name__ == '__main__':
    main()
