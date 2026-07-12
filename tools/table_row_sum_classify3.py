"""#2110 v3 — v2 + 한글 측 중첩 표 필터 (앵커 List==0 = 본문 최상위).

v2(table_row_sum_classify2.py)의 잔여 한계 — 한글 HeadCtrl 이 셀/글상자 내부의
중첩 표까지 열거해 다표 문서에서 표 개수 불일치(예: rhwp 45 vs 한글 479)로 서수
매칭이 깨지던 24건 — 를 해소한다.

판별자: `ctrl.GetAnchorPos(0)` 의 ListParaPos `List` 항목. 본문(모든 구역 공통)
앵커는 List==0, 셀·글상자 내부 앵커는 해당 리스트 id(>0). 검증:
  - 20496055 (1구역): 전체 tbl 27 → List==0 정확히 1 = rhwp 최상위 1
  - 68874 (10구역): 전체 tbl 43 → List==0 정확히 39 = rhwp 최상위 39
    (구역 1+ 본문도 List==0 — 본문 리스트는 구역 전체에서 단일)

나머지 파이프라인(rhwp dump/dump-pages 측정, 서수 매칭, 분류)은 v2와 동일.

사용: python tools/table_row_sum_classify3.py --list <경로목록.txt> [--exe rhwp] -o out.tsv
요구: Windows + 한컴 + pyhwpx.
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOL = 8.0
MM_TO_PX = 96.0 / 25.4
HU_TO_PX = 96.0 / 7200.0

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8", errors="replace")

PARA = re.compile(r"^--- 문단 (\d+)\.(\d+) ---")
TBL = re.compile(r"^\s*\[(\d+)\] 표: (\d+)행×(\d+)열")
DECL = re.compile(r"size=\d+×(\d+)\(")
DRIFT = re.compile(r"TABLE_DRIFT: pi=(\d+) sec=(\d+) eff_h=([\d.]+) .* mt_sum=([\d.]+) mt_rows=(\d+)")


def rhwp_tables(exe: str, doc: str):
    """dump → 문서 순서의 최상위 표 [(sec, pi, rows, declared_px)]. 내부표 제외."""
    out = subprocess.run([exe, "dump", doc], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", timeout=240)
    tables = []
    sec = pi = None
    pending = None  # (sec, pi, rows) — 다음 size= 대기
    for ln in out.stdout.splitlines():
        m = PARA.match(ln)
        if m:
            sec, pi = int(m.group(1)), int(m.group(2))
            continue
        if "내부표" in ln:
            continue
        m = TBL.match(ln)
        if m and sec is not None:
            pending = (sec, pi, int(m.group(2)))
            continue
        if pending:
            d = DECL.search(ln)
            if d:
                tables.append((*pending, int(d.group(1)) * HU_TO_PX))
                pending = None
    return tables


def rhwp_measured(exe: str, doc: str):
    """dump-pages(RHWP_TABLE_DRIFT) → {(sec,pi): mt_sum_px}."""
    env = dict(os.environ, RHWP_TABLE_DRIFT="1")
    out = subprocess.run([exe, "dump-pages", doc], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", env=env, timeout=240)
    res = {}
    for m in DRIFT.finditer(out.stdout + out.stderr):
        key = (int(m.group(2)), int(m.group(1)))
        res.setdefault(key, float(m.group(4)))
    return res


def hangul_top_tables(doc: str):
    """COM 1세션 — 본문 최상위(tbl 앵커 List==0) 표의 (행수, 합 px) 문서 순서 리스트."""
    from pyhwpx import Hwp

    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    hwp = Hwp(new=True, visible=False)
    try:
        hwp.open(doc)
        # 1) 앵커 선수집 + 중첩 필터 (순회 중 캐럿 이동으로 ctrl 체인이 흔들리지 않게)
        anchors = []
        n_all = 0
        ctrl = hwp.HeadCtrl
        while ctrl is not None:
            if ctrl.CtrlID == "tbl":
                n_all += 1
                try:
                    pos = ctrl.GetAnchorPos(0)
                    if pos.Item("List") == 0:
                        anchors.append(pos)
                except Exception:  # noqa: BLE001
                    pass
            ctrl = ctrl.Next
        # 2) 표별 행높이 — 실패 표는 (0, None) 로 기록하고 계속
        out = []
        for pos in anchors:
            try:
                hwp.SetPosBySet(pos)
                hwp.FindCtrl()
                if not hwp.ShapeObjTableSelCell():
                    out.append((0, None))
                    continue
                n = int(hwp.get_row_num())
                hwp.Cancel()
                hs = []
                for _ in range(n):
                    hs.append(float(hwp.get_row_height()))
                    if not hwp.TableLowerCell():
                        break
                hwp.Cancel()
                out.append((len(hs), sum(hs) * MM_TO_PX))
            except Exception:  # noqa: BLE001
                out.append((0, None))
        return out, n_all
    finally:
        try:
            hwp.quit()
        except Exception:
            pass
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)


def classify_table(decl, hangul, rhwp):
    if rhwp is None or hangul is None:
        return "NO_DATA", 0.0
    d = rhwp - hangul
    if abs(d) <= TOL:
        return "ALIGNED", d
    if d > TOL:
        return "RHWP_INFLATE", d
    if decl is not None and abs(rhwp - decl) <= TOL:
        return "HANGUL_SHRINK", d
    return "MIXED", d


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", required=True)
    ap.add_argument("--exe", default=str(ROOT / "target/release/rhwp.exe"))
    ap.add_argument("-o", "--output", required=True)
    a = ap.parse_args()
    docs = [l.strip() for l in open(a.list, encoding="utf-8-sig") if l.strip()]
    counts: dict[str, int] = {}
    with open(a.output, "w", encoding="utf-8", newline="") as f:
        f.write("sample\tn_tbl\tworst_tbl\tdecl_px\thangul_px\trhwp_px\tdiff_px\tclass\tmatch_note\n")
        for i, doc in enumerate(docs, 1):
            name = Path(doc).name
            try:
                rts = rhwp_tables(a.exe, doc)
                meas = rhwp_measured(a.exe, doc)
                hts, n_all = hangul_top_tables(doc)
            except Exception as e:  # noqa: BLE001
                f.write(f"{name}\t\t\t\t\t\t\tERR\t{type(e).__name__}\n")
                f.flush()
                counts["ERR"] = counts.get("ERR", 0) + 1
                print(f"# {i}/{len(docs)} ERR {name[:40]}", file=sys.stderr, flush=True)
                continue
            note = "" if len(rts) == len(hts) else f"tbl_count rhwp={len(rts)} hangul={len(hts)}(all={n_all})"
            worst = None  # (abs_diff, idx, decl, h, r, cls, d)
            for k, (sec, pi, rows, decl) in enumerate(rts):
                h = hts[k][1] if k < len(hts) and hts[k][1] is not None else None
                r = meas.get((sec, pi))
                cls, d = classify_table(decl, h, r)
                if cls == "NO_DATA":
                    continue
                if worst is None or abs(d) > worst[0]:
                    worst = (abs(d), k, decl, h, r, cls, d)
            if worst is None:
                cls = "NO_DATA"
                f.write(f"{name}\t{len(rts)}\t\t\t\t\t\t{cls}\t{note}\n")
            else:
                _, k, decl, h, r, cls, d = worst
                f.write(f"{name}\t{len(rts)}\t{k}\t{decl:.1f}\t{h:.1f}\t{r:.1f}\t{d:+.1f}\t{cls}\t{note}\n")
            f.flush()
            counts[cls] = counts.get(cls, 0) + 1
            print(f"# {i}/{len(docs)} {name[:36]} -> {cls} {counts}", file=sys.stderr, flush=True)
        print(f"# done {counts}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
