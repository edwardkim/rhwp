"""rhwp 렌더링 레이아웃 vs OLE HWP(한글) 페이지↔PI 매칭 검증.

rhwp 가 각 문단(PI)을 배치한 페이지를, 한글(OLE 자동화)이 같은 문단을 배치한 페이지와
대조한다. 두 도구 모두 동일 .hwpx/.hwp 의 본문 `<hp:p>` 문단을 0-기반으로 동일 카운트하므로
PI 인덱스가 1:1 정렬된다(다중 구역은 구역별 문단수 누적 오프셋으로 본문 연속 인덱스에 매핑).

- rhwp:   `rhwp dump-pages` → 각 pi 의 시작 페이지(해당 pi 가 처음 등장한 1-기반 페이지).
- 한글:   `SetPos(0, para, 0)` 후 `current_page`(1-기반 절대 페이지).

판정(파일 단위):
  MATCH      : 모든 PI 의 페이지가 일치 + 총 페이지수 동일
  PI_MISMATCH: 일부 PI 가 다른 페이지 (페이지수는 같을 수도)
  PAGE_DELTA : 총 페이지수 불일치 (대개 PI_MISMATCH 동반)
  PARA_COUNT : rhwp/한글 문단수 불일치(정렬 불가) — 별도 분류
  ERR        : 한글 열기/처리 실패

PI_MISMATCH/PAGE_DELTA 1건↑ 종료코드 1.

알려진 한계 — 캐럿-개체 분리 (시각 정합인데 PI_MISMATCH 로 나오는 오탐, #1757):
  rhwp 는 "pi 가 처음 등장한 쪽"(표 몸체 시작 쪽), 한글은 SetPos 캐럿 쪽을 보고한다.
  1) 자리차지 다쪽 표 anchor — 표 몸체는 양쪽 동일 렌더인데 한글 캐럿(anchor 줄)은
     표가 끝나는 쪽 (예: 17991519 공항시설법 별표3, pi1 rhwp 1쪽 ↔ 한글 4쪽).
  2) 쪽 경계 TAC(글자처럼) 표 문단 — 표는 양쪽 모두 다음 쪽에 통째 렌더(이전 쪽
     하단은 빈 공간)인데 한글 캐럿은 이전 쪽 (예: 2789777 군수품 별표3 pi4,
     36389863 물품검사 조서 pi9 — 한글 PDF 시각 확인 완료).
  판별: 해당 pi 문단이 다쪽 자리차지/쪽 경계 TAC 표이면 한글 PDF 를 생성해 시각
  대조 후 오탐 여부 확정. 상세: mydocs/manual/verify_pi_page_vs_hangul.md

사용:
    python tools/verify_pi_page_vs_hangul.py --batch <원본폴더> [--sample N] [--seed S] -o out.tsv
    python tools/verify_pi_page_vs_hangul.py --files a.hwpx b.hwp -o out.tsv

요구: Windows + 한컴오피스 + pyhwpx. rhwp release 바이너리.
산출 TSV: sample / verdict / rhwp_pages / hwp_pages / n_mismatch / detail
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
PG = re.compile(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+),")
PI = re.compile(r"\bpi=(\d+)")


def git_head() -> str:
    try:
        return subprocess.run(["git", "rev-parse", "--short", "HEAD"],
                              capture_output=True, text=True, timeout=10).stdout.strip() or "?"
    except Exception:
        return "?"


def rhwp_pi_pages(path: Path):
    """rhwp dump-pages → ({(section,pi): start_page_1based}, total_pages, section_para_counts)."""
    try:
        out = subprocess.run([str(RHWP), "dump-pages", str(path)], capture_output=True,
                             text=True, encoding="utf-8", errors="replace", timeout=120)
    except Exception as e:  # noqa: BLE001
        return None, None, None, f"rhwp:{e}"
    if out.returncode != 0:
        return None, None, None, "rhwp:rc"
    start: dict[tuple[int, int], int] = {}
    pages = set()
    cur_page = 0
    cur_sec = 0
    max_pi: dict[int, int] = {}
    for ln in out.stdout.splitlines():
        m = PG.search(ln)
        if m:
            cur_page = int(m.group(1))
            cur_sec = int(m.group(2))
            pages.add(cur_page)
            continue
        q = PI.search(ln)
        if q and cur_page:
            pi = int(q.group(1))
            key = (cur_sec, pi)
            if key not in start or cur_page < start[key]:
                start[key] = cur_page
            max_pi[cur_sec] = max(max_pi.get(cur_sec, 0), pi)
    if not pages:
        return None, None, None, "rhwp:nopages"
    sec_counts = {s: max_pi[s] + 1 for s in max_pi}
    return start, len(pages), sec_counts, None


def hwp_para_pages(hwp, path: Path):
    """한글 SetPos+current_page → {abs_para: page_1based}, total_pages."""
    hwp.open(str(path))
    total = hwp.PageCount
    hwp.MoveDocEnd()
    end = hwp.GetPos()  # (list, para, pos)
    max_para = end[1]
    mp = {}
    for para in range(max_para + 1):
        hwp.SetPos(0, para, 0)
        mp[para] = hwp.current_page
    hwp.clear(option=1)
    return mp, total, max_para + 1


def compare(rhwp_start, sec_counts, hwp_pages):
    """rhwp (section,pi)→page 를 연속 abs 인덱스로 변환 후 한글과 비교."""
    # 구역별 누적 오프셋 (구역 0 부터 정렬된 순서)
    offsets = {}
    acc = 0
    for s in sorted(sec_counts):
        offsets[s] = acc
        acc += sec_counts[s]
    rhwp_abs = {}
    for (s, pi), pg in rhwp_start.items():
        rhwp_abs[offsets.get(s, 0) + pi] = pg
    mism = []
    for idx in sorted(set(rhwp_abs) | set(hwp_pages)):
        rp = rhwp_abs.get(idx)
        hp = hwp_pages.get(idx)
        if rp != hp:
            mism.append((idx, rp, hp))
    return mism, acc


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--batch", type=Path, help="원본 폴더(재귀)")
    g.add_argument("--files", nargs="+", type=Path)
    ap.add_argument("--sample", type=int, default=0)
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("-o", "--out", type=Path, required=True)
    ap.add_argument("--restart-every", type=int, default=400)
    args = ap.parse_args()

    if not RHWP.exists():
        print(f"오류: rhwp 바이너리 없음 {RHWP}", file=sys.stderr)
        return 2
    try:
        from pyhwpx import Hwp
    except ImportError:
        print("오류: pyhwpx 미설치", file=sys.stderr)
        return 2

    if args.batch:
        files = [p for p in sorted(args.batch.rglob("*"))
                 if p.suffix.lower() in (".hwpx", ".hwp")]
    else:
        files = list(args.files)
    if args.sample and len(files) > args.sample:
        rng = random.Random(args.seed)
        files = sorted(rng.sample(files, args.sample))

    head = git_head()
    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    hwp = Hwp(new=True, visible=False)

    n_match = n_mism = n_delta = n_para = n_err = 0
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["# git_head=" + head])
        w.writerow(["sample", "verdict", "rhwp_pages", "hwp_pages", "n_mismatch", "detail"])
        for i, f in enumerate(files):
            rel = str(f.relative_to(args.batch)) if args.batch else f.name
            rel = rel.replace("\\", "/")
            rstart, rpages, sec_counts, rerr = rhwp_pi_pages(f)
            if rerr:
                n_err += 1
                w.writerow([rel, "ERR", "", "", "", rerr])
                fh.flush()
                continue
            try:
                hpages, htotal, hcount = hwp_para_pages(hwp, f)
            except Exception as e:  # noqa: BLE001
                n_err += 1
                w.writerow([rel, "ERR", rpages, "", "", f"hwp:{type(e).__name__}"])
                fh.flush()
                try:
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
                    hwp = Hwp(new=True, visible=False)
                except Exception:
                    pass
                continue
            rcount = sum(sec_counts.values())
            if rcount != hcount:
                n_para += 1
                w.writerow([rel, "PARA_COUNT", rpages, htotal, abs(rcount - hcount),
                            f"rhwp_paras={rcount} hwp_paras={hcount}"])
                fh.flush()
                continue
            mism, _ = compare(rstart, sec_counts, hpages)
            if not mism and rpages == htotal:
                n_match += 1
                # MATCH 도 TSV 에 기록 — 성공 샘플과 미실행 샘플을 산출물에서 구분.
                w.writerow([rel, "MATCH", rpages, htotal, 0, ""])
                fh.flush()
                continue
            verdict = "PAGE_DELTA" if rpages != htotal else "PI_MISMATCH"
            if verdict == "PAGE_DELTA":
                n_delta += 1
            else:
                n_mism += 1
            detail = " ; ".join(f"pi{idx} rhwp_p{rp}|hwp_p{hp}" for idx, rp, hp in mism[:40])
            if len(mism) > 40:
                detail += f" ; (+{len(mism) - 40} more)"
            w.writerow([rel, verdict, rpages, htotal, len(mism), detail])
            fh.flush()
            if (i + 1) % args.restart_every == 0:
                try:
                    hwp.quit()
                    subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
                    hwp = Hwp(new=True, visible=False)
                except Exception:
                    pass
    try:
        hwp.quit()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    except Exception:
        pass
    total = len(files)
    print(f"\n[pi-page-vs-hangul] HEAD={head} 처리={total}")
    print(f"  MATCH={n_match} PI_MISMATCH={n_mism} PAGE_DELTA={n_delta} "
          f"PARA_COUNT={n_para} ERR={n_err}")
    print(f"  → {args.out}")
    return 1 if (n_mism + n_delta) > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
