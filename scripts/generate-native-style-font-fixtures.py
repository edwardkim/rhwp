#!/usr/bin/env python3
"""공개 OFL 글꼴에서 Native 스타일 공급 검사용 SFNT를 재생성한다."""
import argparse
import hashlib
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
# 최초 fixture의 SFNT 수정 시각을 고정해 재생성 시 바이트도 유지한다.
CASES = [("Regular", 400, 3874872193), ("Bold", 700, 3874872201)]


def generate(output_dir):
    output_dir.mkdir(parents=True, exist_ok=True)
    for label, weight, modified in CASES:
        source = ROOT / "assets/fonts" / f"NotoSansKR-{label}.woff2"
        font = TTFont(source, recalcTimestamp=False)
        assert font["OS/2"].usWeightClass == weight
        options = subset.Options()
        options.name_IDs = ["*"]
        options.name_legacy = True
        options.name_languages = ["*"]
        options.recalc_timestamp = False
        selected = subset.Subsetter(options=options)
        selected.populate(text="AW 총투자")
        selected.subset(font)
        for name in font["name"].names:
            if name.nameID in (1, 16, 21):
                value = "RHWP Supplied Style Contract"
            elif name.nameID in (2, 17, 22):
                value = label
            elif name.nameID in (3, 4):
                value = "RHWP Supplied Style Contract " + label
            elif name.nameID == 6:
                value = "RHWPSuppliedStyleContract-" + label
            else:
                continue
            name.string = value.encode(name.getEncoding())
        font.flavor = None
        font["head"].modified = modified
        target = output_dir / f"RHWPSuppliedStyle-{label}.ttf"
        font.save(target)
        print(target.name, hashlib.sha256(target.read_bytes()).hexdigest())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path,
                        default=ROOT / "tests/fixtures/native_font_styles")
    generate(parser.parse_args().output_dir)
