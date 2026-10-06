"""Release promotion adapters preserve exact caller and read-only metadata contracts."""

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ADAPTER = ".github/workflows/release-operations-contracts.yml"


class ReleaseOperationsPromotionTests(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads((ROOT / "scripts/workflow_promotion_policy.json").read_text())["workflows"]
        self.adapter = (ROOT / ADAPTER).read_text()

    def test_metadata_adapter_is_read_only_and_dispatch_only(self):
        trigger = self.adapter.split("permissions:", 1)[0]
        self.assertEqual(re.findall(r"^  ([a-z_]+):", trigger, re.M), ["workflow_dispatch"])
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
        for name in ("issue-form-labels", "refresh-nextest-duration", "trusted-postmerge-ci-reuse"):
            entry = self.policy[f".github/workflows/{name}.yml"]
            self.assertEqual(entry["evidencePath"], ADAPTER)
            self.assertEqual(entry["executionMode"], "contracts-only")
            self.assertEqual(entry["requiredJobs"], ["Release operations contracts"])
            self.assertEqual(entry["allowedEvents"], ["workflow_dispatch"])
            self.assertEqual(entry["allowedActors"], ["edwardkim"])
            self.assertIn("permissions", entry["sensitiveSurfaces"])
        self.assertEqual(self.policy[ADAPTER]["executionMode"], "contracts-only")

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
