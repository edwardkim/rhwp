#!/usr/bin/env python3
"""Portable rejection tests; these do not execute the real GTK compositor."""
import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
from qualify_linux_tile_group import validate_stage


def fixture():
    stamp = {"documentId": 1, "revision": 2, "fontGeneration": 0,
        "viewGeneration": 3, "deviceGeneration": 7, "deviceScale": 1.25}
    placement = {"snapshot": 19, "scale": .55, "originX": 2040.25, "originY": .5}
    rects = [{"left": 0, "top": 0, "width": 2049, "height": 240},
             {"left": 2049, "top": 0, "width": 2049, "height": 240}]
    members = [{"textureId": ident, "deviceGeneration": 7, "acceptedStamp": stamp,
        "renderedStamp": stamp, "registered": True, "closing": False,
        "liveRasterSlots": 2, "rasterSlotLimit": 4, "framesSubmitted": 3,
        **dict.fromkeys(("wholeFrameReadbacks", "abandonedTargets", "orphanedOwners",
            "orphanedRasterSlots", "pendingRetireSlots", "snapshotReleaseFailures"), 0)} for ident in (11,12)]
    def proof(left, width, factor):
        return {"snapshot": 19, "stamp": stamp, "coordinateSpace": "viewportPixels",
            "viewport": {"width": width * factor, "height": 240 * factor},
            "placement": {"scale": .55 * factor, "originX": (2040.25-left) * factor, "originY": .5 * factor},
            "descriptor": {"width": width * factor, "height": 240 * factor,
                "stride": width * factor * 4, "pixelFormat": "rgba8Premultiplied", "colorSpace": "srgb", "origin": "topLeft"}}
    return {"protocol": 1, "nonce": "owned", "index": 3, "pid": 42,
        "width": 4098, "height": 240, "deviceScale": 1.25, "physicalOffset": [0,0],
        "stamp": stamp, "snapshotIds": [19], "tileRects": rects,
        "group": {"groupId": 11, "deviceGeneration": 7, "members": members},
        "submission": {"groupId": 11, "deviceGeneration": 7, "stamp": stamp,
            "tiles": [{"textureId": ident, "width": 2049, "height": 240,
                "placements": [{**placement, "originX": placement["originX"]-rect["left"]}]} for ident, rect in zip((11,12),rects)]},
        "strips": [{"left": left, "width": min(1024,4098-left),
            "cpu": proof(left,min(1024,4098-left),1), "reference": proof(left,min(1024,4098-left),2)} for left in range(0,4098,1024)]}


class TileQualification(unittest.TestCase):
    def test_large_actual_group_and_independent_bounded_source_strip_contract(self):
        self.assertEqual(validate_stage(fixture(), "owned", 3, 42), (4098,240,(0,0)))

    def test_missing_tile_or_aliasing_identity_cannot_qualify(self):
        for mutate in (lambda v: v["group"]["members"].pop(),
                       lambda v: v["group"]["members"][1].update(textureId=11),
                       lambda v: v["group"]["members"][1].update(deviceGeneration=8),
                       lambda v: v["group"]["members"][1].update(renderedStamp={})):
            value = copy.deepcopy(fixture()); mutate(value)
            with self.assertRaises(ValueError): validate_stage(value, "owned", 3, 42)

    def test_gap_overlap_or_wrong_tile_transform_cannot_qualify(self):
        for mutate in (lambda v: v["tileRects"][1].update(left=2050),
                       lambda v: v["tileRects"][1].update(left=2048),
                       lambda v: v["submission"]["tiles"][1]["placements"][0].update(originX=0),
                       lambda v: v["submission"]["tiles"][1]["placements"][0].update(snapshot=20)):
            value = copy.deepcopy(fixture()); mutate(value)
            with self.assertRaises(ValueError): validate_stage(value, "owned", 3, 42)

    def test_independent_source_must_cover_every_pixel_with_same_stamp_and_translation(self):
        for mutate in (lambda v: v["strips"].pop(),
                       lambda v: v["strips"][2]["reference"].update(snapshot=20),
                       lambda v: v["strips"][2]["reference"]["placement"].update(originX=0),
                       lambda v: v["strips"][2]["reference"]["descriptor"].update(stride=4096)):
            value = copy.deepcopy(fixture()); mutate(value)
            with self.assertRaises(ValueError): validate_stage(value, "owned", 3, 42)


if __name__ == "__main__":
    unittest.main()
