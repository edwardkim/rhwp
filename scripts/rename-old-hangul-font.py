#!/usr/bin/env python3
"""OFL 옛한글 서브셋의 윤곽·고지를 보존하고 RHWP 고유 이름을 지정한다."""
import argparse
from pathlib import Path

from fontTools.ttLib import TTFont

FAMILY = "RHWP Old Hangul"
POSTSCRIPT = "RHWPOldHangul-Regular"


def generate(source, woff2_output, otf_output):
    font = TTFont(source, recalcTimestamp=False)
    names = {1: FAMILY, 16: FAMILY, 21: FAMILY,
             3: POSTSCRIPT + ";1.0", 4: FAMILY + " Regular",
             6: POSTSCRIPT, 18: FAMILY + " Regular"}
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID].encode(record.getEncoding())
    cff = font["CFF "].cff
    cff.fontNames = [POSTSCRIPT]
    for top in cff.topDictIndex:
        top.FamilyName = FAMILY
        top.FullName = FAMILY + " Regular"
        for index, private_face in enumerate(top.FDArray):
            if hasattr(private_face, "FontName"):
                private_face.FontName = f"{POSTSCRIPT}-{index}"
    # name 0/13/14와 CFF Notice 등 원본 권리·라이선스 고지는 변경하지 않는다.
    for target, flavor in [(woff2_output, "woff2"), (otf_output, None)]:
        target.parent.mkdir(parents=True, exist_ok=True)
        font.flavor = flavor
        font.save(target)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--woff2-output", type=Path,
                        default=Path("assets/fonts/RHWPOldHangul-Regular.woff2"))
    parser.add_argument("--otf-output", type=Path,
                        default=Path("ttfs/opensource/RHWPOldHangul-Regular.otf"))
    args = parser.parse_args()
    generate(args.source, args.woff2_output, args.otf_output)
