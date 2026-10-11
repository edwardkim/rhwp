#!/usr/bin/env python3
"""Generate OFL subsets for the real-document Native equation font contract."""
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
CASES = [
    ("NotoSansKR-ExtraLight.ttf", "RHWPEquationCJKLight.ttf", "Noto Sans KR",
     "평점입찰가격배한도최저해당추정의상0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz =+-×%()"),
    ("NotoSansKR-Regular.ttf", "RHWPEquationLatinOnly.ttf", "Batang",
     "".join(chr(c) for c in range(32, 127))),
]


def main():
    for source, target, family, characters in CASES:
        font = TTFont(ROOT / "ttfs/opensource" / source, recalcTimestamp=False)
        options = subset.Options()
        options.name_IDs = ["*"]
        options.name_legacy = True
        options.name_languages = ["*"]
        options.recalc_timestamp = False
        selected = subset.Subsetter(options=options)
        selected.populate(text=characters)
        selected.subset(font)
        for name in font["name"].names:
            if name.nameID in (1, 16):
                name.string = family.encode(name.getEncoding())
        font.save(ROOT / "tests/fixtures/fonts" / target)


if __name__ == "__main__":
    main()
