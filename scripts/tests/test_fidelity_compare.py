from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import subprocess
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest import mock

MODULE_PATH = (
    Path(__file__).resolve().parents[2]
    / "tools"
    / "fidelity_compare"
    / "fidelity_compare.py"
)
SPEC = importlib.util.spec_from_file_location("fidelity_compare", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"fidelity_compare 모듈을 불러올 수 없습니다: {MODULE_PATH}")
FIDELITY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = FIDELITY
SPEC.loader.exec_module(FIDELITY)


class ExecutableDiscoveryTests(unittest.TestCase):
    def test_find_rhwp_uses_platform_specific_release_test_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            binary = repo / "target" / "release-test" / "rhwp"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"binary")

            resolved = FIDELITY.find_rhwp(repo=repo, env={}, os_name="posix")

            self.assertEqual(resolved, str(binary))

    def test_find_rhwp_accepts_path_discovered_override(self) -> None:
        with mock.patch.object(
            FIDELITY.shutil, "which", return_value="/opt/rhwp/bin/rhwp"
        ):
            resolved = FIDELITY.find_rhwp(env={"RHWP_BIN": "rhwp-custom"})

        self.assertEqual(resolved, "/opt/rhwp/bin/rhwp")

    def test_find_chrome_uses_linux_path_lookup(self) -> None:
        def which(name: str) -> str | None:
            return "/usr/bin/chromium" if name == "chromium" else None

        with mock.patch.object(FIDELITY.shutil, "which", side_effect=which):
            resolved = FIDELITY.find_chrome(env={}, os_name="posix", platform="linux")

        self.assertEqual(resolved, "/usr/bin/chromium")

    def test_find_chrome_uses_windows_program_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            chrome = (
                Path(directory) / "Google" / "Chrome" / "Application" / "chrome.exe"
            )
            chrome.parent.mkdir(parents=True)
            chrome.write_bytes(b"binary")
            with mock.patch.object(FIDELITY.shutil, "which", return_value=None):
                resolved = FIDELITY.find_chrome(
                    env={"PROGRAMFILES": directory}, os_name="nt", platform="win32"
                )

        self.assertEqual(resolved, str(chrome))


class ChromeCaptureTests(unittest.TestCase):
    def test_capture_retries_once_and_surfaces_first_stderr(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.html"
            output = root / "capture.png"
            source.write_text("<html></html>", encoding="utf-8")
            calls = 0

            def fake_run(
                *_args: object, **_kwargs: object
            ) -> subprocess.CompletedProcess[str]:
                nonlocal calls
                calls += 1
                if calls == 1:
                    return subprocess.CompletedProcess(
                        [], 1, stdout="", stderr="first failure"
                    )
                output.write_bytes(b"png")
                return subprocess.CompletedProcess([], 0, stdout="", stderr="")

            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                succeeded = FIDELITY.capture_with_chrome(
                    "chrome", source, output, 800, 600, run=fake_run
                )

        self.assertTrue(succeeded)
        self.assertEqual(calls, 2)
        self.assertIn("first failure", stderr.getvalue())
        self.assertIn("1/2", stderr.getvalue())

    def test_capture_returns_false_after_two_failures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.svg"
            output = root / "capture.png"
            source.write_text("<svg></svg>", encoding="utf-8")

            def fake_run(
                *_args: object, **_kwargs: object
            ) -> subprocess.CompletedProcess[str]:
                return subprocess.CompletedProcess(
                    [], 2, stdout="", stderr="still failing"
                )

            with contextlib.redirect_stderr(io.StringIO()):
                succeeded = FIDELITY.capture_with_chrome(
                    "chrome", source, output, 800, 600, run=fake_run
                )

        self.assertFalse(succeeded)
        self.assertFalse(output.exists())


class TextLayerComparisonTests(unittest.TestCase):
    def test_multiset_comparison_ignores_order_whitespace_and_unicode_form(
        self,
    ) -> None:
        reference = "A e\u0301 ·\n"
        rendered = "éA×"

        missing, extra = FIDELITY.compare_text_layers(reference, rendered)

        self.assertEqual(missing, {"·": 1})
        self.assertEqual(extra, {"×": 1})

    def test_svg_text_reads_only_text_elements(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            svg = Path(directory) / "page.svg"
            svg.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg"><style>ignored</style>'
                '<text>A<tspan>가</tspan></text><path d="M0 0"/></svg>',
                encoding="utf-8",
            )

            extracted = FIDELITY.svg_text(svg)

        self.assertEqual(extracted, "A가")

    def test_adjacent_reciprocal_text_difference_is_owner_shift_candidate(self) -> None:
        moved = Counter("각주26) footnote owner")

        candidates = FIDELITY.adjacent_text_owner_shift_candidates(
            {
                25: (Counter(), moved),
                26: (moved, Counter()),
            }
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["page"], 25)
        self.assertEqual(candidates[0]["next_page"], 26)
        self.assertEqual(candidates[0]["direction"], "rhwp_earlier_than_reference")
        self.assertEqual(candidates[0]["shared_count"], sum(moved.values()))

    def test_adjacent_partial_text_overlap_is_not_owner_shift_candidate(self) -> None:
        candidates = FIDELITY.adjacent_text_owner_shift_candidates(
            {
                0: (Counter(), Counter("abcdefgh")),
                1: (Counter("abcxxxxx"), Counter()),
            }
        )

        self.assertEqual(candidates, [])

    def test_adjacent_ordered_sequence_detects_counter_diluted_late_owner(self) -> None:
        moved = "60)http://www.who.int/transplantation/ConsensusStatementShort.pdf?ua=1"
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                51: (f"p52 본문 {moved}", "p52 본문"),
                52: ("p53 본문", f"p53 본문 {moved}"),
            }
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["page"], 51)
        self.assertEqual(candidates[0]["next_page"], 52)
        self.assertEqual(candidates[0]["direction"], "rhwp_later_than_reference")
        self.assertEqual(candidates[0]["sequence"], moved)

    def test_adjacent_ordered_sequence_normalizes_target_page_line_breaks(self) -> None:
        moved = "60) http://www.who.int/transplantation/ConsensusStatementShort.pdf?ua=1"
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                51: (f"p52 본문 {moved}", "p52 본문"),
                52: (
                    "p53 본문",
                    "p53 본문\n60)    http://www.who.int/transplantation/"
                    "ConsensusStatementShort.pdf?ua=1",
                ),
            }
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["page"], 51)
        self.assertEqual(candidates[0]["next_page"], 52)
        self.assertEqual(candidates[0]["direction"], "rhwp_later_than_reference")

    def test_adjacent_ordered_sequence_detects_a_multistep_late_owner_chain(self) -> None:
        moved60 = "60) https://example.test/zzzzzzzzzzzzzzzzzzzzzzzz"
        moved62 = "62) qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq"
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                51: (f"p52 본문 {moved60}", "p52 본문"),
                52: (f"p53 본문 {moved62}", f"p53 본문\n{moved60}"),
                53: ("p54 본문", f"p54 본문\n{moved62}"),
            }
        )

        self.assertEqual(
            [(candidate["page"], candidate["next_page"], candidate["direction"])
            for candidate in candidates],
            [(51, 52, "rhwp_later_than_reference"), (52, 53, "rhwp_later_than_reference")],
        )

    def test_adjacent_ordered_sequence_ignores_intra_page_reorder(self) -> None:
        moved = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        tail = "qrstuvwxyz" * 10
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                0: (f"prefix{moved}{tail}", f"prefix{tail}{moved}"),
                1: ("p2", f"p2{moved}"),
            }
        )

        self.assertEqual(candidates, [])

    def test_adjacent_ordered_sequence_detects_early_owner(self) -> None:
        moved = "26)11번참고문헌내Adametal논문"
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                25: ("p26 본문", f"p26 본문 {moved}"),
                26: (f"p27 본문 {moved}", "p27 본문"),
            }
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["page"], 25)
        self.assertEqual(candidates[0]["next_page"], 26)
        self.assertEqual(candidates[0]["direction"], "rhwp_earlier_than_reference")

    def test_ordered_sequence_ignores_text_still_owned_by_next_reference_page(self) -> None:
        moved = "동일한문장이정상적으로다음쪽에도있음"
        candidates = FIDELITY.adjacent_text_owner_sequence_candidates(
            {
                0: (f"p1 {moved}", "p1"),
                1: (f"p2 {moved}", f"p2 {moved}"),
            }
        )

        self.assertEqual(candidates, [])

    def test_owner_shift_ledger_uses_one_based_page_numbers(self) -> None:
        moved = Counter("각주26) footnote owner")
        with tempfile.TemporaryDirectory() as directory:
            FIDELITY.write_text_owner_shift_ledger(
                Path(directory),
                {
                    25: (Counter(), moved),
                    26: (moved, Counter()),
                },
            )
            report = (Path(directory) / "text-owner-shift-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn("26\t27\trhwp_earlier_than_reference", report)

    def test_owner_sequence_ledger_uses_one_based_page_numbers(self) -> None:
        moved = "60)http://www.who.int/transplantation/ConsensusStatementShort.pdf?ua=1"
        with tempfile.TemporaryDirectory() as directory:
            FIDELITY.write_text_owner_sequence_ledger(
                Path(directory),
                {
                    51: (f"p52 {moved}", "p52"),
                    52: ("p53", f"p53 {moved}"),
                },
            )
            report = (Path(directory) / "text-owner-sequence-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn("52\t53\trhwp_later_than_reference", report)
        self.assertIn(moved, report)

    def test_page_boundary_ledger_keeps_short_reciprocal_owner_shift(self) -> None:
        moved = Counter("②구내운반차사용의")
        with tempfile.TemporaryDirectory() as directory:
            FIDELITY.write_page_boundary_fidelity_ledger(
                Path(directory),
                {
                    69: (Counter(), moved),
                    70: (moved, Counter()),
                },
                {},
            )
            report = (Path(directory) / "page-boundary-fidelity-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn(
            "70\t71\ttext_owner_shift\trhwp_earlier_than_reference\t9\t0",
            report,
        )

    def test_successor_top_float_refines_early_owner_shift(self) -> None:
        moved = Counter("그림앞문단이기준PDF에서는다음쪽으로이어짐")
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "pi": 1276,
                            "ci": 0,
                            "textWrap": "TopAndBottom",
                            "bbox": {"x": 100, "y": 80, "w": 400, "h": 300},
                        }
                    ],
                }
            ],
        }
        differences = {
            117: (Counter(), moved),
            118: (moved, Counter()),
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tree_dir = root / "render_tree"
            tree_dir.mkdir()
            (tree_dir / "document_119.json").write_text(
                json.dumps(tree), encoding="utf-8"
            )

            candidates = FIDELITY.successor_float_owner_shift_candidates(
                tree_dir, [117, 118], differences
            )
            FIDELITY.write_successor_float_owner_shift_ledger(
                root, tree_dir, [117, 118], differences
            )
            report = (root / "float-owner-shift-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["direction"], "rhwp_earlier_than_reference")
        self.assertEqual(candidates[0]["float"]["pi"], 1276)
        self.assertEqual(candidates[0]["float"]["text_wrap"], "TopAndBottom")
        self.assertIn("118\t119\trhwp_earlier_than_reference", report)
        self.assertIn("\t1276\t0\tTopAndBottom\t", report)

    def test_successor_float_owner_shift_ignores_lower_page_float(self) -> None:
        moved = Counter("그림앞문단이기준PDF에서는다음쪽으로이어짐")
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Body",
                    "children": [
                        {
                            "type": "Image",
                            "textWrap": "TopAndBottom",
                            "bbox": {"x": 100, "y": 400, "w": 400, "h": 300},
                        }
                    ],
                }
            ],
        }
        with tempfile.TemporaryDirectory() as directory:
            tree_dir = Path(directory)
            (tree_dir / "document_119.json").write_text(
                json.dumps(tree), encoding="utf-8"
            )
            candidates = FIDELITY.successor_float_owner_shift_candidates(
                tree_dir,
                [117, 118],
                {
                    117: (Counter(), moved),
                    118: (moved, Counter()),
                },
            )

        self.assertEqual(candidates, [])

    def test_numbered_page_count_ignores_manifest_and_non_page_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in (
                "document_001.svg",
                "document_002.svg",
                "export-svg-manifest.json",
                "document_002.copy.svg",
            ):
                (root / name).write_text("", encoding="utf-8")

            count = FIDELITY.numbered_page_count(root, ".svg")

        self.assertEqual(count, 2)

    def test_page_count_ledger_keeps_partial_svg_scope_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            FIDELITY.write_page_count_ledger(
                Path(directory),
                reference_page_count=215,
                full_svg_page_count=None,
                full_render_tree_page_count=219,
            )
            report = (Path(directory) / "page-count-ledger.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn("reference_pdf\t215\t0\tfull PDF", report)
        self.assertIn("rhwp_svg\t-\t-\tnot counted", report)
        self.assertIn("rhwp_render_tree\t219\t4\tfull render tree", report)


class SvgTableBorderClipCandidateTests(unittest.TestCase):
    def test_reports_right_table_border_hidden_by_parent_clip(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "pi": 6,
                    "ci": 0,
                    "rows": 12,
                    "cols": 5,
                    "bbox": {"x": 80, "y": 200, "w": 650, "h": 700},
                    "children": [
                        {"type": "Line", "bbox": {"x": 730, "y": 200, "w": 2, "h": 700}}
                    ],
                }
            ],
        }
        svg = """<svg xmlns=\"http://www.w3.org/2000/svg\">
          <defs><clipPath id=\"body-clip\"><rect x=\"75\" y=\"100\" width=\"650\" height=\"850\"/></clipPath></defs>
          <g clip-path=\"url(#body-clip)\"><line x1=\"730\" y1=\"200\" x2=\"730\" y2=\"900\" stroke=\"#000\" stroke-width=\"2\"/></g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p004.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_table_border_clip_candidates(svg_path, tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 6)
        self.assertEqual(candidates[0]["edge"], "right")
        self.assertEqual(candidates[0]["visible_width_ratio"], 0.0)
        self.assertEqual(candidates[0]["clip_ids"], ("body-clip",))

    def test_ignores_an_unclipped_or_non_table_vertical_stroke(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "bbox": {"x": 80, "y": 200, "w": 650, "h": 700},
                    "children": [
                        {"type": "Line", "bbox": {"x": 730, "y": 200, "w": 2, "h": 700}}
                    ],
                }
            ],
        }
        svg = """<svg xmlns=\"http://www.w3.org/2000/svg\">
          <line x1=\"730\" y1=\"200\" x2=\"730\" y2=\"900\" stroke=\"#000\" stroke-width=\"2\"/>
          <line x1=\"30\" y1=\"200\" x2=\"30\" y2=\"900\" stroke=\"#000\" stroke-width=\"2\"/>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p004.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_table_border_clip_candidates(svg_path, tree)

        self.assertEqual(candidates, [])


class SvgTableHorizontalBorderClipCandidateTests(unittest.TestCase):
    def test_reports_bottom_table_frame_hidden_by_parent_clip(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "pi": 7,
                    "ci": 1,
                    "rows": 1,
                    "cols": 1,
                    "bbox": {"x": 80, "y": 200, "w": 650, "h": 700},
                    "children": [
                        {"type": "Line", "bbox": {"x": 80, "y": 900, "w": 650, "h": 2}}
                    ],
                }
            ],
        }
        svg = """<svg xmlns="http://www.w3.org/2000/svg">
          <defs><clipPath id="cell-clip"><rect x="75" y="100" width="660" height="798"/></clipPath></defs>
          <g clip-path="url(#cell-clip)"><line x1="80" y1="900" x2="730" y2="900" stroke="#000" stroke-width="2"/></g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p010.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_table_horizontal_border_clip_candidates(svg_path, tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 7)
        self.assertEqual(candidates[0]["edge"], "bottom")
        self.assertEqual(candidates[0]["visible_height_ratio"], 0.0)
        self.assertEqual(candidates[0]["clip_ids"], ("cell-clip",))

    def test_reports_missing_physical_bottom_frame_before_source_border(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "pi": 7,
                    "ci": 1,
                    "rows": 1,
                    "cols": 1,
                    "bbox": {"x": 80, "y": 200, "w": 650, "h": 700},
                    "children": [
                        {"type": "Line", "bbox": {"x": 80, "y": 900, "w": 650, "h": 2}}
                    ],
                }
            ],
        }
        svg = """<svg xmlns="http://www.w3.org/2000/svg">
          <defs><clipPath id="cell-clip"><rect x="75" y="100" width="660" height="750"/></clipPath></defs>
          <g clip-path="url(#cell-clip)"><line x1="80" y1="900" x2="730" y2="900" stroke="#000" stroke-width="2"/></g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p010.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_table_horizontal_border_clip_candidates(svg_path, tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["edge"], "bottom")
        self.assertEqual(candidates[0]["line_y"], 850.0)
        self.assertEqual(candidates[0]["visible_height_ratio"], 0.0)


class TableCellTextOverlapCandidateTests(unittest.TestCase):
    def test_reports_painted_lines_overlapping_within_one_cell(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "pi": 7,
                    "ci": 1,
                    "rows": 2,
                    "cols": 2,
                    "bbox": {"x": 80, "y": 200, "w": 600, "h": 300},
                    "children": [
                        {
                            "type": "Cell",
                            "row": 1,
                            "col": 1,
                            "bbox": {"x": 380, "y": 250, "w": 290, "h": 180},
                            "children": [
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 400, "y": 280, "w": 240, "h": 20},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "첫 줄",
                                            "bbox": {"x": 400, "y": 280, "w": 240, "h": 20},
                                        }
                                    ],
                                },
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 420, "y": 286, "w": 220, "h": 20},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "겹친 줄",
                                            "bbox": {"x": 420, "y": 286, "w": 220, "h": 20},
                                        }
                                    ],
                                },
                                {
                                    "type": "Cell",
                                    "row": 0,
                                    "col": 0,
                                    "bbox": {"x": 430, "y": 280, "w": 100, "h": 50},
                                    "children": [
                                        {
                                            "type": "TextLine",
                                            "bbox": {"x": 430, "y": 280, "w": 100, "h": 20},
                                            "children": [
                                                {
                                                    "type": "TextRun",
                                                    "text": "중첩 셀",
                                                    "bbox": {"x": 430, "y": 280, "w": 100, "h": 20},
                                                }
                                            ],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

        candidates = FIDELITY.table_cell_text_overlap_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 7)
        self.assertEqual(candidates[0]["row"], 1)
        self.assertEqual(candidates[0]["overlap_pair_count"], 1)
        self.assertEqual(candidates[0]["max_overlap_y_px"], 14.0)

    def test_ignores_separated_lines_and_empty_guides(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": [
                {
                    "type": "Table",
                    "bbox": {"x": 80, "y": 200, "w": 600, "h": 300},
                    "children": [
                        {
                            "type": "Cell",
                            "bbox": {"x": 80, "y": 200, "w": 600, "h": 300},
                            "children": [
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 100, "y": 240, "w": 300, "h": 18},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "정상 줄",
                                            "bbox": {"x": 100, "y": 240, "w": 300, "h": 18},
                                        }
                                    ],
                                },
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 100, "y": 270, "w": 300, "h": 18},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "다음 줄",
                                            "bbox": {"x": 100, "y": 270, "w": 300, "h": 18},
                                        }
                                    ],
                                },
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 100, "y": 242, "w": 300, "h": 18},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "  ",
                                            "bbox": {"x": 100, "y": 242, "w": 300, "h": 18},
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.table_cell_text_overlap_candidates(tree), [])


class TableCellTextBoundaryCandidateTests(unittest.TestCase):
    def test_reports_visible_line_crossing_owning_cell_only(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Table",
                    "pi": 9,
                    "ci": 2,
                    "rows": 2,
                    "cols": 2,
                    "children": [
                        {
                            "type": "Cell",
                            "row": 1,
                            "col": 0,
                            "bbox": {"x": 100, "y": 100, "w": 200, "h": 100},
                            "children": [
                                {
                                    "type": "TextLine",
                                    "bbox": {"x": 100, "y": 118, "w": 220, "h": 18},
                                    "children": [
                                        {
                                            "type": "TextRun",
                                            "text": "우측선을 침범",
                                            "bbox": {
                                                "x": 120,
                                                "y": 120,
                                                "w": 185,
                                                "h": 15,
                                            },
                                        }
                                    ],
                                },
                                {
                                    "type": "Cell",
                                    "row": 0,
                                    "col": 0,
                                    "bbox": {"x": 285, "y": 150, "w": 50, "h": 30},
                                    "children": [
                                        {
                                            "type": "TextLine",
                                            "bbox": {
                                                "x": 300,
                                                "y": 155,
                                                "w": 30,
                                                "h": 10,
                                            },
                                            "children": [
                                                {
                                                    "type": "TextRun",
                                                    "text": "중첩 셀",
                                                    "bbox": {
                                                        "x": 300,
                                                        "y": 155,
                                                        "w": 30,
                                                        "h": 10,
                                                    },
                                                }
                                            ],
                                        }
                                    ],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

        candidates = FIDELITY.table_cell_text_boundary_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 9)
        self.assertEqual(candidates[0]["row"], 1)
        self.assertEqual(candidates[0]["candidate_kind"], "line_boundary_overflow")
        self.assertEqual(candidates[0]["node_type"], "TextLine")
        self.assertEqual(candidates[0]["edges"], ("right",))
        self.assertEqual(candidates[0]["overflow_right_px"], 20.0)

    def test_reports_visible_ending_natural_width_risk(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Cell",
                    "bbox": {"x": 100, "y": 100, "w": 200, "h": 100},
                    "children": [
                        {
                            "type": "TextLine",
                            "bbox": {"x": 110, "y": 120, "w": 189.2, "h": 15},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "자연 폭이 우측선을 침범",
                                    "bbox": {"x": 120, "y": 120, "w": 185, "h": 15},
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        candidates = FIDELITY.table_cell_text_boundary_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["candidate_kind"], "natural_visible_width_risk")
        self.assertEqual(candidates[0]["node_type"], "TextRun")
        self.assertEqual(candidates[0]["overflow_right_px"], 5.0)
        self.assertEqual(candidates[0]["edge_clearance_px"], 0.8)

    def test_ignores_natural_width_when_final_line_has_saved_margin(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Cell",
                    "bbox": {"x": 213.7, "y": 77.1, "w": 487.6, "h": 426.9},
                    "children": [
                        {
                            "type": "TextLine",
                            "bbox": {"x": 270.8, "y": 231.2, "w": 423.7, "h": 17.3},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "댓수는 자율안전확인신고가 형식별로 이루어 짐에 ",
                                    "bbox": {"x": 270.8, "y": 231.2, "w": 438.0, "h": 17.3},
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.table_cell_text_boundary_candidates(tree), [])

    def test_keeps_visible_ending_risk_even_when_line_box_is_inside(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Cell",
                    "bbox": {"x": 100, "y": 100, "w": 200, "h": 100},
                    "children": [
                        {
                            "type": "TextLine",
                            "bbox": {"x": 110, "y": 120, "w": 183, "h": 15},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "우측선을 침범",
                                    "bbox": {"x": 120, "y": 120, "w": 185, "h": 15},
                                }
                            ],
                        }
                    ],
                }
            ],
        }

        candidates = FIDELITY.table_cell_text_boundary_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["candidate_kind"], "natural_visible_width_risk")
        self.assertEqual(candidates[0]["edge_clearance_px"], 7.0)

    def test_ignores_small_overflow_blank_run_and_detached_continuation(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Cell",
                    "bbox": {"x": 100, "y": 100, "w": 200, "h": 100},
                    "children": [
                        {
                            "type": "TextLine",
                            "bbox": {"x": 110, "y": 120, "w": 191.9, "h": 15},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "허용 오차",
                                    "bbox": {
                                        "x": 110,
                                        "y": 120,
                                        "w": 191.9,
                                        "h": 15,
                                    },
                                }
                            ],
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 120, "y": 210, "w": 100, "h": 15},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "이전 fragment 잔존 노드",
                                    "bbox": {
                                        "x": 120,
                                        "y": 210,
                                        "w": 100,
                                        "h": 15,
                                    },
                                }
                            ],
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 295, "y": 150, "w": 20, "h": 15},
                            "children": [
                                {
                                    "type": "TextRun",
                                    "text": "  ",
                                    "bbox": {
                                        "x": 295,
                                        "y": 150,
                                        "w": 20,
                                        "h": 15,
                                    },
                                }
                            ],
                        },
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.table_cell_text_boundary_candidates(tree), [])


class SvgTextBandClipCandidateTests(unittest.TestCase):
    def test_reports_partial_top_and_bottom_clip_only(self) -> None:
        svg = """<svg xmlns="http://www.w3.org/2000/svg">
          <defs><clipPath id="cell-clip"><rect x="0" y="40" width="100" height="20"/></clipPath></defs>
          <g clip-path="url(#cell-clip)">
            <text x="10" y="45" font-size="10">top</text>
            <text x="20" y="61" font-size="10">bottom</text>
            <text x="30" y="50" font-size="10">inside</text>
            <text x="40" y="20" font-size="10">stale</text>
            <text x="150" y="47" font-size="10">disjoint</text>
            <text x="50" y="47" font-size="10">   </text>
          </g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p034.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_text_band_clip_candidates(svg_path)

        self.assertEqual([candidate["text"] for candidate in candidates], ["top", "bottom"])
        self.assertEqual(candidates[0]["edges"], ("top",))
        self.assertEqual(candidates[0]["clipped_top_px"], 3.0)
        self.assertEqual(candidates[1]["edges"], ("bottom",))
        self.assertEqual(candidates[1]["clipped_bottom_px"], 3.0)
        self.assertEqual(candidates[1]["clip_ids"], ("cell-clip",))

    def test_ignores_wholly_clipped_nested_stale_and_transformed_text(self) -> None:
        svg = """<svg xmlns="http://www.w3.org/2000/svg">
          <defs>
            <clipPath id="body-clip"><rect x="0" y="0" width="100" height="100"/></clipPath>
            <clipPath id="cell-clip"><rect x="0" y="40" width="100" height="20"/></clipPath>
          </defs>
          <g clip-path="url(#body-clip)">
            <g clip-path="url(#cell-clip)">
              <text x="10" y="20" font-size="10">stale</text>
              <g transform="translate(0 1)">
                <text x="10" y="47" font-size="10">unknown transform</text>
              </g>
            </g>
          </g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "p034.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_text_band_clip_candidates(svg_path)

        self.assertEqual(candidates, [])

    def test_ignores_stage96_p65_body_clip_when_ink_band_is_inside(self) -> None:
        svg = """<svg xmlns="http://www.w3.org/2000/svg">
          <defs><clipPath id="body-clip-3"><rect x="75.6" y="75.6" width="642.5" height="971.3"/></clipPath></defs>
          <g clip-path="url(#body-clip-3)">
            <text x="75.6" y="91.6" font-size="20">나. 각 대안의 활동별 비용·편익 분석 결과</text>
          </g>
        </svg>"""

        with tempfile.TemporaryDirectory() as directory:
            svg_path = Path(directory) / "86712_regulatory_analysis_065.svg"
            svg_path.write_text(svg, encoding="utf-8")
            candidates = FIDELITY.svg_text_band_clip_candidates(svg_path)

        self.assertEqual(candidates, [])


class LayoutCandidateTests(unittest.TestCase):
    @staticmethod
    def body_table_tree(
        *,
        pi: int = 100,
        ci: int = 0,
        rows: int = 5,
        cols: int = 3,
        x: float = 80.0,
        y: float = 700.0,
        width: float = 560.0,
        height: float = 180.0,
        footer_y: float | None = None,
    ) -> dict[str, object]:
        children: list[dict[str, object]] = [
            {
                "type": "Body",
                "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                "children": [
                    {
                        "type": "Table",
                        "pi": pi,
                        "ci": ci,
                        "rows": rows,
                        "cols": cols,
                        "bbox": {"x": x, "y": y, "w": width, "h": height},
                    }
                ],
            }
        ]
        if footer_y is not None:
            children.append(
                {
                    "type": "Footer",
                    "bbox": {"x": 50, "y": footer_y, "w": 700, "h": 30},
                }
            )
        return {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1000},
            "children": children,
        }

    def test_table_fragment_ledger_records_same_source_table_on_adjacent_pages(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tree_dir = root / "render_tree"
            tree_dir.mkdir()
            (tree_dir / "render_tree_001.json").write_text(
                json.dumps(self.body_table_tree(y=760.0, height=160.0)),
                encoding="utf-8",
            )
            (tree_dir / "render_tree_002.json").write_text(
                json.dumps(self.body_table_tree(y=80.0, height=130.0)),
                encoding="utf-8",
            )

            FIDELITY.write_table_fragment_ledger(
                root,
                tree_dir,
                [0],
                {0: (Counter("a" * 30), Counter())},
            )
            report = (root / "table-fragment-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn("page\tnext_page\tpi\tci\trows\tcols", report)
        self.assertIn("1\t2\t100\t0\t5\t3\t5\t3", report)
        self.assertIn("80.0,760.0,560.0,160.0", report)
        self.assertIn("same_pi_ci_adjacent_fragment", report)
        self.assertIn("page_bottom_near_material_text_delta", report)
        self.assertIn("does not assert PDF table row owner", report)

    def test_page_boundary_ledger_promotes_table_fragment_with_owner_drift(self) -> None:
        moved = "일시적반복적근거설명구내운반차안전조치"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tree_dir = root / "render_tree"
            tree_dir.mkdir()
            (tree_dir / "render_tree_081.json").write_text(
                json.dumps(self.body_table_tree(pi=842, y=760.0, height=160.0)),
                encoding="utf-8",
            )
            (tree_dir / "render_tree_082.json").write_text(
                json.dumps(self.body_table_tree(pi=842, y=80.0, height=220.0)),
                encoding="utf-8",
            )
            FIDELITY.write_page_boundary_fidelity_ledger(
                root,
                {
                    80: (Counter(moved), Counter()),
                    81: (Counter(), Counter(moved)),
                },
                {
                    80: (f"p81 {moved}", "p81"),
                    81: ("p82", f"p82 {moved}"),
                },
                tree_dir=tree_dir,
                requested_pages=[80, 81],
            )
            report = (root / "page-boundary-fidelity-candidates.tsv").read_text(
                encoding="utf-8"
            )

        self.assertIn(
            "81\t82\ttable_fragment_text_owner_drift\trhwp_later_than_reference",
            report,
        )
        self.assertIn("pi=842,ci=0,rows=5,cols=3", report)

    def test_table_fragment_candidates_include_footer_and_frame_geometry_signals(self) -> None:
        tree = self.body_table_tree(
            x=-5.0,
            y=870.0,
            width=560.0,
            height=100.0,
            footer_y=920.0,
        )
        with tempfile.TemporaryDirectory() as directory:
            tree_dir = Path(directory)
            (tree_dir / "render_tree_001.json").write_text(
                json.dumps(tree), encoding="utf-8"
            )
            candidates = FIDELITY.table_fragment_candidates(tree_dir, [0], {})

        self.assertEqual(len(candidates), 1)
        self.assertEqual(
            candidates[0]["signals"],
            ["page_table_footer", "page_table_outside_frame"],
        )

    def test_bottom_near_table_requires_material_text_delta(self) -> None:
        tree = self.body_table_tree(y=780.0, height=100.0)
        with tempfile.TemporaryDirectory() as directory:
            tree_dir = Path(directory)
            (tree_dir / "render_tree_001.json").write_text(
                json.dumps(tree), encoding="utf-8"
            )
            below_threshold = FIDELITY.table_fragment_candidates(
                tree_dir,
                [0],
                {0: (Counter("a" * 23), Counter())},
            )
            material_delta = FIDELITY.table_fragment_candidates(
                tree_dir,
                [0],
                {0: (Counter("a" * 24), Counter())},
            )

        self.assertEqual(below_threshold, [])
        self.assertEqual(len(material_delta), 1)
        self.assertEqual(
            material_delta[0]["signals"],
            ["page_bottom_near_material_text_delta"],
        )

    def test_square_wrapped_image_crossed_by_three_body_lines_is_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "pi": 1355,
                            "ci": 0,
                            "textWrap": "Square",
                            "bbox": {"x": 400, "y": 120, "w": 220, "h": 260},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 150, "w": 560, "h": 16},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 180, "w": 560, "h": 16},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 210, "w": 560, "h": 16},
                        },
                    ],
                }
            ],
        }

        candidates = FIDELITY.square_wrap_text_overlap_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 1355)
        self.assertEqual(candidates[0]["overlap_line_count"], 3)
        self.assertEqual(FIDELITY.layout_candidates(tree)[4], 1)

    def test_square_wrapped_image_edge_contact_is_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "pi": 1692,
                            "ci": 1,
                            "textWrap": "Square",
                            "bbox": {"x": 400, "y": 120, "w": 220, "h": 260},
                        },
                        *[
                            {
                                "type": "TextLine",
                                "bbox": {"x": 100, "y": y, "w": 300, "h": 16},
                                "children": [{"type": "TextRun", "text": "본문"}],
                            }
                            for y in (150, 180, 210)
                        ],
                    ],
                }
            ],
        }

        candidates = FIDELITY.square_wrap_text_overlap_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 1692)
        self.assertEqual(candidates[0]["candidate_kind"], "edge_clearance_loss")
        self.assertEqual(candidates[0]["edge"], "left")
        self.assertEqual(candidates[0]["edge_contact_line_count"], 3)
        self.assertEqual(candidates[0]["min_clearance_px"], 0.0)
        self.assertEqual(FIDELITY.layout_candidates(tree)[4], 1)

    def test_square_wrapped_image_with_pdf_like_edge_clearance_is_not_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "textWrap": "Square",
                            "bbox": {"x": 400, "y": 120, "w": 220, "h": 260},
                        },
                        *[
                            {
                                "type": "TextLine",
                                "bbox": {"x": 100, "y": y, "w": 294, "h": 16},
                                "children": [{"type": "TextRun", "text": "본문"}],
                            }
                            for y in (150, 180, 210)
                        ],
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.square_wrap_text_overlap_candidates(tree), [])

    def test_deferred_square_picture_below_body_top_is_a_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 80, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Column",
                            "bbox": {"x": 50, "y": 80, "w": 700, "h": 900},
                            "children": [
                                {
                                    "type": "Image",
                                    "pi": 1355,
                                    "ci": 0,
                                    "textWrap": "Square",
                                    "bbox": {"x": 440, "y": 128, "w": 220, "h": 260},
                                },
                                {
                                    "type": "TextLine",
                                    "pi": 1356,
                                    "bbox": {"x": 90, "y": 80, "w": 320, "h": 16},
                                    "children": [{"type": "TextRun", "text": "본문"}],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

        candidates = FIDELITY.deferred_square_picture_page_top_drift_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 1355)
        self.assertEqual(candidates[0]["candidate_kind"], "deferred_page_start_offset_drift")
        self.assertEqual(candidates[0]["image_top_drift_px"], 48.0)
        self.assertEqual(FIDELITY.layout_candidates(tree)[5], 1)

    def test_square_wrap_ignores_empty_full_width_guide_lines(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "textWrap": "Square",
                            "bbox": {"x": 400, "y": 120, "w": 220, "h": 260},
                        },
                        *[
                            {
                                "type": "TextLine",
                                "bbox": {"x": 100, "y": y, "w": 560, "h": 16},
                                "children": [{"type": "TextRun", "text": ""}],
                            }
                            for y in (150, 180, 210)
                        ],
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.square_wrap_text_overlap_candidates(tree), [])

    def test_square_wrap_keeps_marker_line_with_empty_text_run(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "pi": 1,
                            "ci": 0,
                            "textWrap": "Square",
                            "bbox": {"x": 300, "y": 200, "w": 200, "h": 200},
                        },
                        *[
                            {
                                "type": "TextLine",
                                "bbox": {"x": 100, "y": y, "w": 500, "h": 16},
                                "children": (
                                    [
                                        {"type": "TextRun", "text": ""},
                                        {"type": "FnMarker"},
                                    ]
                                    if y == 300
                                    else [{"type": "TextRun", "text": "본문"}]
                                ),
                            }
                            for y in (230, 300, 360)
                        ],
                    ],
                }
            ],
        }

        candidates = FIDELITY.square_wrap_text_overlap_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["overlap_line_count"], 3)

    def test_in_front_image_is_not_a_square_wrap_overlap_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 800, "h": 1100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 50, "y": 50, "w": 700, "h": 900},
                    "children": [
                        {
                            "type": "Image",
                            "textWrap": "InFrontOfText",
                            "bbox": {"x": 400, "y": 120, "w": 220, "h": 260},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 150, "w": 560, "h": 16},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 180, "w": 560, "h": 16},
                        },
                        {
                            "type": "TextLine",
                            "bbox": {"x": 100, "y": 210, "w": 560, "h": 16},
                        },
                    ],
                }
            ],
        }

        self.assertEqual(FIDELITY.square_wrap_text_overlap_candidates(tree), [])


class RegistryAndArgumentsTests(unittest.TestCase):
    def test_recognized_reference_patterns_use_pdf_directory_and_version_suffix(
        self,
    ) -> None:
        for key in ("plan", "manual", "korexam", "math", "eng"):
            with self.subTest(key=key):
                fixture = FIDELITY.REG[key]
                self.assertTrue(fixture.reference_pattern.startswith("pdf/"))
                self.assertRegex(fixture.reference_pattern, r"-20(?:22|24)\.pdf$")
                self.assertIn("기준 PDF", fixture.reference_grade)

    def test_legacy_sample_reference_is_explicitly_downgraded(self) -> None:
        fixture = FIDELITY.REG["bunjang"]

        self.assertTrue(fixture.reference_pattern.startswith("samples/"))
        self.assertIn("참고 PDF", fixture.reference_grade)
        self.assertIn("별도 확인 필요", fixture.reference_grade)

    def test_out_dir_is_parsed_as_exact_path(self) -> None:
        args = FIDELITY.parse_args(
            ["plan", "0", "9", "--out-dir", "/tmp/fidelity-plan"]
        )

        self.assertEqual(args.out_dir, Path("/tmp/fidelity-plan"))


if __name__ == "__main__":
    unittest.main()
