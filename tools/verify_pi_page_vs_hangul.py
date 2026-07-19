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
  PROTECTED_SKIP : [#2261] 보호/배포용 HWP5 — 한글 정상 개방(PageCount 정상)이나
                   캐럿이 본문 진입 불가(max_para==0 → hcount==1)인데 rhwp 는 본문
                   문단 다수. per-PI 대조 불가라 PageCount 만 대조·per-PI 스킵.
  ERR        : 한글 열기/처리 실패

PI_MISMATCH/PAGE_DELTA 1건↑ 종료코드 1. PROTECTED_SKIP 은 사각지대 분리라 실패 미계상.

알려진 한계 — 캐럿-개체 분리 (시각 정합인데 PI_MISMATCH 로 나오는 오탐, #1757):
  rhwp 는 "pi 가 처음 등장한 쪽"(표 몸체 시작 쪽), 한글은 SetPos 캐럿 쪽을 보고한다.
  1) 자리차지 다쪽 표 anchor — 표 몸체는 양쪽 동일 렌더인데 한글 캐럿(anchor 줄)은
     표가 끝나는 쪽 (예: 17991519 공항시설법 별표3, pi1 rhwp 1쪽 ↔ 한글 4쪽).
  2) 쪽 경계 TAC(글자처럼) 표 문단 — 표는 양쪽 모두 다음 쪽에 통째 렌더(이전 쪽
     하단은 빈 공간)인데 한글 캐럿은 이전 쪽 (예: 2789777 군수품 별표3 pi4,
     36389863 물품검사 조서 pi9 — 한글 PDF 시각 확인 완료).
  3) [#1920] 쪽 하단 빈 문단 — rhwp 는 저장 lineseg 대로 쪽 하단에 배치하는데
     한글 캐럿은 다음 쪽 상단으로 보고 (예: 36398160 pi3, rhwp 1쪽 ↔ 한글 2쪽).
     불일치 PI 전부가 "빈 문단(text_len=0, controls=0) + rhwp 쪽 = 한글 쪽 - 1"
     이면 verdict 를 `PI_MISMATCH_CARET` 로 분리하고 detail 에 [empty-caret?]
     태그를 단다 (종료코드 실패로 계상하지 않음 — 시각 확정 전 후보).
  판별: 해당 pi 문단이 다쪽 자리차지/쪽 경계 TAC 표이면 한글 PDF 를 생성해 시각
  대조 후 오탐 여부 확정. 상세: mydocs/manual/verification/verify_pi_page_vs_hangul.md

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
    # [#2136/#1757] 표 앵커 캐럿 오탐 판별용: (sec,pi) → (span_end_page, is_tac)
    tbl_info: dict[tuple[int, int], tuple[int, bool]] = {}
    for ln in out.stdout.splitlines():
        m = PG.search(ln)
        if m:
            cur_page = int(m.group(1))
            cur_sec = int(m.group(2))
            pages.add(cur_page)
            continue
        # [#2152] 미주 문단 PageItem 은 para_index 가 본문 인덱스 뒤에 이어 붙는다
        # (typeset.rs en_para_idx = body_len + 미주 로컬). dump-pages 가 [미주] 라벨을
        # 달아 주므로 본문 pi 카운트/매핑에서 제외 — 한글 SetPos 문단 공간(본문)과 정렬.
        if "[미주]" in ln:
            continue
        q = PI.search(ln)
        if q and cur_page:
            pi = int(q.group(1))
            key = (cur_sec, pi)
            if key not in start or cur_page < start[key]:
                start[key] = cur_page
            max_pi[cur_sec] = max(max_pi.get(cur_sec, 0), pi)
            st = ln.lstrip()
            if st.startswith(("Table", "PartialTable")):
                prev = tbl_info.get(key)
                tac = "tac=true" in ln
                end = max(cur_page, prev[0]) if prev else cur_page
                tbl_info[key] = (end, tac or (prev[1] if prev else False))
    if not pages:
        return None, None, None, None, "rhwp:nopages"
    sec_counts = {s: max_pi[s] + 1 for s in max_pi}
    return start, len(pages), sec_counts, tbl_info, None


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
    """rhwp (section,pi)→page 를 연속 abs 인덱스로 변환 후 한글과 비교.

    반환 mism 항목: (abs_idx, rhwp_page, hwp_page, section, pi) — 구역-로컬
    (section, pi)는 오탐 판별용 rhwp dump 재조회에 쓴다 (#1920).
    """
    # 구역별 누적 오프셋 (구역 0 부터 정렬된 순서)
    offsets = {}
    acc = 0
    for s in sorted(sec_counts):
        offsets[s] = acc
        acc += sec_counts[s]
    rhwp_abs = {}
    for (s, pi), pg in rhwp_start.items():
        rhwp_abs[offsets.get(s, 0) + pi] = (pg, s, pi)
    mism = []
    for idx in sorted(set(rhwp_abs) | set(hwp_pages)):
        entry = rhwp_abs.get(idx)
        rp = entry[0] if entry else None
        hp = hwp_pages.get(idx)
        if rp != hp:
            sec, pi = (entry[1], entry[2]) if entry else (0, idx)
            mism.append((idx, rp, hp, sec, pi))
    return mism, acc


EMPTY_PARA = re.compile(r"--- 문단 \d+\.\d+ --- cc=\d+, text_len=(\d+), controls=(\d+)")


def pi_is_empty_para(path: Path, sec: int, pi: int) -> bool:
    """[#1920] 오탐 후보 판별 — 해당 문단이 빈 문단(text_len=0, controls=0)인지.

    쪽 하단의 빈 문단은 rhwp(저장 lineseg 배치)와 한글(캐럿을 다음 쪽 상단으로
    보고)이 쪽 번호를 다르게 셀 수 있다 — 시각 정합인데 PI_MISMATCH 로 잡히는
    캐럿 의미차 오탐 유형 (기지 한계 (1)(2)의 확장, 36398160 pi=3 실측).
    """
    try:
        out = subprocess.run(
            [str(RHWP), "dump", str(path), "-s", str(sec), "-p", str(pi)],
            capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=60,
        )
    except Exception:  # noqa: BLE001
        return False
    m = EMPTY_PARA.search(out.stdout)
    return bool(m) and m.group(1) == "0" and m.group(2) == "0"


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--batch", type=Path, help="원본 폴더(재귀)")
    g.add_argument("--files", nargs="+", type=Path)
    g.add_argument("--list", type=Path, help="파일 목록 텍스트(줄당 절대경로) — 표본 재현용")
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
    elif args.list:
        # utf-8-sig: PowerShell Out-File 등이 넣는 BOM 이 첫 경로를 오염시키는 것 방지
        files = [Path(l.strip()) for l in args.list.read_text(encoding="utf-8-sig").splitlines()
                 if l.strip()]
    else:
        files = list(args.files)
    if args.sample and len(files) > args.sample:
        rng = random.Random(args.seed)
        files = sorted(rng.sample(files, args.sample))

    head = git_head()

    def fresh_hwp():
        """[r12] taskkill 직후 즉시 CreateObject 는 COM 서버 정리 전 레이스로 실패
        ('개체가 준비되지 않았습니다', 고부하 시 재현). kill 후 대기 + 생성 재시도,
        얕은 probe 는 통과해도 실제 Open 이 실패하므로 실파일 open 으로 확인."""
        import time

        last = None
        probe_doc = ROOT / "samples" / "byeolpyo1.hwp"
        for attempt in range(4):
            subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
            time.sleep(2 + attempt * 3)
            try:
                h = Hwp(new=True, visible=False)
            except Exception as e:  # noqa: BLE001
                last = e
                continue
            for _ in range(10):
                try:
                    h.open(str(probe_doc))
                    _ = h.PageCount
                    h.clear(option=1)
                    return h
                except Exception as e:  # noqa: BLE001
                    last = e
                    time.sleep(2)
            try:
                h.quit()
            except Exception:
                pass
        raise last

    hwp = fresh_hwp()

    n_match = n_mism = n_delta = n_para = n_err = n_caret = n_protected = 0
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(["# git_head=" + head])
        w.writerow(["sample", "verdict", "rhwp_pages", "hwp_pages", "n_mismatch", "detail"])
        for i, f in enumerate(files):
            rel = str(f.relative_to(args.batch)) if args.batch else f.name
            rel = rel.replace("\\", "/")
            rstart, rpages, sec_counts, tbl_info, rerr = rhwp_pi_pages(f)
            if rerr:
                n_err += 1
                w.writerow([rel, "ERR", "", "", "", rerr])
                fh.flush()
                continue
            try:
                hpages, htotal, hcount = hwp_para_pages(hwp, f)
            except Exception:  # noqa: BLE001
                # [r12] 인스턴스 재생성 후 같은 문서 1회 재시도 — 재시작 직후
                # not-ready/dead-proxy 로 정상 문서가 ERR 로 소모되는 것을 방지.
                try:
                    hwp = fresh_hwp()
                    hpages, htotal, hcount = hwp_para_pages(hwp, f)
                except Exception as e:  # noqa: BLE001
                    n_err += 1
                    w.writerow([rel, "ERR", rpages, "", "", f"hwp:{type(e).__name__} {str(e)[:80]}"])
                    fh.flush()
                    try:
                        hwp = fresh_hwp()
                    except Exception:
                        pass
                    continue
            rcount = sum(sec_counts.values())
            if rcount != hcount:
                # [#2261] 보호/배포용 HWP5 — 한글이 정상 개방(PageCount 정상)하나
                # 캐럿이 본문 진입 불가(MoveDocEnd 후 GetPos para=0 → hcount==1)라
                # per-PI 대조 불가. rhwp 는 본문 문단 다수 파싱. PageCount 만 대조하고
                # per-PI 는 스킵해 PARA_COUNT(구조적 정렬 불가)와 분리 집계한다.
                if hcount == 1 and rcount > 1:
                    n_protected += 1
                    page_tag = (
                        "page_match" if rpages == htotal
                        else f"page_delta={rpages - htotal:+d}"
                    )
                    w.writerow([rel, "PROTECTED_SKIP", rpages, htotal, abs(rpages - htotal),
                                f"rhwp_paras={rcount} hwp_paras=1(caret-blocked) {page_tag}"])
                    fh.flush()
                    continue
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
            # [#1920] 캐럿 의미차 오탐 후보 판별: "빈 문단 + 쪽 번호 ±1"
            # (rhwp 는 저장 lineseg 대로 배치, 한글 캐럿은 인접 쪽으로 보고 —
            # 시각 정합 오탐 후보. #2136/r12 에서 양방향 확장 — 1220000 시각 확증.)
            # [#1757/#2136 r12] 표 앵커 캐럿 오탐 2형:
            #   (1) 자리차지(비TAC) 다쪽 표 — 한글 캐럿은 표가 끝나는 쪽:
            #       rp = 표 시작쪽 < hp ≤ 표 span 끝쪽 이면 오탐.
            #   (2) 쪽 경계 TAC 표 — 표는 양쪽 모두 다음 쪽 통째 렌더인데 한글
            #       캐럿은 이전 쪽: hp == rp - 1 이면 오탐.
            def _caret_like(rp, hp, sec, pi):
                if rp is None or hp is None:
                    return False
                ti = tbl_info.get((sec, pi))
                if ti is not None:
                    end, tac = ti
                    if not tac and rp < hp <= end:
                        return True  # (1) 다쪽 표 anchor
                    if tac and hp == rp - 1:
                        return True  # (2) 쪽 경계 TAC 표
                return abs(hp - rp) == 1 and pi_is_empty_para(f, sec, pi)

            verdict = "PAGE_DELTA" if rpages != htotal else "PI_MISMATCH"
            if verdict == "PAGE_DELTA":
                n_delta += 1
            else:
                caret_all = all(
                    _caret_like(rp, hp, sec, pi) for _, rp, hp, sec, pi in mism
                )
                if caret_all:
                    verdict = "PI_MISMATCH_CARET"
                    n_caret += 1
                else:
                    n_mism += 1
            marks = []
            for idx, rp, hp, sec, pi in mism[:40]:
                tag = ""
                if (verdict == "PI_MISMATCH_CARET"
                        or _caret_like(rp, hp, sec, pi)):
                    tag = "[empty-caret?]"
                marks.append(f"pi{idx} rhwp_p{rp}|hwp_p{hp}{tag}")
            detail = " ; ".join(marks)
            if len(mism) > 40:
                detail += f" ; (+{len(mism) - 40} more)"
            w.writerow([rel, verdict, rpages, htotal, len(mism), detail])
            fh.flush()
            if (i + 1) % args.restart_every == 0:
                try:
                    hwp.quit()
                except Exception:
                    pass
                try:
                    hwp = fresh_hwp()
                except Exception:
                    pass
    try:
        hwp.quit()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
    except Exception:
        pass
    total = len(files)
    print(f"\n[pi-page-vs-hangul] HEAD={head} 처리={total}")
    print(f"  MATCH={n_match} PI_MISMATCH={n_mism} PI_MISMATCH_CARET={n_caret} "
          f"PAGE_DELTA={n_delta} PARA_COUNT={n_para} PROTECTED_SKIP={n_protected} ERR={n_err}")
    print(f"  → {args.out}")
    # CARET(오탐 후보)는 실패로 계상하지 않는다 — 시각 대조로 확정 전까지 후보.
    return 1 if (n_mism + n_delta) > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
