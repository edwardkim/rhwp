"""Relational grid/ownership contracts; no document layout or visual baseline.

2026-11-07 KST: these checks use the native public 512-pixel grid and RGBA
enclosing-window contract, rather than fixing any real document coordinates.
"""

import gc
import math
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
import weakref

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_viewport_tiles import CLIENT_CACHE_BYTES, MAX_EXACT_PIXEL, TileImageCache, ViewportTile, frame_byte_cost, plan_visible_tiles


def visible(x=0, y=0, width=700, height=800, scale=1, page=0):
    return {"page": page, "rasterScale": scale,
            "pageRegion": {"x": x, "y": y, "width": width, "height": height}}


class TileContracts(unittest.TestCase):
    def test_fixed_grid_is_independent_of_viewport_scroll_origin(self):
        pages = [{"width": 2000, "height": 2000}]
        first = plan_visible_tiles(pages, [visible(width=800, height=800)], 1)
        moved = plan_visible_tiles(pages, [visible(x=200, y=200, width=800, height=800)], 1)
        self.assertEqual(first, moved)
        crossed = plan_visible_tiles(pages, [visible(x=511.5, width=1, height=1)], 1)
        self.assertEqual([(tile.x, tile.y) for tile in crossed], [(0, 0), (512, 0)])

    def test_fractional_page_extent_uses_ceil_and_keeps_the_final_pixel(self):
        pages = [{"width": 700.2, "height": 411.3}]
        tiles = plan_visible_tiles(pages, [visible(width=700.2, height=411.3, scale=1.3)], 2)
        right = math.ceil(700.2 * 1.3)
        bottom = math.ceil(411.3 * 1.3)
        self.assertEqual(max(t.x + t.width for t in tiles), right)
        self.assertEqual(max(t.y + t.height for t in tiles), bottom)
        self.assertEqual(sum(t.width * t.height for t in tiles), right * bottom)
        self.assertTrue(all(t.device_scale == 2 for t in tiles))

    def test_logical_request_rounding_is_validated_by_actual_metadata(self):
        tile = ViewportTile(0, 512, 512, 512, 512, 1.3, 2)
        frame = {"originX": 511, "originY": 511, "width": 514, "height": 514,
                 "stride": 514 * 4, "byteLength": 514 * 514 * 4}
        self.assertEqual(frame_byte_cost(tile, frame), 514 * 514 * 4)
        for changed in ({"originX": 513}, {"width": 513}, {"byteLength": 2**63}, {"stride": 0}, {"height": True}):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                frame_byte_cost(tile, {**frame, **changed})

    def test_huge_viewport_is_rejected_before_any_grid_iteration(self):
        pages = [{"width": MAX_EXACT_PIXEL, "height": MAX_EXACT_PIXEL}]
        with patch("native_viewport_tiles.range", side_effect=AssertionError("unbounded grid iteration"), create=True):
            with self.assertRaisesRegex(ValueError, "bounded tile plan"):
                plan_visible_tiles(pages, [visible(width=MAX_EXACT_PIXEL, height=MAX_EXACT_PIXEL)], 1)

    def test_total_plan_is_bounded_across_pages(self):
        pages = [{"width": 4096, "height": 4096}] * 2
        with self.assertRaisesRegex(ValueError, "bounded tile plan"):
            plan_visible_tiles(pages, [visible(width=4096, height=4096), visible(width=1, height=1, page=1)], 1)
        with self.assertRaises(ValueError):
            plan_visible_tiles(pages, [visible(), visible()], 1)
        self.assertLessEqual(64 * 514 * 514 * 4, CLIENT_CACHE_BYTES)

    def test_invalid_numeric_inputs_and_bounds_are_rejected(self):
        for scale in (0, -1, True, float("nan"), float("inf"), 1e-300, 65):
            with self.subTest(scale=scale), self.assertRaises(ValueError):
                plan_visible_tiles([{"width": 700, "height": 800}], [visible(scale=scale)], 1)
        for width in (0, -1, 10**1000, float("inf")):
            with self.subTest(width=width), self.assertRaises(ValueError):
                plan_visible_tiles([{"width": 700, "height": 800}], [visible(width=width)], 1)
        with self.assertRaises(ValueError):
            plan_visible_tiles([{"width": 700, "height": 800}], [visible(x=MAX_EXACT_PIXEL + 1)], 1)
        with self.assertRaises(ValueError):
            plan_visible_tiles([{"width": 700, "height": 800}], [visible()] * 65, 1)

    def test_off_page_region_needs_no_tile_storage(self):
        self.assertEqual(plan_visible_tiles([{"width": 700, "height": 800}], [visible(x=-600, width=100)], 1), ())

    def test_tile_identity_includes_document_revision_font_scale_and_dpr(self):
        tile = ViewportTile(0, 0, 0, 512, 512, 1, 1)
        key = tile.key((1, 7, 3, 0))
        for identity in ((2, 7, 3, 0), (1, 8, 3, 0), (1, 7, 4, 0), (1, 7, 3, 1)):
            self.assertNotEqual(key, tile.key(identity))
        self.assertNotEqual(key, ViewportTile(0, 0, 0, 512, 512, 2, 1).key((1, 7, 3, 0)))
        self.assertNotEqual(key, ViewportTile(0, 0, 0, 512, 512, 1, 2).key((1, 7, 3, 0)))

    def test_lru_honors_actual_bytes_entries_and_touch_order(self):
        cache = TileImageCache(max_bytes=10, max_entries=2)
        cache.put("first", "A", 4)
        cache.put("second", "B", 4)
        self.assertEqual(cache.get("first"), "A")
        cache.put("third", "C", 5)
        self.assertIn("first", cache)
        self.assertNotIn("second", cache)
        self.assertEqual(cache.retained_bytes, 9)
        cache.put("first", "replacement", 7)
        self.assertNotIn("third", cache)
        self.assertEqual(cache.retained_bytes, 7)
        self.assertFalse(cache.put("oversized", object(), 11))
        self.assertEqual(cache.retained_bytes, 7)
        self.assertEqual(len(cache), 1)

    def test_cache_clear_and_eviction_release_owned_images(self):
        class Image:
            pass
        cache = TileImageCache(max_bytes=4, max_entries=1)
        image = Image()
        reference = weakref.ref(image)
        cache.put("old", image, 4)
        del image
        self.assertIsNotNone(reference())
        cache.put("new", Image(), 4)
        gc.collect()
        self.assertIsNone(reference())
        cache.clear()
        self.assertEqual((len(cache), cache.retained_bytes), (0, 0))

    def test_zero_limits_never_retain_images(self):
        for cache in (TileImageCache(0, 1), TileImageCache(4, 0)):
            self.assertFalse(cache.put("tile", object(), 4))
            self.assertEqual((len(cache), cache.retained_bytes), (0, 0))


if __name__ == "__main__":
    unittest.main(verbosity=2)
