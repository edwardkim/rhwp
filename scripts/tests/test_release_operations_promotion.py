"""Release promotion adapters preserve exact caller and read-only metadata contracts."""

import json
import hashlib
import re
import unittest
from datetime import UTC, datetime
from pathlib import Path

from scripts import workflow_promotion_preflight as promotion

ROOT = Path(__file__).resolve().parents[2]
ADAPTER = ".github/workflows/release-operations-contracts.yml"
CALLER = ".github/workflows/ci.yml"
CONTRACT_JOB = "release-operations-contracts / Release operations contracts"


class ReleaseOperationsPromotionTests(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads((ROOT / "scripts/workflow_promotion_policy.json").read_text())["workflows"]
        self.adapter = (ROOT / ADAPTER).read_text()

    def test_metadata_adapter_is_read_only_and_reusable(self):
        trigger = self.adapter.split("permissions:", 1)[0]
        self.assertEqual(re.findall(r"^  ([a-z_]+):", trigger, re.M), ["workflow_call"])
        self.assertIn("  contents: read", self.adapter)
        self.assertNotRegex(self.adapter, r"(?m)^\s+[a-z-]+: write$")
        self.assertIn("ref: ${{ github.sha }}", self.adapter)
        self.assertNotIn("secrets.", self.adapter)
        for action in re.findall(r"uses: ([^\s#]+)", self.adapter):
            self.assertRegex(action, r"@[0-9a-f]{40}$")

    def test_shipped_metadata_scripts_and_failure_cases_are_executed(self):
        for name in (
            "issue-form-labels", "collect-postmerge-duration-data",
            "nextest-target-duration-policy", "trusted-postmerge-duration-evidence",
            "verify-trusted-postmerge-ci-reuse", "verify-trusted-postmerge-ci-reuse-squash",
        ):
            path = f"scripts/tests/{name}.test.mjs"
            self.assertIn(path, self.adapter)
            self.assertTrue((ROOT / path).is_file())
        for name in ("test_postmerge_duration_workflow", "test_trusted_postmerge_ci_reuse_workflow"):
            self.assertIn(f"scripts.tests.{name}", self.adapter)

    def test_each_metadata_workflow_requires_exact_adapter_evidence(self):
        for name in ("issue-form-labels", "refresh-nextest-duration", "trusted-postmerge-ci-reuse", "release-operations-contracts"):
            entry = self.policy[f".github/workflows/{name}.yml"]
            self.assertEqual(entry["evidencePath"], CALLER)
            self.assertEqual(entry["executionMode"], "contracts-only")
            self.assertEqual(entry["requiredJobs"], ["CI preflight", "Build & Test", CONTRACT_JOB])
            self.assertEqual(entry["allowedEvents"], ["workflow_dispatch"])
            self.assertEqual(entry["allowedActors"], ["edwardkim"])
            self.assertIn("permissions", entry["sensitiveSurfaces"])
        self.assertEqual(self.policy[ADAPTER]["executionMode"], "contracts-only")

    def test_registered_ci_dispatch_calls_same_commit_adapter_without_secrets(self):
        ci = (ROOT / CALLER).read_text()
        block = re.search(r"(?ms)^  release-operations-contracts:\n.*?(?=^  [\w-]+:\n|\Z)", ci)
        self.assertIsNotNone(block, "New adapter must be reachable through the registered CI workflow")
        caller = block.group()
        self.assertIn(f"uses: ./{ADAPTER}", caller)
        self.assertIn("needs: [preflight]", caller)
        self.assertIn("github.event_name == 'workflow_dispatch'", caller)
        self.assertIn("needs.preflight.result == 'success'", caller)
        self.assertIn("contents: read", caller)
        self.assertNotIn("secrets:", caller)
        self.assertNotRegex(caller, r"(?m)^\s+[a-z-]+: write$")

    def test_same_ci_run_requires_adapter_success_for_every_metadata_source(self):
        candidate = "b" * 40
        entries = []
        runs = []
        for name in ("issue-form-labels", "refresh-nextest-duration", "trusted-postmerge-ci-reuse", "release-operations-contracts"):
            path = f".github/workflows/{name}.yml"
            digest = hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
            entries.append({"path": path, "classification": "executable", "after": {"sha256": digest}, **self.policy[path]})
            runs.append({
                "id": 42, "url": "https://github.com/edwardkim/rhwp/actions/runs/42",
                "path": CALLER, "event": "workflow_dispatch", "headSha": candidate,
                "workflowSha256": digest, "executionMode": "contracts-only", "actor": "edwardkim",
                "paginationComplete": True, "status": "completed", "conclusion": "success",
                "jobs": [{"name": job, "status": "completed", "conclusion": "success"} for job in ("CI preflight", "Build & Test", CONTRACT_JOB)],
            })
        inventory = {"schemaVersion": 1, "baseSha": "a" * 40, "candidateSha": candidate, "repository": "edwardkim/rhwp", "entries": entries}
        inventory["inventorySha256"] = promotion._canonical_sha256(inventory)
        def verify():
            return promotion.verify_evidence(inventory, runs, [], now=datetime(2026, 10, 6, tzinfo=UTC), trusted_maintainers=frozenset({"edwardkim"}))
        self.assertTrue(verify()["ok"])
        for observed in ("skipped", "failure"):
            for run, entry in zip(runs, entries):
                run["jobs"][-1]["conclusion"] = observed
                verdict = verify()
                self.assertFalse(verdict["ok"])
                self.assertIn(f"job-not-green:{CONTRACT_JOB}:{observed}", verdict["errors"])
                self.assertEqual({item["path"] for item in verdict["acceptedRuns"]}, {item["path"] for item in entries if item is not entry})
                run["jobs"][-1]["conclusion"] = "success"

    def test_nextest_reusables_require_real_four_archive_caller_jobs(self):
        ci = (ROOT / ".github/workflows/ci.yml").read_text()
        for name, template in (
            ("build-nextest-archives", "build-test-archive-{a} / Build test archive ({a})"),
            ("run-nextest-archives", "test-archive-{a}-shard-1 / Default-feature tests (Archive {upper})"),
        ):
            entry = self.policy[f".github/workflows/{name}.yml"]
            self.assertEqual(entry["evidencePath"], ".github/workflows/ci.yml")
            self.assertEqual(entry["executionMode"], "direct")
            expected = ["CI preflight", "Build & Test"] + [
                template.format(a=a, upper=a.upper()) for a in "abcd"
            ]
            self.assertEqual(entry["requiredJobs"], expected)
            for a in "abcd":
                job = expected[2 + "abcd".index(a)].split(" / ")[0]
                block = re.search(rf"(?ms)^  {job}:\n.*?(?=^  [\w-]+:\n|\Z)", ci)
                self.assertIsNotNone(block)
                self.assertIn(f"uses: ./.github/workflows/{name}.yml", block.group())
                self.assertIn(f"archive_label: {a}", block.group())
