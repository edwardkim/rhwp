"""Focused regression tests for the PDF/SVG fidelity candidate extractors."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("fidelity_compare.py")
SPEC = importlib.util.spec_from_file_location("fidelity_compare", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
fidelity_compare = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = fidelity_compare
SPEC.loader.exec_module(fidelity_compare)


class VisibleSvgTextTests(unittest.TestCase):
    def test_ancestor_clip_excludes_off_page_text_but_keeps_partial_line(self) -> None:
        svg = """<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\">
<defs><clipPath id=\"body\"><rect x=\"0\" y=\"0\" width=\"100\" height=\"100\"/></clipPath>
<clipPath id=\"cell\"><rect x=\"0\" y=\"40\" width=\"100\" height=\"10\"/></clipPath></defs>
<g clip-path=\"url(#body)\"><text x=\"10\" y=\"-10\" font-size=\"10\">hidden-top</text>
<text x=\"10\" y=\"20\" font-size=\"10\">body-visible</text>
<g clip-path=\"url(#cell)\"><text x=\"10\" y=\"20\" font-size=\"10\">hidden-cell</text>
<text x=\"10\" y=\"47\" font-size=\"10\">partial-cell</text></g></g></svg>"""
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sample.svg"
            path.write_text(svg, encoding="utf-8")
            visible, excluded = fidelity_compare.svg_visible_text(path)

        self.assertEqual(visible, "body-visiblepartial-cell")
        self.assertGreaterEqual(excluded, len("hidden-tophidden-cell"))

    def test_visible_text_excess_requires_preserved_reference_text(self) -> None:
        candidates = fidelity_compare.visible_text_excess_candidates(
            {
                0: (Counter(), Counter("x" * 48)),
                1: (Counter("missing" * 4), Counter("y" * 100)),
            },
            {0: 30, 1: 0},
        )

        self.assertEqual(candidates, [{
            "page": 0,
            "reference_only": 0,
            "visible_svg_only": 48,
            "clip_excluded_chars": 30,
        }])


class SvgExportFontFallbackTests(unittest.TestCase):
    def test_single_page_export_enables_local_font_fallback_aliases(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command: list[str], **_: object) -> subprocess.CompletedProcess[str]:
            calls.append(command)
            (svg_dir / "sample_001.svg").write_text("<svg/>", encoding="utf-8")
            return subprocess.CompletedProcess(command, 0, "", "")

        with tempfile.TemporaryDirectory() as directory:
            svg_dir = Path(directory)
            original_run = fidelity_compare.subprocess.run
            fidelity_compare.subprocess.run = fake_run
            try:
                rendered = fidelity_compare.render_svg(
                    "rhwp", Path("sample.hwp"), svg_dir, 0
                )
            finally:
                fidelity_compare.subprocess.run = original_run

        self.assertTrue(rendered)
        self.assertEqual(len(calls), 1)
        self.assertIn("--font-style", calls[0])

    def test_full_export_enables_local_font_fallback_aliases(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command: list[str], **_: object) -> subprocess.CompletedProcess[str]:
            calls.append(command)
            return subprocess.CompletedProcess(command, 0, '{"pageCount":1}', "")

        with tempfile.TemporaryDirectory() as directory:
            svg_dir = Path(directory)
            original_run = fidelity_compare.subprocess.run
            fidelity_compare.subprocess.run = fake_run
            try:
                rendered = fidelity_compare.render_all_svg(
                    "rhwp", Path("sample.hwp"), svg_dir
                )
                manifest = (svg_dir / "export-svg-manifest.json").read_text(
                    encoding="utf-8"
                )
            finally:
                fidelity_compare.subprocess.run = original_run

        self.assertTrue(rendered)
        self.assertEqual(manifest, '{"pageCount":1}')
        self.assertEqual(len(calls), 1)
        self.assertIn("--font-style", calls[0])


if __name__ == "__main__":
    unittest.main(verbosity=2)
