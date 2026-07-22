# -*- coding: utf-8 -*-
"""#2668 실측 워커 — 문서 1건. 한글 COM 크래시 격리를 위해 별도 프로세스로 실행.

usage: python fn_geom_worker.py <hwp경로> <출력json>

v2 변경: 구분선 아래 span 의 **텍스트/좌표를 보존**한다.
v1 에서 fn_bot 이 각주 본문이 아니라 **쪽번호**(꼬리말 밴드의 짧은 숫자)일 수
있다는 교란을 배제하기 위함. 쪽번호는 짧은 숫자 단독 span 으로 분리 집계한다.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"C:\Users\planet\rhwp")
EXE = str(ROOT / "target/debug/rhwp.exe")
PT2PX = 4.0 / 3.0
PAGENUM = re.compile(r"^[\s\-–—()<>[\]]*\d{1,4}[\s\-–—()<>[\]]*$")


def rhwp_areas(path):
    t = subprocess.run([EXE, "dump-pages", path], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=900).stdout
    return [(float(m.group(1)), float(m.group(1)) + float(m.group(2)))
            for m in re.finditer(
                r"body_area: x=[\d.-]+ y=([\d.-]+) w=[\d.-]+ h=([\d.-]+)", t)]


def measure(pdf, areas):
    import fitz
    d = fitz.open(pdf)
    rows = []
    for pno in range(d.page_count):
        pg = d[pno]
        body_top, body_bot = areas[pno] if pno < len(areas) else (
            areas[-1] if areas else (0.0, 0.0))
        pw = pg.rect.width * PT2PX
        seps = []
        for dr in pg.get_drawings():
            for it in dr["items"]:
                pts = None
                if it[0] == "l":
                    p0, p1 = it[1], it[2]
                    if abs(p0.y - p1.y) < 0.6:
                        pts = (p0.x * PT2PX, p1.x * PT2PX, p0.y * PT2PX)
                elif it[0] == "re":
                    r = it[1]
                    if r.height * PT2PX < 1.2:
                        pts = (r.x0 * PT2PX, r.x1 * PT2PX, r.y0 * PT2PX)
                if pts:
                    x0, x1 = sorted(pts[:2])
                    y, w = pts[2], x1 - x0
                    if y > (body_top + body_bot) / 2 and 0.12 * pw < w < 0.65 * pw:
                        seps.append(y)
        spans = []
        for blk in pg.get_text("dict")["blocks"]:
            for ln in blk.get("lines", []):
                for sp in ln["spans"]:
                    tx = sp["text"].strip()
                    if not tx:
                        continue
                    b = sp["bbox"]
                    spans.append({"t": tx, "x0": b[0] * PT2PX, "x1": b[2] * PT2PX,
                                  "y0": b[1] * PT2PX, "y1": b[3] * PT2PX,
                                  "sz": sp.get("size", 0)})
        if not spans:
            continue
        sep_y = None
        for y in sorted(seps, reverse=True):
            if any(s["y0"] > y - 1 for s in spans):
                sep_y = y
                break
        row = {"page": pno + 1, "body_top": body_top, "body_bot": body_bot,
               "page_h": pg.rect.height * PT2PX, "n_sep": len(seps)}
        if sep_y is None:
            row["fn"] = False
            rows.append(row)
            continue
        below = [s for s in spans if s["y0"] > sep_y - 1]
        above = [s for s in spans if s["y0"] <= sep_y - 1]
        if not below:
            row["fn"] = False
            rows.append(row)
            continue
        # 쪽번호 후보: 짧은 숫자 단독 + 그 줄에 다른 span 없음
        by_line = {}
        for s in below:
            by_line.setdefault(round(s["y1"], 1), []).append(s)
        pnum, fnspans = [], []
        for _y, ss in by_line.items():
            joined = "".join(x["t"] for x in ss)
            if len(ss) <= 2 and PAGENUM.match(joined):
                pnum.extend(ss)
            else:
                fnspans.extend(ss)
        row["fn"] = bool(fnspans)
        row["sep_y"] = sep_y
        row["body_bot_txt"] = max(s["y1"] for s in above) if above else None
        if pnum:
            row["pnum_bot"] = max(s["y1"] for s in pnum)
            row["pnum_txt"] = "|".join(s["t"] for s in pnum)[:20]
        if fnspans:
            row["fn_bot"] = max(s["y1"] for s in fnspans)
            row["fn_top"] = min(s["y0"] for s in fnspans)
            row["fn_sz"] = max(s["sz"] for s in fnspans)
            row["over"] = row["fn_bot"] - body_bot
            row["bot_txt"] = max(fnspans, key=lambda s: s["y1"])["t"][:30]
        rows.append(row)
    d.close()
    return rows


def main():
    src, out = sys.argv[1], sys.argv[2]
    tmp = str(Path(out).with_suffix(".pdf"))
    areas = rhwp_areas(src)
    from pyhwpx import Hwp
    hwp = Hwp(visible=False)
    try:
        if os.path.exists(tmp):
            os.remove(tmp)
        hwp.open(src)
        hwp.save_as(tmp, format="PDF")
        hwp.clear(option=1)
        rows = measure(tmp, areas)
    finally:
        try:
            hwp.quit()
        except Exception:  # noqa: BLE001
            pass
    json.dump({"path": src, "rhwp_pages": len(areas), "rows": rows},
              open(out, "w", encoding="utf-8"), ensure_ascii=False)
    if os.path.exists(tmp):
        os.remove(tmp)


if __name__ == "__main__":
    main()
