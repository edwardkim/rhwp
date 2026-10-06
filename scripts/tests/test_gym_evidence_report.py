"""[#6669] Gym 증적 seal·신원·fail-closed 계약."""

from __future__ import annotations

from contextlib import redirect_stderr
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "evidence_report.py"
FIXTURE = (
    REPO_ROOT / "scripts" / "tests" / "fixtures" / "gym-evidence-report" / "complete"
)
SAMPLE_REPORT = REPO_ROOT / "gym" / "examples" / "evidence-report.html"


def load():
    spec = importlib.util.spec_from_file_location("gym_evidence_report", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_valid_evidence(root: Path):
    for source in FIXTURE.iterdir():
        if source.name != "evidence-manifest.json":
            shutil.copy2(source, root / source.name)


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
            self.assertNotIn(b"/opt/rhwp-fixture", first)
            self.assertEqual(manifest["identity"]["binaryName"], "rhwp")
            self.m.verify_seal(root)

    def test_tracked_public_fixture_reproduces_manifest_and_sample_html(self):
        fixture_names = {path.name for path in FIXTURE.iterdir() if path.is_file()}
        self.assertEqual(
            fixture_names,
            set(self.m.REQUIRED_INPUT_FILES) | {"evidence-manifest.json"},
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            shutil.copytree(FIXTURE, root, dirs_exist_ok=True)
            self.m.seal_evidence(root)
            self.assertEqual(
                (root / "evidence-manifest.json").read_bytes(),
                (FIXTURE / "evidence-manifest.json").read_bytes(),
            )
            _, manifest, output, report_hash = self.m.render_evidence(root, root / "report.html")
            self.assertEqual(output.read_bytes(), SAMPLE_REPORT.read_bytes())
            self.assertEqual(
                report_hash,
                hashlib.sha256(SAMPLE_REPORT.read_bytes()).hexdigest(),
            )
            self.assertEqual(manifest["resultStatus"], self.m.STATUS_PASS)
            self.assertEqual(manifest["scoreErrorAccounting"]["reportedCount"], 1)
            self.assertEqual(manifest["scoreErrorAccounting"]["intendedCount"], 1)
            self.assertEqual(manifest["scoreErrorAccounting"]["unexplainedCount"], 0)

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
            rewrite_json(root, "discrimination", lambda report: report.update(taskCount=3, discriminating=3))
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
            def mutate(report):
                report["results"][0].pop("error")
                report["scoreErrors"] = ["orphan error"]

            rewrite_json(root, "discrimination", mutate)
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
                report["discriminating"] = 1
                report["falsePass"] = ["pack-a/T01"]
                report["falsePassControls"] = ["pack-a/T01 (wrong-answer)"]
                report["results"][0]["discriminates"] = False
                report["results"][0].pop("error")
                report["scoreErrors"] = []

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


class HtmlReportContractTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_pass_report_is_deterministic_self_contained_and_path_safe(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            self.m.seal_evidence(root)
            _, _, first_path, first_hash = self.m.render_evidence(root, root / "first.html")
            _, _, second_path, second_hash = self.m.render_evidence(root, root / "second.html")
            first = first_path.read_bytes()
            second = second_path.read_bytes()
            text = first.decode("utf-8")

            self.assertEqual(first, second)
            self.assertEqual(first_hash, second_hash)
            self.assertIn("Gym evidence report", text)
            self.assertIn("trajectory.ok</dt><dd>true", text)
            self.assertIn("trajectory.trusted</dt><dd>true", text)
            self.assertIn("pack-a", text)
            self.assertIn("PASS", text)
            self.assertNotIn("<script", text.lower())
            self.assertNotIn("http://", text.lower())
            self.assertNotIn("https://", text.lower())
            self.assertNotIn("/opt/rhwp-fixture", text)
            self.assertNotIn("fixture-host", text)

    def test_valid_fail_writes_non_green_report_and_cli_returns_one(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)

            def mutate(report):
                report["ok"] = False
                report["discriminating"] = 1
                report["falsePass"] = ["pack-a/T01"]
                report["falsePassControls"] = ["pack-a/T01 (wrong-answer)"]
                report["results"][0]["discriminates"] = False
                report["results"][0].pop("error")
                report["scoreErrors"] = []

            rewrite_json(root, "discrimination", mutate)
            (root / "discrimination.exit").write_text("1\n", encoding="ascii")
            self.m.seal_evidence(root)
            output = root / "fail.html"
            run = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "--evidence-dir",
                    str(root),
                    "--out",
                    str(output),
                ],
                cwd=REPO_ROOT,
                check=False,
                capture_output=True,
            )
            summary = json.loads(run.stdout)
            text = output.read_text(encoding="utf-8")

            self.assertEqual(run.returncode, self.m.EXIT_RESULT_NOT_PASS)
            self.assertEqual(summary["resultStatus"], self.m.STATUS_FAIL)
            self.assertTrue(summary["generated"])
            self.assertIn("status-fail", text)
            self.assertIn("FAIL", text)
            self.assertIn("pack-a/T01 (wrong-answer)", text)

    def test_invalid_seal_returns_two_without_overwriting_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            self.m.seal_evidence(root)
            output = root / "existing.html"
            output.write_bytes(b"preserve-existing-output\n")
            (root / "platform.txt").write_text(
                "Linux changed-host 6.8.0 x86_64\n", encoding="utf-8", newline="\n"
            )
            run = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "--evidence-dir",
                    str(root),
                    "--out",
                    str(output),
                ],
                cwd=REPO_ROOT,
                check=False,
                capture_output=True,
            )
            error = json.loads(run.stderr)

            self.assertEqual(run.returncode, self.m.EXIT_INPUT_INVALID)
            self.assertEqual(error["kind"], self.m.ERROR_KIND)
            self.assertEqual(output.read_bytes(), b"preserve-existing-output\n")

    def test_free_text_is_escaped_redacted_and_bounded(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            secret = '<img src=x onerror=alert(1)> /home/edward/private/secret.hwp ' + "x" * 500

            def mutate(report):
                row = report["results"][0]
                row.update(ok=False, kind="failed-score", message=secret)
                report.update(
                    ok=False,
                    exit=1,
                    built=1,
                    failed=1,
                    failedScore=1,
                )

            rewrite_json(root, "positive", mutate)
            (root / "positive.exit").write_text("1\n", encoding="ascii")
            self.m.seal_evidence(root)
            _, _, path, _ = self.m.render_evidence(root, root / "redacted.html")
            text = path.read_text(encoding="utf-8")

            self.assertNotIn("<img src=x", text)
            self.assertNotIn("/home/edward", text)
            self.assertNotIn("secret.hwp", text)
            self.assertIn("&lt;img src=x onerror=alert(1)&gt;", text)
            self.assertIn("[absolute-path]", text)
            self.assertIn("source chars", text)
            self.assertIn(hashlib.sha256(secret.encode("utf-8")).hexdigest(), text)

    def test_untrusted_trajectory_is_visibly_incomplete_not_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)

            def mutate(report):
                report["trusted"] = False
                report["exceptions"] = [
                    {
                        "kind": "missing-reference",
                        "pack": "pack-a",
                        "task": "T02",
                        "path": "/private/corpus/hidden.hwpx",
                        "message": "missing /private/corpus/hidden.hwpx",
                    }
                ]
                report["exceptionCount"] = 1

            rewrite_json(root, "trajectory", mutate)
            self.m.seal_evidence(root)
            _, _, path, _ = self.m.render_evidence(root, root / "incomplete.html")
            text = path.read_text(encoding="utf-8")

            self.assertIn("status-incomplete", text)
            self.assertIn("trajectory.ok</dt><dd>true", text)
            self.assertIn("trajectory.trusted</dt><dd>false", text)
            self.assertNotIn("/private/corpus", text)
            self.assertNotIn("hidden.hwpx", text)
            self.assertIn("[absolute-path]", text)

    def test_cli_requires_exactly_one_operation_and_protects_seal(self):
        with redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as missing:
                self.m.parse_args(["--evidence-dir", "x"])
            with self.assertRaises(SystemExit) as both:
                self.m.parse_args(["--evidence-dir", "x", "--seal", "--out", "x.html"])
        self.assertEqual(missing.exception.code, 2)
        self.assertEqual(both.exception.code, 2)

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_valid_evidence(root)
            self.m.seal_evidence(root)
            original = (root / "evidence-manifest.json").read_bytes()
            with self.assertRaises(self.m.EvidenceError) as ctx:
                self.m.render_evidence(root, root / "evidence-manifest.json")
            self.assertIn("protected-output", {row["code"] for row in ctx.exception.errors})
            self.assertEqual((root / "evidence-manifest.json").read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
