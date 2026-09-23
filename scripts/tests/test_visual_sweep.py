from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from PIL import Image, ImageDraw


MODULE_PATH = Path(__file__).resolve().parents[1] / "visual_sweep.py"
SPEC = importlib.util.spec_from_file_location("visual_sweep", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"visual_sweep 모듈을 불러올 수 없습니다: {MODULE_PATH}")
SWEEP = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SWEEP
SPEC.loader.exec_module(SWEEP)


class SubpixelTolerantContentMatchTests(unittest.TestCase):
    def test_one_pixel_silhouette_shift_is_accepted_within_radius(self) -> None:
        rhwp = Image.new("RGB", (32, 32), "white")
        pdf = Image.new("RGB", (32, 32), "white")
        ImageDraw.Draw(rhwp).line((10, 4, 10, 27), fill="black", width=1)
        ImageDraw.Draw(pdf).line((11, 4, 11, 27), fill="black", width=1)
        self.assertEqual(
            SWEEP.subpixel_tolerant_content_match_percent(rhwp, pdf, radius_px=1), 100.0
        )

    def test_displacement_beyond_radius_remains_visible(self) -> None:
        rhwp = Image.new("RGB", (32, 32), "white")
        pdf = Image.new("RGB", (32, 32), "white")
        ImageDraw.Draw(rhwp).line((8, 4, 8, 27), fill="black", width=1)
        ImageDraw.Draw(pdf).line((13, 4, 13, 27), fill="black", width=1)
        value = SWEEP.subpixel_tolerant_content_match_percent(rhwp, pdf, radius_px=2)
        self.assertIsNotNone(value)
        self.assertLess(value, 100.0)


class PrReviewGateTests(unittest.TestCase):
    def test_help_renders_the_percent_threshold(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(MODULE_PATH), "--help"],
            check=True,
            text=True,
            capture_output=True,
        )
        self.assertIn("90% 실루엣 gate", completed.stdout)

    def test_below_ninety_requires_re_review(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 3, "tolerant_content_match_percent": 89.99}],
            font_mismatch_evidence=None,
        )
        self.assertEqual(gate["status"], "re_review_required")
        self.assertEqual(
            gate["below_threshold_pages"],
            [{"page": 3, "tolerant_content_match_percent": 89.99}],
        )

    def test_ninety_and_above_passes(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 3, "tolerant_content_match_percent": 90.0}],
            font_mismatch_evidence=None,
        )
        self.assertEqual(gate["status"], "passed")

    def test_missing_requested_metric_requires_re_review(self) -> None:
        gate = SWEEP.pr_review_gate(
            [{"page": 7, "tolerant_content_match_percent": 99.0}],
            expected_pages=[7, 8],
            font_mismatch_evidence=None,
        )
        self.assertEqual(gate["status"], "re_review_required")
        self.assertEqual(gate["unavailable_metric_pages"], [8])

    def test_hashed_font_mismatch_evidence_is_the_only_exception(self) -> None:
        evidence = {"path": "scratch/font-mismatch.md", "sha256": "a" * 64}
        gate = SWEEP.pr_review_gate(
            [{"page": 3, "tolerant_content_match_percent": 28.4}],
            font_mismatch_evidence=evidence,
        )
        self.assertEqual(gate["status"], "font_mismatch_exception")
        self.assertEqual(gate["font_mismatch_evidence"], evidence)


class LabelWrapTests(unittest.TestCase):
    """[#7349] 라벨이 canvas 폭을 넘으면 접는다 — 문서 이미지는 건드리지 않는다."""

    def setUp(self) -> None:
        self.font = SWEEP.label_font()

    def width_of(self, text: str) -> int:
        bbox = self.font.getbbox(text)
        return bbox[2] - bbox[0]

    def test_short_label_stays_one_line(self) -> None:
        lines = SWEEP.wrap_label_lines("t34 p014 overlay", self.font, 600)
        self.assertEqual(lines, ["t34 p014 overlay"])

    def test_long_label_wraps_within_width(self) -> None:
        text = (
            "chemical-rewind (WASM) p014 overlay pixel_match=93.797% "
            "ink_match=21.711% diff=55306/891662"
        )
        max_width = 400
        lines = SWEEP.wrap_label_lines(text, self.font, max_width)
        self.assertGreater(len(lines), 1)
        for line in lines:
            self.assertLessEqual(self.width_of(line), max_width, line)
        self.assertEqual(
            "".join(lines).replace(" ", ""),
            text.replace(" ", ""),
            "접어도 글자를 잃지 않는다",
        )

    def test_unbroken_token_is_split_by_characters(self) -> None:
        text = "x" * 400
        max_width = 120
        lines = SWEEP.wrap_label_lines(text, self.font, max_width)
        self.assertGreater(len(lines), 1)
        for line in lines:
            self.assertLessEqual(self.width_of(line), max_width, line)
        self.assertEqual("".join(lines), text)

    def test_label_line_height_is_positive(self) -> None:
        self.assertGreaterEqual(SWEEP.label_line_height(self.font), 12)


class OverlayLabelFitTests(unittest.TestCase):
    """[#7349] overlay PNG 의 라벨이 canvas 오른쪽에서 잘리지 않는다 — 문서 영역은 그대로."""

    def make_pages(self, temp_dir: Path) -> tuple[Path, Path]:
        rhwp = Image.new("RGB", (794, 240), "white")
        ImageDraw.Draw(rhwp).rectangle([40, 40, 300, 120], fill=(0, 0, 0))
        pdf = Image.new("RGB", (794, 240), "white")
        ImageDraw.Draw(pdf).rectangle([48, 40, 308, 120], fill=(0, 0, 0))
        rhwp_path = temp_dir / "rhwp_014.png"
        pdf_path = temp_dir / "pdf-014.png"
        rhwp.save(rhwp_path)
        pdf.save(pdf_path)
        return rhwp_path, pdf_path

    def test_overlay_summary_keeps_tolerant_content_metric(self) -> None:
        with tempfile.TemporaryDirectory() as raw_dir:
            temp_dir = Path(raw_dir)
            rhwp_path, pdf_path = self.make_pages(temp_dir)
            result = SWEEP.make_overlay_compares(
                [rhwp_path], [pdf_path], temp_dir / "overlay", "summary", pixel_diff_threshold=16
            )
            summary = result["summary"]
            self.assertIsInstance(summary["average_tolerant_content_match_percent"], float)
            self.assertIsInstance(summary["worst_tolerant_content_match_percent"], float)

    def test_long_key_label_ink_stays_inside_canvas(self) -> None:
        with tempfile.TemporaryDirectory() as raw_dir:
            temp_dir = Path(raw_dir)
            rhwp_path, pdf_path = self.make_pages(temp_dir)
            out_path = temp_dir / "overlay_014.png"
            metrics = SWEEP.make_overlay_page(
                rhwp_path,
                pdf_path,
                out_path,
                "chemical-rewind (WASM) 긴 진단 라벨 폭 초과 재현 key",
                13,
                pixel_diff_threshold=16,
            )
            canvas = Image.open(out_path).convert("L")
            label_height = canvas.height - 240
            self.assertGreater(label_height, 0, "라벨 영역이 있어야 한다")
            pixels = canvas.load()
            right_edge = [
                x
                for x in range(canvas.width)
                for y in range(label_height)
                if pixels[x, y] < 128
            ]
            self.assertTrue(right_edge, "라벨 잉크가 있어야 한다")
            self.assertLess(
                max(right_edge),
                canvas.width - 1,
                "라벨이 canvas 오른쪽 끝까지 닿으면 잘린 것이다",
            )
            # 문서 영역은 접기 전후로 같다 — 라벨만 늘어난다.
            document = canvas.crop((0, label_height, canvas.width, canvas.height))
            self.assertEqual(document.size, (794, 240))
            self.assertEqual(metrics["width"], 794)

    def test_review_uses_the_overlay_footer_once(self) -> None:
        """overlay의 한국어 지표는 review 패널에서 한 번만 보여야 한다."""
        with tempfile.TemporaryDirectory() as raw_dir:
            temp_dir = Path(raw_dir)
            rhwp_path, pdf_path = self.make_pages(temp_dir)
            (temp_dir / "compare").mkdir()
            compare = SWEEP.make_compares(
                [rhwp_path], [pdf_path], temp_dir / "compare", "footer"
            )
            overlay_path = temp_dir / "overlay" / "overlay_014.png"
            metrics = SWEEP.make_overlay_page(
                rhwp_path,
                pdf_path,
                overlay_path,
                "footer",
                13,
                pixel_diff_threshold=16,
            )
            review = SWEEP.make_review_panels(
                compare, [overlay_path], [metrics], temp_dir / "review"
            )[0]
            with Image.open(review) as review_image, Image.open(overlay_path) as overlay_image:
                self.assertEqual(
                    review_image.height,
                    overlay_image.height,
                    "review가 overlay 하단 지표를 다시 붙이면 안 된다",
                )


class LabelFontTests(unittest.TestCase):
    def setUp(self) -> None:
        SWEEP.label_font.cache_clear()

    def tearDown(self) -> None:
        SWEEP.label_font.cache_clear()

    def test_fontconfig_label_font_path_uses_existing_match(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            font_path = Path(temp_dir) / "NotoSansCJK-Regular.ttc"
            font_path.write_bytes(b"fake font")

            def fake_run(cmd: list[str], **_: object) -> subprocess.CompletedProcess[str]:
                return subprocess.CompletedProcess(cmd, 0, stdout=f"{font_path}\n", stderr="")

            with (
                patch.object(SWEEP.shutil, "which", return_value="/usr/bin/fc-match"),
                patch.object(SWEEP.subprocess, "run", side_effect=fake_run),
            ):
                self.assertEqual(SWEEP.fontconfig_label_font_path(), font_path)

    def test_configured_label_font_paths_puts_env_before_fontconfig(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            env_font = Path(temp_dir) / "review-label.ttf"
            fc_font = Path(temp_dir) / "NotoSansCJK-Regular.ttc"
            with (
                patch.dict(os.environ, {SWEEP.LABEL_FONT_ENV: str(env_font)}),
                patch.object(SWEEP, "fontconfig_label_font_path", return_value=fc_font),
                patch.object(SWEEP, "known_label_font_paths", return_value=[]),
            ):
                self.assertEqual(SWEEP.configured_label_font_paths(), [env_font, fc_font])

    def test_env_label_font_paths_accepts_platform_path_separator(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            first = Path(temp_dir) / "first.ttf"
            second = Path(temp_dir) / "second.ttf"
            value = os.pathsep.join([str(first), str(second)])
            with patch.dict(os.environ, {SWEEP.LABEL_FONT_ENV: value}):
                self.assertEqual(SWEEP.env_label_font_paths(), [first, second])

    def test_known_label_font_paths_prioritizes_current_platform(self) -> None:
        paths_by_platform = {
            "Linux": ("linux-font.ttf",),
            "Darwin": ("mac-font.ttf",),
            "Windows": ("windows-font.ttf",),
        }
        with (
            patch.object(SWEEP.platform, "system", return_value="Windows"),
            patch.object(SWEEP, "LABEL_FONT_PATHS_BY_PLATFORM", paths_by_platform),
        ):
            self.assertEqual(
                SWEEP.known_label_font_paths(),
                [Path("windows-font.ttf"), Path("linux-font.ttf"), Path("mac-font.ttf")],
            )

    def test_configured_label_font_paths_dedupes_repeated_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            font_path = Path(temp_dir) / "same.ttf"
            with (
                patch.dict(os.environ, {SWEEP.LABEL_FONT_ENV: str(font_path)}),
                patch.object(SWEEP, "fontconfig_label_font_path", return_value=font_path),
                patch.object(SWEEP, "known_label_font_paths", return_value=[font_path]),
            ):
                self.assertEqual(SWEEP.configured_label_font_paths(), [font_path])

    def test_label_font_loads_truetype_before_default_font(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            font_path = Path(temp_dir) / "NanumGothic.ttf"
            font_path.write_bytes(b"fake font")
            sentinel = object()
            with (
                patch.object(SWEEP, "configured_label_font_paths", return_value=[font_path]),
                patch.object(SWEEP.ImageFont, "truetype", return_value=sentinel) as truetype,
                patch.object(SWEEP.ImageFont, "load_default") as load_default,
            ):
                self.assertIs(SWEEP.label_font(), sentinel)
                truetype.assert_called_once_with(str(font_path), 18)
                load_default.assert_not_called()


class SelectedRasterTests(unittest.TestCase):
    def test_singleton_document_year_is_not_the_physical_page(self) -> None:
        svg = Path("two-digits-wrap-2020.svg")
        tree = Path("page_2020.json")
        pdf = Path("pdf-1.png")
        self.assertEqual(
            SWEEP.select_source_page_paths([svg], [tree], [pdf], [1]),
            [(1, svg, tree, pdf)],
        )

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


class WasmSweepTests(unittest.TestCase):
    def test_font_policy_preserves_geometry_and_uses_only_font_faces(self) -> None:
        source = '<svg width="100"><text x="12" y="34" font-family="휴먼명조">조문</text></svg>'
        face = '@font-face { font-family: "휴먼명조"; src: local("HCR Batang"); }'
        policy = f'<svg><style>{face} text {{display:none}}</style><text x="99">다른 본문</text></svg>'
        result = SWEEP.apply_svg_font_policy(source, policy + policy)
        self.assertEqual(result, source.replace('width="100">', f'width="100"><style>{face}</style>'))

    def test_explicit_font_change_invalidates_resume(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fonts = root / 'fonts'
            fonts.mkdir()
            font = fonts / 'source.ttf'
            font.write_bytes(b'original font')
            args, before = SWEEP.svg_font_export_options(root, 'full', [fonts])
            self.assertEqual(args, ['--embed-fonts=full', '--font-path', str(fonts)])
            target = SWEEP.Target('input', Path('input.hwp'), Path('reference.pdf'))
            SWEEP.run_manifest_for_target(root / 'out', target, {'font_supply': before}, 96, 32, resume=False)
            font.write_bytes(b'changed font')
            _, after = SWEEP.svg_font_export_options(root, 'full', [fonts])
            with self.assertRaises(SystemExit):
                SWEEP.run_manifest_for_target(root / 'out', target, {'font_supply': after}, 96, 32, resume=True)

    def test_embedded_font_policy_does_not_replace_wasm_text_or_coordinates(self) -> None:
        source = '<svg><text x="12" y="34">original</text></svg>'
        face = '@font-face {font-family:"Source";src:url("data:font/ttf;base64,AAAA");}'
        result = SWEEP.apply_svg_font_policy(source, '<svg><style>' + face + '</style><text x="99">native</text></svg>')
        self.assertEqual(result, source.replace('<svg>', '<svg><style>' + face + '</style>'))

    def test_font_policy_does_not_replace_a_wasm_owned_face(self) -> None:
        source = '<svg><style>@font-face {font-family:"Owned";src:url("wasm.woff2")}</style></svg>'
        self.assertEqual(SWEEP.apply_svg_font_policy(source, '@font-face {font-family:"Owned";src:local("Other")}'), source)

    def test_changed_wasm_package_invalidates_resume(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            package = root / 'pkg'
            package.mkdir()
            (root / 'scripts').mkdir()
            (root / 'scripts/export-wasm-for-sweep.mjs').write_text('exporter')
            (package / 'rhwp.js').write_text('module')
            (package / 'rhwp_bg.wasm').write_bytes(b'first')
            before = SWEEP.wasm_package_provenance(root, package)
            target = SWEEP.Target('input', Path('input.hwp'), Path('reference.pdf'))
            SWEEP.run_manifest_for_target(root / 'out', target, {'wasm': before}, 96, 32, resume=False)
            (package / 'rhwp_bg.wasm').write_bytes(b'second')
            after = SWEEP.wasm_package_provenance(root, package)
            self.assertNotEqual(before, after)
            with self.assertRaises(SystemExit):
                SWEEP.run_manifest_for_target(root / 'out', target, {'wasm': after}, 96, 32, resume=True)

    def test_wasm_export_keeps_wasm_tree_and_page_count(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def fake_run(command, **kwargs):
                if command[0] == 'node':
                    output = Path(command[command.index('--out') + 1])
                    (output / 'raw_svg').mkdir(parents=True)
                    (output / 'render_tree').mkdir()
                    for page in (1, 2):
                        (output / f'raw_svg/wasm_{page:03}.svg').write_text('<svg><text x="4">WASM</text></svg>')
                        (output / f'render_tree/render_tree_{page:03}.json').write_text('{"type":"Page","bbox":{"x":4}}')
                    (output / 'manifest.json').write_text('{"pageCount":2}')
                else:
                    # Native pagination is deliberately different: only its font CSS is used.
                    (Path(command[command.index('-o') + 1]) / 'native.svg').write_text('<svg><style>@font-face {font-family: "A";src:local("A")}</style><text x="99">Native</text></svg>')
                return subprocess.CompletedProcess(command, 0, '', '')
            with patch.object(SWEEP, 'run', side_effect=fake_run):
                SWEEP.export_wasm_target(root, root / 'input.hwp', root / 'pkg', 'rhwp', root / 'out')
            svg = (root / 'out/svg/wasm_002.svg').read_text()
            self.assertIn('<text x="4">WASM</text>', svg)
            self.assertNotIn('Native', svg)
            self.assertEqual(json.loads((root / 'out/render_tree/render_tree_002.json').read_text())['bbox']['x'], 4)

            environment = root / 'environment.json'
            with patch.object(SWEEP, 'run', side_effect=fake_run) as run:
                SWEEP.export_wasm_target(root, root / 'input.hwp', root / 'pkg', 'rhwp', root / 'out', environment)
            for call in run.call_args_list:
                command = call.args[0]
                self.assertEqual(command[command.index('--font-environment') + 1], str(environment))

            def incomplete_run(command, **kwargs):
                result = fake_run(command, **kwargs)
                if command[0] == 'node':
                    (Path(command[-1]) / 'render_tree/render_tree_002.json').unlink()
                return result

            with patch.object(SWEEP, 'run', side_effect=incomplete_run):
                with self.assertRaises(SystemExit):
                    SWEEP.export_wasm_target(root, root / 'input.hwp', root / 'pkg', 'rhwp', root / 'out')
            self.assertFalse((root / 'out/wasm-export-complete.json').exists())


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
    def test_interior_frames_are_not_physical_page_boundaries(self) -> None:
        rhwp = Image.new("RGB", (794, 1123), "white")
        pdf = Image.new("RGB", (794, 1122), "white")

        self.assertTrue(
            SWEEP.frames_are_interior_decorations(
                rhwp,
                (26, 75, 767, 1096),
                pdf,
                (26, 75, 767, 1095),
            )
        )
        self.assertFalse(
            SWEEP.frames_are_interior_decorations(
                rhwp,
                (26, 75, 767, 1112),
                pdf,
                (26, 75, 767, 1111),
            )
        )

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


class RhwpBinaryFreshnessTests(unittest.TestCase):
    def test_rejects_stale_implicit_debug_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            binary = root / "target" / "pr-review" / "debug" / "rhwp"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"rhwp")
            os.utime(binary, (100.0, 100.0))
            with patch.object(SWEEP, "git_head_commit_timestamp", return_value=101.0):
                with self.assertRaisesRegex(SystemExit, "현재 HEAD보다 오래되었습니다"):
                    SWEEP.ensure_default_rhwp_binary_is_current(root, SWEEP.DEFAULT_RHWP_BIN)

    def test_allows_explicit_binary_and_current_default_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            binary = root / "target" / "pr-review" / "debug" / "rhwp"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"rhwp")
            os.utime(binary, (102.0, 102.0))
            with patch.object(SWEEP, "git_head_commit_timestamp", return_value=101.0):
                SWEEP.ensure_default_rhwp_binary_is_current(root, SWEEP.DEFAULT_RHWP_BIN)
                SWEEP.ensure_default_rhwp_binary_is_current(root, "target/debug/rhwp")


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
