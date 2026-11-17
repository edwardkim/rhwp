"""Generate tiny OFL program-selection controls; never use proprietary outlines."""
from copy import deepcopy
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTCollection, TTFont

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "tests/fixtures/fonts"
FAMILY = "RHWP Native Container Probe"
font = TTFont(DEST / "RHWPEquationLatinOnly.ttf", recalcTimestamp=False)
options = subset.Options()
options.name_IDs = ["*"]
options.name_legacy = True
options.name_languages = ["*"]
subsetter = subset.Subsetter(options=options)
subsetter.populate(text="A ")
subsetter.subset(font)


def variant(label, narrow=False, family=FAMILY):
    result = deepcopy(font)
    for record in result["name"].names:
        if record.nameID in (1, 3, 4, 6, 16, 17, 18, 21):
            value = family if record.nameID in (1, 16, 21) else (
                "Regular" if record.nameID == 17 else family.replace(" ", "") + "-" + label
            )
            record.string = value.encode(record.getEncoding())
    if narrow:
        glyph = result.getBestCmap()[ord("A")]
        coords = result["glyf"][glyph].coordinates
        for index, (x, y) in enumerate(coords):
            coords[index] = (round(x / 2), y)
        advance, lsb = result["hmtx"][glyph]
        result["hmtx"][glyph] = (round(advance / 2), round(lsb / 2))
    return result


variant("TTF").save(DEST / "RHWPNativeContainer.ttf")
collection = TTCollection()
# The requested family deliberately occupies a nonzero collection index.
collection.fonts = [variant("Control", family=FAMILY + " Control"), variant("TTC", True)]
collection.save(DEST / "RHWPNativeContainer.ttc")
