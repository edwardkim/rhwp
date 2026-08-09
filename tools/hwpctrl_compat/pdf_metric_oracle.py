"""한글 PDF 를 **정밀 폰트-metric 오라클**로 삼아 rhwp 의 ASCII 폭 표를 검증한다.

## 왜 PDF 인가

`KeyIndicator.pos` 는 캐럿 x 를 반각 단위로 반올림해 sub-pixel 폭을 못 가른다(§4.28). 한글이
저장한 **PDF 의 글리프 origin** 은 실제 렌더 advance 라 훨씬 정밀하다(HCRBatang 확정, §4.29).
글리프 origin 델타를 폰트 크기로 나누면 advance/em 이 나오고, 이를 rhwp 의 per-glyph 표
(`HAANSOFT_BATANG_ASCII` 등)와 직접 견준다.

## 전역 장평 정규화

한글은 이 표본을 **장평 97%** 로 렌더한다(전각 한국어가 1.0em 이 아니라 0.972em). 그래서
라틴 advance 도 다 같은 비율로 눌린다. 한국어 전각을 1.0 기준으로 삼아 나누면 그 전역 배율이
지워지고 **글자별 표 오차**만 남는다.

## 쓰임

    python tools/hwpctrl_compat/pdf_metric_oracle.py samples/re-font-batang-hancom.hwp
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / "output" / "poc" / "hwpctrl" / "caret_pdf"

# rhwp 의 함초롬바탕 ASCII 표(text_measurement.rs HAANSOFT_BATANG_ASCII, 0x20~0x7E).
HAANSOFT_BATANG_ASCII = [
    0.3330, 0.4160, 0.4160, 0.8330, 0.6250, 0.9160, 0.8330, 0.2500,
    0.5000, 0.5000, 0.5000, 0.8330, 0.2910, 0.8330, 0.2910, 0.3330,
    0.5830, 0.5830, 0.5830, 0.5830, 0.5830, 0.5830, 0.5830, 0.5830,
    0.5830, 0.5830, 0.3330, 0.3330, 0.8330, 0.8330, 0.8330, 0.5000,
    1.0000, 0.7500, 0.6660, 0.6660, 0.7080, 0.6660, 0.6250, 0.7080,
    0.7500, 0.3750, 0.4580, 0.7500, 0.6250, 0.9160, 0.7500, 0.7080,
    0.6250, 0.7080, 0.6660, 0.6250, 0.7500, 0.7500, 0.7080, 0.9580,
    0.6660, 0.6660, 0.6250, 0.5000, 0.3330, 0.5000, 1.0000, 0.5000,
    0.5830, 0.5000, 0.5410, 0.5000, 0.5410, 0.5410, 0.3750, 0.5410,
    0.5410, 0.2910, 0.2910, 0.5410, 0.2910, 0.8330, 0.5410, 0.5410,
    0.5410, 0.5410, 0.4160, 0.5000, 0.3750, 0.5410, 0.5410, 0.7910,
    0.5830, 0.5830, 0.4580, 0.5830, 0.5830, 0.5830, 0.7910,
]


def table(c: str) -> float | None:
    cp = ord(c)
    return HAANSOFT_BATANG_ASCII[cp - 0x20] if 0x20 <= cp < 0x7F else None


def export_pdf(sample: str, pdf: Path, timeout: int) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    code = f'''
from pyhwpx import Hwp
hwp = Hwp(new=True, visible=False); com = hwp.hwp
com.Open(r"{REPO / sample}", "", "")
assert com.EditMode == 1, "읽기 전용"
ok = com.SaveAs(r"{pdf}", "PDF", "")
com.Quit()
raise SystemExit(0 if ok else 1)
'''
    proc = subprocess.run([sys.executable, "-c", code], timeout=timeout, check=False)
    if proc.returncode != 0 or not pdf.exists():
        raise SystemExit(f"PDF 내보내기 실패: {sample}")


def pdf_advances(pdf: Path) -> tuple[dict[str, float], float, set]:
    """PDF 첫 줄의 글자별 advance/em(중복은 평균)과 한국어 전각 배율을 낸다."""
    import fitz

    doc = fitz.open(str(pdf))
    d = doc[0].get_text("rawdict")
    fonts = set()
    glyphs = []
    size = None
    for block in d["blocks"]:
        for line in block.get("lines", []):
            for span in line["spans"]:
                fonts.add((span["font"], round(span["size"], 2)))
                size = span["size"]
                for ch in span["chars"]:
                    glyphs.append((ch["c"], ch["origin"][0], ch["origin"][1]))
    y0 = glyphs[0][2]
    line = [g for g in glyphs if abs(g[2] - y0) < 2]
    adv: dict[str, list] = defaultdict(list)
    korean = []
    for i in range(len(line) - 1):
        c = line[i][0]
        a = (line[i + 1][1] - line[i][1]) / size
        if "가" <= c <= "힣":  # 완성형 한글 = 전각
            korean.append(a)
        elif c != " " and table(c) is not None:
            adv[c].append(a)
    scale = sum(korean) / len(korean) if korean else 1.0
    return {c: sum(v) / len(v) for c, v in adv.items()}, scale, fonts


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("sample")
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--reexport", action="store_true", help="PDF 를 새로 내보낸다")
    args = ap.parse_args()

    pdf = OUT / (Path(args.sample).stem + ".pdf")
    if args.reexport or not pdf.exists():
        export_pdf(args.sample, pdf, args.timeout)

    adv, scale, fonts = pdf_advances(pdf)
    print(f"PDF 폰트: {fonts}")
    print(f"전역 장평(한국어 전각 배율): {scale:.4f}\n")
    print(f"{'글자':<5}{'PDF/em':>8}{'정규화':>8}{'표값':>8}{'차%':>7}")
    worst = 0.0
    flags = []
    for c in sorted(adv):
        norm = adv[c] / scale  # 전역 장평 제거
        t = table(c)
        pct = (norm - t) / t * 100
        worst = max(worst, abs(pct))
        mark = " <<<" if abs(pct) > 8 else ""
        if abs(pct) > 8:
            flags.append((c, round(norm, 3), t, round(pct, 1)))
        print(f"'{c}'  {adv[c]:8.3f}{norm:8.3f}{t:8.3f}{pct:+6.1f}%{mark}")
    print(f"\n최대 편차 {worst:.1f}% · 8% 초과 {len(flags)}자: "
          f"{[(c, n, t) for c, n, t, _ in flags]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
