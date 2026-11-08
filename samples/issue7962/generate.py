"""Generate exact-size valid PDFs in an explicit local output directory."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def pdf(padding):
    content = b'BT /F1 12 Tf 20 100 Td (SIZE_BOUNDARY) Tj ET'
    objects = [
        b'<< /Type /Catalog /Pages 2 0 R >>',
        b'<< /Type /Pages /Count 1 /Kids [3 0 R] >>',
        b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
        b'<< /Length ' + str(len(content)).encode() + b' >>\nstream\n' + content + b'\nendstream',
        b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
        b'<< /Length ' + str(padding).encode() + b' >>\nstream\n' + b'\0' * padding + b'\nendstream',
    ]
    data = bytearray(b'%PDF-1.4\n')
    offsets = []
    for i, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data += str(i).encode() + b' 0 obj\n' + obj + b'\nendobj\n'
    xref = len(data)
    data += b'xref\n0 7\n0000000000 65535 f \n'
    for offset in offsets:
        data += f'{offset:010} 00000 n \n'.encode()
    data += f'trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode()
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--large', action='store_true', help='also generate local-only 64 MiB boundaries')
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    sizes = [16777215, 16777216, 16777217]
    if args.large:
        sizes += [67108863, 67108864, 67108865]
    results = []
    for target in sizes:
        padding = target - 1000
        for _ in range(10):
            data = pdf(padding)
            if len(data) == target:
                break
            padding += target - len(data)
        assert len(data) == target
        file = args.out / f'valid-pdf-{target}-bytes.pdf'
        if file.exists():
            if file.read_bytes() != data:
                raise ValueError(f'existing file differs: {file}')
        else:
            file.write_bytes(data)
        subprocess.run(['qpdf', '--check', str(file)], check=True, capture_output=True)
        results.append({'path': str(file.resolve()), 'bytes': target,
                        'sha256': hashlib.sha256(file.read_bytes()).hexdigest(), 'qpdfCheck': 'passed'})
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
