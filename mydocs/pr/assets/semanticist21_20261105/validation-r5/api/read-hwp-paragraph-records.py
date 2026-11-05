"""rhwp에 의존하지 않고 HWP5 본문 문단의 원시 레코드를 읽는다."""
from pathlib import Path
import hashlib
import json
import struct
import sys
import zlib

import olefile


def read_document(filename):
    path = Path(filename)
    with olefile.OleFileIO(path) as compound:
        header = compound.openstream('FileHeader').read()
        body = compound.openstream('BodyText/Section0').read()
    if header[36] & 1:
        body = zlib.decompress(body, -15)
    paragraphs = []
    offset = 0
    while offset < len(body):
        header = struct.unpack_from('<I', body, offset)[0]
        offset += 4
        tag, level, size = header & 1023, (header >> 10) & 1023, header >> 20
        if size == 4095:
            size = struct.unpack_from('<I', body, offset)[0]
            offset += 4
        data = body[offset:offset + size]
        offset += size
        assert len(data) == size, '레코드 경계'
        if tag == 66 and level == 0:
            paragraphs.append({
                'char_count': struct.unpack_from('<I', data)[0] & 0x7fffffff,
                'para_text_units': None,
                'line_starts': [],
            })
        elif paragraphs and level == 1:
            if tag == 67:
                paragraphs[-1]['para_text_units'] = list(
                    struct.unpack('<' + 'H' * (len(data) // 2), data)
                )
            elif tag == 69:
                paragraphs[-1]['line_starts'] = [
                    struct.unpack_from('<I', data, index)[0]
                    for index in range(0, len(data), 36)
                ]
    return {
        'path': str(path.resolve()),
        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'paragraphs': paragraphs,
    }


if __name__ == '__main__':
    if len(sys.argv) < 2:
        raise SystemExit('사용법: read-hwp-paragraph-records.py <HWP> [HWP ...]')
    print(json.dumps([read_document(name) for name in sys.argv[1:]],
                     ensure_ascii=False, indent=2))
