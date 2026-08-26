#!/usr/bin/env python3
"""[Issue #5162] 표를 감싼 0길이 누름틀의 FIELD_END 오배치 재현·관찰 스크립트.

HWPX 를 HWP5 로 저장할 때, 텍스트 없이 표 하나만 감싼 CLICK_HERE 누름틀은
텍스트 축 0길이라 HWP5 직렬화기의 후행 컨트롤 경로로 온다. 그 경로가 FIELD_END 를
감싼 표(개체) 앞에 찍으면 누름틀이 비고, 한글 2022 는 그 자리에 안내문을 본문으로
렌더한다(정품 SaveAs 는 안내문을 내지 않는다).

사용법:
    # 1) 표본 만들기 (samples/issue5162_field_wraps_table.hwpx 를 재생성)
    python tools/issue5162_field_wraps_table.py build

    # 2) HWP5 로 변환하고 PARA_TEXT 확장 제어 배치를 관찰
    #    수정 전: FIELD_BEGIN -> FIELD_END -> OBJ(tbl)  (표가 필드 밖 = 버그)
    #    수정 후: FIELD_BEGIN -> OBJ(tbl) -> FIELD_END  (표가 필드 안 = 정상)
    python tools/issue5162_field_wraps_table.py show <converted.hwp>

표본은 samples/issue2808_single_table_form_physical_ladder.hwpx 의 표를
issue1893 의 실제 CLICK_HERE fieldBegin/fieldEnd(신규 id)로 감싸 만든다.
"""
import os
import re
import struct
import sys
import zipfile
import zlib

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE_HWPX = os.path.join(REPO, "samples", "issue2808_single_table_form_physical_ladder.hwpx")
FIELD_SRC = os.path.join(REPO, "samples", "issue1893_clickhere_field_roundtrip.hwpx")
OUT_HWPX = os.path.join(REPO, "samples", "issue5162_field_wraps_table.hwpx")


def _read_section(hwpx):
    with zipfile.ZipFile(hwpx) as z:
        for n in z.namelist():
            if n.lower().endswith("section0.xml"):
                return n, z.read(n).decode("utf-8"), z.namelist()
    raise SystemExit("section0.xml 없음")


def build():
    _, base_sec, names = _read_section(BASE_HWPX)
    _, fsrc, _ = _read_section(FIELD_SRC)

    # issue1893 의 실제 fieldBegin 을 가져와 신규 id 로 치환한다.
    m = re.search(r"<hp:fieldBegin\b.*?</hp:fieldBegin>", fsrc, re.S)
    fb = (
        m.group(0)
        .replace('id="1549188898"', 'id="800000001"')
        .replace('fieldid="627272811"', 'fieldid="800000002"')
    )
    fe = '<hp:fieldEnd beginIDRef="800000001" fieldid="800000002"/>'

    # 표를 감싼 run 을 [ctrl fieldBegin][tbl][ctrl fieldEnd] 세 run 으로 교체한다.
    x = base_sec
    j = x.find("<hp:tbl")
    te = x.find("</hp:tbl>", j) + len("</hp:tbl>")
    run_start = x.rfind("<hp:run", 0, j)
    run_end = x.find("</hp:run>", te) + len("</hp:run>")
    cid_m = re.search(r'charPrIDRef="(\d+)"', x[run_start:run_end])
    cid = cid_m.group(1) if cid_m else "0"
    tbl = x[j:te]
    new_runs = (
        f'<hp:run charPrIDRef="{cid}"><hp:ctrl>{fb}</hp:ctrl></hp:run>'
        f'<hp:run charPrIDRef="{cid}">{tbl}<hp:t/></hp:run>'
        f'<hp:run charPrIDRef="{cid}"><hp:ctrl>{fe}</hp:ctrl></hp:run>'
    )
    new_sec = x[:run_start] + new_runs + x[run_end:]
    assert new_sec.count("<hp:fieldBegin") == 1 and new_sec.count("<hp:tbl") == 1

    sec_name, _, _ = _read_section(BASE_HWPX)
    with zipfile.ZipFile(BASE_HWPX) as zin, zipfile.ZipFile(OUT_HWPX, "w") as zout:
        for n in names:
            data = new_sec.encode("utf-8") if n == sec_name else zin.read(n)
            comp = zipfile.ZIP_STORED if n == "mimetype" else zipfile.ZIP_DEFLATED
            zout.writestr(n, data, compress_type=comp)
    print("wrote", OUT_HWPX, os.path.getsize(OUT_HWPX), "bytes")


def _records(buf):
    i = 0
    while i + 4 <= len(buf):
        h = struct.unpack("<I", buf[i:i + 4])[0]
        i += 4
        tag = h & 0x3FF
        sz = (h >> 20) & 0xFFF
        if sz == 0xFFF:
            sz = struct.unpack("<I", buf[i:i + 4])[0]
            i += 4
        yield tag, buf[i:i + sz]
        i += sz


def show(hwp_path):
    try:
        import olefile
    except ImportError:
        raise SystemExit("pip install olefile 필요")
    ole = olefile.OleFileIO(hwp_path)
    raw = ole.openstream("BodyText/Section0").read()
    ole.close()
    try:
        body = zlib.decompress(raw, -15)
    except zlib.error:
        body = raw
    names = {0x03: "FIELD_BEGIN", 0x04: "FIELD_END", 0x0B: "OBJ"}
    ext = set(range(1, 4)) | set(range(4, 9)) | set(range(11, 13)) | set(range(14, 24))
    pn = -1
    for tag, data in _records(body):
        if tag == 66:
            pn += 1
        if tag != 67:
            continue
        cus = struct.unpack("<%dH" % (len(data) // 2), data[: len(data) // 2 * 2])
        seq, k = [], 0
        while k < len(cus):
            ch = cus[k]
            if ch in ext:
                seq.append("%s@%d" % (names.get(ch, "x%02X" % ch), k))
                k += 8
            else:
                k += 1
        if any(t.startswith(("FIELD", "OBJ")) for t in seq):
            print("para#%d: %s" % (pn, " -> ".join(seq)))


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "build"
    if cmd == "build":
        build()
    elif cmd == "show":
        show(sys.argv[2])
    else:
        raise SystemExit(__doc__)
