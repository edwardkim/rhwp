"""[#6669] Gym 증적 seal·신원·fail-closed 계약."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "evidence_report.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_evidence_report", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _audit():
    return {
        "kind": "gymAudit",
        "schemaVersion": "1.0",
        "ok": True,
        "packCount": 1,
        "taskCount": 1,
        "referenceCount": 1,
        "packs": [
            {
                "id": "pack-a",
                "issues": [],
                "structured": [],
                "taskCount": 1,
                "referenceCount": 1,
                "empty": False,
            }
        ],
        "okPacks": ["pack-a"],
        "emptyPacks": [],
        "taskIdCollisions": {},
        "issueCount": 0,
        "issues": [],
        "issueCountsByCode": {},
        "issueCountsByFamily": {},
        "toolErrors": [],
        "missingPacksRoot": False,
        "toolFailed": False,
        "exit": 0,
    }


def _oracle(mode):
    if mode == "structural":
        return {
            "kind": "gymOracleProbe",
            "schemaVersion": "1.0",
            "ok": True,
            "mode": "structural",
            "exports": ["probe"],
            "required": ["probe"],
            "issues": [],
            "issueCount": 0,
            "probes": {},
        }
    return {
        "kind": "gymOracleProbe",
        "schemaVersion": "1.0",
        "ok": True,
        "mode": "selftest",
        "checks": [{"name": "synthetic", "ok": True}],
        "failed": [],
        "issueCount": 0,
        "checkCount": 1,
    }


def _authority():
    entries = [
        {
            "key": "pack-a/T01",
            "authority": "self-live",
            "baselineSource": "self-live",
            "explicitAuthority": False,
        }
    ]
    return {
        "kind": "gymAuthorityLedger",
        "schemaVersion": "1.0",
        "ok": True,
        "root": "gym/packs",
        "authorityClasses": {},
        "classificationRule": "synthetic",
        "taskCount": 1,
        "referenceCount": 1,
        "entryCount": 1,
        "summary": {
            "byAuthority": {
                "self-live": 1,
                "contract-constant": 0,
                "independent-fixture": 0,
                "external-oracle": 0,
            },
            "byBaselineSource": {"self-live": 1, "contract-constant": 0},
            "explicitAuthorityCount": 0,
        },
        "entries": entries,
        "issueCount": 0,
        "issues": [],
        "exit": 0,
    }


def _positive(run_id, bin_path):
    return {
        "kind": "gymBaselineVerification",
        "schemaVersion": "1.0",
        "ok": True,
        "exit": 0,
        "binPath": bin_path,
        "agent": f"maintainer-{run_id}",
        "packs": ["pack-a"],
        "taskCount": 1,
        "built": 1,
        "failed": 0,
        "skipped": 0,
        "missingArtifact": 0,
        "failedScore": 0,
        "buildError": 0,
        "results": [
            {"ok": True, "kind": "ok", "pack": "pack-a", "task": "T01", "message": None}
        ],
    }


def _discrimination(bin_path):
    return {
        "kind": "gymDiscrimination",
        "schemaVersion": "1.0",
        "ok": True,
        "taskCount": 1,
        "controlCount": 1,
        "discriminating": 1,
        "falsePass": [],
        "falsePassControls": [],
        "results": [
            {
                "pack": "pack-a",
                "task": "T01",
                "control": "wrong-answer",
                "discriminates": True,
            }
        ],
        "loadErrors": [],
        "scoreErrors": [],
        "buildErrors": [],
        "skipped": [],
        "toolFailed": False,
        "toolErrors": [],
        "controlKinds": ["wrong-answer", "input-copy", "garbage"],
        "binPath": bin_path,
    }


def _trajectory(bin_path):
    return {
        "kind": "gymTrajectoryNecessity",
        "schemaVersion": "1.0",
        "ok": True,
        "taskCount": 1,
        "loadBearing": 1,
        "theater": [],
        "exceptions": [],
        "exceptionCount": 0,
        "skipped": [],
        "skipCount": 0,
        "results": [
            {
                "pack": "pack-a",
                "task": "T01",
                "loadBearing": True,
                "steps": 2,
                "removedStep": "run",
            }
        ],
        "trusted": True,
        "toolFailed": False,
        "toolErrors": [],
        "exit": 0,
        "missingBin": False,
        "binPath": bin_path,
    }


def write_valid_evidence(root: Path):
    run_id = "20260903-100000"
    bin_path = "/tmp/rhwp-gym-run/target/debug/rhwp"
    metadata = {
        "run-id.txt": run_id,
        "gym-runner-head.txt": "a" * 40,
        "gym-runner-tree.txt": "b" * 40,
        "product-source-head.txt": "c" * 40,
        "rhwp-version.txt": "rhwp 0.8.6",
        "rhwp-bin.sha256": f"{'d' * 64}  {bin_path}",
        "run-started.txt": "2026-09-03T10:00:00+09:00",
        "platform.txt": "Linux test-host 6.8.0 x86_64",
        "python-version.txt": "Python 3.12.3",
        "rust-version.txt": "rustc 1.89.0",
    }
    for name, value in metadata.items():
        (root / name).write_text(value + "\n", encoding="utf-8", newline="\n")

    reports = {
        "audit": _audit(),
        "oracle-structural": _oracle("structural"),
        "oracle-selftest": _oracle("selftest"),
        "authority-ledger": _authority(),
        "positive": _positive(run_id, bin_path),
        "discrimination": _discrimination(bin_path),
        "trajectory": _trajectory(bin_path),
    }
    for base, report in reports.items():
        (root / f"{base}.json").write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        (root / f"{base}.stderr").write_bytes(b"")
        (root / f"{base}.exit").write_text("0\n", encoding="ascii")
        (root / f"{base}.seconds").write_text("1\n", encoding="ascii")
    (root / "unit.txt").write_text("Ran 1 test\n\nOK\n", encoding="utf-8", newline="\n")
    (root / "unit.exit").write_text("0\n", encoding="ascii")
    (root / "unit.seconds").write_text("1\n", encoding="ascii")


def rewrite_json(root: Path, base: str, mutate):
    path = root / f"{base}.json"
    report = json.loads(path.read_text(encoding="utf-8"))
    mutate(report)
    path.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )


class SealContractTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_complete_bundle_seals_deterministically_without_raw_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            bundle, manifest = self.m.seal_evidence(root)
            first = (root / "evidence-manifest.json").read_bytes()
            bundle2, manifest2 = self.m.seal_evidence(root)
            second = (root / "evidence-manifest.json").read_bytes()

            self.assertEqual(first, second)
            self.assertEqual(manifest, manifest2)
            self.assertEqual(bundle["status"]["overall"], self.m.STATUS_PASS)
            self.assertEqual(bundle2["identityFingerprint"], bundle["identityFingerprint"])
            self.assertEqual(len(manifest["inputs"]), len(self.m.REQUIRED_INPUT_FILES))
            self.assertNotIn(b"/tmp/rhwp-gym-run", first)
            self.assertEqual(manifest["identity"]["binaryName"], "rhwp")
            self.m.verify_seal(root)

    def test_missing_input_rejects_without_overwriting_existing_manifest(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            sentinel = b"preserve-existing-manifest\n"
            (root / "evidence-manifest.json").write_bytes(sentinel)
            (root / "trajectory.json").unlink()
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.seal_evidence(root)
            self.assertIn("missing-input", {row["code"] for row in ctx.exception.errors})
            self.assertEqual((root / "evidence-manifest.json").read_bytes(), sentinel)

    def test_malformed_and_duplicate_json_keys_are_rejected(self):
        for payload in ("{not-json\n", '{"kind":"x","kind":"y"}\n'):
            with self.subTest(payload=payload):
                with tempfile.TemporaryDirectory() as tmp:
                    root = Path(tmp)
                    write_valid_evidence(root)
                    (root / "audit.json").write_text(payload, encoding="utf-8")
                    with self.assertRaises(self.m.EvidenceError) as ctx:
                        self.m.load_evidence(root)
                    self.assertIn("malformed-json", {row["code"] for row in ctx.exception.errors})

    def test_schema_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            rewrite_json(root, "positive", lambda report: report.update(schemaVersion="9.9"))
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.load_evidence(root)
            self.assertIn("schema-mismatch", {row["code"] for row in ctx.exception.errors})

    def test_pass_cardinality_mismatch_is_rejected_as_mixed_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            rewrite_json(root, "discrimination", lambda report: report.update(taskCount=2, discriminating=2))
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.load_evidence(root)
            self.assertIn("run-cardinality-mismatch", {row["code"] for row in ctx.exception.errors})

    def test_binary_path_and_run_agent_mismatch_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            rewrite_json(root, "positive", lambda report: report.update(agent="maintainer-other"))
            rewrite_json(root, "discrimination", lambda report: report.update(binPath="/tmp/other/rhwp"))
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.load_evidence(root)
            codes = {row["code"] for row in ctx.exception.errors}
            self.assertIn("run-identity-mismatch", codes)
            self.assertGreaterEqual(sum(row["code"] == "run-identity-mismatch" for row in ctx.exception.errors), 2)

    def test_positive_forged_counts_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            rewrite_json(root, "positive", lambda report: report.update(built=0))
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.load_evidence(root)
            self.assertTrue(any("built" in row["message"] for row in ctx.exception.errors))

    def test_seal_change_is_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            self.m.seal_evidence(root)
            (root / "platform.txt").write_text(
                "Linux changed-host 6.8.0 x86_64\n", encoding="utf-8", newline="\n"
            )
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.verify_seal(root)
            self.assertIn("manifest-mismatch", {row["code"] for row in ctx.exception.errors})


class StatusContractTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_explained_score_error_is_intended_rejection_and_can_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            error = "채점 예외: ValueError: synthetic"

            def mutate(report):
                report["results"][0]["error"] = error
                report["scoreErrors"] = [f"pack-a/T01 (wrong-answer): {error}"]

            rewrite_json(root, "discrimination", mutate)
            bundle = self.m.load_evidence(root)
            accounting = bundle["scoreErrorAccounting"]
            self.assertEqual(accounting["reportedCount"], 1)
            self.assertEqual(accounting["intendedCount"], 1)
            self.assertEqual(accounting["unexplainedCount"], 0)
            self.assertEqual(bundle["status"]["overall"], self.m.STATUS_PASS)

    def test_unexplained_score_error_is_incomplete_not_structural_rejection(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            rewrite_json(root, "discrimination", lambda report: report.update(scoreErrors=["orphan error"]))
            bundle, manifest = self.m.seal_evidence(root)
            self.assertEqual(bundle["status"]["roles"]["discrimination"]["status"], self.m.STATUS_INCOMPLETE)
            self.assertEqual(bundle["status"]["overall"], self.m.STATUS_INCOMPLETE)
            self.assertEqual(manifest["scoreErrorAccounting"]["unexplainedCount"], 1)

    def test_valid_false_pass_seals_as_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)

            def mutate(report):
                report["ok"] = False
                report["discriminating"] = 0
                report["falsePass"] = ["pack-a/T01"]
                report["falsePassControls"] = ["pack-a/T01 (wrong-answer)"]
                report["results"][0]["discriminates"] = False

            rewrite_json(root, "discrimination", mutate)
            (root / "discrimination.exit").write_text("1\n", encoding="ascii")
            bundle, manifest = self.m.seal_evidence(root)
            self.assertEqual(bundle["status"]["roles"]["discrimination"]["status"], self.m.STATUS_FAIL)
            self.assertEqual(bundle["status"]["overall"], self.m.STATUS_FAIL)
            self.assertEqual(manifest["resultStatus"], self.m.STATUS_FAIL)

    def test_ok_true_trusted_false_trajectory_is_incomplete(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)

            def mutate(report):
                report["trusted"] = False
                report["exceptions"] = [
                    {"kind": "missing-reference", "pack": "pack-a", "task": "T02", "path": "x", "head": "x"}
                ]
                report["exceptionCount"] = 1

            rewrite_json(root, "trajectory", mutate)
            bundle = self.m.load_evidence(root)
            self.assertEqual(bundle["status"]["roles"]["trajectory"]["status"], self.m.STATUS_INCOMPLETE)
            self.assertEqual(bundle["status"]["overall"], self.m.STATUS_INCOMPLETE)


class ProducerProvenanceTests(unittest.TestCase):
    def test_discrimination_run_audit_records_binary_path(self):
        m = load()
        discriminate = m._tool_module("discriminate")
        with tempfile.TemporaryDirectory() as tmp:
            with mock.patch.object(discriminate, "discriminate", return_value=discriminate.empty_report()):
                report = discriminate.run_audit(
                    "/tmp/product/rhwp",
                    gym_root=str(Path(tmp) / "gym"),
                    neg_root=str(Path(tmp) / "negative"),
                )
        self.assertEqual(report["binPath"], "/tmp/product/rhwp")
        self.assertIn("binPath", discriminate.OPTIONAL_REPORT_KEYS)


if __name__ == "__main__":
    unittest.main()
