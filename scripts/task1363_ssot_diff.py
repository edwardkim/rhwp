#!/usr/bin/env python3
"""Task #1363 — 미주 높이 모델 SSOT divergence 측정 하니스.

미주 para 단위로 typeset 누적(acc)이 layout 순차 렌더 높이(line_advances_sum)를
얼마나 정확히 예측하는지 정량화한다. `RHWP_EN_SSOT` 레벨(legacy/A/B/on)별로
(1) para별 SSOT 등식 잔차(acc − line_adv_sum), (2) exam별 미주 본문 초과(overflow px)
를 산출하여 divergence 이전 전후 변화를 비교한다.

사용:
  scripts/task1363_ssot_diff.py --level A
  scripts/task1363_ssot_diff.py --level A --baseline legacy   # legacy 대비 변화
  scripts/task1363_ssot_diff.py --level A --out mydocs/report/task1363_ssot_diff_stage3.tsv

stderr 의 `EN_SSOT pi=... acc_legacy=... acc=... line_adv_sum=...` 라인을 수집한다.
overflow 는 export-svg 산출 SVG 의 max text y 와 페이지 height 차이 합(issue_1082 와 동일식).
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# issue_1082 골든 가드 대상 exam (비대상 0.0 유지 / 대상 50.1 감소 판정).
EXAMS = [
    ("3-09'23 hwp", "samples/3-09월_교육_통합_2023.hwp"),
    ("3-09'23 hwpx", "samples/3-09월_교육_통합_2023.hwpx"),
    ("3-09'22 hwp", "samples/3-09월_교육_통합_2022.hwp"),
    ("3-10'22 hwp", "samples/3-10월_교육_통합_2022.hwp"),
    ("3-11'22 hwp", "samples/3-11월_실전_통합_2022.hwp"),
    ("3-09'24 sep20/20 (대상)", "samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp"),
]

EN_SSOT_RE = re.compile(
    r"EN_SSOT pi=(\d+) rewind=(\w+) acc_legacy=([\d.]+) acc=([\d.]+) "
    r"line_adv_sum=([\d.]+) fit=([\d.]+) h4f=([\d.]+)"
)
SVG_HEIGHT_RE = re.compile(r'height="([\d.]+)"')
TEXT_Y_RE = re.compile(r'<text[^>]* y="([\d.]+)"')


@dataclass
class ParaRow:
    pi: int
    rewind: bool
    acc_legacy: float
    acc: float
    line_adv_sum: float
    fit: float
    h4f: float

    @property
    def ssot_residual(self) -> float:
        """SSOT 등식 잔차: acc 가 line_adv_sum 을 얼마나 예측 못 하는지."""
        return self.acc - self.line_adv_sum

    @property
    def legacy_residual(self) -> float:
        return self.acc_legacy - self.line_adv_sum


@dataclass
class ExamResult:
    name: str
    rel: str
    rows: list[ParaRow] = field(default_factory=list)
    overflow_px: float = 0.0

    @property
    def rewind_rows(self) -> list[ParaRow]:
        return [r for r in self.rows if r.rewind]

    @property
    def abs_ssot_residual(self) -> float:
        return sum(abs(r.ssot_residual) for r in self.rows)

    @property
    def abs_legacy_residual(self) -> float:
        return sum(abs(r.legacy_residual) for r in self.rows)


def run_exam(rel: str, level: str) -> ExamResult:
    src = REPO / rel
    if not src.exists():
        print(f"  ! 누락: {rel}", file=sys.stderr)
        return ExamResult(rel, rel)
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ)
        env["RHWP_EN_SSOT_DEBUG"] = "1"
        # 기본값이 B(Stage 4 승격)이므로 모든 레벨을 명시적으로 설정한다.
        # (미설정 = 기본 B 이므로 legacy 를 unset 으로 두면 안 됨.)
        env["RHWP_EN_SSOT"] = level
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--bin", "rhwp", "--",
             "export-svg", str(src), "-o", tmp],
            cwd=REPO, env=env, capture_output=True, text=True,
        )
        res = ExamResult(rel, rel)
        for line in proc.stderr.splitlines():
            m = EN_SSOT_RE.search(line)
            if m:
                res.rows.append(ParaRow(
                    pi=int(m.group(1)),
                    rewind=m.group(2) == "true",
                    acc_legacy=float(m.group(3)),
                    acc=float(m.group(4)),
                    line_adv_sum=float(m.group(5)),
                    fit=float(m.group(6)),
                    h4f=float(m.group(7)),
                ))
        res.overflow_px = compute_overflow(Path(tmp))
        return res


def compute_overflow(svg_dir: Path) -> float:
    """issue_1082 와 동일식: 페이지별 (max text y − height) 양수 합산."""
    total = 0.0
    for svg_path in sorted(svg_dir.glob("*.svg")):
        text = svg_path.read_text(encoding="utf-8", errors="ignore")
        hm = SVG_HEIGHT_RE.search(text)
        if not hm:
            continue
        height = float(hm.group(1))
        max_y = max((float(y) for y in TEXT_Y_RE.findall(text)), default=0.0)
        if max_y > height:
            total += max_y - height
    return total


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--level", default="A", choices=["legacy", "A", "B", "on"])
    ap.add_argument("--baseline", default=None,
                    help="비교 기준 레벨(예: legacy). 지정 시 overflow/잔차 델타 출력")
    ap.add_argument("--out", default=None, help="para 행 TSV 출력 경로")
    args = ap.parse_args()

    print(f"=== RHWP_EN_SSOT={args.level} ===")
    results = [run_exam(rel, args.level) for _, rel in EXAMS]
    base = None
    if args.baseline:
        print(f"=== baseline RHWP_EN_SSOT={args.baseline} ===")
        base = [run_exam(rel, args.baseline) for _, rel in EXAMS]

    print(f"\n{'exam':<28} {'overflow':>9} {'|ssot_res|':>11} {'rewind#':>8}", end="")
    if base:
        print(f" {'Δoverflow':>10}", end="")
    print()
    for i, (name, _) in enumerate(EXAMS):
        r = results[i]
        line = f"{name:<28} {r.overflow_px:>9.1f} {r.abs_ssot_residual:>11.1f} {len(r.rewind_rows):>8}"
        if base:
            line += f" {r.overflow_px - base[i].overflow_px:>+10.1f}"
        print(line)

    if args.out:
        out = Path(args.out)
        out.parent.mkdir(parents=True, exist_ok=True)
        with out.open("w", encoding="utf-8") as f:
            f.write("exam\tpi\trewind\tacc_legacy\tacc\tline_adv_sum\tssot_res\tlegacy_res\n")
            for i, (name, _) in enumerate(EXAMS):
                for row in results[i].rows:
                    f.write(f"{name}\t{row.pi}\t{row.rewind}\t{row.acc_legacy:.1f}\t"
                            f"{row.acc:.1f}\t{row.line_adv_sum:.1f}\t"
                            f"{row.ssot_residual:.1f}\t{row.legacy_residual:.1f}\n")
        print(f"\nTSV → {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
