from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
POLICY_WORKFLOW = ROOT / ".github/workflows/ci-impact-policy.yml"
CI_WORKFLOW = ROOT / ".github/workflows/ci.yml"
CODEQL_WORKFLOW = ROOT / ".github/workflows/codeql.yml"
RENDER_WORKFLOW = ROOT / ".github/workflows/render-diff.yml"


class CiImpactPolicyWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = POLICY_WORKFLOW.read_text(encoding="utf-8")
        cls.ci_workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        cls.codeql_workflow = CODEQL_WORKFLOW.read_text(encoding="utf-8")
        cls.render_workflow = RENDER_WORKFLOW.read_text(encoding="utf-8")

    def test_controller_uses_default_branch_registered_triggers(self) -> None:
        self.assertIn("  pull_request_target:\n", self.workflow)
        self.assertIn("  workflow_run:\n", self.workflow)
        self.assertIn("    workflows: [CI, CodeQL, Render Diff]", self.workflow)
        self.assertIn("merging this file to devel is not activation", self.workflow)

    def test_privileged_jobs_have_narrow_permissions(self) -> None:
        self.assertIn("permissions: {}", self.workflow)
        self.assertEqual(self.workflow.count("      statuses: write"), 1)
        self.assertEqual(self.workflow.count("      actions: read"), 1)
        self.assertNotIn("      actions: write", self.workflow)
        self.assertNotIn("      checks: write", self.workflow)
        self.assertNotIn("      contents: write", self.workflow)

    def test_controller_never_checks_out_or_executes_pull_request_head(self) -> None:
        self.assertEqual(
            self.workflow.count("ref: ${{ steps.resolve.outputs.base_sha }}"),
            1,
        )
        self.assertEqual(self.workflow.count("persist-credentials: false"), 1)
        self.assertNotIn("ref: ${{ github.event.pull_request.head.sha }}", self.workflow)
        self.assertNotIn("ref: ${{ github.event.workflow_run.head_sha }}", self.workflow)
        self.assertNotIn("refs/pull/", self.workflow)
        self.assertNotIn("download-artifact", self.workflow)
        self.assertNotIn("artifacts.download", self.workflow)
        self.assertNotIn(
            "checkout@",
            self.workflow.replace(
                "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
                "",
            ),
        )

    def test_controller_executes_only_sparse_checked_out_base_policy(self) -> None:
        self.assertEqual(
            self.workflow.count("node trusted-base/scripts/ci-impact-classifier.cjs"),
            1,
        )
        self.assertEqual(
            self.workflow.count("node trusted-base/scripts/ci-impact-policy.cjs"),
            1,
        )
        self.assertEqual(
            self.workflow.count("            scripts/ci-impact-classifier.cjs\n"),
            1,
        )
        self.assertEqual(
            self.workflow.count("            scripts/ci-impact-policy.cjs\n"),
            1,
        )

    def test_audit_reloads_all_same_head_workflow_metadata(self) -> None:
        self.assertIn("github.rest.actions.listWorkflowRunsForRepo", self.workflow)
        self.assertIn("branch: process.env.HEAD_BRANCH", self.workflow)
        self.assertNotIn("head_sha: process.env.HEAD_SHA", self.workflow)
        self.assertIn("selectLatestWorkflowRun(runs", self.workflow)
        self.assertIn("headBranch: process.env.HEAD_BRANCH", self.workflow)
        self.assertIn("headRepository: process.env.HEAD_REPOSITORY", self.workflow)
        self.assertIn("github.rest.actions.listJobsForWorkflowRun", self.workflow)
        self.assertIn("steps: (job.steps || []).map", self.workflow)
        self.assertIn("CI: '.github/workflows/ci.yml'", self.workflow)
        self.assertIn("CodeQL: '.github/workflows/codeql.yml'", self.workflow)
        self.assertIn("'Render Diff': '.github/workflows/render-diff.yml'", self.workflow)

    def test_job_collection_errors_are_serialized_as_incomplete_evidence(self) -> None:
        self.assertIn("let jobsCollected = false;", self.workflow)
        self.assertIn("jobsCollected = true;", self.workflow)
        self.assertIn("jobsCollected,", self.workflow)
        self.assertIn("audit will remain pending", self.workflow)

    def test_status_is_exact_head_bound_and_supports_pending_aggregation(self) -> None:
        self.assertEqual(self.workflow.count("context: 'CI Impact Policy'"), 1)
        self.assertEqual(
            sum(
                line.strip() == "sha: process.env.HEAD_SHA,"
                for line in self.workflow.splitlines()
            ),
            1,
        )
        self.assertIn("if (process.env.DECISION === 'blocked') state = 'failure'", self.workflow)
        self.assertIn("else if (noWorkflowExpected) state = 'success'", self.workflow)
        self.assertIn("new Set(['pending', 'success', 'failure'])", self.workflow)
        self.assertIn("candidate.head.sha === run.head_sha", self.workflow)
        self.assertIn("core.setOutput('trigger_head_sha', triggerHeadSha)", self.workflow)
        self.assertIn("input.currentHeadSha = process.env.CURRENT_HEAD_SHA", self.workflow)
        self.assertIn("github.rest.pulls.get({", self.workflow)
        self.assertIn("livePull.head.sha !== process.env.HEAD_SHA", self.workflow)
        self.assertIn("cancel-in-progress: true", self.workflow)

    def test_cancelled_controller_cannot_summarize_or_publish(self) -> None:
        guarded = "if: ${{ always() && !cancelled() && steps.resolve.outputs.active == 'true' }}"
        self.assertEqual(self.workflow.count(guarded), 5)
        for step_name in [
            "Classify with trusted base implementation",
            "Prepare trusted policy input",
            "Evaluate trusted policy and aggregate audit",
            "Summarize trusted policy",
            "Publish exact-head policy status",
        ]:
            marker = f"      - name: {step_name}\n        {guarded}\n"
            if step_name in {
                "Classify with trusted base implementation",
                "Evaluate trusted policy and aggregate audit",
            }:
                marker = f"      - name: {step_name}\n        id: "
                start = self.workflow.index(marker)
                block = self.workflow[start : start + 240]
                self.assertIn(guarded, block)
            else:
                self.assertIn(marker, self.workflow)

    def test_workers_consume_only_exact_trusted_review_reuse_status(self) -> None:
        for workflow in (self.ci_workflow, self.codeql_workflow, self.render_workflow):
            self.assertIn("status.context === 'CI Impact Policy'", workflow)
            self.assertIn("fields.get('v') === '5'", workflow)
            self.assertIn("fields.get('rfp') === '1'", workflow)
            self.assertIn("fields.get('b') === pr.base.sha", workflow)
            self.assertIn("run.name === 'CI Impact Policy Controller'", workflow)
            self.assertIn("run.event === 'pull_request_target'", workflow)
            self.assertIn("statuses: read", workflow)
            self.assertNotIn("skip_eligible", workflow)

    def test_controller_collects_full_candidate_and_review_lineage(self) -> None:
        self.assertIn("selectReviewOnlyCandidate", self.workflow)
        self.assertIn("pull-commit-list-incomplete", self.workflow)
        self.assertIn("pull-commit-head-mismatch", self.workflow)
        self.assertIn("allowPriorBase", self.workflow)
        self.assertIn("github-advanced-security", self.workflow)
        self.assertIn(".ci-impact-review-reuse.json", self.workflow)
        self.assertIn("reviewReuse.mergeTreeVerified", self.workflow)
        self.assertIn("REVIEW_FAST_PASS", self.workflow)
        self.assertIn("else if (process.env.REVIEW_FAST_PASS === 'true') state = 'success'", self.workflow)
        self.assertIn("process.env.AUDIT_CONCLUSION === 'pending'", self.workflow)

    def test_merge_bridge_fetches_objects_without_checking_out_pr_head(self) -> None:
        self.assertIn("name: Verify trusted current-base merge bridge", self.workflow)
        self.assertIn("git -C trusted-base fetch", self.workflow)
        self.assertIn("git -C trusted-base merge-tree --write-tree", self.workflow)
        self.assertIn("verify_review_only_merge_resolution.py", self.workflow)
        self.assertNotIn("ref: refs/pull/", self.workflow)


if __name__ == "__main__":
    unittest.main()
