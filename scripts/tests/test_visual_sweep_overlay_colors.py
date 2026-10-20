"""차이 소속의 색상과 실제 overlay 산출을 검증한다."""

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

SPEC = importlib.util.spec_from_file_location(
    "visual_sweep_overlay_colors", Path(__file__).resolve().parents[1] / "visual_sweep.py"
)
assert SPEC is not None and SPEC.loader is not None
SWEEP = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SWEEP
SPEC.loader.exec_module(SWEEP)


class OverlayColorOwnershipTests(unittest.TestCase):
    def test_extra_rhwp_content_is_blue_and_missing_content_is_red(self):
        self.assertEqual(
            SWEEP.overlay_color((0, 0, 0), (255, 255, 255), threshold=32),
            ((40, 100, 255), True, True, True),
        )
        self.assertEqual(
            SWEEP.overlay_color((255, 255, 255), (0, 0, 0), threshold=32),
            ((255, 40, 40), True, True, True),
        )

    def test_matching_and_both_sided_differences_have_distinct_colors(self):
        self.assertEqual(
            SWEEP.overlay_color((0, 0, 0), (0, 0, 0), threshold=32),
            ((0, 0, 0), False, True, False),
        )
        self.assertEqual(
            SWEEP.overlay_color((0, 0, 0), (0, 0, 80), threshold=32),
            ((255, 150, 0), True, True, True),
        )

    def test_generated_overlay_preserves_color_ownership_and_difference_counts(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            rhwp = Image.new("RGB", (32, 32), "white")
            pdf = Image.new("RGB", (32, 32), "white")
            ImageDraw.Draw(rhwp).rectangle((2, 2, 4, 4), fill="black")
            ImageDraw.Draw(pdf).rectangle((20, 20, 22, 22), fill="black")
            rhwp.save(base / "rhwp_001.png")
            pdf.save(base / "pdf_001.png")
            metrics = SWEEP.make_overlay_page(
                base / "rhwp_001.png", base / "pdf_001.png", base / "overlay_001.png",
                "색상 소속", 0, pixel_diff_threshold=32,
            )
            with Image.open(base / "overlay_001.png") as output:
                pixels = [
                    output.getpixel((x, y))
                    for y in range(output.height)
                    for x in range(output.width)
                ]
            self.assertEqual(pixels.count((40, 100, 255)), 9)
            self.assertEqual(pixels.count((255, 40, 40)), 9)
            self.assertEqual(metrics["diff_pixels"], 18)
            self.assertEqual(metrics["ink_diff_pixels"], 18)
            self.assertEqual(metrics["tolerant_content_match_percent"], 0.0)
            self.assertEqual(metrics["overlay_color_legend"]["rhwp_only"], "blue")
            self.assertEqual(metrics["overlay_color_legend"]["pdf_only"], "red")


if __name__ == "__main__":
    unittest.main()
