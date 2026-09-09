"""[#6628] Gym 정답 권위·기준풀이 출처 원장 계약."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "authority_ledger.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_authority_ledger", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_json(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")


def live_task(task_id="A01"):
    return {
        "id": task_id,
        "tier": 1,
        "title": "live",
        "input": "samples/x.hwp",
        "instructions": "i",
        "submit": {"kind": "answer", "files": ["answer.json"]},
        "checks": [{
            "name": "pages",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": ["info", "{input}", "--json"],
            "path": "pageCount",
        }],
    }


def live_reference(task_id="A01"):
    return {
        "id": task_id,
        "steps": [{
            "answer": {
                "pages": {
                    "cmd": ["info", "{input}", "--json"],
                    "path": "pageCount",
                },
            },
        }],
    }


def write_case(root: Path, task=None, reference=None, *, pack="p1", evidence=True):
    task = live_task() if task is None else task
    task_id = task.get("id", "A01") if isinstance(task, dict) else "A01"
    reference = live_reference(task_id) if reference is None else reference
    task_path = root / "gym" / "packs" / pack / "tasks" / f"{task_id}.json"
    reference_path = root / "gym" / "packs" / pack / "reference" / f"{task_id}.json"
    write_json(task_path, task)
    write_json(reference_path, reference)
    sample = root / "samples" / "x.hwp"
    sample.parent.mkdir(parents=True, exist_ok=True)
    sample.write_bytes(b"fixture")
    if evidence:
        oracle = root / "evidence" / "oracle.json"
        write_json(oracle, {"source": "independent"})
    return root / "gym"


def codes(report):
    return [row["code"] for row in report["issues"]]


class AuthorityClassificationTests(unittest.TestCase):
    def setUp(self):
        self.mod = load()

    def test_live_check_dominates_supporting_fixture_and_contract_signals(self):
        task = live_task()
        task["checks"].extend([
            {"name": "changed", "op": "differs_from_input", "file": "out.hwp"},
            {"name": "exists", "op": "file_exists", "file": "out.hwp"},
        ])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gym = write_case(root, task=task)
            report = self.mod.build_ledger(gym, root)
        self.assertTrue(report["ok"], report["issues"])
        entry = report["entries"][0]
        self.assertEqual(entry["authority"], "self-live")
        self.assertEqual(
            entry["authoritySignals"],
            ["self-live", "independent-fixture", "contract-constant"],
        )
        self.assertIn("not an independent product oracle", entry["caveat"])

    def test_task_authored_value_is_contract_constant(self):
        task = live_task()
        task["checks"] = [{
            "name": "planned", "op": "json_value_eq",
            "file": "answer.json", "path": "planned", "value": 2,
        }]
        reference = {
            "id": task["id"],
            "steps": [{"answer": {"planned": {"const": 2}}}],
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = self.mod.build_ledger(write_case(root, task, reference), root)
        self.assertTrue(report["ok"], report["issues"])
        self.assertEqual(report["entries"][0]["authority"], "contract-constant")
        self.assertEqual(report["entries"][0]["baselineSource"], "contract-constant")

    def test_input_relation_is_independent_fixture_with_narrow_caveat(self):
        task = live_task()
        task["submit"] = {"kind": "artifact", "files": ["out.hwp"]}
        task["checks"] = [
            {"name": "changed", "op": "differs_from_input", "file": "out.hwp"},
            {"name": "exists", "op": "file_exists", "file": "out.hwp"},
        ]
        reference = {"id": task["id"], "steps": [{"run": ["edit", "{input}", "-o", "{sub:out.hwp}"]}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = self.mod.build_ledger(write_case(root, task, reference), root)
        self.assertTrue(report["ok"], report["issues"])
        entry = report["entries"][0]
        self.assertEqual(entry["authority"], "independent-fixture")
        self.assertEqual(entry["baselineSource"], "self-live")
        fixture = [row for row in entry["authorityEvidence"] if row["role"] == "public-input-fixture"]
        self.assertEqual([row["path"] for row in fixture], ["samples/x.hwp"])
        self.assertIn("cited public fixture relation", entry["caveat"])

    def test_external_oracle_requires_explicit_existing_repo_evidence(self):
        task = live_task()
        task["submit"] = {"kind": "artifact", "files": ["out.json"]}
        task["checks"] = [{"name": "exists", "op": "file_exists", "file": "out.json"}]
        task["authority"] = {
            "class": "external-oracle",
            "evidence": ["evidence/oracle.json"],
        }
        reference = {"id": task["id"], "steps": [{"run": ["info", "{input}", "--json"]}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = self.mod.build_ledger(write_case(root, task, reference), root)
        self.assertTrue(report["ok"], report["issues"])
        entry = report["entries"][0]
        self.assertEqual(entry["authority"], "external-oracle")
        self.assertEqual(entry["explicitAuthority"], True)
        self.assertIn("evidence/oracle.json", [row["path"] for row in entry["authorityEvidence"]])


class FailClosedTests(unittest.TestCase):
    def setUp(self):
        self.mod = load()

    def report_for(self, task, reference=None, *, evidence=True):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        gym = write_case(root, task, reference, evidence=evidence)
        return self.mod.build_ledger(gym, root)

    def test_multiple_primary_authorities_are_rejected(self):
        task = live_task()
        task["authority"] = ["self-live", "external-oracle"]
        report = self.report_for(task)
        self.assertFalse(report["ok"])
        self.assertIn("multiple-authority", codes(report))

    def test_external_oracle_without_evidence_is_rejected(self):
        task = live_task()
        task["checks"] = [{"name": "exists", "op": "file_exists", "file": "out"}]
        task["authority"] = {"class": "external-oracle", "evidence": []}
        report = self.report_for(task)
        self.assertFalse(report["ok"])
        self.assertIn("authority-evidence-required", codes(report))

    def test_missing_declared_evidence_path_is_rejected(self):
        task = live_task()
        task["checks"] = [{"name": "exists", "op": "file_exists", "file": "out"}]
        task["authority"] = {
            "class": "external-oracle",
            "evidence": ["evidence/missing.json"],
        }
        report = self.report_for(task, evidence=False)
        self.assertFalse(report["ok"])
        self.assertIn("evidence-path-missing", codes(report))

    def test_evidence_cannot_escape_repository(self):
        task = live_task()
        task["checks"] = [{"name": "exists", "op": "file_exists", "file": "out"}]
        task["authority"] = {"class": "external-oracle", "evidence": ["../outside.json"]}
        report = self.report_for(task)
        self.assertFalse(report["ok"])
        self.assertIn("evidence-outside-repo", codes(report))

    def test_live_check_cannot_be_promoted_to_external_oracle(self):
        task = live_task()
        task["authority"] = {
            "class": "external-oracle",
            "evidence": ["evidence/oracle.json"],
        }
        report = self.report_for(task)
        self.assertFalse(report["ok"])
        self.assertIn("authority-conflict", codes(report))
        self.assertEqual(report["entries"][0]["authority"], "self-live")

    def test_missing_input_fixture_fails_independent_classification(self):
        task = live_task()
        task["input"] = "samples/missing.hwp"
        task["checks"] = [{"name": "changed", "op": "differs_from_input", "file": "out"}]
        reference = {"id": task["id"], "steps": [{"run": ["edit", "{input}"]}]}
        report = self.report_for(task, reference)
        self.assertFalse(report["ok"])
        self.assertIn("evidence-path-missing", codes(report))

    def test_mixed_baseline_sources_are_rejected(self):
        task = live_task()
        reference = {
            "id": task["id"],
            "steps": [{
                "run": ["info", "{input}", "--json"],
                "answer": {"pages": {"const": 1}},
            }],
        }
        report = self.report_for(task, reference)
        self.assertFalse(report["ok"])
        self.assertIn("multiple-baseline-source", codes(report))
        self.assertIsNone(report["entries"][0]["baselineSource"])

    def test_missing_reference_prevents_silent_complete_ledger(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gym = write_case(root)
            (gym / "packs" / "p1" / "reference" / "A01.json").unlink()
            report = self.mod.build_ledger(gym, root)
        self.assertFalse(report["ok"])
        self.assertIn("reference-missing", codes(report))
        self.assertIn("entry-count-mismatch", codes(report))

    def test_reference_without_task_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gym = write_case(root)
            write_json(
                gym / "packs" / "p1" / "reference" / "ORPHAN.json",
                live_reference("ORPHAN"),
            )
            report = self.mod.build_ledger(gym, root)
        self.assertFalse(report["ok"])
        self.assertIn("reference-without-task", codes(report))


class RealRepositoryLedgerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.report = cls.mod.build_ledger(REPO_ROOT / "gym", REPO_ROOT)

    def test_every_real_task_appears_exactly_once(self):
        report = self.report
        self.assertTrue(report["ok"], report["issues"][:10])
        self.assertEqual(report["taskCount"], 1035)
        self.assertEqual(report["referenceCount"], 1035)
        self.assertEqual(report["entryCount"], 1035)
        keys = [entry["key"] for entry in report["entries"]]
        self.assertEqual(len(keys), len(set(keys)))

    def test_summary_is_recomputed_from_entries(self):
        self.assertEqual(
            self.report["summary"],
            self.mod.recompute_summary(self.report["entries"]),
        )
        self.assertEqual(sum(self.report["summary"]["byAuthority"].values()), 1035)
        self.assertEqual(sum(self.report["summary"]["byBaselineSource"].values()), 1035)

    def test_current_distribution_exposes_zero_external_oracles(self):
        self.assertEqual(
            self.report["summary"]["byAuthority"],
            {
                "self-live": 987,
                "contract-constant": 28,
                "independent-fixture": 20,
                "external-oracle": 0,
            },
        )
        self.assertEqual(
            self.report["summary"]["byBaselineSource"],
            {"self-live": 1031, "contract-constant": 4},
        )

    def test_output_is_deterministic_and_contains_no_absolute_repo_path(self):
        again = self.mod.build_ledger(REPO_ROOT / "gym", REPO_ROOT)
        first_json = json.dumps(self.report, ensure_ascii=False, sort_keys=True)
        second_json = json.dumps(again, ensure_ascii=False, sort_keys=True)
        self.assertEqual(first_json, second_json)
        self.assertNotIn(str(REPO_ROOT), first_json)

    def test_cli_json_uses_current_interpreter_and_exits_zero(self):
        process = subprocess.run(
            [sys.executable, str(TOOL), "--json"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        report = json.loads(process.stdout)
        self.assertEqual(report["kind"], "gymAuthorityLedger")
        self.assertEqual(report["entryCount"], 1035)


if __name__ == "__main__":
    unittest.main()
