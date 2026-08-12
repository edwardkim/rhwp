"""Focused regression tests for the PDF/SVG fidelity candidate extractors."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest.mock import patch


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

    def test_full_font_mode_embeds_selected_face(self) -> None:
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
                with patch.dict(
                    fidelity_compare.os.environ,
                    {"RHWP_SVG_FONT_MODE": "full"},
                    clear=False,
                ):
                    rendered = fidelity_compare.render_svg(
                        "rhwp", Path("sample.hwp"), svg_dir, 0
                    )
            finally:
                fidelity_compare.subprocess.run = original_run

        self.assertTrue(rendered)
        self.assertIn("--embed-fonts=full", calls[0])
        self.assertNotIn("--font-style", calls[0])

    def test_invalid_font_mode_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "RHWP_SVG_FONT_MODE"):
            fidelity_compare.svg_font_export_option({"RHWP_SVG_FONT_MODE": "invalid"})

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

    def test_render_tree_does_not_receive_svg_only_font_path_option(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command: list[str], **_: object) -> subprocess.CompletedProcess[str]:
            calls.append(command)
            return subprocess.CompletedProcess(command, 0, "", "")

        with tempfile.TemporaryDirectory() as directory:
            tree_dir = Path(directory)
            original_run = fidelity_compare.subprocess.run
            fidelity_compare.subprocess.run = fake_run
            try:
                with patch.dict(
                    fidelity_compare.os.environ,
                    {"RHWP_FONT_PATH_DIR": "/fonts"},
                    clear=False,
                ):
                    rendered = fidelity_compare.render_all_render_tree(
                        "rhwp", Path("sample.hwp"), tree_dir
                    )
            finally:
                fidelity_compare.subprocess.run = original_run

        self.assertTrue(rendered)
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][:2], ["rhwp", "export-render-tree"])
        self.assertNotIn("--font-path", calls[0])

    def test_font_path_list_is_split_for_svg_export(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command: list[str], **_: object) -> subprocess.CompletedProcess[str]:
            calls.append(command)
            (svg_dir / "sample_001.svg").write_text("<svg/>", encoding="utf-8")
            return subprocess.CompletedProcess(command, 0, "", "")

        with tempfile.TemporaryDirectory() as directory:
            svg_dir = Path(directory)
            fonts_a = svg_dir / "fonts-a"
            fonts_b = svg_dir / "fonts-b"
            fonts_a.mkdir()
            fonts_b.mkdir()
            original_run = fidelity_compare.subprocess.run
            fidelity_compare.subprocess.run = fake_run
            try:
                with patch.dict(
                    fidelity_compare.os.environ,
                    {
                        "RHWP_FONT_PATH_DIR": os.pathsep.join(
                            [str(fonts_a), str(fonts_b)]
                        )
                    },
                    clear=False,
                ):
                    rendered = fidelity_compare.render_svg(
                        "rhwp", Path("sample.hwp"), svg_dir, 0
                    )
            finally:
                fidelity_compare.subprocess.run = original_run

        self.assertTrue(rendered)
        self.assertEqual(
            [calls[0][index + 1] for index, value in enumerate(calls[0]) if value == "--font-path"],
            [str(fonts_a.resolve()), str(fonts_b.resolve())],
        )

    def test_linux_chrome_fontconfig_uses_same_font_path_list(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fonts_a = root / "fonts-a"
            fonts_b = root / "fonts-b"
            fonts_a.mkdir()
            fonts_b.mkdir()
            env = {
                "RHWP_FONT_PATH_DIR": os.pathsep.join([str(fonts_a), str(fonts_b)]),
                "PATH": os.environ.get("PATH", ""),
            }
            configured = fidelity_compare.chrome_fontconfig_environment(
                root / "work", env, os_name="posix", platform="linux"
            )

            self.assertIsNotNone(configured)
            assert configured is not None
            config_path = Path(configured["FONTCONFIG_PATH"]) / configured["FONTCONFIG_FILE"]
            config = config_path.read_text(encoding="utf-8")

        self.assertEqual(configured["FONTCONFIG_FILE"], "fonts.conf")
        self.assertIn(f"<dir>{fonts_a.resolve()}</dir>", config)
        self.assertIn(f"<dir>{fonts_b.resolve()}</dir>", config)


if __name__ == "__main__":
    unittest.main(verbosity=2)
