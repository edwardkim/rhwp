"""#1658 valign(세로정렬) offset 회귀 게이트 — 합성 fixture 의 CENTERME 마커 위치 검증.

has_nested 셀의 total_content_height(stored vpos vs computed) 산정이 over-count 되면 Center/Bottom
정렬 offset 이 0 에 수렴해 상단정렬(데이터 손실 아님이나 시각 오류)된다(kkyu8925 제보, #1658).
본 게이트는 외부 셀 마커 `CENTERME` 의 baseline Y(pt)를 측정해 BUG(상단)↔FIX(중앙/하단) 를 판정한다.

원리: rhwp SVG 는 문자별 `<text x y>`. y 로 줄을 묶어 `CENTERME` 줄의 baseline y(px)→pt(×72/96).
fixture 별 BUG/FIX 기대값(README, devel 30931679 기준)과 대조. 측정 baseline 은 README 대비 약 +8pt
일관 오프셋이 있으므로 **BUG/FIX 중 가까운 쪽**으로 판정(절대값 아닌 분류).

사용:
  python tools/valign_offset_gate.py [--dir samples/valign_fixtures] [--exe <rhwp>]
종료코드: over-count fixture 중 하나라도 BUG(상단)면 1 (= 회귀/미수정), 전부 FIX 면 0.
"""
from __future__ import annotations

import argparse
import glob
import os
import re
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
# fixture: (외부셀 valign, BUG pt, FIX pt). undercount 가드는 BUG==FIX(불변 검증).
FIXTURES = {
    "centered_cell_nested_table": ("Center", 107.7, 185.7),
    "cell_vcenter_multi_nested_overcount": ("Center", 105.6, 197.7),
    "cell_vbottom_nested_overcount": ("Bottom", 109.8, 265.8),
    "cell_vcenter_nested_undercount": ("Center", 212.2, 212.2),
}


def _winpath(p: str) -> str:
    m = re.match(r"^/([a-zA-Z])/(.*)$", p)
    return f"{m.group(1).upper()}:/{m.group(2)}" if (m and sys.platform == "win32") else p


def centerme_pt(svg_dir: str) -> float | None:
    """SVG 디렉토리에서 CENTERME 줄 baseline y(pt)."""
    for s in sorted(glob.glob(os.path.join(svg_dir, "*.svg"))):
        t = Path(s).read_text(encoding="utf-8")
        rows: dict[float, list[tuple[float, str]]] = defaultdict(list)
        for m in re.finditer(r'<text x="([\d.]+)" y="([\d.]+)"[^>]*>([^<])', t):
            rows[round(float(m.group(2)), 1)].append((float(m.group(1)), m.group(3)))
        for y, chs in sorted(rows.items()):
            if "CENTERME" in "".join(c for _, c in sorted(chs)):
                return y * 72.0 / 96.0
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", default=str(REPO / "samples" / "valign_fixtures"))
    ap.add_argument("--exe", default="target/release/rhwp.exe"
                    if sys.platform == "win32" else "target/release/rhwp")
    a = ap.parse_args()
    fix_dir = _winpath(a.dir)
    exe = _winpath(a.exe)

    n_bug = 0
    n_missing = 0
    for name, (valign, bug, fix) in FIXTURES.items():
        src = os.path.join(fix_dir, name + ".hwpx")
        if not os.path.exists(_winpath(src)):
            print(f"  없음 {name}")
            n_missing += 1
            continue
        with tempfile.TemporaryDirectory() as td:
            subprocess.run([exe, "export-svg", _winpath(src), "-o", td],
                           capture_output=True, text=True, encoding="utf-8",
                           errors="replace", timeout=120)
            pt = centerme_pt(td)
        if pt is None:
            print(f"  마커없음 {name}")
            n_missing += 1
            continue
        is_guard = abs(bug - fix) < 1.0
        near_bug = abs(pt - bug) < abs(pt - fix)
        if is_guard:
            verdict = "가드OK" if abs(pt - fix) < 30 else "가드깨짐"
            bad = abs(pt - fix) >= 30
        else:
            verdict = "BUG(상단)" if near_bug else f"FIX({valign})"
            bad = near_bug
        if bad:
            n_bug += 1
        print(f"  {name}: y={pt:.1f}pt (BUG~{bug}/FIX~{fix}) -> {verdict}")
    print(f"\n[valign-gate] BUG(미수정)={n_bug} 누락={n_missing}")
    return 1 if n_bug > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
