# -*- coding: utf-8 -*-
"""#2279 선결1 — footer 흡수/분할 전환점 실측 워커 (문서 1건).

하단 여백(pagePr/margin bottom, HWPUNIT)을 이분 탐색으로 스윕하며 한글
warm-open PageCount 의 전환점(m*)을 찾는다. 변형본은 HWPX zip 의
section0.xml margin bottom 만 치환해 생성(저장 lineseg 사다리 불변).

warm 프로토콜: 새 인스턴스에서 더미 문서를 먼저 열어 세션을 데운 뒤 측정
(#2138 stage1 fresh/warm 상태 의존성 — 권위는 warm).

usage: python footer_flip_worker.py <doc.hwpx> <dummy.hwpx> <tmpdir>
stdout 마지막 줄에 JSON 결과 1줄.

요구: Windows + 한컴오피스 + pyhwpx.
"""
import json
import re
import subprocess
import sys
import time
import zipfile
from pathlib import Path

PRECISION_HU = 10  # ~0.13px
HI_EXTEND_HU = 16000  # 흡수 문서의 상방 탐색 한계 (+213px)

MARGIN_RE = re.compile(r'(<hp:margin[^>]*bottom=")(\d+)(")')


def main() -> None:
    doc, dummy, tmpdir = sys.argv[1], sys.argv[2], sys.argv[3]
    doc_id = Path(doc).name.split("_")[0]
    sec = "Contents/section0.xml"

    with zipfile.ZipFile(doc) as zin:
        infos = zin.infolist()
        data = {i.filename: zin.read(i.filename) for i in infos}
    xml = data[sec].decode("utf-8")
    m = MARGIN_RE.search(xml)
    if not m:
        print(json.dumps({"doc": doc_id, "error": "margin not found"}))
        return
    margin0 = int(m.group(2))

    def variant(mval: int) -> str:
        out = Path(tmpdir) / f"{doc_id}_m{mval}.hwpx"
        if not out.exists():
            new_xml = xml[: m.start(2)] + str(mval) + xml[m.end(2):]
            with zipfile.ZipFile(out, "w") as zo:
                for zi in infos:
                    payload = new_xml.encode("utf-8") if zi.filename == sec else data[zi.filename]
                    zo.writestr(zi, payload, zi.compress_type)
        return str(out)

    from pyhwpx import Hwp

    hwp = None
    for _ in range(3):
        try:
            hwp = Hwp(visible=False)
            break
        except Exception:  # noqa: BLE001
            subprocess.run(["taskkill", "/IM", "Hwp.exe", "/F"], capture_output=True)
            time.sleep(5)
    if hwp is None:
        print(json.dumps({"doc": doc_id, "error": "Hwp instance creation failed x3"}))
        return

    opens = 0
    cache: dict[int, int] = {}

    def pc(mval: int) -> int:
        nonlocal opens
        if mval in cache:
            return cache[mval]
        hwp.open(variant(mval))
        opens += 1
        n = hwp.PageCount
        cache[mval] = n
        return n

    res = {"doc": doc_id, "path": doc, "margin0": margin0}
    try:
        hwp.open(dummy)  # warm-up
        p0 = pc(margin0)
        res["p0_warm"] = p0
        if p0 < 1:
            res["error"] = f"bad p0 {p0}"
            print(json.dumps(res))
            return
        # 전환 술어 S(m): "분할 상태" = PageCount >= split_pages. m* = inf{ m : S(m) }.
        # 여백을 줄이면 body 가 커져 흡수(쪽수 감소), 늘리면 분할(증가).
        split_pages = None
        if pc(0) < p0:
            split_pages, lo, hi = p0, 0, margin0
        elif pc(margin0 + HI_EXTEND_HU) > p0:
            split_pages, lo, hi = p0 + 1, margin0, margin0 + HI_EXTEND_HU
        else:
            res.update(flip="NOFLIP", pc_at_0=cache.get(0),
                       pc_at_hi=cache.get(margin0 + HI_EXTEND_HU), opens=opens)
            print(json.dumps(res, ensure_ascii=False))
            return
        while hi - lo > PRECISION_HU:
            mid = (lo + hi) // 2
            if pc(mid) >= split_pages:
                hi = mid
            else:
                lo = mid
        res.update(flip="OK", split_pages=split_pages, m_lo_absorb=lo, m_hi_split=hi,
                   m_star=(lo + hi) / 2.0, delta_px=((lo + hi) / 2.0 - margin0) / 75.0,
                   opens=opens)
    except Exception as e:  # noqa: BLE001
        res["error"] = repr(e)
    finally:
        try:
            hwp.quit()
        except Exception:  # noqa: BLE001
            pass
    print(json.dumps(res, ensure_ascii=False))


if __name__ == "__main__":
    main()
