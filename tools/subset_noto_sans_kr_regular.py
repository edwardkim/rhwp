#!/usr/bin/env python3
"""CanvasKit용 Noto Sans KR Regular 정적 서브셋을 생성한다."""

from __future__ import annotations

import argparse
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont


# 기존 Regular 번들의 cmap을 유지해 문서 본문의 한글/라틴 coverage를 바꾸지 않는다.
BASE_UNICODES = (
    "U+0020-007E,U+00A0-00FF,U+0152-0153,U+02BB,U+0300-0301,U+0304,"
    "U+2002,U+2013-2014,U+2018-201A,U+201C-201E,U+2022,U+2026,"
    "U+2032-2033,U+2039-203A,U+20AC,U+2122,U+2191,U+2193,U+2212,"
    "U+2215,U+3131-318E,U+3200-321C,U+3260-327B,U+AC00-D7A3"
)

# KS X 1001 글머리/도형과 표 테두리에 쓰이는 범위를 명시적으로 포함한다.
REQUIRED_SYMBOL_UNICODES = "U+2500-257F,U+25A0-25FF"
REQUIRED_CODEPOINTS = (0x2500, 0x25A0, 0x25AA, 0x25A1, 0x25CB)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True, help="Google Fonts Noto Sans KR variable TTF")
    parser.add_argument(
        "--ttf-output",
        type=Path,
        default=Path("ttfs/opensource/NotoSansKR-Regular.ttf"),
        help="정적 Regular TTF 출력 경로",
    )
    parser.add_argument(
        "--woff2-output",
        type=Path,
        default=Path("assets/fonts/NotoSansKR-Regular.woff2"),
        help="웹 번들 WOFF2 출력 경로",
    )
    return parser.parse_args()


def verify_coverage(font: TTFont) -> None:
    cmap = font.getBestCmap()
    missing = [f"U+{codepoint:04X}" for codepoint in REQUIRED_CODEPOINTS if codepoint not in cmap]
    if missing:
        raise RuntimeError(f"필수 기호가 서브셋에 없습니다: {', '.join(missing)}")


def set_regular_names(font: TTFont) -> None:
    """가변 source의 Thin 기본 이름이 정적 Regular 번들에 남지 않게 한다."""
    replacements = {
        1: "Noto Sans KR",
        2: "Regular",
        4: "Noto Sans KR Regular",
        6: "NotoSansKR-Regular",
        16: "Noto Sans KR",
        17: "Regular",
    }
    for record in font["name"].names:
        replacement = replacements.get(record.nameID)
        if replacement is not None:
            record.string = replacement.encode(record.getEncoding())


def main() -> None:
    args = parse_args()
    font = TTFont(args.source)
    if "fvar" not in font:
        raise RuntimeError("입력 폰트가 가변 폰트가 아닙니다")

    regular = instantiateVariableFont(font, {"wght": 400}, inplace=False)
    set_regular_names(regular)
    options = subset.Options()
    options.layout_features = ["*"]
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=subset.parse_unicodes(f"{BASE_UNICODES},{REQUIRED_SYMBOL_UNICODES}"))
    subsetter.subset(regular)
    verify_coverage(regular)

    # 입력 source의 timestamp를 보존해 동일 입력에서 바이트 단위 재현성을 유지한다.
    regular.recalcTimestamp = False
    args.ttf_output.parent.mkdir(parents=True, exist_ok=True)
    args.woff2_output.parent.mkdir(parents=True, exist_ok=True)
    regular.flavor = None
    regular.save(args.ttf_output)
    regular.flavor = "woff2"
    regular.save(args.woff2_output)
    print(f"wrote {args.ttf_output} and {args.woff2_output}")


if __name__ == "__main__":
    main()
