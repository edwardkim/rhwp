"""페이지↔PI 매칭 오라클 — 저장 전후(원본 ↔ rt) rhwp 페이지네이션의 PI→페이지 배치 비교.

IR 게이트(hwpx-roundtrip/hwp5-roundtrip)는 IR 뼈대 diff 만 본다. 이 도구는 한 단계
더 들어가 **렌더링 레이아웃(페이지네이션) 충실도**를 본다: 원본을 파싱·페이지네이션해
얻은 `{(section, pi): {global_page,...}}` 맵과, 재저장본(rt)의 동일 맵을 비교해
**같은 PI(문단/표)가 다른 페이지로 이동**하면 손실로 보고한다.

판정(파일 단위):
  - SAME      : 모든 (section,pi) 의 페이지 집합 동일 + 총 페이지수 동일
  - PI_MOVED  : 일부 (section,pi) 가 다른 페이지에 배치 (페이지수는 같을 수도)
  - PAGE_DELTA: 총 페이지수 변동 (대개 PI_MOVED 동반; 페이지붕괴/확장)
  - ERR       : dump-pages 실패(파싱/렌더 오류)

PI_MOVED 또는 PAGE_DELTA 가 1건 이상이면 종료 코드 1.

사용:
    # inventory 기준 표본 (IR_DIFF 전건 + PASS 무작위 N)
    python tools/verify_pi_page_roundtrip.py \
        --inventory output/poc/fidelity3/hwpx/inventory.tsv \
        --orig-root C:/Users/planet/hwpdocs --rt-root output/poc/fidelity3/hwpx \
        --status IR_DIFF --pass-sample 300 --seed 42 \
        -o output/poc/fidelity3/pi_page_diff_hwpx.tsv

    # 폴더 배치(원본↔rt 재귀 매칭)
    python tools/verify_pi_page_roundtrip.py --batch <원본_폴더> <rt_폴더> -o out.tsv

산출 TSV 컬럼:
    sample / verdict / orig_pages / rt_pages / n_moved / moved (세부: sec:pi orig{..}→rt{..} ; ...)
"""
from __future__ import annotations

import argparse
import csv
import random
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RHWP = ROOT / "target" / "release" / ("rhwp.exe" if sys.platform == "win32" else "rhwp")

PAGE_HDR = re.compile(r"=== 페이지 \d+ \(global_idx=(\d+), section=(\d+), page_num=")
PI_RE = re.compile(r"\bpi=(\d+)")


def git_head() -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, timeout=10,
        )
        return out.stdout.strip() or "unknown"
    except Exception:
        return "unknown"


def dump_pages_map(path: Path, timeout: int = 120):
    """rhwp dump-pages 출력을 파싱해 {(section, pi): set(global_page)} 와 페이지수를 반환.

    실패 시 (None, None, error_message).
    """
    try:
        proc = subprocess.run(
            [str(RHWP), "dump-pages", str(path)],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return None, None, "TIMEOUT"
    except Exception as e:  # noqa: BLE001
        return None, None, f"EXC:{e}"
    if proc.returncode != 0:
        msg = (proc.stderr or proc.stdout or "").strip().splitlines()
        return None, None, "RC=%d %s" % (proc.returncode, msg[-1] if msg else "")

    mapping: dict[tuple[int, int], set[int]] = {}
    pages: set[int] = set()
    cur_section = 0
    cur_global = -1
    for line in proc.stdout.splitlines():
        m = PAGE_HDR.search(line)
        if m:
            cur_global = int(m.group(1))
            cur_section = int(m.group(2))
            pages.add(cur_global)
            continue
        pm = PI_RE.search(line)
        if pm and cur_global >= 0:
            key = (cur_section, int(pm.group(1)))
            mapping.setdefault(key, set()).add(cur_global)
    if not pages:
        # 파싱은 됐으나 페이지가 없음(빈 문서/예외) — 오류로 취급
        return None, None, "NO_PAGES"
    return mapping, len(pages), None


def compare(orig_map, rt_map):
    """두 맵을 비교해 페이지 집합이 다른 (section,pi) 목록을 반환."""
    moved = []
    for key in sorted(set(orig_map) | set(rt_map)):
        o = orig_map.get(key, set())
        r = rt_map.get(key, set())
        if o != r:
            sec, pi = key
            moved.append((sec, pi, sorted(o), sorted(r)))
    return moved


def find_rt(rt_root: Path, rel: Path):
    stem = rel.with_suffix("")
    for suffix in (".rt.hwpx", ".rt.hwp"):
        cand = rt_root / (str(stem) + suffix)
        if cand.exists():
            return cand
    return None


def collect_inventory(inv: Path, orig_root: Path, rt_root: Path,
                      status_filter: set[str] | None, pass_sample: int, seed: int):
    rows = []
    with open(inv, encoding="utf-8") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        for r in reader:
            rows.append(r)
    selected = []
    pass_pool = []
    for r in rows:
        status = r.get("status", "")
        sample = r.get("sample", "")
        if status == "PARSE_FAIL":
            continue
        rel = Path(sample)
        # inventory sample 컬럼은 확장자 없는 stem 일 수 있으므로 원본 후보를 탐색
        orig = None
        for suffix in (".hwpx", ".hwp"):
            cand = orig_root / (str(rel.with_suffix("")) + suffix)
            if cand.exists():
                orig = cand
                break
        if orig is None and (orig_root / rel).exists():
            orig = orig_root / rel
        if orig is None:
            continue
        rt = find_rt(rt_root, rel)
        if rt is None:
            continue
        relkey = str(rel).replace("\\", "/")
        if status_filter and status in status_filter:
            selected.append((orig, rt, relkey))
        elif status == "PASS":
            pass_pool.append((orig, rt, relkey))
    if pass_sample > 0 and pass_pool:
        random.Random(seed).shuffle(pass_pool)
        selected.extend(pass_pool[:pass_sample])
    return selected


def collect_batch(orig_root: Path, rt_root: Path):
    pairs = []
    for orig in sorted(orig_root.rglob("*")):
        if orig.suffix.lower() not in (".hwpx", ".hwp"):
            continue
        rel = orig.relative_to(orig_root)
        rt = find_rt(rt_root, rel)
        if rt is not None:
            pairs.append((orig, rt, str(rel).replace("\\", "/")))
    return pairs


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--batch", nargs=2, metavar=("ORIG_ROOT", "RT_ROOT"))
    ap.add_argument("--inventory", type=Path)
    ap.add_argument("--orig-root", type=Path)
    ap.add_argument("--rt-root", type=Path)
    ap.add_argument("--status", default="IR_DIFF",
                    help="inventory 모드에서 전건 포함할 status (콤마구분, 기본 IR_DIFF)")
    ap.add_argument("--pass-sample", type=int, default=0,
                    help="PASS 중 무작위 표본 수 (기본 0)")
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--limit", type=int, default=0, help="총 처리 상한(디버그)")
    ap.add_argument("-o", "--out", type=Path, required=True)
    args = ap.parse_args()

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 — {RHWP}", file=sys.stderr)
        return 2

    if args.batch:
        pairs = collect_batch(Path(args.batch[0]), Path(args.batch[1]))
    elif args.inventory:
        if not (args.orig_root and args.rt_root):
            print("오류: --inventory 모드는 --orig-root, --rt-root 필요", file=sys.stderr)
            return 2
        status_filter = {s for s in args.status.split(",") if s}
        pairs = collect_inventory(args.inventory, args.orig_root, args.rt_root,
                                  status_filter, args.pass_sample, args.seed)
    else:
        print("오류: --batch 또는 --inventory 중 하나 필요", file=sys.stderr)
        return 2

    if args.limit > 0:
        pairs = pairs[:args.limit]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    n_same = n_moved = n_delta = n_err = 0
    head = git_head()
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["sample", "verdict", "orig_pages", "rt_pages", "n_moved", "moved"])
        for idx, (orig, rt, relkey) in enumerate(pairs):
            omap, opages, oerr = dump_pages_map(orig)
            if oerr:
                n_err += 1
                w.writerow([relkey, "ERR", "", "", "", f"orig:{oerr}"])
                continue
            rmap, rpages, rerr = dump_pages_map(rt)
            if rerr:
                n_err += 1
                w.writerow([relkey, "ERR", opages, "", "", f"rt:{rerr}"])
                continue
            moved = compare(omap, rmap)
            if not moved and opages == rpages:
                n_same += 1
                continue  # SAME 는 TSV 생략(노이즈 절감)
            verdict = "PAGE_DELTA" if opages != rpages else "PI_MOVED"
            if verdict == "PAGE_DELTA":
                n_delta += 1
            else:
                n_moved += 1
            detail = " ; ".join(
                f"s{sec}:pi{pi} {o}->{r}" for sec, pi, o, r in moved[:40]
            )
            if len(moved) > 40:
                detail += f" ; (+{len(moved) - 40} more)"
            w.writerow([relkey, verdict, opages, rpages, len(moved), detail])
            if (idx + 1) % 100 == 0:
                print(f"  ... {idx + 1}/{len(pairs)} 처리", file=sys.stderr)

    total = len(pairs)
    print(f"\n[pi-page-roundtrip] HEAD={head} 처리={total}")
    print(f"  SAME={n_same}  PI_MOVED={n_moved}  PAGE_DELTA={n_delta}  ERR={n_err}")
    print(f"  → {args.out}")
    return 1 if (n_moved + n_delta) > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
