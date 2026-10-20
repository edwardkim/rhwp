#!/usr/bin/env python3
"""Generate OFL subsets for the real-document Native equation font contract."""
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
FAMILY = "RHWP Equation Contract"
CJK = "평점입찰가격배한도최저해당추정의상0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz =+-×%()"
CASES = [
    ("NotoSansKR-Regular.ttf", "RHWPEquationCJKRegular.ttf", FAMILY, CJK),
    ("NotoSansKR-ExtraLight.ttf", "RHWPEquationCJKLight.ttf", FAMILY,
     "평점입찰가격배한도최저해당추정의상0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz =+-×%()"),
    ("NotoSansKR-Regular.ttf", "RHWPEquationLatinOnly.ttf", FAMILY,
     "".join(chr(c) for c in range(32, 127))),
]


def name_fixture(font, target, family):
    """윤곽선 출처의 고지는 보존하고 수정본의 식별 이름만 고유하게 지정한다."""
    suffix = Path(target).stem.removeprefix("RHWPEquation")
    names = {
        1: family, 16: family, 21: family,
        3: f"RHWP Equation Contract {suffix};1.0",
        4: f"{family} {suffix}",
        6: f"RHWPEquationContract-{suffix}",
        18: f"{family} {suffix}",
    }
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID].encode(record.getEncoding())


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
        name_fixture(font, target, family)
        font.save(ROOT / "tests/fixtures/fonts" / target)


if __name__ == "__main__":
    main()
