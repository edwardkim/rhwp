#!/usr/bin/env python3
"""로드맵 진행률 기계 산출 — 트랙 문서 태그 파서 (R94 P2, #4107).

## 왜 있는가

로드맵 R1~R100 의 진행률이 손집계였다. 손집계는 지도가 늘면 실물과 갈라진다 —
설계 문서(mydocs/tech/autonomous_maintenance/progress_machine_readable.md) §1 이
#3907 안에서 같은 부류 오류 6건 재발을 실측했다. 이 도구는 canonical 인 트랙 문서
세트(mydocs/tech/agent_roadmap/track_*.md)의 단계 제목 태그를 파싱해 집계를
기계가 내고, README 의 집계 블록을 그 산출로 대체한다.

## 무엇을 검증하는가 (설계 문서 §7 의 P2 — 형식 검증만)

- V1: R1~R100 결번·중복 없음
- V2: 등급이 통제 어휘(완료·실측·문서·이슈·가설) 안
- V6: README 의 기계 집계 블록 = 파싱 집계 (--write 로 블록 재생성)

## 무엇을 하지 않는가 (설계 문서 §5.5)

- 이슈 본문(#3907)을 읽거나 고치지 않는다 — 증거 토큰 검증(V3~V5)은
  P1(완료 정의 합의) 뒤의 몫이다.
- 등급을 추천하지 않는다. 새 R 항목을 만들지 않는다. 전부 사람 몫이다.

## 쓰는 법

    py tools/roadmap_progress.py            # 검증 — 집계 출력 + README 블록 대조
    py tools/roadmap_progress.py --write    # README 집계 블록 재생성

종료 코드: 0 통과 / 1 위반·불일치 / 2 사용법 오류
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

for _s in (sys.stdout, sys.stderr):
    if hasattr(_s, "reconfigure"):
        _s.reconfigure(encoding="utf-8", errors="replace")

ROADMAP_DIR = Path("mydocs/tech/agent_roadmap")
README_NAME = "README.md"
GRADES = ("완료", "실측", "문서", "이슈", "가설")

# 단계 제목 형식의 원천은 트랙 문서 자신이다(형식이 바뀌면 이 파서도 같은 PR 에서
# 바뀌어야 한다 — R94 착수 게이트 "태그 형식 안정"의 함의).
HEAD = re.compile(r"^##\s+R(\d+)\s+(.*?)\s*`\[([^\]`]+)\]`\s*$")
# 태그 없는 R 헤딩 — 형식 위반으로 별도 보고한다.
HEAD_LOOSE = re.compile(r"^##\s+R(\d+)\b")

BEGIN_MARK = "<!-- roadmap-progress:begin"
BEGIN_LINE = (
    "<!-- roadmap-progress:begin — tools/roadmap_progress.py --write 가 생성한다. "
    "손으로 고치지 말 것 -->"
)
END_MARK = "<!-- roadmap-progress:end -->"

TRACK_LABEL = {
    "a": "A 봉투",
    "b": "B 가드·보안",
    "c": "C 동시성",
    "d": "D 발견",
    "e": "E 실물 능력",
    "f": "F 규모",
    "g": "G 바인딩",
    "h": "H MCP",
    "i": "I 표준",
    "j": "J 자율",
}


def parse_tracks(root: Path):
    entries: dict[int, tuple[str, str, str]] = {}  # n -> (grade, track_key, title)
    problems: list[str] = []
    files = sorted((root / ROADMAP_DIR).glob("track_*.md"))
    if not files:
        problems.append(f"트랙 문서 없음 — {root / ROADMAP_DIR}")
        return entries, problems
    for f in files:
        m = re.match(r"track_([a-z])_", f.name)
        track_key = m.group(1) if m else "?"
        for i, line in enumerate(
            f.read_text(encoding="utf-8", errors="replace").splitlines(), 1
        ):
            hm = HEAD.match(line)
            if hm:
                n, title, grade = int(hm.group(1)), hm.group(2), hm.group(3)
                if grade not in GRADES:
                    problems.append(
                        f"{f.name}:{i} — R{n} 등급 `[{grade}]` 가 통제 어휘 밖 "
                        f"(허용: {'·'.join(GRADES)})"
                    )
                if n in entries:
                    problems.append(f"{f.name}:{i} — R{n} 중복 (이미 트랙 {entries[n][1]} 에)")
                entries[n] = (grade, track_key, title)
                continue
            lm = HEAD_LOOSE.match(line)
            if lm and not HEAD.match(line):
                problems.append(f"{f.name}:{i} — R{lm.group(1)} 헤딩에 등급 태그가 없다")
    missing = [n for n in range(1, 101) if n not in entries]
    if missing:
        problems.append(f"결번 {len(missing)}개 — {' '.join(f'R{n}' for n in missing)}")
    extra = [n for n in entries if not 1 <= n <= 100]
    if extra:
        problems.append(f"범위 밖 {' '.join(f'R{n}' for n in sorted(extra))}")
    return entries, problems


def build_block(entries) -> str:
    totals = {g: 0 for g in GRADES}
    per_track: dict[str, dict[str, int]] = {}
    for _n, (grade, track_key, _t) in sorted(entries.items()):
        if grade in totals:
            totals[grade] += 1
        per_track.setdefault(track_key, {g: 0 for g in GRADES})
        if grade in GRADES:
            per_track[track_key][grade] += 1
    total_line = " · ".join(f"{g} {totals[g]}" for g in GRADES)
    lines = [
        BEGIN_LINE,
        "",
        f"집계 (단계 제목 태그 전수, 기계 산출): **{total_line} = {sum(totals.values())}**",
        "",
        "| 트랙 | " + " | ".join(GRADES) + " | 계 |",
        "|---|" + "---|" * (len(GRADES) + 1),
    ]
    for key in sorted(per_track):
        row = per_track[key]
        label = TRACK_LABEL.get(key, key.upper())
        lines.append(
            f"| {label} | "
            + " | ".join(str(row[g]) for g in GRADES)
            + f" | {sum(row.values())} |"
        )
    lines.append(
        "| **합계** | "
        + " | ".join(f"**{totals[g]}**" for g in GRADES)
        + f" | **{sum(totals.values())}** |"
    )
    lines.append("")
    lines.append(END_MARK)
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="로드맵 진행률 기계 산출 (R94 P2)")
    ap.add_argument("--repo", default=".", help="저장소 루트 (기본: 현재 디렉터리)")
    ap.add_argument("--write", action="store_true", help="README 집계 블록을 재생성한다")
    args = ap.parse_args()

    root = Path(args.repo).resolve()
    readme = root / ROADMAP_DIR / README_NAME
    if not readme.exists():
        print(f"오류: {readme} 없음", file=sys.stderr)
        return 2

    entries, problems = parse_tracks(root)
    block = build_block(entries)

    for line in block.splitlines():
        if line.startswith("집계") or line.startswith("| "):
            print(line)

    if problems:
        print()
        print(f"형식 위반 {len(problems)}건:")
        for p in problems:
            print(f"  · {p}")
        return 1

    text = readme.read_text(encoding="utf-8", errors="replace")
    b, e = text.find(BEGIN_MARK), text.find(END_MARK)
    if b < 0 or e < 0 or e < b:
        print()
        print(
            f"README 에 기계 집계 블록 마커가 없다 — {BEGIN_MARK} … {END_MARK} 를 넣고 "
            "--write 로 채워라",
        )
        return 1
    current = text[b : e + len(END_MARK)]

    if args.write:
        if current == block:
            print()
            print("README 블록이 이미 최신 — 변경 없음")
            return 0
        readme.write_text(text[:b] + block + text[e + len(END_MARK) :], encoding="utf-8")
        print()
        print(f"README 블록 갱신 — {readme.relative_to(root)}")
        return 0

    if current != block:
        print()
        print("README 집계 블록이 파싱 집계와 다르다 — `--write` 로 재생성하라 (V6 위반)")
        return 1
    print()
    print("전부 통과 — 결번 0 · 중복 0 · 등급 어휘 일치 · README 집계 일치")
    return 0


if __name__ == "__main__":
    sys.exit(main())
