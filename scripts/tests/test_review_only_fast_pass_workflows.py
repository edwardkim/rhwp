from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = {
    "ci": ROOT / ".github/workflows/ci.yml",
    "codeql": ROOT / ".github/workflows/codeql.yml",
    "render-diff": ROOT / ".github/workflows/render-diff.yml",
}


class ReviewOnlyFastPassWorkflowTests(unittest.TestCase):
    def test_base_advance_does_not_invalidate_a_trailing_review_record(self) -> None:
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertNotIn("isCurrentBaseAncestor", workflow)
                self.assertNotIn("candidate does not include current base", workflow)
                self.assertNotIn("no-green-current-base", workflow)
                self.assertIn("for (const candidateSha of reviewOnlyCandidates)", workflow)

    def test_ci_and_codeql_bind_a_reused_result_to_the_same_pr_source(self) -> None:
        for name in ("ci", "codeql"):
            with self.subTest(workflow=name):
                workflow = WORKFLOWS[name].read_text(encoding="utf-8")
                self.assertIn("event: 'pull_request'", workflow)
                self.assertNotIn("branch: pr.head.ref", workflow)
                self.assertIn("run.head_sha === candidateSha", workflow)
                self.assertIn("run.head_branch === pr.head.ref", workflow)
                self.assertIn("run.head_repository?.id === pr.head.repo?.id", workflow)
                self.assertIn("listJobsForWorkflowRun", workflow)
                self.assertIn("runCreatedAt < pullCreatedAt", workflow)

    def test_current_base_update_merge_requires_an_automatic_merge_tree(self) -> None:
        for name in ("ci", "codeql"):
            with self.subTest(workflow=name):
                workflow = WORKFLOWS[name].read_text(encoding="utf-8")
                self.assertIn("isCurrentBaseUpdateMerge", workflow)
                self.assertIn("pending-base-merge-tree", workflow)
                self.assertIn("multiple-current-base-update-merges", workflow)
                self.assertIn("git merge-tree --write-tree", workflow)
                self.assertIn("current-base-merge-tree-mismatch", workflow)
                self.assertIn("current-base-update-merge-tree-green", workflow)
                self.assertIn(
                    "ref: refs/pull/${{ github.event.pull_request.number }}/head",
                    workflow,
                )
                self.assertIn("lfs: false", workflow)
                self.assertIn("persist-credentials: false", workflow)

    def test_render_diff_keeps_its_existing_pr_identity_guard(self) -> None:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        self.assertIn("render-diff-workflow-pr-identity-mismatch", workflow)
        self.assertIn("renderDiffRun.head_branch !== pr.head.ref", workflow)
        self.assertIn("renderDiffRun.head_repository?.id !== pr.head.repo?.id", workflow)


if __name__ == "__main__":
    unittest.main()
