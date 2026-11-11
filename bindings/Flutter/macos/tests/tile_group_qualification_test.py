#!/usr/bin/env python3
"""Portable rejection checks for actual two-AppKit-view capture evidence."""

import copy
import unittest

from native_view_qualification_test import qualifier, result, stage


def tiled_stage():
    value = stage()
    value.update(width=321, tileGroup=True, groupId=11, tileTextureIds=[11, 12],
                 physicalTiles=[[0, 0, 161, 240], [161, 0, 160, 240]],
                 geometryProof={"cpu": {"snapshot": 99}})
    value["limits"].update(tileGroups=True, sharedSnapshotLeases=True,
                           maxTileGroupTiles=2, maxTileGroupSnapshots=4)
    value["nativeTileMembers"] = [
        {"textureId": identity, "deviceGeneration": 1,
         "acceptedStamp": copy.deepcopy(value["stamp"]), "renderedStamp": copy.deepcopy(value["stamp"]),
         "acceptedSnapshots": [99], "ready": True, "viewBound": True, "layerReady": True,
         "hasFrame": True, "covered": False, "drawableWidth": width, "drawableHeight": 240,
         "drawableDeviceScale": value["deviceScale"], "frames": 1}
        for identity, width in [(11, 161), (12, 160)]
    ]
    return value


class TileGroupCaptureContracts(unittest.TestCase):
    def validate(self, value):
        return qualifier.validate_stage(value, nonce="owned", index=1, pid=123, tile_group=True)

    def test_odd_width_pair_and_vertical_pair_cover_the_complete_viewport(self):
        value = tiled_stage()
        self.assertEqual(self.validate(value)[:2], (321, 240))
        value.update(width=240, height=321, physicalTiles=[[0, 0, 240, 161], [0, 161, 240, 160]])
        for member, height in zip(value["nativeTileMembers"], [161, 160]):
            member.update(drawableWidth=240, drawableHeight=height)
        self.assertEqual(self.validate(value)[:2], (240, 321))

    def test_missing_member_duplicate_identity_or_foreign_group_is_rejected(self):
        for mutate in [
            lambda v: v.pop("tileGroup"),
            lambda v: v["nativeTileMembers"].pop(),
            lambda v: v["nativeTileMembers"][1].update(textureId=11),
            lambda v: v.update(groupId=12),
            lambda v: v.update(tileTextureIds=[12, 11]),
            lambda v: v["nativeTileMembers"][1].update(deviceGeneration=2),
        ]:
            with self.subTest(mutation=mutate):
                value = tiled_stage(); mutate(value)
                with self.assertRaises(ValueError): self.validate(value)

    def test_gap_overlap_fractional_or_wrong_actual_drawable_is_rejected(self):
        for mutate in [
            lambda v: v.update(physicalTiles=[[0, 0, 160, 240], [161, 0, 160, 240]]),
            lambda v: v.update(physicalTiles=[[0, 0, 162, 240], [161, 0, 160, 240]]),
            lambda v: v.update(physicalTiles=[[0, 0, 161.0, 240], [161, 0, 160, 240]]),
            lambda v: v["nativeTileMembers"][1].update(drawableWidth=161),
            lambda v: v["nativeTileMembers"][1].update(drawableDeviceScale=1),
        ]:
            value = tiled_stage(); mutate(value)
            with self.assertRaises(ValueError): self.validate(value)

    def test_each_full_stamp_field_and_shared_snapshot_receipt_is_required(self):
        for field in ("documentId", "revision", "fontGeneration", "viewGeneration", "deviceGeneration", "deviceScale"):
            value = tiled_stage()
            value["nativeTileMembers"][1]["renderedStamp"][field] += 1
            with self.assertRaisesRegex(ValueError, "full stamp"): self.validate(value)
        for ids in ([], [98], [99, 99]):
            value = tiled_stage(); value["nativeTileMembers"][1]["acceptedSnapshots"] = ids
            with self.assertRaisesRegex(ValueError, "CPU snapshot"): self.validate(value)

    def test_one_ready_tile_or_covered_native_view_cannot_pass(self):
        for key, changed in [("ready", False), ("hasFrame", False), ("layerReady", False),
                             ("viewBound", False), ("covered", True), ("frames", 0)]:
            value = tiled_stage(); value["nativeTileMembers"][1][key] = changed
            with self.assertRaises(ValueError): self.validate(value)

    def test_held_cover_requires_two_mounted_native_views_and_matching_serials(self):
        previous = tiled_stage()
        cover = {key: previous[key] for key in ("protocol", "nonce", "pid", "platform", "width", "height", "deviceScale", "physicalOffset")}
        cover.update(tileGroup=True, nativeViewCount=2, nativeCovered=True, nativeViewMounted=True,
                     viewBound=True, layerReady=True, presentationKind="nativeView", visibilitySerial=5,
                     clearColor=0xff55575a, nativeTileMembers=copy.deepcopy(previous["nativeTileMembers"]))
        for member in cover["nativeTileMembers"]: member.update(covered=True, visibilitySerial=5)
        self.assertEqual(qualifier.validate_cover(cover, nonce="owned", pid=123, previous=previous, tile_group=True), bytes((85, 87, 90, 255)))
        for mutate in [lambda v: v.update(nativeViewCount=1),
                       lambda v: v["nativeTileMembers"][1].update(covered=False),
                       lambda v: v["nativeTileMembers"][1].update(visibilitySerial=4)]:
            value = copy.deepcopy(cover); mutate(value)
            with self.assertRaises(ValueError):
                qualifier.validate_cover(value, nonce="owned", pid=123, previous=previous, tile_group=True)

    def test_close_rejects_one_missing_or_unfinished_unacknowledged_owner(self):
        previous = tiled_stage(); completed = result()
        completed.update(tileGroup=True, nativeTileMembers=copy.deepcopy(previous["nativeTileMembers"]))
        for member in completed["nativeTileMembers"]:
            member.update(closed=True, ownerFinished=True, registered=False, nativeViewLayerDetached=True,
                          quarantined=False, acknowledged=True, frames=32)
        qualifier.validate_result(completed, nonce="owned", frames=32, tile_group=True, previous=previous)
        for key, changed in [("closed", False), ("ownerFinished", False), ("nativeViewLayerDetached", False),
                             ("quarantined", True), ("acknowledged", False), ("registered", True),
                             ("frames", 31), ("deviceGeneration", 2)]:
            value = copy.deepcopy(completed); value["nativeTileMembers"][1][key] = changed
            with self.assertRaises(ValueError):
                qualifier.validate_result(value, nonce="owned", frames=32, tile_group=True, previous=previous)


if __name__ == "__main__":
    unittest.main()
