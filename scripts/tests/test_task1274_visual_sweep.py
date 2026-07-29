from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

from PIL import Image, ImageDraw


MODULE_PATH = Path(__file__).resolve().parents[1] / "task1274_visual_sweep.py"
SPEC = importlib.util.spec_from_file_location("task1274_visual_sweep", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"task1274_visual_sweep 모듈을 불러올 수 없습니다: {MODULE_PATH}")
SWEEP = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SWEEP
SPEC.loader.exec_module(SWEEP)


class SelectedRasterTests(unittest.TestCase):
    def test_raster_paths_limits_multi_page_svg_to_requested_page(self) -> None:
        paths = [Path("rhwp_001.svg"), Path("rhwp_002.svg"), Path("rhwp_003.svg")]

        selected = SWEEP.raster_paths_for_selected_pages(paths, [2])

        self.assertEqual(selected, [Path("rhwp_002.svg")])

    def test_raster_paths_preserves_singleton_filename_fallback(self) -> None:
        paths = [Path("rhwp_177.svg")]

        selected = SWEEP.raster_paths_for_selected_pages(paths, [1])

        self.assertEqual(selected, paths)

    def test_pdf_raster_commands_limits_each_requested_pdf_page(self) -> None:
        commands = SWEEP.pdf_raster_commands(
            Path("reference.pdf"), 144, Path("out/pdf"), [1, 3]
        )

        self.assertEqual(len(commands), 2)
        self.assertEqual(commands[0][1:5], ["-f", "1", "-l", "1"])
        self.assertEqual(commands[1][1:5], ["-f", "3", "-l", "3"])
        self.assertEqual(commands[0][-2:], ["reference.pdf", "out/pdf"])

    def test_pdf_raster_commands_keeps_full_document_default(self) -> None:
        commands = SWEEP.pdf_raster_commands(
            Path("reference.pdf"), 144, Path("out/pdf"), None
        )

        self.assertEqual(commands, [["pdftoppm", "-r", "144", "-png", "reference.pdf", "out/pdf"]])


class LegacyGlyphVisualCandidateTests(unittest.TestCase):
    def test_old_hangul_run_with_local_pdf_mismatch_is_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "TextRun",
                    "bbox": {"x": 10, "y": 10, "w": 20, "h": 10},
                    "text": "ᄒᆞᆫ글",
                    "pi": 135,
                }
            ],
        }
        rhwp = Image.new("RGB", (100, 100), "white")
        ImageDraw.Draw(rhwp).rectangle((10, 10, 29, 19), fill="black")
        pdf = Image.new("RGB", (100, 100), "white")

        candidates = SWEEP.render_tree_legacy_glyph_visual_candidates(
            tree,
            rhwp,
            pdf,
            pixel_diff_threshold=32,
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 135)
        self.assertEqual(candidates[0]["codepoints"], ["U+1112", "U+119E", "U+11AB"])
        self.assertEqual(candidates[0]["ink_match_percent"], 0.0)

    def test_modern_hangul_run_is_not_a_legacy_glyph_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "TextRun",
                    "bbox": {"x": 10, "y": 10, "w": 20, "h": 10},
                    "text": "한글",
                    "pi": 135,
                }
            ],
        }
        rhwp = Image.new("RGB", (100, 100), "white")
        ImageDraw.Draw(rhwp).rectangle((10, 10, 29, 19), fill="black")
        pdf = Image.new("RGB", (100, 100), "white")

        candidates = SWEEP.render_tree_legacy_glyph_visual_candidates(
            tree,
            rhwp,
            pdf,
            pixel_diff_threshold=32,
        )

        self.assertEqual(candidates, [])

    def test_private_use_run_with_local_mismatch_is_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "TextRun",
                    "bbox": {"x": 10, "y": 10, "w": 20, "h": 10},
                    "text": "\ue001",
                    "pi": 136,
                }
            ],
        }
        rhwp = Image.new("RGB", (100, 100), "white")
        ImageDraw.Draw(rhwp).rectangle((10, 10, 29, 19), fill="black")
        pdf = Image.new("RGB", (100, 100), "white")

        candidates = SWEEP.render_tree_legacy_glyph_visual_candidates(
            tree,
            rhwp,
            pdf,
            pixel_diff_threshold=32,
        )

        self.assertEqual(candidates[0]["codepoints"], ["U+E001"])


if __name__ == "__main__":
    unittest.main()
