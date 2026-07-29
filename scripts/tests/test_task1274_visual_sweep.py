from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


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


if __name__ == "__main__":
    unittest.main()
