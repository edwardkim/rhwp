"""HWP5 셀 LIST_HEADER 확장 플래그(bytes 6-7 상위 바이트) 플립 실험용 패처 (#1831).

인과 실험 인프라: 셀 플래그 비트를 바꾼 변형 .hwp 를 만들어 한글 렌더링 차이를
관찰한다. 압축 스트림은 deflate 뒤 꼬리 바이트를 보존해 재기록한다(미보존 시
문서가 다르게 해석된 것으로 오인하는 실험 오염 발생 — #1831 교훈).

사용: python tools/patch_cell_flags.py <src.hwp> <dst.hwp> <clear|clearat|set> [row|ordinal]
  clear  : 모든 셀의 b7 bit2 를 클리어
  clearat: b7 bit2 세트 셀 중 ordinal 번째 하나만 클리어
  set    : row(기본 0) 행 셀들의 b7 bit2 를 세트
pythoncom StructuredStorage 로 스트림 크기 변경 재기록. 요구: Windows + pywin32 + olefile.
"""
import shutil
import struct
import sys
import zlib

import olefile
import pythoncom
from win32com import storagecon

HWPTAG_TABLE = 77
HWPTAG_LIST_HEADER = 72


def patch_section(data: bytes, mode: str, target_row: int) -> tuple[bytes, int]:
    buf = bytearray(data)
    pos, n, patched = 0, len(data), 0
    tl = None
    flagged_ordinal = -1  # b7&0x04 인 셀의 등장 순번 (clearat 모드용)
    while pos + 4 <= n:
        (hdr,) = struct.unpack_from("<I", buf, pos)
        tag, level, size = hdr & 0x3FF, (hdr >> 10) & 0x3FF, (hdr >> 20) & 0xFFF
        pos += 4
        if size == 0xFFF:
            (size,) = struct.unpack_from("<I", buf, pos)
            pos += 4
        if tag == HWPTAG_TABLE:
            tl = level
        elif tl is not None and level < tl:
            tl = None
        if tag == HWPTAG_LIST_HEADER and tl is not None and level == tl and size >= 40:
            row = buf[pos + 10] | (buf[pos + 11] << 8)
            if buf[pos + 7] & 0x04:
                flagged_ordinal += 1
            if mode == "clear" and buf[pos + 7] & 0x04:
                buf[pos + 7] &= ~0x04
                patched += 1
            elif mode == "clearat" and buf[pos + 7] & 0x04 and flagged_ordinal == target_row:
                buf[pos + 7] &= ~0x04
                patched += 1
            elif mode == "set" and row == target_row and not (buf[pos + 7] & 0x04):
                buf[pos + 7] |= 0x04
                patched += 1
        pos += size
    return bytes(buf), patched


def main():
    src, dst, mode = sys.argv[1], sys.argv[2], sys.argv[3]
    target_row = int(sys.argv[4]) if len(sys.argv) > 4 else 0
    shutil.copyfile(src, dst)

    ole = olefile.OleFileIO(src)
    header = ole.openstream("FileHeader").read()
    compressed = header[36] & 1
    sections = {}
    for entry in ole.listdir():
        if len(entry) == 2 and entry[0] == "BodyText":
            raw = ole.openstream(entry).read()
            if compressed:
                do = zlib.decompressobj(-15)
                data = do.decompress(raw)
                tail = do.unused_data  # deflate 스트림 뒤 꼬리 바이트 보존 (한글이 참조)
            else:
                data, tail = raw, b""
            patched_data, cnt = patch_section(data, mode, target_row)
            if cnt:
                if compressed:
                    co = zlib.compressobj(9, zlib.DEFLATED, -15)
                    out = co.compress(patched_data) + co.flush() + tail
                else:
                    out = patched_data
                sections[entry[1]] = out
                print(f"{entry[1]}: {cnt} cells patched, {len(raw)} -> {len(out)} bytes (tail {len(tail)}B)")
    ole.close()

    if not sections:
        print("패치 대상 없음")
        return

    m = storagecon.STGM_READWRITE | storagecon.STGM_SHARE_EXCLUSIVE
    stg = pythoncom.StgOpenStorage(dst, None, m)
    body = stg.OpenStorage("BodyText", None, m)
    for name, out in sections.items():
        stm = body.OpenStream(name, None, m)
        stm.SetSize(len(out))
        stm.Write(out)
    body.Commit(0)
    stg.Commit(0)
    print(f"저장: {dst}")


if __name__ == "__main__":
    main()
