# -*- coding: utf-8 -*-
"""HWPX→HWP roundtrip 페이지네이션 fidelity 하니스.

`hwpx-roundtrip`/`hwpx_roundtrip_baseline` 이 검사하는 것은 HWPX→HWPX 의 IR 뼈대
보존이다. 본 하니스는 이와 상보적으로 **HWPX→HWP 변환 후 페이지네이션이 원본 HWPX 와
일치하는지**를 측정한다: 임의 HWPX 를 `rhwp convert` 로 HWP 로 변환하고, orig-HWPX 와
변환-HWP 의 `dump-pages` `(section,pi)→page` 를 대조한다. 불일치(PI_MOVED/PAGE_DELTA)는
HWPX↔HWP5 파서/typeset 경로 divergence 를 의미한다(예: 빈-앵커 host_line_spacing
소스-의존 억제 #1836 계열).

동시에 **native HWP** 파일이 함께 있으면 native 렌더 정합(별도 축)을 확인할 수 있다
(예: rowbreak-problem-pages 의 .hwp/.hwpx 두 인코딩 대조).

용도:
  1. 후보 수정 바이너리마다 코퍼스에 대해 fidelity 를 측정해 HWPX↔변환HWP divergence
     회귀를 정량화(두 바이너리 산출 TSV 를 roundtrip_fidelity_diff.py 로 전이 분류).
  2. native .hwp 와 .hwpx 가 모두 있는 문서에서 실제갭 vs phantom갭을 한 하니스에서 비교.

사용:
    python tools/roundtrip_fidelity_harness.py --corpus <HWPX폴더> --workdir <작업폴더> -o out.tsv
    python tools/roundtrip_fidelity_harness.py --files a.hwpx b.hwpx --workdir wd -o out.tsv

산출 TSV: sample / verdict(SAME|PI_MOVED|PAGE_DELTA|ERR) / hwpx_pages / hwp_pages / n_moved / detail
종료코드: PI_MOVED+PAGE_DELTA > 0 이면 1.
"""
from __future__ import annotations

import argparse
import csv
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RHWP = ROOT / "target" / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")
PAGE_HDR = re.compile(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+), page_num=")
PI_RE = re.compile(r"\bpi=(\d+)")


def dump_pi_pages(path: Path):
    """dump-pages → ({(section,pi): first_page}, total_pages) 또는 (None, err)."""
    try:
        out = subprocess.run(
            [str(RHWP), "dump-pages", str(path)],
            capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180,
        )
    except Exception as e:  # noqa: BLE001
        return None, f"exc:{e}"
    if out.returncode != 0:
        return None, "rc"
    cur_page, cur_sec = 0, 0
    pages = set()
    first: dict[tuple[int, int], int] = {}
    for ln in out.stdout.splitlines():
        m = PAGE_HDR.search(ln)
        if m:
            cur_page = int(m.group(1))
            cur_sec = int(m.group(2))
            pages.add(cur_page)
            continue
        for q in PI_RE.finditer(ln):
            key = (cur_sec, int(q.group(1)))
            if key not in first:
                first[key] = cur_page
    if not pages:
        return None, "nopages"
    return (first, len(pages)), None


def compare(hwpx: Path, hwp: Path):
    a, ea = dump_pi_pages(hwpx)
    if ea:
        return "ERR", "", "", 0, f"hwpx:{ea}"
    b, eb = dump_pi_pages(hwp)
    if eb:
        return "ERR", str(a[1]), "", 0, f"hwp:{eb}"
    (fa, pa), (fb, pb) = a, b
    moved = []
    for key in sorted(set(fa) | set(fb)):
        if fa.get(key) != fb.get(key):
            moved.append((key, fa.get(key), fb.get(key)))
    if not moved and pa == pb:
        return "SAME", str(pa), str(pb), 0, ""
    verdict = "PAGE_DELTA" if pa != pb else "PI_MOVED"
    detail = " ; ".join(f"s{s}:pi{p} {oa}->{ob}" for (s, p), oa, ob in moved[:30])
    return verdict, str(pa), str(pb), len(moved), detail


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--corpus", type=Path, help="HWPX 폴더(재귀)")
    g.add_argument("--files", nargs="+", type=Path)
    g.add_argument("--file-list", type=Path, help="줄당 HWPX 경로 목록 파일(재현용 코퍼스)")
    ap.add_argument("--workdir", type=Path, required=True, help="변환 HWP 임시 폴더")
    ap.add_argument("-o", "--out", type=Path, required=True)
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 {RHWP}", file=sys.stderr)
        return 2
    if args.corpus:
        files = sorted(p for p in args.corpus.rglob("*.hwpx"))
    elif args.file_list:
        files = [
            Path(ln.strip())
            for ln in args.file_list.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.lstrip().startswith("#")
        ]
    else:
        files = list(args.files)
    if args.limit and len(files) > args.limit:
        files = files[: args.limit]
    args.workdir.mkdir(parents=True, exist_ok=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)

    n_same = n_moved = n_delta = n_err = 0
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["sample", "verdict", "hwpx_pages", "hwp_pages", "n_moved", "detail"])
        for i, hwpx in enumerate(files):
            gen = args.workdir / (hwpx.stem + ".gen.hwp")
            conv = subprocess.run(
                [str(RHWP), "convert", str(hwpx), str(gen)],
                capture_output=True, text=True, encoding="utf-8", errors="replace",
            )
            if conv.returncode != 0 or not gen.exists():
                n_err += 1
                w.writerow([hwpx.name, "ERR", "", "", 0, "convert_fail"])
                fh.flush()
                continue
            verdict, pa, pb, nm, detail = compare(hwpx, gen)
            w.writerow([hwpx.name, verdict, pa, pb, nm, detail])
            fh.flush()
            if verdict == "SAME":
                n_same += 1
            elif verdict == "PI_MOVED":
                n_moved += 1
            elif verdict == "PAGE_DELTA":
                n_delta += 1
            else:
                n_err += 1
    print(f"[roundtrip-fidelity] 처리={len(files)}")
    print(f"  SAME={n_same} PI_MOVED={n_moved} PAGE_DELTA={n_delta} ERR={n_err}")
    print(f"  → {args.out}")
    return 1 if (n_moved + n_delta) > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
