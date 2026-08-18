#!/usr/bin/env python3
"""M-fid fatten_text_layer 단위·픽스처 시험."""

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent

FATTEN_SPEC = importlib.util.spec_from_file_location(
    "fatten_text_layer", TOOL_DIR / "fatten_text_layer.py"
)
assert FATTEN_SPEC is not None and FATTEN_SPEC.loader is not None
fatten = importlib.util.module_from_spec(FATTEN_SPEC)
sys.modules[FATTEN_SPEC.name] = fatten
FATTEN_SPEC.loader.exec_module(fatten)

HARNESS_SPEC = importlib.util.spec_from_file_location(
    "fidelity_compare", TOOL_DIR / "fidelity_compare.py"
)
assert HARNESS_SPEC is not None and HARNESS_SPEC.loader is not None
harness = importlib.util.module_from_spec(HARNESS_SPEC)
sys.modules[HARNESS_SPEC.name] = harness
HARNESS_SPEC.loader.exec_module(harness)


class ClassifyTests(unittest.TestCase):
    def test_four_kinds(self) -> None:
        self.assertEqual(
            harness.classify_text_layer_delta(
                harness.normalized_characters("갑"),
                harness.normalized_characters(""),
            ),
            harness.TEXT_LAYER_LOSS,
        )
        self.assertEqual(
            harness.classify_text_layer_delta(
                harness.normalized_characters(""),
                harness.normalized_characters("을"),
            ),
            harness.TEXT_LAYER_EXCESS,
        )
        self.assertEqual(
            harness.classify_text_layer_delta(
                harness.normalized_characters("갑"),
                harness.normalized_characters("을"),
            ),
            harness.TEXT_LAYER_SUBSTITUTION,
        )
        missing, extra = harness.compare_text_layers("갑을", "을갑")
        self.assertEqual(
            harness.classify_text_layer_delta(missing, extra),
            harness.TEXT_LAYER_MATCH,
        )

    def test_nfc_and_space_are_neutral(self) -> None:
        missing, extra = harness.compare_text_layers("가 나", "가\n나\u00a0")
        self.assertEqual(harness.classify_text_layer_delta(missing, extra), "match")

    def test_write_text_report_header(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = harness.write_text_report(
                Path(directory),
                [(0, 2, 0, "U+AC00:\\uac00×2", "", "")],
            )
            text = path.read_text(encoding="utf-8")
        self.assertTrue(text.startswith(harness.TEXT_REPORT_HEADER))
        self.assertIn("1\t2\t0\t", text)

    def test_text_only_artifacts_do_not_include_png(self) -> None:
        names = harness.text_only_artifact_names(
            export_all_svg=True, layout_ledger=True
        )
        self.assertIn("text-report.tsv", names)
        self.assertIn("svg/export-svg-manifest.json", names)
        self.assertIn("layout-candidates.tsv", names)
        self.assertNotIn("cmp-p000.png", names)
        self.assertFalse(any(name.endswith(".png") for name in names))


class GenerateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.out = Path(cls.tmp.name)
        cls.summary = fatten.run(cls.out)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_minimum_corpus(self) -> None:
        self.assertGreaterEqual(self.summary["caseCount"], 200)
        self.assertGreaterEqual(self.summary["pathCount"], 20)
        self.assertGreaterEqual(self.summary["svgFixtureCount"], 3)
        kinds = self.summary["kindCounts"]
        for kind in ("loss", "excess", "substitution", "match"):
            self.assertGreater(kinds.get(kind, 0), 0, kind)

    def test_constraints(self) -> None:
        self.assertFalse(self.summary["constraints"]["visualSweepTouched"])
        self.assertFalse(self.summary["constraints"]["engineTouched"])
        self.assertFalse(self.summary["constraints"]["gymTouched"])

    def test_tables_exist(self) -> None:
        for name in ("loss.tsv", "excess.tsv", "substitution.tsv", "match.tsv"):
            path = self.out / "tables" / name
            self.assertTrue(path.is_file(), name)
            lines = path.read_text(encoding="utf-8").splitlines()
            self.assertGreater(len(lines), 2, name)
            self.assertTrue(lines[0].startswith("id\tkind\t"))

    def test_case_matches_live_classifier(self) -> None:
        cases_dir = self.out / "fixtures" / "text_layer" / "cases"
        files = list(cases_dir.glob("*.json"))
        self.assertEqual(len(files), self.summary["caseCount"])
        for path in files:
            case = json.loads(path.read_text(encoding="utf-8"))
            missing, extra = harness.compare_text_layers(
                case["referenceText"], case["svgText"]
            )
            self.assertEqual(
                harness.classify_text_layer_delta(missing, extra),
                case["classification"],
                case["id"],
            )
            self.assertTrue(case["candidateNotVerdict"])
            self.assertTrue(case["textOnly"])
            self.assertFalse(case["chromeRequired"])

    def test_owner_shift_threshold(self) -> None:
        hits = [
            json.loads(path.read_text(encoding="utf-8"))
            for path in (self.out / "fixtures" / "text_layer" / "cases").glob(
                "owner-*.json"
            )
        ]
        self.assertGreaterEqual(len(hits), 8)
        self.assertTrue(any(case["ownerShift"] or case["sequence"] for case in hits))

    def test_text_only_paths_parse(self) -> None:
        path_dir = self.out / "fixtures" / "text_only_paths"
        for path in path_dir.glob("path-*.json"):
            spec = json.loads(path.read_text(encoding="utf-8"))
            if spec["expectsError"]:
                stderr = io.StringIO()
                with self.assertRaises(SystemExit), redirect_stderr(stderr):
                    harness.parse_args(spec["argv"])
                self.assertIn(spec["errorNeedle"], stderr.getvalue())
            else:
                args = harness.parse_args(spec["argv"])
                self.assertTrue(args.text_only)
                self.assertEqual(bool(args.export_all_svg), spec["exportAllSvg"])
                self.assertEqual(bool(args.layout_ledger), spec["layoutLedger"])
                expected = harness.text_only_artifact_names(
                    export_all_svg=spec["exportAllSvg"],
                    layout_ledger=spec["layoutLedger"],
                )
                self.assertEqual(list(expected), spec["artifacts"])
                self.assertFalse(spec["chromeRequired"])
                self.assertFalse(spec["pypdfium2Required"])

    def test_svg_visible_fixture(self) -> None:
        svg = self.out / "fixtures" / "svg" / "svg-clip-body-cell.svg"
        visible, excluded = harness.svg_visible_text(svg)
        self.assertEqual(visible, "body-visiblepartial-cell")
        self.assertGreaterEqual(excluded, len("hidden-tophidden-cell"))

    def test_svg_glyph_fixture(self) -> None:
        svg = self.out / "fixtures" / "svg" / "svg-pua-and-fffd.svg"
        text = harness.svg_text(svg)
        risks = harness.svg_glyph_risks(text)
        self.assertGreater(sum(risks.values()), 0)
        self.assertIn("\uFFFD", risks)

    def test_working_doc_mentions_issue(self) -> None:
        text = (self.out / "WORKING.md").read_text(encoding="utf-8")
        self.assertIn("5467", text)
        self.assertIn("visual_sweep.py", text)
        self.assertIn("소실", text)


class CheckedInArtifactsTests(unittest.TestCase):
    def test_checked_in_catalog_is_current(self) -> None:
        index = TOOL_DIR / "fixtures" / "text_layer" / "index.json"
        if not index.is_file():
            self.skipTest("generator output is not checked in yet")
        data = json.loads(index.read_text(encoding="utf-8"))
        self.assertGreaterEqual(data["caseCount"], 200)
        for row in data["cases"]:
            case_path = TOOL_DIR / "fixtures" / "text_layer" / "cases" / f"{row['id']}.json"
            self.assertTrue(case_path.is_file(), row["id"])
            case = json.loads(case_path.read_text(encoding="utf-8"))
            missing, extra = harness.compare_text_layers(
                case["referenceText"], case["svgText"]
            )
            self.assertEqual(
                harness.classify_text_layer_delta(missing, extra),
                case["classification"],
                row["id"],
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
