"""Optional real-Chrome capture contract, independent of document layout."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from PIL import Image


ROOT = Path(__file__).resolve().parents[2]
CHROME = os.environ.get("VISUAL_SWEEP_CHROME")


@unittest.skipUnless(CHROME, "VISUAL_SWEEP_CHROME is required for browser raster checks")
class WebfontRasterViewportTest(unittest.TestCase):
    def test_all_four_page_corners_survive_capture_at_both_scales(self):
        with tempfile.TemporaryDirectory(prefix="rhwp-raster-viewport-") as directory:
            root = Path(directory)
            source = root / "corners.svg"
            source.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg" width="794" height="1123">'
                '<rect width="794" height="1123" fill="white"/>'
                '<rect x="0" y="0" width="20" height="20" fill="red"/>'
                '<rect x="774" y="0" width="20" height="20" fill="lime"/>'
                '<rect x="0" y="1103" width="20" height="20" fill="blue"/>'
                '<rect x="774" y="1103" width="20" height="20" fill="magenta"/>'
                '</svg>', encoding="utf-8",
            )
            for zoom in (1, 2):
                with self.subTest(zoom=zoom):
                    output = root / f"corners-{zoom}.png"
                    result = subprocess.run(
                        ["node", str(ROOT / "scripts/rasterize-svg-webfonts.mjs"),
                         "--input", str(source), "--output", str(output),
                         "--zoom", str(zoom), "--chrome", CHROME],
                        cwd=ROOT, capture_output=True, text=True, timeout=60,
                        check=False,
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    with Image.open(output) as raster:
                        rgb = raster.convert("RGB")
                    self.assertEqual(rgb.size, (794 * zoom, 1123 * zoom))
                    for x, y, color in (
                        (10, 10, (255, 0, 0)),
                        (784, 10, (0, 255, 0)),
                        (10, 1113, (0, 0, 255)),
                        (784, 1113, (255, 0, 255)),
                    ):
                        self.assertEqual(rgb.getpixel((x * zoom, y * zoom)), color)


if __name__ == "__main__":
    unittest.main()
