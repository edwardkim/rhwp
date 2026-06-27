"""한글 페이지 충실도 오라클 — 원본 ↔ 저장본(rt)의 한컴 PageCount 를 배치 비교.

IR 게이트(`hwpx-roundtrip`/`hwp5-roundtrip`)나 rhwp 자체 페이지수로는 검출되지 않는
**한글에서만 나타나는 페이지 붕괴**(예: #1557 secCnt, 잔여 단일구역 2→1)를 검출한다.
CLAUDE.md 권위 등급상 Windows+한컴에디터가 1차 정답지이므로, 이 오라클을 정식 도구화한다.

사용:
    # 배치: 원본 폴더 ↔ rt 폴더(재귀, 상대경로 매칭)
    python tools/verify_hangul_pages.py --batch <원본_폴더> <rt_폴더> [-o out.tsv]

    # 인벤토리: roundtrip 산출 inventory.tsv 의 sample 컬럼 기준
    python tools/verify_hangul_pages.py --inventory output/poc/fidelity3/hwpx/inventory.tsv \
        --orig-root C:/Users/planet/hwpdocs/samples --rt-root output/poc/fidelity3/hwpx \
        --status IR_DIFF,PASS --sample 40 --seed 42 [--pdf] [-o out.tsv]

판정: 원본=저장본 → OK / 저장본<원본 → COLLAPSE / 저장본>원본 → EXPAND.
COLLAPSE 가 1건 이상이면 종료 코드 1(게이트). MISSING/ERR 은 비집계 경고.

요구사항: Windows + 한컴오피스 2010+, `pip install pyhwpx`. (`--pdf` 는 PyMuPDF 도 필요)
"""
from __future__ import annotations

import argparse
import csv
import random
import subprocess
import sys
import time
from pathlib import Path


def git_head() -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, timeout=10,
        )
        return out.stdout.strip() or "unknown"
    except Exception:
        return "unknown"


def find_rt(rt_root: Path, rel: Path) -> Path | None:
    """원본 상대경로 rel 에 대응하는 rt 파일(.rt.hwpx/.rt.hwp)을 찾는다."""
    stem = rel.with_suffix("")
    for suffix in (".rt.hwpx", ".rt.hwp"):
        cand = rt_root / (str(stem) + suffix)
        if cand.exists():
            return cand
    return None


def collect_pairs_batch(orig_root: Path, rt_root: Path) -> list[tuple[Path, Path, str]]:
    pairs: list[tuple[Path, Path, str]] = []
    for orig in sorted(orig_root.rglob("*")):
        if orig.suffix.lower() not in (".hwpx", ".hwp"):
            continue
        rel = orig.relative_to(orig_root)
        rt = find_rt(rt_root, rel)
        if rt is not None:
            pairs.append((orig, rt, str(rel).replace("\\", "/")))
    return pairs


def collect_pairs_inventory(
    inv_tsv: Path, orig_root: Path, rt_root: Path, status_filter: set[str] | None
) -> list[tuple[Path, Path, str]]:
    """roundtrip inventory.tsv(sample/status 컬럼) 기준으로 쌍을 수집."""
    pairs: list[tuple[Path, Path, str]] = []
    with open(inv_tsv, encoding="utf-8") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            sample = (row.get("sample") or "").strip()
            if not sample:
                continue
            if status_filter and (row.get("status") or "") not in status_filter:
                continue
            rel = Path(sample.replace("/", "\\"))
            orig = orig_root / rel
            rt = find_rt(rt_root, rel)
            if orig.exists() and rt is not None:
                pairs.append((orig, rt, sample.replace("\\", "/")))
    return pairs


def run(pairs, out_tsv, visible, use_pdf, resume=False) -> int:
    try:
        from pyhwpx import Hwp
    except ImportError:
        print("ERROR: pyhwpx 미설치. `pip install pyhwpx` 실행하세요.", file=sys.stderr)
        return 2
    fitz = None
    if use_pdf:
        try:
            import fitz as _fitz  # PyMuPDF
            fitz = _fitz
        except ImportError:
            print("ERROR: --pdf 에는 PyMuPDF 필요. `pip install pymupdf`.", file=sys.stderr)
            return 2

    if not pairs:
        print("ERROR: 비교할 원본↔rt 쌍이 없습니다.", file=sys.stderr)
        return 2

    head = git_head()

    # 깨끗한 시작 — 잔존 Hwp.exe(이전 배치 누수)를 정리한 뒤 인스턴스 생성.
    # 오염된 COM 환경에서 시작하면 첫 인스턴스부터 com_error 다발(관측).
    try:
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"],
                       capture_output=True, timeout=30)
        time.sleep(1)
    except Exception:
        pass

    # [resume] 기존 out_tsv 의 처리분을 읽어 건너뛴다(증분 기록과 짝). 전수 배치 중
    # COM 크래시 시 재실행으로 이어서 진행하기 위함.
    done_rows = []  # (verdict, o, r, note, rel) — 성공분만(ERR 제외 → 재시도)
    done_rels = set()
    if resume and out_tsv is not None and out_tsv.exists():
        err_retry = 0
        with open(out_tsv, encoding="utf-8") as fh:
            for line in fh:
                line = line.rstrip("\n")
                if not line or line.startswith("#") or line.startswith("verdict\t"):
                    continue
                parts = line.split("\t")
                if len(parts) == 5:
                    if parts[0] == "ERR":
                        err_retry += 1  # ERR 은 done 처리 안 함 → 재시도
                        continue
                    done_rows.append(tuple(parts))
                    done_rels.add(parts[4])
        # ERR 행을 버리고 성공분만 남겨 TSV 재작성(중복 방지) — 이후 증분 append.
        out_tsv.parent.mkdir(parents=True, exist_ok=True)
        with open(out_tsv, "w", encoding="utf-8") as fh:
            fh.write(f"# git_head={head} pdf={use_pdf}\n")
            fh.write("verdict\torig_pg\trt_pg\tnote\trel\n")
            for rec in done_rows:
                fh.write("\t".join(str(x) for x in rec) + "\n")
        pairs = [p for p in pairs if p[2] not in done_rels]
        print(f"# [resume] 성공 {len(done_rels)}건 건너뜀, ERR {err_retry}건 재시도 포함 남은 {len(pairs)}건")

    print(f"# 한글 페이지 오라클 | git HEAD={head} | 대상 {len(pairs)}건")

    # 증분 기록 핸들 — 각 행 처리 직후 flush 하여 크래시 내성 확보.
    inc_fh = None
    if out_tsv is not None:
        out_tsv.parent.mkdir(parents=True, exist_ok=True)
        if resume and out_tsv.exists():
            inc_fh = open(out_tsv, "a", encoding="utf-8")  # 재작성된 파일에 이어쓰기
        else:
            inc_fh = open(out_tsv, "w", encoding="utf-8")  # fresh: truncate
            inc_fh.write(f"# git_head={head} pdf={use_pdf}\n")
            inc_fh.write("verdict\torig_pg\trt_pg\tnote\trel\n")
            inc_fh.flush()

    hwp = Hwp(new=True, visible=visible)
    tmp_pdf = Path.cwd() / "_hpv_tmp.pdf"

    def page_count(p: Path) -> int:
        hwp.open(str(p))
        n = hwp.PageCount
        if use_pdf and fitz is not None:
            if tmp_pdf.exists():
                tmp_pdf.unlink()
            hwp.save_as(str(tmp_pdf), "PDF")
            doc = fitz.open(str(tmp_pdf))
            n = len(doc)
            doc.close()
        hwp.clear(option=1)
        return n

    rows = list(done_rows)
    # 기존(resume) 분 카운트 반영
    collapse = sum(1 for x in done_rows if x[0] == "COLLAPSE")
    ok = sum(1 for x in done_rows if x[0] == "OK")
    other = sum(1 for x in done_rows if x[0] in ("EXPAND", "ERR"))

    def emit(rec):
        rows.append(rec)
        if inc_fh is not None:
            inc_fh.write("\t".join(str(x) for x in rec) + "\n")
            inc_fh.flush()

    # COM 인스턴스는 수천 건 누적 시 사망(과거 ~1868건에서 전멸) → 주기적 하드 재시작.
    # 주의: hwp.quit() 만으로는 Hwp.exe 프로세스가 남아 누적(200+ 누수 관측) → taskkill 병행.
    restart_every = 600

    def restart_hwp():
        nonlocal hwp
        try:
            hwp.quit()
        except Exception:
            pass
        # 잔존 Hwp.exe 강제 종료(누수 방지) 후 새 인스턴스 생성.
        try:
            subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"],
                           capture_output=True, timeout=30)
        except Exception:
            pass
        time.sleep(1)
        hwp = Hwp(new=True, visible=visible)

    try:
        for i, (orig, rt, rel) in enumerate(pairs):
            if i > 0 and i % restart_every == 0:
                restart_hwp()  # 주기적 재시작
            try:
                o = page_count(orig)
                r = page_count(rt)
            except Exception as exc:  # 파일별 격리
                emit(("ERR", -1, -1, type(exc).__name__, rel))
                other += 1
                print(f"  [{i+1:>4}/{len(pairs)}] {'ERR':>8}  {rel}", flush=True)
                restart_hwp()  # ERR 후 COM 상태 불량 가능 → 재생성
                continue
            if o == r:
                verdict, ok = "OK", ok + 1
            elif r < o:
                verdict, collapse = "COLLAPSE", collapse + 1
            else:
                verdict, other = "EXPAND", other + 1
            emit((verdict, o, r, "", rel))
            print(f"  [{i+1:>4}/{len(pairs)}] {verdict:>8}  pg {o}->{r}  {rel}", flush=True)
    finally:
        try:
            hwp.quit()
        except Exception:
            pass
        if tmp_pdf.exists():
            try:
                tmp_pdf.unlink()
            except Exception:
                pass
        if inc_fh is not None:
            inc_fh.close()

    if out_tsv is not None:
        print(f"\nTSV 저장(증분): {out_tsv}")

    total = len(rows)
    rate = 100.0 * collapse / total if total else 0.0
    print(
        f"\n=== 한글 페이지 오라클: {total}건 / OK={ok} COLLAPSE={collapse} "
        f"기타(EXPAND/ERR)={other} (붕괴율 {rate:.0f}%) ==="
    )
    return 1 if collapse > 0 else 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="한글 페이지 충실도 오라클(원본↔rt PageCount 비교)")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--batch", nargs=2, metavar=("ORIG_DIR", "RT_DIR"),
                   help="원본 폴더와 rt 폴더(재귀, 상대경로 매칭)")
    g.add_argument("--inventory", type=Path, metavar="TSV",
                   help="roundtrip inventory.tsv (sample/status 컬럼)")
    ap.add_argument("--orig-root", type=Path, help="--inventory 모드 원본 루트")
    ap.add_argument("--rt-root", type=Path, help="--inventory 모드 rt 루트")
    ap.add_argument("--status", help="--inventory 상태 필터(쉼표구분, 예: IR_DIFF,PASS)")
    ap.add_argument("--sample", type=int, default=0, help="무작위 표본 수(0=전수)")
    ap.add_argument("--seed", type=int, default=42, help="표본 시드(재현성)")
    ap.add_argument("--pdf", action="store_true", help="PDF 페이지수 교차검증(PyMuPDF)")
    ap.add_argument("-o", "--out", type=Path, default=None, help="결과 TSV 경로")
    ap.add_argument("--visible", action="store_true", help="한글 창 표시(디버깅)")
    ap.add_argument("--resume", action="store_true",
                   help="기존 -o TSV 의 처리분을 건너뛰고 이어서 진행(전수 배치 크래시 내성)")
    args = ap.parse_args(argv)

    if args.batch:
        orig_root, rt_root = Path(args.batch[0]), Path(args.batch[1])
        if not orig_root.is_dir() or not rt_root.is_dir():
            print("ERROR: --batch 의 두 경로는 모두 폴더여야 합니다.", file=sys.stderr)
            return 2
        pairs = collect_pairs_batch(orig_root, rt_root)
    else:
        if not args.orig_root or not args.rt_root:
            print("ERROR: --inventory 모드는 --orig-root 와 --rt-root 가 필요합니다.", file=sys.stderr)
            return 2
        status_filter = set(s.strip() for s in args.status.split(",")) if args.status else None
        pairs = collect_pairs_inventory(args.inventory, args.orig_root, args.rt_root, status_filter)

    if args.sample and len(pairs) > args.sample:
        rng = random.Random(args.seed)
        pairs = rng.sample(pairs, args.sample)
        pairs.sort(key=lambda p: p[2])

    return run(pairs, args.out, args.visible, args.pdf, resume=args.resume)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
