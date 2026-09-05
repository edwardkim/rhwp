"""Trusted post-merge worker-reuse workflow contracts."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
REUSABLE = REPO_ROOT / ".github/workflows/trusted-postmerge-ci-reuse.yml"
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"
WORKFLOWS = {
    "ci": REPO_ROOT / ".github/workflows/ci.yml",
    "codeql": REPO_ROOT / ".github/workflows/codeql.yml",
}


class TrustedPostmergeReuseWorkflowTests(unittest.TestCase):
    def test_reusable_workflow_actions_are_pinned_to_full_commit_shas(self) -> None:
        workflow = REUSABLE.read_text(encoding="utf-8")
        pins = re.findall(
            r"^\s*uses:\s+[^@\s]+@([0-9a-f]+)\b", workflow, flags=re.MULTILINE
        )
        self.assertGreater(len(pins), 0)
        self.assertTrue(all(len(pin) == 40 for pin in pins))

    def test_reusable_verifier_is_read_only_and_fail_closed(self) -> None:
        workflow = REUSABLE.read_text(encoding="utf-8")
        self.assertIn("actions: read", workflow)
        self.assertIn("contents: read", workflow)
        self.assertIn("pull-requests: read", workflow)
        self.assertIn("Default to full verification", workflow)
        self.assertIn("Resolve trusted source base parent", workflow)
        self.assertIn("ref: ${{ steps.source-base.outputs.sha }}", workflow)
        self.assertIn("caller_event_name:", workflow)
        self.assertIn("caller_ref:", workflow)
        self.assertIn("caller_sha:", workflow)
        self.assertIn("inputs.caller_event_name == 'push'", workflow)
        self.assertIn("CALLER_SHA: ${{ inputs.caller_sha }}", workflow)
        self.assertIn("one squash parent or two merge parents", workflow)
        self.assertIn("exactly one same-repository merged PR", workflow)
        self.assertIn("trusted-base-verifier-unavailable", workflow)
        self.assertIn("event: 'pull_request'", workflow)
        self.assertIn("listPullRequestsAssociatedWithCommit", workflow)
        self.assertIn("compareCommits", workflow)
        self.assertIn("listWorkflowRuns", workflow)
        self.assertIn("listFiles", workflow)
        self.assertIn("listCommits", workflow)
        self.assertIn("classifyReviewOnlyCommit", workflow)
        self.assertIn("listWorkflowRunArtifacts", workflow)
        self.assertIn("listJobsForWorkflowRun", workflow)
        self.assertIn("fullLaneRunIds", workflow)
        self.assertIn('"ci.yml", "codeql.yml"', workflow)
        self.assertIn('`nextest-target-durations-${workflowRun.id}-${label}`', workflow)
        self.assertIn("never checks out or executes", workflow)
        self.assertIn("the merged PR head", workflow)
        self.assertIn("Capture PR merge-tree evidence", workflow)
        self.assertIn("Upload PR merge-tree evidence", workflow)
        self.assertIn("trusted-postmerge-merge-tree-v1-", workflow)
        self.assertIn("mergeTreeEvidenceByRunId", workflow)
        self.assertIn("parents[0] !== pullRequest.base.sha", workflow)
        self.assertIn("recordMergeTreeEvidence", workflow)
        self.assertIn("testedParents[1] === workflowRun.head_sha", workflow)
        self.assertIn("testedTreeSha === encodedTreeSha", workflow)

    def test_all_duplicate_postmerge_workflows_call_the_shared_verifier(self) -> None:
        expected_workflow_files = {
            "ci": "workflow_file: ci.yml",
            "codeql": "workflow_file: codeql.yml",
        }
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn("trusted_postmerge_reuse:", workflow)
                self.assertIn("./.github/workflows/trusted-postmerge-ci-reuse.yml", workflow)
                self.assertIn(expected_workflow_files[name], workflow)
                self.assertIn("needs.trusted_postmerge_reuse.outputs.reuse", workflow)
                self.assertIn(
                    "caller_event_name: ${{ github.event_name }}", workflow
                )
                self.assertIn("caller_ref: ${{ github.ref }}", workflow)
                self.assertIn("caller_sha: ${{ github.sha }}", workflow)

    def test_ci_requires_candidate_duration_artifacts_before_worker_reuse(self) -> None:
        ci = WORKFLOWS["ci"].read_text(encoding="utf-8")
        self.assertIn("require_duration_artifacts: true", ci)
        self.assertIn("postmerge_source_run_id", ci)
        self.assertIn("Download trusted PR Archive B duration measurement", ci)
        self.assertIn("Download trusted PR Archive C duration measurement", ci)

    def test_direct_review_only_reuse_requires_the_exact_skipped_worker(self) -> None:
        workflow = REUSABLE.read_text(encoding="utf-8")
        self.assertIn("reviewOnlyFastPassRunIds", workflow)
        self.assertIn('job.conclusion === "skipped"', workflow)

    def test_trusted_reuse_evaluator_contracts_are_invoked_by_ci(self) -> None:
        ci = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            "scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs", ci
        )
        self.assertIn(
            "scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs", ci
        )


if __name__ == "__main__":
    unittest.main()


# #6779: contract checks for trusted frontend evidence and separate timing output.
import pathlib as frontend_pathlib
import unittest as frontend_unittest


class FrontendOnlyPostmergeReuseWorkflowTests(frontend_unittest.TestCase):
    def test_frontend_evidence_uses_the_trusted_classifier_and_exact_run_jobs(self):
        root = frontend_pathlib.Path(__file__).resolve().parents[2]
        workflow = (root / '.github/workflows/trusted-postmerge-ci-reuse.yml').read_text()
        self.assertIn('scripts/ci-impact-classifier.cjs', workflow)
        self.assertIn("ref: ${{ steps.source-base.outputs.sha }}", workflow)
        self.assertIn("classifyChanges({ eventName: 'pull_request', files: pullFiles })", workflow)
        self.assertIn("run_id: finalHeadRun.id, filter: 'latest'", workflow)
        self.assertIn('frontendOnlyCiRunIsReusable(impact, jobs)', workflow)
        self.assertIn('frontendOnlyRunIds.push(String(finalHeadRun.id))', workflow)
        self.assertIn('artifact.expired !== true', workflow)

    def test_frontend_reuse_does_not_require_or_refresh_rust_timings(self):
        root = frontend_pathlib.Path(__file__).resolve().parents[2]
        workflow = (root / '.github/workflows/trusted-postmerge-ci-reuse.yml').read_text()
        ci = (root / '.github/workflows/ci.yml').read_text()
        self.assertIn("if (result.reuse && process.env.REQUIRE_DURATION_ARTIFACTS === 'true')", workflow)
        self.assertIn('if (result.refreshDurationData !== false)', workflow)
        self.assertIn("core.setOutput('refresh_duration_data'", workflow)
        self.assertIn("postmerge_refresh_duration_data: ${{ needs.trusted_postmerge_reuse.outputs.refresh_duration_data || 'false' }}", ci)
        self.assertIn("needs.preflight.outputs.postmerge_reuse != 'true'", ci)
        self.assertIn("needs.preflight.outputs.postmerge_refresh_duration_data == 'true'", ci)
