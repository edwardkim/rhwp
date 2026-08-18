#!/usr/bin/env python3
"""Generator + on-disk fixture tests for M-fill fatten."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import catalogs
import fatten_form_fill as fatten
import form_fill as ff


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


class GeneratorSmokeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        cls.bundle = fatten.run(cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_minimum_catalog_size(self) -> None:
        self.assertGreaterEqual(len(self.bundle.forms), 40)
        self.assertGreaterEqual(len(self.bundle.fills), 20)
        self.assertGreaterEqual(len(self.bundle.dry_runs), 20)
        self.assertGreaterEqual(len(self.bundle.verifies), 20)
        self.assertGreaterEqual(len(self.bundle.hongs), 20)
        self.assertGreaterEqual(len(self.bundle.batches), 12)
        self.assertGreaterEqual(len(self.bundle.paths), 20)

    def test_schemas_exist(self) -> None:
        for name in (
            "form_catalog.v1.json",
            "fill_case.v1.json",
            "batch_row.v1.json",
            "path_contract.v1.json",
            "honggildong_4781.v1.json",
        ):
            self.assertTrue((self.root / "schema" / name).is_file(), name)

    def test_field01_first_field_is_company(self) -> None:
        payload = load_json(self.root / "fixtures" / "forms" / "field-01.json")
        self.assertEqual(payload["firstFieldName"], "회사명")
        self.assertEqual(payload["fields"][0]["name"], "회사명")

    def test_honggildong_first_only_pass(self) -> None:
        payload = load_json(
            self.root / "fixtures" / "honggildong_4781" / "field-01-first-only.json"
        )
        self.assertEqual(payload["verdict"], "pass")
        self.assertFalse(payload["allowedClone"])
        self.assertEqual(payload["detect"]["cloneCount"], 0)
        self.assertEqual(payload["afterFirst"], "홍길동")
        self.assertEqual(payload["otherHongCount"], 0)

    def test_honggildong_clone_forbidden(self) -> None:
        payload = load_json(
            self.root / "fixtures" / "honggildong_4781" / "field-01-clone.json"
        )
        self.assertEqual(payload["verdict"], "clone_forbidden")
        self.assertGreater(payload["detect"]["cloneCount"], 0)

    def test_dry_run_has_no_output_key(self) -> None:
        payload = load_json(self.root / "fixtures" / "dry_run" / "field-01.json")
        self.assertTrue(payload["envelope"]["dryRun"])
        self.assertNotIn("output", payload["envelope"])
        self.assertFalse(payload["writesFile"])

    def test_verify_reports_identical(self) -> None:
        payload = load_json(self.root / "fixtures" / "verify" / "field-01.json")
        self.assertTrue(payload["envelope"]["verify"]["identical"])
        self.assertEqual(payload["exit"], 0)

    def test_occurrence_leaves_untouched(self) -> None:
        payload = load_json(self.root / "fixtures" / "occurrence" / "reg-80168.json")
        after = payload["afterTargeted"]["피규제집단명"]
        self.assertEqual(after[0], "가상협회 회원사")
        form = catalogs.form_by_id("reg-80168")
        self.assertEqual(after[1], form.values_of("피규제집단명")[1])

    def test_ambiguous_incomplete(self) -> None:
        payload = load_json(self.root / "fixtures" / "occurrence" / "reg-80168.json")
        self.assertTrue(payload["ambiguous"]["incomplete"])
        self.assertGreaterEqual(payload["ambiguous"]["envelope"]["ambiguous"][0]["total"], 3)

    def test_batch_dry_run_writes_nothing(self) -> None:
        payload = load_json(self.root / "fixtures" / "batch" / "field-01-dry-run.json")
        self.assertFalse(payload["writesFiles"])
        self.assertFalse(payload["anyOutputKey"])

    def test_paths_are_existing_cli(self) -> None:
        for path in (self.root / "fixtures" / "paths").glob("*.json"):
            payload = load_json(path)
            self.assertTrue(payload["existingCliOnly"])
            self.assertIn(payload["argv"][0], {"fields", "edit", "batch"})
            if payload["argv"][0] == "edit":
                self.assertIn(payload["argv"][1], {"fill-fields", "no-such-action"})
            if payload["argv"][0] == "batch":
                self.assertEqual(payload["argv"][1], "fill")

    def test_envelopes_recompute_from_live_functions(self) -> None:
        for rel in (
            "fixtures/fill/field-01-plain.json",
            "fixtures/dry_run/gian-1.json",
            "fixtures/verify/form-01.json",
        ):
            payload = load_json(self.root / rel)
            form = catalogs.form_by_id(payload["form"])
            again = ff.fill_envelope(
                form,
                payload["data"],
                dry_run=payload["envelope"]["dryRun"],
                verify=payload["envelope"].get("verify") is not None,
                output=payload["envelope"].get("output"),
            )
            self.assertEqual(again["filledCount"], payload["envelope"]["filledCount"])
            self.assertEqual(again["notFound"], payload["envelope"]["notFound"])
            self.assertEqual(again["dryRun"], payload["envelope"]["dryRun"])

    def test_summary_matches_counts(self) -> None:
        summary = load_json(self.root / "reports" / "fatten_summary.json")
        self.assertEqual(summary["formCount"], len(self.bundle.forms))
        self.assertEqual(summary["honggildongCount"], len(self.bundle.hongs))
        self.assertFalse(summary["inventedFillLogic"])
        self.assertFalse(summary["touchedGym"])
        self.assertTrue(summary["existingCliOnly"])


class OnDiskRepoFixtures(unittest.TestCase):
    def test_repo_fixtures_present_after_generate(self) -> None:
        index = HERE / "fixtures" / "index.json"
        if not index.is_file():
            self.skipTest("generator has not been run in-tree yet")
        payload = load_json(index)
        self.assertGreaterEqual(len(payload["written"]), 200)
        first = HERE / "fixtures" / "honggildong_4781" / "field-01-first-only.json"
        self.assertTrue(first.is_file())
        hong = load_json(first)
        self.assertEqual(hong["verdict"], "pass")


if __name__ == "__main__":
    unittest.main()
