from __future__ import annotations

import importlib.util
import csv
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

MODULE_PATH = Path(__file__).resolve().parents[1] / "visual_sweep.py"
SPEC = importlib.util.spec_from_file_location("visual_sweep_alpha_test", MODULE_PATH)
SWEEP = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SWEEP
SPEC.loader.exec_module(SWEEP)


class WhitePaperAlphaTests(unittest.TestCase):
    def test_transparent_black_padding_is_blank_paper(self):
        source = Image.new("RGBA", (32, 33), (255, 255, 255, 255))
        ImageDraw.Draw(source).rectangle((0, 32, 31, 32), fill=(0, 0, 0, 0))
        reference = Image.new("RGB", (32, 32), "white")
        details = SWEEP.subpixel_tolerant_content_match_details(source, reference)
        self.assertEqual(details["tolerant_content_match_percent"], 100.0)
        self.assertEqual(details["silhouette_content_union_pixels"], 0)

    def test_hidden_rgb_does_not_become_visible_ink(self):
        source = Image.new("RGBA", (8, 8), (255, 0, 0, 0))
        left, right = SWEEP.padded_pair(source, Image.new("RGB", (8, 8), "white"))
        self.assertEqual(left.tobytes(), right.tobytes())

    def test_content_analysis_does_not_count_transparent_ink(self):
        source = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        self.assertIsNone(SWEEP.content_bounds(source, x_min=0, x_max=31, y_min=0, y_max=31))
        self.assertEqual(SWEEP.row_bands(
            source, frame=(0, 0, 31, 31), predicate=SWEEP.is_dark_pixel,
            min_pixels_per_row=1, gap=0,
        ), [])

    def test_partial_alpha_is_composited_before_content_classification(self):
        source = Image.new("RGBA", (16, 16), (0, 0, 0, 16))
        reference = Image.new("RGB", (16, 16), (239, 239, 239))
        left, right = SWEEP.padded_pair(source, reference)
        self.assertEqual(left.tobytes(), right.tobytes())
        self.assertEqual(SWEEP.subpixel_tolerant_content_match_percent(source, reference), 100.0)

    def test_luminance_alpha_has_the_same_white_paper_contract(self):
        left, right = SWEEP.padded_pair(
            Image.new("LA", (8, 8), (0, 128)),
            Image.new("RGB", (8, 8), (127, 127, 127)),
        )
        self.assertEqual(left.tobytes(), right.tobytes())

    def test_palette_transparency_survives_png_loading(self):
        source = Image.new("P", (16, 16), 0)
        source.putpalette([0, 0, 0, 255, 0, 0] + [0] * 762)
        source.info["transparency"] = 0
        source.putpixel((8, 8), 1)
        reference = Image.new("RGB", (16, 16), "white")
        reference.putpixel((8, 8), (255, 0, 0))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "palette.png"
            source.save(path)
            with Image.open(path) as loaded:
                left, right = SWEEP.padded_pair(loaded, reference)
                self.assertEqual(left.tobytes(), right.tobytes())

    def test_opaque_rgb_and_rgba_keep_every_pixel(self):
        source = Image.new("RGB", (16, 16))
        source.putdata([(x * 13 % 256, x * 47 % 256, x * 113 % 256) for x in range(256)])
        for image in [source, source.convert("RGBA")]:
            left, right = SWEEP.padded_pair(image, source)
            self.assertEqual(left.tobytes(), source.tobytes())
            self.assertEqual(right.tobytes(), source.tobytes())

    def test_visible_opaque_displacement_is_still_a_mismatch(self):
        source = Image.new("RGBA", (32, 32), "white")
        reference = Image.new("RGBA", (32, 32), "white")
        ImageDraw.Draw(source).rectangle((4, 4, 8, 20), fill="black")
        ImageDraw.Draw(reference).rectangle((18, 4, 22, 20), fill="black")
        self.assertLess(SWEEP.subpixel_tolerant_content_match_percent(source, reference), 100.0)

    def test_transparent_render_does_not_hide_missing_reference_content(self):
        source = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        reference = Image.new("RGB", (32, 32), "white")
        ImageDraw.Draw(reference).rectangle((12, 4, 20, 24), fill="black")
        self.assertEqual(SWEEP.subpixel_tolerant_content_match_percent(source, reference), 0.0)

    def test_overlay_and_compare_use_the_same_white_paper(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            source = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
            ImageDraw.Draw(source).rectangle((12, 12, 28, 36), fill=(0, 0, 0, 255))
            reference = Image.new("RGB", source.size, "white")
            ImageDraw.Draw(reference).rectangle((12, 12, 28, 36), fill="black")
            left, right = folder / "rhwp_001.png", folder / "pdf_001.png"
            source.save(left)
            reference.save(right)
            metrics = SWEEP.make_overlay_page(
                left, right, folder / "overlay.png", "alpha", 0, pixel_diff_threshold=32
            )
            self.assertEqual(metrics["diff_pixels"], 0)
            self.assertEqual(metrics["tolerant_content_match_percent"], 100.0)
            compares = folder / "compare"
            compares.mkdir()
            result = SWEEP.make_compares([left], [right], compares, "alpha")
            with Image.open(result[0]) as image:
                label_height = image.height - source.height
                self.assertEqual(image.getpixel((0, label_height)), (255, 255, 255))

    def test_png_pair_cli_keeps_alpha_policy_with_spawned_workers(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            left, right, output = folder / "left", folder / "right", folder / "result"
            left.mkdir()
            right.mkdir()
            for page in [1, 2]:
                source = Image.new("RGBA", (32, 33), (255, 255, 255, 255))
                ImageDraw.Draw(source).rectangle((0, 32, 31, 32), fill=(0, 0, 0, 0))
                source.save(left / f"rhwp_{page:03d}.png")
                Image.new("RGB", (32, 32), "white").save(right / f"pdf_{page:03d}.png")
            result = subprocess.run(
                [sys.executable, str(MODULE_PATH), "--png-pair", str(left), str(right),
                 "--silhouette-only", "--key", "alpha", "--out", str(output), "--page-workers", "2"],
                cwd=MODULE_PATH.parents[1], capture_output=True, text=True, timeout=45,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            with (output / "silhouette.tsv").open() as stream:
                rows = list(csv.DictReader(stream, delimiter="\t"))
            self.assertEqual([row["tolerant_content_match_percent"] for row in rows], ["100.00000"] * 2)
            manifest = json.loads((output / "silhouette_manifest.json").read_text())
            self.assertEqual(manifest["radius_px"], 2)
            self.assertEqual(manifest["color_tolerance"], 32)
            self.assertEqual(manifest["image_decoding"], {
                "alpha_background": "#ffffff",
                "script_sha256": hashlib.sha256(MODULE_PATH.with_name("visual_sweep_images.py").read_bytes()).hexdigest(),
            })


if __name__ == "__main__":
    unittest.main()
