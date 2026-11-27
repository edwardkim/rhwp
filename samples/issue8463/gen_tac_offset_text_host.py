#!/usr/bin/env python3
"""[#8463] 저장 줄 없는 host 의 양수 세로 오프셋 TAC 표: 한컴 저장본을 레코드 단위로 고친 공개 재현물.

  python3 -I samples/issue8463/gen_tac_offset_text_host.py \\
      samples/issue6861/3194097-performance-rating-roster.hwp samples/issue8463/tac-offset-text-host.hwp

바탕: 한컴오피스 2020(11.0.0.8969) 저장 HWP5 공개 샘플(1쪽). 문단 2 는 2×1 TAC 표(칸 안에 중첩 표)
하나와 문단 끝만 갖는다. BodyText/Section0 에서 다음만 바꾼다.
  1. 본문(최상위) 문단의 PARA_LINE_SEG(레벨 1)만 지우고 그 PARA_HEADER 의 줄 수를 0 으로 둔다.
     칸 안 문단의 저장 줄은 남긴다. 칸 줄까지 지우면 rhwp 가 칸 문단 줄 높이를 다시 계산해 중첩 표가
     2~3px 위에 그려지는 별개 차이가 섞인다(쪽 배분은 같고, 한컴은 원본과 같은 자리에 그린다).
  2. 문단 2 첫 표 CTRL_HEADER 의 세로 오프셋을 3000HU 로 둔다(4바이트 제자리).
  3. 문단 2 PARA_TEXT 의 문단 끝 앞에 공백 1칸을 넣고 PARA_HEADER 글자 수를 1 늘린다.
다른 레코드의 바이트·순서, FileHeader·DocInfo 등 다른 스트림, OLE(CFB) 섹터 배치는 원본 그대로다.
새 Section0 은 원래 스트림 길이에 정확히 맞춰 같은 자리에 쓴다(deflate 를 sync flush 로 끝낸 뒤 빈
deflate 블록으로 채운다. 끝까지 유효한 deflate 이며 0 을 덧붙이지 않는다). CFB 를 새로 쓴 파일은
한컴 2020 이 열지 못했다. 필요: olefile.
"""
import shutil
import struct
import sys
import zlib

import olefile

PARA_HEADER, PARA_TEXT, PARA_LINE_SEG, CTRL_HEADER = 66, 67, 69, 71
LINE_COUNT_OFFSET = 16  # nchars 4 + ctrl mask 4 + shape 2 + style 1 + break 1 + cs 2 + range 2
HOST, V_OFFSET, SPACES = 2, 3000, 1


def records(buf):
    pos, out = 0, []
    while pos + 4 <= len(buf):
        h = struct.unpack_from('<I', buf, pos)[0]
        tag, level, size, hdr = h & 0x3FF, (h >> 10) & 0x3FF, h >> 20, 4
        if size == 0xFFF:
            size = struct.unpack_from('<I', buf, pos + 4)[0]
            hdr = 8
        out.append([tag, level, bytearray(buf[pos + hdr:pos + hdr + size])])
        pos += hdr + size
    assert pos == len(buf), '레코드 경계'
    return out


def serialize(recs):
    out = bytearray()
    for tag, level, data in recs:
        size = len(data)
        out += struct.pack('<I', tag | (level << 10) | (min(size, 0xFFF) << 20))
        if size >= 0xFFF:
            out += struct.pack('<I', size)
        out += data
    return bytes(out)


def edit(plain):
    keep, top, header, table_done = [], -1, None, False
    for rec in records(plain):
        tag, level, data = rec
        if tag == PARA_HEADER and level == 0:
            top += 1
        if tag == PARA_LINE_SEG and level == 1:
            struct.pack_into('<H', header[2], LINE_COUNT_OFFSET, 0)
            continue
        if top == HOST and level == 1:
            if tag == CTRL_HEADER and data[0:4] == b' lbt' and not table_done:
                struct.pack_into('<i', data, 8, V_OFFSET)
                table_done = True
            if tag == PARA_TEXT:
                assert data[-2:] == b'\r\x00', '문단 끝'
                rec[2] = data[:-2] + b' \x00' * SPACES + b'\r\x00'
                nchars = struct.unpack_from('<I', header[2], 0)[0]
                struct.pack_into('<I', header[2], 0, (nchars & 0x8000_0000) | ((nchars & 0x7FFF_FFFF) + SPACES))
        if tag == PARA_HEADER:
            header = rec
        keep.append(rec)
    assert table_done, '문단 2 표'
    return serialize(keep)


def empty_tail(k, final_fixed):
    """비마지막 고정 허프만 빈 블록 k 개 + 마지막 빈 블록(고정 또는 저장)."""
    bits = []
    for _ in range(k):
        bits += [0, 1, 0] + [0] * 7
    bits += [1, 1, 0] + [0] * 7 if final_fixed else [1, 0, 0]
    bits += [0] * (-len(bits) % 8)
    out = bytes(sum(b << i for i, b in enumerate(bits[j:j + 8])) for j in range(0, len(bits), 8))
    return out if final_fixed else out + b'\x00\x00\xff\xff'


def fit(plain, length):
    for level in range(9, 0, -1):
        c = zlib.compressobj(level, zlib.DEFLATED, -15)
        head = c.compress(plain) + c.flush(zlib.Z_SYNC_FLUSH)
        pad = length - len(head)
        for m in range(max(pad // 5, 0) + 1):
            for k in range(8):
                for final_fixed in (False, True):
                    data = head + b'\x00\x00\x00\xff\xff' * m + empty_tail(k, final_fixed)
                    if len(data) == length:
                        assert zlib.decompress(data, -15) == plain
                        return data
    raise SystemExit('원래 스트림 길이에 맞출 수 없다')


src, out = sys.argv[1:3]
ole = olefile.OleFileIO(src)
assert struct.unpack_from('<I', ole.openstream('FileHeader').read(), 36)[0] & 1, '압축 문서'
raw = ole.openstream('BodyText/Section0').read()
ole.close()
new = fit(edit(zlib.decompress(raw, -15)), len(raw))
shutil.copyfile(src, out)
ole = olefile.OleFileIO(out, write_mode=True)
ole.write_stream('BodyText/Section0', new)
ole.close()
