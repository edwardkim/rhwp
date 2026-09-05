"""#6689 canonical devel -> main workflow 승격 gate 배선 계약."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"
GITHUB_OPERATIONS = REPO_ROOT / "mydocs/manual/github_operations.md"
PUBLISH_GUIDE = REPO_ROOT / "mydocs/manual/publish_guide.md"


def job_block(workflow: str, job_id: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_id)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
    )
    return match.group(0) if match else ""


class WorkflowPromotionGateWiringTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WORKFLOW.read_text(encoding="utf-8")
        cls.gate = job_block(cls.ci, "workflow-promotion-preflight")
        cls.aggregate = job_block(cls.ci, "build-and-test")
        cls.operations = GITHUB_OPERATIONS.read_text(encoding="utf-8")
        cls.publish = PUBLISH_GUIDE.read_text(encoding="utf-8")

    def test_gate_runs_only_for_same_repository_devel_to_main_pr(self) -> None:
        self.assertTrue(self.gate, "workflow-promotion-preflight job이 없다")
        for condition in [
            "github.event_name == 'pull_request'",
            "github.base_ref == 'main'",
            "github.head_ref == 'devel'",
            "github.event.pull_request.head.repo.full_name == github.repository",
        ]:
            self.assertIn(condition, self.gate)
        trigger = self.ci.split("env:", maxsplit=1)[0]
        self.assertNotIn("  pull_request_target:", trigger)

    def test_gate_is_read_only_and_checks_out_exact_candidate(self) -> None:
        for permission in ["actions: read", "contents: read", "issues: read"]:
            self.assertIn(permission, self.gate)
        self.assertNotRegex(self.gate, r"(?m)^\s+(actions|contents|issues): write$")
        self.assertIn("ref: ${{ github.event.pull_request.head.sha }}", self.gate)
        self.assertIn("fetch-depth: 0", self.gate)
        self.assertIn("git merge-base --is-ancestor", self.gate)

    def test_gate_collects_then_verifies_without_dispatching(self) -> None:
        self.assertIn("scripts/workflow_promotion_evidence.py", self.gate)
        self.assertIn("scripts/workflow_promotion_preflight.py inventory", self.gate)
        self.assertIn("scripts/workflow_promotion_preflight.py verify", self.gate)
        self.assertIn("workflow-promotion-evidence-${{ github.run_id }}", self.gate)
        self.assertNotIn("workflow run", self.gate)
        self.assertNotIn("dispatches", self.gate)

    def test_build_and_test_requires_gate_only_for_canonical_promotion(self) -> None:
        self.assertIn("- workflow-promotion-preflight", self.aggregate)
        self.assertIn(
            "PROMOTION_RESULT: ${{ needs['workflow-promotion-preflight'].result }}",
            self.aggregate,
        )
        self.assertIn("CANONICAL_PROMOTION:", self.aggregate)
        self.assertIn('[[ "${PROMOTION_RESULT}" == "success" ]]', self.aggregate)
        self.assertIn('[[ "${PROMOTION_RESULT}" == "skipped" ]]', self.aggregate)
        self.assertLess(
            self.aggregate.index("CANONICAL_PROMOTION"),
            self.aggregate.index('if [[ "${FAST_PASS}" == "true" ]]'),
            "promotion 결과는 fast-pass 조기 종료보다 먼저 검사해야 한다",
        )

    def test_operations_and_release_guides_keep_the_same_boundary(self) -> None:
        for text in (self.operations, self.publish):
            self.assertIn("Workflow promotion preflight", text)
            self.assertIn("devel", text)
            self.assertIn("main", text)
            self.assertIn("exact", text)
            self.assertIn("#6634", text)
        self.assertIn("rhwp-workflow-promotion-waiver:v1", self.operations)
        self.assertIn("workflow-promotion-evidence-", self.operations)


if __name__ == "__main__":
    unittest.main()
