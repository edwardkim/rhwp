#!/usr/bin/env python3
"""[Issue #5169] HWP5 의 BodyText 와 ViewText 컨트롤/글자 인구를 대조한다.

배포용(FileHeader 0x04) 이 아닌데도 ViewText 가 있는 문서(예: 변경 추적 0x4000)는,
한글이 ViewText 를 렌더한다. 종전 rhwp 는 배포용일 때만 ViewText 를 읽어 BodyText(=한글이
렌더하지 않는 판본)를 읽었다. 이 스크립트로 두 스트림의 표/개체/글자 수 차이를 확인한다.

    python tools/issue5169_bodytext_vs_viewtext.py samples/issue5169_viewtext_changetracking.hwp

기대 출력(재현 표본): BodyText tbl=4 chars=3540 vs ViewText tbl=10 gso=2 chars=10210,
FileHeader distribution=0 changetrack=1 → ViewText 우선이 정답.
"""
import struct
import sys
import zlib

try:
    import olefile
except ImportError:
    raise SystemExit("pip install olefile 필요")


def census(body):
    i = 0
    tbl = gso = secd = chars = 0
    while i + 4 <= len(body):
        h = struct.unpack("<I", body[i:i + 4])[0]
        i += 4
        tag = h & 0x3FF
        sz = (h >> 20) & 0xFFF
        if sz == 0xFFF:
            sz = struct.unpack("<I", body[i:i + 4])[0]
            i += 4
        d = body[i:i + sz]
        if tag == 67:  # PARA_TEXT (UTF-16 code units)
            chars += sz // 2
        elif tag == 71 and sz >= 4:  # CTRL_HEADER, fourcc little-endian
            cid = d[:4]
            if cid == b" lbt":
                tbl += 1
            elif cid == b" osg":
                gso += 1
            elif cid == b"dces":
                secd += 1
        i += sz
    return tbl, gso, secd, chars


def main(path):
    o = olefile.OleFileIO(path)
    fh = o.openstream("FileHeader").read()
    flags = struct.unpack("<I", fh[36:40])[0]
    print(
        "FileHeader flags=0x%08x compressed=%d encrypted=%d distribution=%d changetrack=%d"
        % (flags, flags & 1, bool(flags & 2), bool(flags & 4), bool(flags & 0x4000))
    )
    compressed = bool(flags & 1)
    names = set("/".join(p) for p in o.listdir())
    for stream in ("BodyText/Section0", "ViewText/Section0"):
        if stream not in names:
            print("%-20s (없음)" % stream)
            continue
        raw = o.openstream(stream).read()
        try:
            body = zlib.decompress(raw, -15) if compressed else raw
        except zlib.error:
            print("%-20s (deflate 실패 — 스텁/암호화, BodyText 로 폴백해야 함)" % stream)
            continue
        tbl, gso, secd, chars = census(body)
        print("%-20s tbl=%d gso=%d secd=%d chars=%d" % (stream, tbl, gso, secd, chars))
    o.close()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    main(sys.argv[1])
