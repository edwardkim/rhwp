#!/usr/bin/env python3
"""감소 잠재량 스캐너 (#2130, §5.1 v2.1)

리팩토링 대상 선정을 "최대 CC 순"이 아니라 "감소 잠재량 순"으로 전환하기 위한
정량 스캐너. 1차 리팩토링 17라운드에서 실증된 4유형을 파일/함수 단위로 점수화한다.

  ① 중복 블록   — strip-정규화 슬라이딩 윈도 해시의 동형 탐지 (R9: 104줄×4벌 → CC −9)
  ② 지역 macro  — fn-지역 macro_rules! × 호출부 수 × 본문 분기 (R16: ir_diff 116→37)
  ③ 판정 체인   — 인접 불리언 체인 let 들의 공통 guard 접두 (R9 판정군)
  ④ 소스분기    — is_hwp3_variant/is_hwpx_source 계열 참조 밀도 (감사 Stage 1 교차)

사용:
  python3 tools/reduction_potential.py                # src/ 전체, 상위 20 요약
  python3 tools/reduction_potential.py --top 40
  python3 tools/reduction_potential.py --file src/renderer/typeset.rs   # 단일 파일 상세
"""

import argparse
import hashlib
import re
import sys
from collections import defaultdict
from pathlib import Path

EXCLUDE_PARTS = ("tests", "diagnostics")
EXCLUDE_FILES = ("font_metrics_data.rs", "johab_map.rs", "pua_oldhangul.rs", "tests.rs")
SOURCE_FLAGS = re.compile(
    r"is_hwp3_variant|is_hwpx_source|is_hwp3_source|is_hwp5_origin_hwpx"
)
WINDOW = 24  # 중복 탐지 슬라이딩 윈도(줄) — R9 중복(104줄)의 안정 검출 하한


def runtime_files(root: Path):
    for p in sorted(root.rglob("*.rs")):
        rp = str(p)
        if any(f"/{part}/" in rp or rp.endswith(f"/{part}") for part in EXCLUDE_PARTS):
            continue
        if p.name in EXCLUDE_FILES:
            continue
        yield p


def normalize(line: str) -> str:
    line = re.sub(r"//.*", "", line)
    return re.sub(r"\s+", " ", line).strip()


def fn_spans(lines):
    """(name, start, end) — 러프한 fn 경계 (중괄호 균형)."""
    spans = []
    i = 0
    while i < len(lines):
        m = re.match(r"\s*(?:pub[^ ]* )?fn (\w+)", lines[i])
        if m and "{" in "".join(lines[i : i + 12]):
            d = 0
            started = False
            for j in range(i, len(lines)):
                d += lines[j].count("{") - lines[j].count("}")
                if "{" in lines[j]:
                    started = True
                if started and d == 0:
                    spans.append((m.group(1), i, j))
                    i = j
                    break
            else:
                break
        i += 1
    return spans


def scan_duplicates(files):
    """유형 ① — 파일 내부 동형 블록 (윈도 해시가 2회 이상)."""
    result = defaultdict(int)  # (file, fn) -> 잠재 줄수
    for p in files:
        lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        norm = [normalize(l) for l in lines]
        seen = defaultdict(list)
        for i in range(0, len(norm) - WINDOW):
            body = [x for x in norm[i : i + WINDOW] if x]
            if len(body) < WINDOW // 2:
                continue
            h = hashlib.md5("\n".join(body).encode()).hexdigest()
            seen[h].append(i)
        dup_lines = set()
        for h, idxs in seen.items():
            # 서로 겹치지 않는 출현 2회 이상만 중복으로 계상
            picked = []
            for i in sorted(idxs):
                if not picked or i >= picked[-1] + WINDOW:
                    picked.append(i)
            if len(picked) >= 2:
                for i in picked[1:]:  # 원본 1벌 제외분이 감소 잠재량
                    dup_lines.update(range(i, i + WINDOW))
        if not dup_lines:
            continue
        spans = fn_spans(lines)
        for name, s, e in spans:
            n = len([x for x in dup_lines if s <= x <= e])
            if n >= WINDOW:
                result[(str(p), name)] += n
    return result


def scan_local_macros(files):
    """유형 ② — fn-지역 macro_rules! 의 (호출부 수 × 본문 분기)."""
    result = defaultdict(int)
    branch_re = re.compile(r"\bif\b|\bmatch\b|&&|\|\|")
    for p in files:
        text = p.read_text(encoding="utf-8", errors="replace")
        lines = text.splitlines()
        spans = fn_spans(lines)
        for name, s, e in spans:
            body = "\n".join(lines[s : e + 1])
            for m in re.finditer(r"macro_rules!\s*(\w+)", body):
                mname = m.group(1)
                # macro 본문 분기 수 (러프: 정의 위치부터 200줄 내)
                seg = body[m.start() : m.start() + 4000]
                branches = len(branch_re.findall(seg))
                calls = len(re.findall(mname + r"!\s*[\(!]", body)) - 1
                if calls > 2 and branches > 2:
                    result[(str(p), name)] += calls * min(branches, 12)
    return result


def scan_guard_chains(files):
    """유형 ③ — 인접 불리언 체인 let 들의 공통 guard 접두 반복."""
    result = defaultdict(int)
    for p in files:
        lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        spans = fn_spans(lines)
        for name, s, e in spans:
            # let X = <chain>; 들의 처음 3개 조건 시그니처 수집
            sigs = []
            i = s
            while i <= e:
                m = re.match(r"\s+let (?:mut )?\w+ = (!?\w[\w.]*)\s*$", lines[i]) or re.match(
                    r"\s+let (?:mut )?\w+ = if (!?\w[\w.]*)", lines[i]
                )
                if m and i + 2 <= e and "&&" in lines[i + 1]:
                    conds = []
                    for j in range(i, min(i + 4, e)):
                        c = re.findall(r"(!?[\w.]+(?:\([^()]*\))?)", lines[j])
                        conds.extend(c[:2])
                    sigs.append(tuple(conds[:3]))
                i += 1
            counts = defaultdict(int)
            for t in sigs:
                counts[t] += 1
            rep = sum((c - 1) * 3 for c in counts.values() if c >= 3)
            if rep:
                result[(str(p), name)] += rep
    return result


def scan_source_flags(files):
    """유형 ④ — 소스분기 참조 밀도 (Phase P / 감사 Stage 1 교차 등재)."""
    result = defaultdict(int)
    for p in files:
        lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        spans = fn_spans(lines)
        for name, s, e in spans:
            n = sum(1 for l in lines[s : e + 1] if SOURCE_FLAGS.search(l))
            if n >= 2:
                result[(str(p), name)] += n * 4
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--top", type=int, default=20)
    ap.add_argument("--file", help="단일 파일만 스캔")
    ap.add_argument("--root", default="src")
    args = ap.parse_args()

    files = [Path(args.file)] if args.file else list(runtime_files(Path(args.root)))
    scans = {
        "①중복": scan_duplicates(files),
        "②macro": scan_local_macros(files),
        "③체인": scan_guard_chains(files),
        "④소스분기": scan_source_flags(files),
    }
    total = defaultdict(lambda: defaultdict(int))
    for kind, res in scans.items():
        for key, v in res.items():
            total[key][kind] = v

    rows = []
    for (f, fn), kinds in total.items():
        score = sum(kinds.values())
        rows.append((score, f, fn, kinds))
    rows.sort(reverse=True)

    print(f"감소 잠재량 상위 {args.top} (점수 = ①중복줄 + ②macro + ③체인 + ④소스분기×4)")
    print(f"{'점수':>5}  {'함수':<44} {'내역'}")
    for score, f, fn, kinds in rows[: args.top]:
        detail = " ".join(f"{k}={v}" for k, v in sorted(kinds.items()) if v)
        loc = f.replace("src/", "")
        print(f"{score:>5}  {fn:<44} {loc}  [{detail}]")


if __name__ == "__main__":
    sys.exit(main())
