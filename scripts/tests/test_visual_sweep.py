from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw


MODULE_PATH = Path(__file__).resolve().parents[1] / "visual_sweep.py"
SPEC = importlib.util.spec_from_file_location("visual_sweep", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"visual_sweep 모듈을 불러올 수 없습니다: {MODULE_PATH}")
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


class ResumeCheckpointTests(unittest.TestCase):
    def test_run_manifest_rejects_changed_provenance(self) -> None:
        target = SWEEP.Target("fixture", Path("source.hwp"), Path("reference.pdf"))
        provenance = {
            "hwp": {"path": "source.hwp", "sha256": "hwp-a"},
            "pdf": {"path": "reference.pdf", "sha256": "pdf-a"},
            "git_head": "commit-a",
            "rhwp_binary": {"configured": "rhwp", "path": "rhwp", "sha256": "bin-a"},
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            base = Path(temp_dir)
            manifest = SWEEP.run_manifest_for_target(
                base, target, provenance, 144, 32, resume=False
            )
            self.assertEqual(manifest["run_state"], "incomplete")
            resumed = SWEEP.run_manifest_for_target(
                base, target, provenance, 144, 32, resume=True
            )
            self.assertEqual(resumed["provenance"], provenance)
            with self.assertRaises(SystemExit):
                SWEEP.run_manifest_for_target(
                    base, target, provenance, 144, 33, resume=True
                )

    def test_page_shards_accumulate_in_same_run(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            base = Path(temp_dir)
            manifest = {
                "requested_pages": [],
                "requested_page_shards": [],
                "run_state": "incomplete",
            }
            first = SWEEP.record_requested_page_shard(base, manifest, [1, 2, 3, 4])
            second = SWEEP.record_requested_page_shard(base, first, [5, 6, 7, 8])

            self.assertEqual(second["requested_pages"], list(range(1, 9)))
            self.assertEqual(second["requested_page_shards"], [[1, 2, 3, 4], [5, 6, 7, 8]])
            stored = json.loads((base / "run_manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(stored["requested_pages"], list(range(1, 9)))

    def test_incomplete_page_manifest_is_not_reused(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            base = Path(temp_dir)
            artifacts = {
                key: f"artifacts/{key}.bin" for key in SWEEP.REQUIRED_PAGE_ARTIFACTS
            }
            page_manifest = {"page": 1, "artifacts": artifacts}
            page_path = base / "pages" / "page-001.json"
            SWEEP.write_json_atomic(page_path, page_manifest)

            self.assertEqual(SWEEP.valid_page_manifests(base), {})

            for relative_path in artifacts.values():
                artifact = base / relative_path
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(b"checkpoint")
            self.assertEqual(set(SWEEP.valid_page_manifests(base)), {1})

    def test_summary_marks_uncheckpointed_requested_pages_incomplete(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            base = Path(temp_dir) / "fixture"
            artifacts = {
                "svg": "artifacts/page.svg",
                "render_tree": "artifacts/page.json",
                "rhwp_png": "artifacts/rhwp.png",
                "pdf_png": "artifacts/pdf.png",
                "compare": "artifacts/compare.png",
                "overlay": "artifacts/overlay.png",
                "review": "artifacts/review.png",
                "analysis": "artifacts/analysis.json",
            }
            for key in ("rhwp_png", "pdf_png", "compare", "overlay", "review"):
                path = base / artifacts[key]
                path.parent.mkdir(parents=True, exist_ok=True)
                Image.new("RGB", (24, 24), "white").save(path)
            (base / artifacts["svg"]).write_text("<svg/>", encoding="utf-8")
            (base / artifacts["render_tree"]).write_text(
                '{"type":"Page","children":[]}', encoding="utf-8"
            )
            visual_metrics = {"page": 1, "flags": []}
            SWEEP.write_json_atomic(base / artifacts["analysis"], visual_metrics)
            page_manifest = {
                "page": 1,
                "artifacts": artifacts,
                "overlay_metrics": {
                    "page": 1,
                    "pixel_match_percent": 100.0,
                    "ink_match_percent": 100.0,
                    "visual_accuracy_proxy_percent": 100.0,
                },
                "visual_metrics": visual_metrics,
            }
            SWEEP.write_json_atomic(base / "pages" / "page-001.json", page_manifest)
            run_manifest = {
                "provenance": {
                    "hwp": {"path": "source.hwp"},
                    "pdf": {"path": "reference.pdf"},
                },
                "requested_pages": [1, 2],
                "requested_page_shards": [[1, 2]],
                "run_state": "incomplete",
            }
            target = SWEEP.Target("fixture", Path("source.hwp"), Path("reference.pdf"))

            summary = SWEEP.write_target_status(
                base,
                base.parent,
                base,
                target,
                run_manifest,
                [base / artifacts["svg"]],
                [base / artifacts["render_tree"]],
                [base / artifacts["pdf_png"]],
                [],
                [],
                32,
            )

            self.assertEqual(summary["run_state"], "incomplete")
            self.assertEqual(summary["completed_pages"], [1])
            self.assertEqual(summary["missing_pages"], [2])
            self.assertEqual(summary["compare_pages"], 1)


class FidelityLayoutBridgeTests(unittest.TestCase):
    @staticmethod
    def square_wrap_overlap_tree() -> dict[str, object]:
        return {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 400, "h": 500},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 20, "y": 20, "w": 360, "h": 420},
                    "children": [
                        {
                            "type": "Image",
                            "pi": 1355,
                            "ci": 0,
                            "textWrap": "Square",
                            "bbox": {"x": 180, "y": 90, "w": 120, "h": 180},
                        },
                        *[
                            {
                                "type": "TextLine",
                                "pi": 1356,
                                "bbox": {"x": 40, "y": y, "w": 300, "h": 14},
                                "children": [{"type": "TextRun", "text": "본문"}],
                            }
                            for y in (110, 140, 170)
                        ],
                    ],
                }
            ],
        }

    @staticmethod
    def deferred_square_top_drift_tree() -> dict[str, object]:
        return {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 400, "h": 500},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 20, "y": 20, "w": 360, "h": 420},
                    "children": [
                        {
                            "type": "Column",
                            "bbox": {"x": 20, "y": 20, "w": 360, "h": 420},
                            "children": [
                                {
                                    "type": "Image",
                                    "pi": 1355,
                                    "ci": 0,
                                    "textWrap": "Square",
                                    "bbox": {"x": 180, "y": 70, "w": 120, "h": 180},
                                },
                                {
                                    "type": "TextLine",
                                    "pi": 1356,
                                    "bbox": {"x": 40, "y": 20, "w": 120, "h": 14},
                                    "children": [{"type": "TextRun", "text": "본문"}],
                                },
                            ],
                        }
                    ],
                }
            ],
        }

    @staticmethod
    def right_table_wrap_tree() -> dict[str, object]:
        return {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 400, "h": 500},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 20, "y": 20, "w": 360, "h": 420},
                    "children": [
                        {
                            "type": "Table",
                            "pi": 44,
                            "ci": 1,
                            "bbox": {"x": 220, "y": 100, "w": 120, "h": 180},
                            "children": [],
                        }
                    ],
                }
            ],
        }

    @staticmethod
    def draw_left_strip_text(image: Image.Image) -> None:
        draw = ImageDraw.Draw(image)
        for y in (120, 145, 170, 195, 220, 245):
            draw.line((32, y, 198, y), fill=(0, 0, 0), width=3)

    def test_uses_fidelity_square_wrap_detector(self) -> None:
        for text_wrap in ("Square", "Tight", "Through"):
            with self.subTest(text_wrap=text_wrap):
                tree = self.square_wrap_overlap_tree()
                tree["children"][0]["children"][0]["textWrap"] = text_wrap
                candidates = SWEEP.render_tree_square_wrap_text_overlap_candidates(tree)

                self.assertEqual(len(candidates), 1)
                self.assertEqual(candidates[0]["pi"], 1355)
                self.assertEqual(candidates[0]["overlap_line_count"], 3)

    def test_uses_fidelity_square_wrap_edge_clearance_detector(self) -> None:
        tree = self.square_wrap_overlap_tree()
        body_children = tree["children"][0]["children"]
        for node in body_children[1:]:
            node["bbox"]["w"] = 140  # x=40 + w=140: image left x=180에 접촉

        candidates = SWEEP.render_tree_square_wrap_text_overlap_candidates(tree)

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["candidate_kind"], "edge_clearance_loss")
        self.assertEqual(candidates[0]["edge_contact_line_count"], 3)

    def test_uses_fidelity_deferred_square_page_top_detector(self) -> None:
        candidates = SWEEP.render_tree_deferred_square_picture_top_drift_candidates(
            self.deferred_square_top_drift_tree()
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 1355)
        self.assertEqual(candidates[0]["image_top_drift_px"], 50.0)

    def test_right_table_left_strip_deficit_detects_dropped_wrap_prefix(self) -> None:
        tree = self.right_table_wrap_tree()
        rhwp = Image.new("RGB", (400, 500), "white")
        pdf = Image.new("RGB", (400, 500), "white")
        self.draw_left_strip_text(pdf)

        candidates = SWEEP.render_tree_right_table_left_strip_text_deficit_candidates(
            tree, rhwp, pdf
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 44)
        self.assertLess(candidates[0]["rhwp_to_pdf_ink_ratio"], 0.15)

    def test_right_table_left_strip_deficit_ignores_matched_wrap_text(self) -> None:
        tree = self.right_table_wrap_tree()
        rhwp = Image.new("RGB", (400, 500), "white")
        pdf = Image.new("RGB", (400, 500), "white")
        self.draw_left_strip_text(rhwp)
        self.draw_left_strip_text(pdf)

        candidates = SWEEP.render_tree_right_table_left_strip_text_deficit_candidates(
            tree, rhwp, pdf
        )

        self.assertEqual(candidates, [])

    def test_rejects_missing_or_malformed_render_tree(self) -> None:
        for tree in (None, {}, {"type": "Page"}):
            with self.subTest(tree=tree):
                with self.assertRaisesRegex(RuntimeError, "render tree"):
                    SWEEP.render_tree_square_wrap_text_overlap_candidates(tree)

    def test_analyze_page_flags_fidelity_square_wrap_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            rhwp_path = root / "rhwp.png"
            pdf_path = root / "pdf.png"
            tree_path = root / "tree.json"
            svg_path = root / "page.svg"
            analysis_dir = root / "analysis"
            Image.new("RGB", (400, 500), "white").save(rhwp_path)
            Image.new("RGB", (400, 500), "white").save(pdf_path)
            svg_path.write_text("<svg/>", encoding="utf-8")
            tree_path.write_text(
                json.dumps(self.square_wrap_overlap_tree()), encoding="utf-8"
            )

            result = SWEEP.analyze_page(
                rhwp_path,
                pdf_path,
                svg_path,
                tree_path,
                analysis_dir,
                "fixture",
                0,
                [],
                {},
                32,
            )

        self.assertIn("square_wrap_text_overlap", result["flags"])
        self.assertEqual(result["square_wrap_text_overlap_candidates"][0]["pi"], 1355)

    def test_analyze_page_flags_deferred_square_page_top_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            rhwp_path = root / "rhwp.png"
            pdf_path = root / "pdf.png"
            tree_path = root / "tree.json"
            svg_path = root / "page.svg"
            analysis_dir = root / "analysis"
            Image.new("RGB", (400, 500), "white").save(rhwp_path)
            Image.new("RGB", (400, 500), "white").save(pdf_path)
            svg_path.write_text("<svg/>", encoding="utf-8")
            tree_path.write_text(
                json.dumps(self.deferred_square_top_drift_tree()), encoding="utf-8"
            )

            result = SWEEP.analyze_page(
                rhwp_path,
                pdf_path,
                svg_path,
                tree_path,
                analysis_dir,
                "fixture",
                0,
                [],
                {},
                32,
            )

        self.assertIn("deferred_square_picture_top_drift", result["flags"])
        self.assertEqual(result["deferred_square_picture_top_drift_candidates"][0]["pi"], 1355)

    def test_summary_includes_fidelity_square_wrap_flag(self) -> None:
        summary, flagged = SWEEP.visual_summary_for_pages(
            [{"page": 127, "flags": ["square_wrap_text_overlap"]}],
            Path("metrics.json"),
            Path("question_flow.json"),
        )

        self.assertEqual(summary["square_wrap_text_overlap_pages"], [127])
        self.assertEqual(flagged[0]["page"], 127)

    def test_summary_includes_deferred_square_page_top_flag(self) -> None:
        summary, flagged = SWEEP.visual_summary_for_pages(
            [{"page": 127, "flags": ["deferred_square_picture_top_drift"]}],
            Path("metrics.json"),
            Path("question_flow.json"),
        )

        self.assertEqual(summary["deferred_square_picture_top_drift_pages"], [127])
        self.assertEqual(flagged[0]["page"], 127)

    def test_summary_includes_right_table_left_strip_text_deficit_flag(self) -> None:
        summary, flagged = SWEEP.visual_summary_for_pages(
            [{"page": 5, "flags": ["right_table_left_strip_text_deficit"]}],
            Path("metrics.json"),
            Path("question_flow.json"),
        )

        self.assertEqual(summary["right_table_left_strip_text_deficit_pages"], [5])
        self.assertEqual(flagged[0]["page"], 5)


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

    def test_display_projection_suppresses_resolved_legacy_glyph_candidate(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "TextRun",
                    "bbox": {"x": 10, "y": 10, "w": 20, "h": 10},
                    "text": "ᄒᆞᆫ글",
                    "displayText": "한글",
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

        self.assertEqual(
            candidates,
            [],
            "source text의 옛자모가 displayText로 이미 해결됐으면 legacy glyph 후보가 아니어야 한다",
        )

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


class FrameDetectionTests(unittest.TestCase):
    def test_frame_tail_uses_raster_coordinates_and_skips_off_page_nodes(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "Body",
                    "children": [
                        {
                            "type": "TextLine",
                            "pi": 7,
                            "bbox": {"x": 10, "y": 94, "w": 80, "h": 4},
                            "children": [{"type": "TextRun", "text": "보이는 꼬리"}],
                        },
                        {
                            "type": "TextLine",
                            "pi": 8,
                            "bbox": {"x": 10, "y": 140, "w": 80, "h": 4},
                            "children": [{"type": "TextRun", "text": "clip 밖 잔여 노드"}],
                        },
                    ],
                }
            ],
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "tree.json"
            path.write_text(json.dumps(tree, ensure_ascii=False), encoding="utf-8")
            raster = Image.new("RGB", (200, 200), "white")
            ImageDraw.Draw(raster).line((20, 190, 180, 190), fill="black", width=1)
            candidates = SWEEP.render_tree_frame_tail_candidates(
                path,
                (0, 0, 200, 190),
                page_tree=tree,
                raster_image=raster,
            )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["pi"], 7)
        self.assertEqual(candidates[0]["bbox"], [18.0, 186.0, 164.0, 12.0])
        self.assertEqual(candidates[0]["render_tree_bbox"], [10.0, 94.0, 80.0, 4.0])

    def test_centered_footnote_separator_is_not_treated_as_page_bottom(self) -> None:
        image = Image.new("RGB", (794, 1123), "white")
        draw = ImageDraw.Draw(image)

        # Chrome's 794px page raster lets a 368px centered footnote separator
        # cross the old 45% row-coverage threshold.  It must not shrink the
        # frame to the footnote line and contaminate all bottom-flow metrics.
        draw.line((213, 1014, 580, 1014), fill="black", width=1)

        _left, _top, _right, bottom = SWEEP.detect_frame(image)

        self.assertEqual(bottom, round(image.height * 0.977))

    def test_bottom_table_border_is_not_treated_as_page_bottom(self) -> None:
        image = Image.new("RGB", (794, 1123), "white")
        draw = ImageDraw.Draw(image)

        # A wide table can exceed the coverage threshold but still sits inside
        # the content area, well above the physical page footer.
        draw.line((81, 1020, 555, 1020), fill="black", width=1)

        _left, _top, _right, bottom = SWEEP.detect_frame(image)

        self.assertEqual(bottom, round(image.height * 0.977))

    def test_matching_page_number_footer_is_not_a_tail_overflow(self) -> None:
        candidates = [
            {
                "text": "- 94 -",
                "overflow_px": 52.1,
                "bbox": [373.9, 1053.1, 46.0, 16.0],
            }
        ]

        active, suppressed = SWEEP.suppress_tolerated_frame_tail_candidates(
            candidates,
            rhwp_out_pixels=87,
            rhwp_outside_frame_bleed_px=62,
            pdf_outside_frame_bleed_px=59,
            content_bottom_delta=-3.0,
            question_marker_drifts=[],
        )

        self.assertEqual(active, [])
        self.assertEqual(suppressed[0]["suppressed_reason"], "page_number_footer_bleed")


class RenderTreeLineOrderTests(unittest.TestCase):
    @staticmethod
    def line(pi: int, y: int, text: str) -> dict[str, object]:
        return {
            "type": "TextLine",
            "pi": pi,
            "bbox": {"x": 90, "y": y, "w": 600, "h": 12},
            "children": [{"type": "TextRun", "text": text}],
        }

    def test_does_not_compare_body_and_footnote_area_as_one_flow(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {"type": "Body", "children": [self.line(1, 963, "본문 마지막 줄")]},
                {"type": "FootnoteArea", "children": [self.line(2, 965, "175) 각주")]},
            ],
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "tree.json"
            path.write_text(json.dumps(tree, ensure_ascii=False), encoding="utf-8")

            candidates = SWEEP.render_tree_line_order_overlap_candidates(path)

        self.assertEqual(candidates, [])

    def test_keeps_overlapping_lines_in_one_flow_as_candidates(self) -> None:
        tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Body",
                    "children": [
                        self.line(1, 963, "본문 첫 줄"),
                        self.line(2, 965, "본문 겹친 줄"),
                    ],
                }
            ],
        }
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "tree.json"
            path.write_text(json.dumps(tree, ensure_ascii=False), encoding="utf-8")

            candidates = SWEEP.render_tree_line_order_overlap_candidates(path)

        self.assertEqual(len(candidates), 1)


class QuestionMarkerFlowTests(unittest.TestCase):
    def test_coloured_chart_is_not_question_flow_without_semantic_marker_drift(self) -> None:
        red_drift = {
            "rhwp_count": 0,
            "pdf_count": 3,
            "max_abs_delta_px": None,
            "mean_abs_delta_px": None,
            "p90_abs_delta_px": None,
        }
        line_drift = {"mean_abs_delta_px": 88.1, "p90_abs_delta_px": 174.0}
        large_region_drift = {"rhwp_count": 3, "pdf_count": 5, "max_abs_delta_px": 301.0}

        self.assertFalse(
            SWEEP.is_question_marker_flow_drift(
                red_drift,
                line_drift,
                large_region_drift,
                has_question_marker_drift=False,
            )
        )

    def test_semantic_question_marker_drift_keeps_structural_signal(self) -> None:
        red_drift = {
            "rhwp_count": 0,
            "pdf_count": 3,
            "max_abs_delta_px": None,
            "mean_abs_delta_px": None,
            "p90_abs_delta_px": None,
        }
        line_drift = {"mean_abs_delta_px": 88.1, "p90_abs_delta_px": 174.0}
        large_region_drift = {"rhwp_count": 3, "pdf_count": 5, "max_abs_delta_px": 301.0}

        self.assertTrue(
            SWEEP.is_question_marker_flow_drift(
                red_drift,
                line_drift,
                large_region_drift,
                has_question_marker_drift=True,
            )
        )


class ColumnTextFlowCollapseCandidateTests(unittest.TestCase):
    def test_detects_large_single_column_band_count_and_y_flow_divergence(self) -> None:
        drifts = [
            {
                "column": 1,
                "drift": {
                    "rhwp_count": 34,
                    "pdf_count": 37,
                    "mean_abs_delta_px": 109.4,
                    "p90_abs_delta_px": 157.0,
                },
            }
        ]

        candidates = SWEEP.column_text_flow_collapse_candidates(
            drifts,
            has_reflowing_float=True,
        )

        self.assertEqual(len(candidates), 1)
        self.assertEqual(candidates[0]["column"], 1)
        self.assertEqual(candidates[0]["band_count_delta"], 3)
        self.assertEqual(candidates[0]["reason"], "column_line_count_and_y_flow_diverge")

    def test_does_not_treat_small_font_baseline_shift_as_flow_collapse(self) -> None:
        drifts = [
            {
                "column": 0,
                "drift": {
                    "rhwp_count": 37,
                    "pdf_count": 37,
                    "mean_abs_delta_px": 95.0,
                    "p90_abs_delta_px": 150.0,
                },
            }
        ]

        self.assertEqual(SWEEP.column_text_flow_collapse_candidates(drifts), [])

    def test_requires_a_reflowing_float_to_avoid_toc_page_number_rail_false_positive(self) -> None:
        drifts = [
            {
                "column": 1,
                "drift": {
                    "rhwp_count": 48,
                    "pdf_count": 36,
                    "mean_abs_delta_px": 147.5,
                    "p90_abs_delta_px": 237.0,
                },
            }
        ]

        self.assertEqual(
            SWEEP.column_text_flow_collapse_candidates(
                drifts,
                has_reflowing_float=False,
            ),
            [],
        )

    def test_detects_square_tight_and_through_image_float_in_render_tree(self) -> None:
        square_tree = {
            "type": "Page",
            "children": [{"type": "Image", "textWrap": "Square"}],
        }
        toc_tree = {
            "type": "Page",
            "children": [
                {
                    "type": "Column",
                    "children": [
                        {"type": "TextRun", "text": "목차\t104"},
                    ],
                }
            ],
        }

        self.assertTrue(SWEEP.render_tree_has_reflowing_text_flow_float(square_tree))
        self.assertFalse(SWEEP.render_tree_has_reflowing_text_flow_float(toc_tree))

    def test_masks_centered_table_strokes_before_column_text_flow_comparison(self) -> None:
        rhwp = Image.new("RGB", (200, 200), "white")
        pdf = Image.new("RGB", (200, 200), "white")
        rhwp_draw = ImageDraw.Draw(rhwp)
        pdf_draw = ImageDraw.Draw(pdf)

        # Same centered table: rhwp rules are disconnected bands while the PDF
        # raster joins them.  These are not paragraph-flow baselines.
        for y in range(20, 131, 10):
            rhwp_draw.line((102, y, 198, y), fill="black", width=1)
        pdf_draw.rectangle((102, 20, 198, 130), fill="black")
        for y in (140, 160, 180):
            rhwp_draw.line((102, y, 198, y), fill="black", width=1)
            pdf_draw.line((102, y, 198, y), fill="black", width=1)

        frame = (0, 0, 200, 200)
        raw = SWEEP.column_line_band_drifts(rhwp, pdf, frame, frame)
        self.assertEqual(len(SWEEP.column_text_flow_collapse_candidates(raw)), 1)

        masked = SWEEP.column_line_band_drifts(
            rhwp,
            pdf,
            frame,
            frame,
            rhwp_mask_rectangles=[(102, 20, 199, 131)],
            pdf_mask_rectangles=[(102, 20, 199, 131)],
        )
        self.assertEqual(SWEEP.column_text_flow_collapse_candidates(masked), [])

    def test_body_table_mask_excludes_footnote_table(self) -> None:
        tree = {
            "type": "Page",
            "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
            "children": [
                {
                    "type": "Body",
                    "bbox": {"x": 0, "y": 0, "w": 100, "h": 100},
                    "children": [
                        {
                            "type": "Table",
                            "bbox": {"x": 10, "y": 20, "w": 30, "h": 40},
                        }
                    ],
                },
                {
                    "type": "FootnoteArea",
                    "children": [
                        {
                            "type": "Table",
                            "bbox": {"x": 50, "y": 60, "w": 20, "h": 20},
                        }
                    ],
                },
            ],
        }

        self.assertEqual(
            SWEEP.render_tree_body_table_masks(tree, Image.new("RGB", (100, 100), "white")),
            [(8, 18, 42, 62)],
        )
        self.assertEqual(
            SWEEP.render_tree_body_raster_frame(tree, Image.new("RGB", (100, 100), "white")),
            (0, 0, 100, 100),
        )


if __name__ == "__main__":
    unittest.main()
