"""Trusted post-merge worker-reuse workflow contracts."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
REUSABLE = REPO_ROOT / ".github/workflows/trusted-postmerge-ci-reuse.yml"
WORKFLOWS = {
    "ci": REPO_ROOT / ".github/workflows/ci.yml",
    "codeql": REPO_ROOT / ".github/workflows/codeql.yml",
    "adapter": REPO_ROOT / ".github/workflows/adapter-diff.yml",
    "proptest": REPO_ROOT / ".github/workflows/proptest-roundtrip.yml",
}


class TrustedPostmergeReuseWorkflowTests(unittest.TestCase):
    def test_reusable_verifier_is_read_only_and_fail_closed(self) -> None:
        workflow = REUSABLE.read_text(encoding="utf-8")
        self.assertIn("actions: read", workflow)
        self.assertIn("contents: read", workflow)
        self.assertIn("pull-requests: read", workflow)
        self.assertIn("Default to full verification", workflow)
        self.assertIn("Resolve trusted source base parent", workflow)
        self.assertIn("ref: ${{ steps.source-base.outputs.sha }}", workflow)
        self.assertIn("trusted-base-verifier-unavailable", workflow)
        self.assertIn("event: 'pull_request'", workflow)
        self.assertIn("listPullRequestsAssociatedWithCommit", workflow)
        self.assertIn("compareCommits", workflow)
        self.assertIn("listWorkflowRuns", workflow)
        self.assertIn("listFiles", workflow)
        self.assertIn("listWorkflowRunArtifacts", workflow)
        self.assertIn("never checks out or executes", workflow)
        self.assertIn("the merged PR head", workflow)

    def test_all_duplicate_postmerge_workflows_call_the_shared_verifier(self) -> None:
        expected_workflow_files = {
            "ci": "workflow_file: ci.yml",
            "codeql": "workflow_file: codeql.yml",
            "adapter": "workflow_file: adapter-diff.yml",
            "proptest": "workflow_file: proptest-roundtrip.yml",
        }
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn("trusted_postmerge_reuse:", workflow)
                self.assertIn("./.github/workflows/trusted-postmerge-ci-reuse.yml", workflow)
                self.assertIn(expected_workflow_files[name], workflow)
                self.assertIn("needs.trusted_postmerge_reuse.outputs.reuse", workflow)

    def test_ci_requires_candidate_duration_artifacts_before_worker_reuse(self) -> None:
        ci = WORKFLOWS["ci"].read_text(encoding="utf-8")
        self.assertIn("require_duration_artifacts: true", ci)
        self.assertIn("postmerge_source_run_id", ci)
        self.assertIn("Download trusted PR Archive B duration measurement", ci)
        self.assertIn("Download trusted PR Archive C duration measurement", ci)


if __name__ == "__main__":
    unittest.main()
