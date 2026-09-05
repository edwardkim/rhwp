from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = {
    "ci": ROOT / ".github/workflows/ci.yml",
    "codeql": ROOT / ".github/workflows/codeql.yml",
    "render-diff": ROOT / ".github/workflows/render-diff.yml",
}
WORKER_PREFLIGHTS = {
    "proptest": (
        ROOT / ".github/workflows/proptest-roundtrip.yml",
        "\n\n      # 기준선 병합을 fast-pass bridge로",
        "fast_pass",
        "true",
        "false",
    ),
}
RESOLUTION_CHECK = ROOT / "scripts/verify_review_only_merge_resolution.py"


class ReviewOnlyFastPassWorkflowTests(unittest.TestCase):
    def test_all_preflights_share_reference_pdf_directories(self) -> None:
        expected = "const pdfPrefixes = ['pdf/'];"
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn(expected, workflow)
                self.assertIn("function isPdfReferencePath(filename)", workflow)
                self.assertIn("function isSampleReferencePath(filename)", workflow)
                self.assertIn("file.status === 'added' || file.status === 'modified'", workflow)
                self.assertIn(
                    "pdfPrefixes.some((prefix) => filename.startsWith(prefix))",
                    workflow,
                )

    def test_render_contract_docs_are_not_review_only(self) -> None:
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn("const renderContractPaths = new Set([", workflow)
                self.assertIn(
                    "'mydocs/tech/canvaskit-parity-implementation.md'",
                    workflow,
                )
                self.assertIn("'mydocs/tech/text-ir-v2.md'", workflow)
                self.assertIn("!renderContractPaths.has(filename)", workflow)

    def test_sample_document_files_are_not_review_only_fast_pass_references(self) -> None:
        workflow_paths = {
            **WORKFLOWS,
            **{name: spec[0] for name, spec in WORKER_PREFLIGHTS.items()},
        }
        for name, workflow_path in workflow_paths.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                sample_function = workflow.split(
                    "function isSampleReferencePath(filename)", maxsplit=1
                )[1].split("function isPdfReferencePath(filename)", maxsplit=1)[0]
                self.assertIn("filename.endsWith('.pdf')", sample_function)
                self.assertIn("filename.endsWith('.png')", sample_function)
                self.assertNotIn("filename.endsWith('.hwp')", sample_function)
                self.assertNotIn("filename.endsWith('.hwpx')", sample_function)

    def test_font_fixture_generators_are_not_review_only_or_enforcement_surface(self) -> None:
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                execution = workflow.split(
                    "function isCiExecutionPath(filename)", maxsplit=1
                )[1].split("function latestRun", maxsplit=1)[0]
                self.assertNotIn("generate_exact_kerning_fixture.py", execution)
                self.assertNotIn("generate_exact_face_collection_fixture.py", execution)

        pull_request_trigger = WORKFLOWS["render-diff"].read_text(encoding="utf-8").split(
            "  workflow_dispatch:",
            maxsplit=1,
        )[0]
        self.assertIn(
            "      - 'scripts/generate_exact_kerning_fixture.py'",
            pull_request_trigger,
        )

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

    def test_current_base_update_merge_allows_only_mydocs_conflict_resolution(self) -> None:
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn("isCurrentBaseUpdateMerge", workflow)
                self.assertIn("pending-base-merge-tree", workflow)
                self.assertIn("multiple-current-base-update-merges", workflow)
                self.assertIn("git merge-tree --write-tree", workflow)
                self.assertIn("current-base-merge-tree-mismatch", workflow)
                self.assertIn("verify_review_only_merge_resolution.py", workflow)
                self.assertIn(
                    "${CURRENT_BASE_SHA}:scripts/verify_review_only_merge_resolution.py",
                    workflow,
                )
                self.assertIn("current-base-merge-resolution-check-unavailable", workflow)
                self.assertIn("current-base-merge-resolution-not-mydocs", workflow)
                self.assertIn("current-base-update-merge-resolution-mydocs-only-green", workflow)
                self.assertIn("current-base-update-merge-tree-green", workflow)
                self.assertIn(
                    "ref: refs/pull/${{ github.event.pull_request.number }}/head",
                    workflow,
                )
                self.assertIn("lfs: false", workflow)
                self.assertIn("persist-credentials: false", workflow)
                self.assertNotIn(
                    "reviewOnlyCandidates.length > 0\n                  && isCurrentBaseUpdateMerge",
                    workflow,
                )

    def test_current_base_merge_reuses_a_green_source_parent_except_ci_changes(self) -> None:
        result_functions = {
            "ci": "buildResult",
            "codeql": "codeqlResult",
            "render-diff": "renderDiffResult",
        }
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                direct_bridge_index = workflow.index("const latestCommitSha = commits.at(-1).sha;")
                candidate_scan_index = workflow.index("const reviewOnlyCandidates = [];")
                self.assertLess(direct_bridge_index, candidate_scan_index)
                self.assertIn("function isCiExecutionPath(filename)", workflow)
                self.assertIn(
                    "current-base-source-ci-execution-change",
                    workflow,
                )
                self.assertIn(
                    f"await {result_functions[name]}(sourceParent.sha, pr",
                    workflow,
                )
                self.assertIn("direct-source-", workflow)

    def test_enforcement_surface_changes_force_current_head_full_validation(self) -> None:
        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn(
                    "filename === 'scripts/ci-impact-policy.cjs'",
                    workflow,
                )
                self.assertIn("const hasCiExecutionSurfaceChange", workflow)
                self.assertIn("isCiExecutionPath(file.filename)", workflow)
                self.assertIn(
                    "isCiExecutionPath(file.previous_filename || '')",
                    workflow,
                )
                guard = "if (hasCiExecutionSurfaceChange) {"
                self.assertLess(
                    workflow.index(guard),
                    workflow.index("prFiles.length > 0 && prFiles.every"),
                )
                self.assertLess(
                    workflow.index(guard),
                    workflow.index("github.rest.pulls.listCommits"),
                )

    def test_render_diff_allows_prior_base_only_for_verified_merge_paths(self) -> None:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        self.assertIn(
            "async function renderDiffResult(candidateSha, pr, allowPriorPrBase = false)",
            workflow,
        )
        self.assertIn("allowPriorPrBase\n                  ? step.name.startsWith(identityPrefix)", workflow)
        self.assertIn("await renderDiffResult(sourceParent.sha, pr, true)", workflow)
        self.assertIn(
            "await renderDiffResult(candidateSha, pr, Boolean(baseMergeBridge))",
            workflow,
        )

    def test_render_diff_trailing_bridge_reuses_prior_base_identity(self) -> None:
        base_sha = "b" * 40
        prior_base_sha = "a" * 40
        code_candidate = "c" * 40
        merge_sha = "m" * 40
        review_tail = "r" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "mydocs/orders/20260827.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "d" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": merge_sha,
                "parents": [{"sha": code_candidate}, {"sha": base_sha}],
                "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
            },
            {
                "sha": review_tail,
                "parents": [{"sha": merge_sha}],
                "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
            },
        ]
        runs, jobs = self._render_diff_green_fixture(
            code_candidate,
            identity_base_sha=prior_base_sha,
        )

        output = self._run_render_diff_preflight(
            files=files,
            commits=commits,
            runs=runs,
            jobs=jobs,
            base_sha=base_sha,
            head_sha=review_tail,
        )

        self.assertEqual(output["fast_pass"], "pending-base-merge-tree", output)
        self.assertEqual(output["candidate_sha"], code_candidate)
        self.assertEqual(output["base_merge_sha"], merge_sha)
        self.assertEqual(output["source_parent_sha"], code_candidate)

    def test_render_diff_prior_base_identity_stays_fail_closed_without_bridge(
        self,
    ) -> None:
        base_sha = "b" * 40
        code_candidate = "c" * 40
        review_tail = "r" * 40
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "d" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": review_tail,
                "parents": [{"sha": code_candidate}],
                "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
            },
        ]
        runs, jobs = self._render_diff_green_fixture(
            code_candidate,
            identity_base_sha="a" * 40,
        )

        output = self._run_render_diff_preflight(
            files=[
                {"filename": "src/renderer/layout.rs", "status": "modified"},
                {"filename": "mydocs/orders/20260827.md", "status": "modified"},
            ],
            commits=commits,
            runs=runs,
            jobs=jobs,
            base_sha=base_sha,
            head_sha=review_tail,
        )

        self.assertEqual(output["fast_pass"], "false")
        self.assertEqual(output["reason"], "canvas-visual-diff-identity-mismatch")

    def test_render_diff_trailing_bridge_prior_base_identity_fail_closed_matrix(
        self,
    ) -> None:
        base_sha = "b" * 40
        code_candidate = "c" * 40
        merge_sha = "m" * 40
        review_tail = "r" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "mydocs/orders/20260827.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "d" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": merge_sha,
                "parents": [{"sha": code_candidate}, {"sha": base_sha}],
                "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
            },
            {
                "sha": review_tail,
                "parents": [{"sha": merge_sha}],
                "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
            },
        ]
        default_runs, default_jobs = self._render_diff_green_fixture(
            code_candidate,
            identity_base_sha="a" * 40,
        )
        matrix = [
            (
                "wrong-repository",
                [{**default_runs[0], "head_repository": {"id": 999}}],
                default_jobs,
                "render-diff-workflow-pr-identity-mismatch",
            ),
            (
                "wrong-branch",
                [{**default_runs[0], "head_branch": "fix/other-branch"}],
                default_jobs,
                "render-diff-workflow-pr-identity-mismatch",
            ),
            (
                "before-pr",
                [{**default_runs[0], "created_at": "2026-08-19T23:59:59Z"}],
                default_jobs,
                "render-diff-workflow-pr-identity-mismatch",
            ),
            (
                "missing-run",
                [],
                default_jobs,
                "no-green-render-candidate",
            ),
            (
                "pending-run",
                [{**default_runs[0], "status": "in_progress", "conclusion": None}],
                default_jobs,
                "no-green-render-candidate",
            ),
            (
                "failed-run",
                [{**default_runs[0], "conclusion": "failure"}],
                default_jobs,
                "latest-render-diff-workflow-not-success:failure",
            ),
            (
                "failed-identity-step",
                default_runs,
                {
                    "101": [
                        {
                            **default_jobs["101"][0],
                            "steps": [
                                {
                                    **default_jobs["101"][0]["steps"][0],
                                    "conclusion": "failure",
                                }
                            ],
                        }
                    ]
                },
                "canvas-visual-diff-identity-mismatch",
            ),
            (
                "wrong-pr-identity-step",
                default_runs,
                {
                    "101": [
                        {
                            **default_jobs["101"][0],
                            "steps": [
                                {
                                    **default_jobs["101"][0]["steps"][0],
                                    "name": f"Render Diff identity PR #999 base {'a' * 40}",
                                }
                            ],
                        }
                    ]
                },
                "canvas-visual-diff-identity-mismatch",
            ),
        ]

        for name, runs, jobs, reason in matrix:
            with self.subTest(case=name):
                output = self._run_render_diff_preflight(
                    files=files,
                    commits=commits,
                    runs=runs,
                    jobs=jobs,
                    base_sha=base_sha,
                    head_sha=review_tail,
                )
                self.assertEqual(output["fast_pass"], "false")
                self.assertEqual(output["reason"], reason)

        second_merge = "n" * 40
        multiple_merge_output = self._run_render_diff_preflight(
            files=files,
            commits=[
                commits[0],
                commits[1],
                {
                    "sha": second_merge,
                    "parents": [{"sha": merge_sha}, {"sha": base_sha}],
                    "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
                },
                {
                    "sha": review_tail,
                    "parents": [{"sha": second_merge}],
                    "files": [{"filename": "mydocs/orders/20260827.md", "status": "modified"}],
                },
            ],
            runs=[],
            jobs={},
            base_sha=base_sha,
            head_sha=review_tail,
        )
        self.assertEqual(multiple_merge_output["fast_pass"], "false")
        self.assertTrue(
            multiple_merge_output["reason"].startswith("multiple-current-base-update-merges:")
        )

    def test_render_diff_skips_a_reused_candidate_before_trying_an_older_canvas_result(
        self,
    ) -> None:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        self.assertIn("state: 'skipped'", workflow)
        self.assertIn("canvas-visual-diff-skipped:${candidateSha}", workflow)
        self.assertLess(
            workflow.index("if (renderDiffJob.conclusion === 'skipped')"),
            workflow.index("if (renderDiffJob.conclusion !== 'success')"),
        )
        self.assertLess(
            workflow.index("if (result.state === 'failed')"),
            workflow.index("candidate not reusable yet: ${candidateSha}"),
        )

    def test_resolution_checker_accepts_only_mydocs_conflicts(self) -> None:
        self.assertEqual(
            self._run_resolution_check("mydocs/orders/20260807.md").returncode,
            0,
        )
        rejected = self._run_resolution_check("src/lib.rs")
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("current-base-merge-resolution-not-mydocs", rejected.stderr)
        render_contract = self._run_resolution_check(
            "mydocs/tech/text-ir-v2.md"
        )
        self.assertNotEqual(render_contract.returncode, 0)
        self.assertIn(
            "current-base-merge-resolution-not-mydocs",
            render_contract.stderr,
        )
        wrong_base = self._run_resolution_check(
            "mydocs/orders/20260807.md",
            expected_base_sha="0" * 40,
        )
        self.assertNotEqual(wrong_base.returncode, 0)
        self.assertIn("current-base-merge-resolution-invalid-merge", wrong_base.stderr)

    def test_worker_preflights_skip_added_review_pdf(self) -> None:
        files = [
            {"filename": "mydocs/pr/archives/pr_5772_review.md", "status": "added"},
            {"filename": "pdf/pr_5772_reference.pdf", "status": "added"},
        ]
        for name, (_, _, output_name, skip_value, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(name, files=files)
                self.assertEqual(output[output_name], skip_value)

    def test_worker_preflights_skip_modified_review_pdf(self) -> None:
        files = [
            {
                "filename": "pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf",
                "status": "modified",
            },
            {
                "filename": "pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf",
                "status": "modified",
            },
            {
                "filename": "pdf/2025 행정업무운영 편람(최종)-hwpx-2020.pdf",
                "status": "modified",
            },
            {
                "filename": "pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf",
                "status": "modified",
            },
        ]
        for name, (_, _, output_name, skip_value, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(name, files=files)
                self.assertEqual(output[output_name], skip_value)

    def test_worker_preflights_reuse_matching_fork_candidate_after_pdf_and_mydocs_tail(
        self,
    ) -> None:
        code_candidate = "c" * 40
        evidence_commit = "e" * 40
        review_commit = "r" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "pdf/pr_5772_reference.pdf", "status": "added"},
            {"filename": "mydocs/orders/20260820.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "b" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": evidence_commit,
                "parents": [{"sha": code_candidate}],
                "files": [{"filename": "pdf/pr_5772_reference.pdf", "status": "added"}],
            },
            {
                "sha": review_commit,
                "parents": [{"sha": evidence_commit}],
                "files": [{"filename": "mydocs/orders/20260820.md", "status": "modified"}],
            },
        ]
        runs = [
            {
                "event": "pull_request",
                "head_sha": code_candidate,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-08-20T12:00:00Z",
            }
        ]
        for name, (_, _, output_name, skip_value, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(
                    name, files=files, commits=commits, runs=runs
                )
                self.assertEqual(output[output_name], skip_value)

    def test_worker_preflights_require_a_verified_base_merge_bridge(self) -> None:
        base_sha = "b" * 40
        code_candidate = "c" * 40
        merge_sha = "m" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "mydocs/orders/20260827.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "d" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": merge_sha,
                "parents": [{"sha": code_candidate}, {"sha": base_sha}],
                "files": files,
            },
        ]
        runs = [
            {
                "event": "pull_request",
                "head_sha": code_candidate,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-08-20T12:00:00Z",
            }
        ]
        for name, (_, _, output_name, _, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(
                    name, files=files, commits=commits, runs=runs, base_sha=base_sha
                )
                self.assertEqual(output[output_name], "pending-base-merge-tree")
                self.assertEqual(output["base_merge_sha"], merge_sha)
                self.assertEqual(output["source_parent_sha"], code_candidate)

    def test_worker_base_merge_fast_passes_fail_closed_for_execution_changes(self) -> None:
        for name, (workflow_path, _, _, _, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn("isCurrentBaseUpdateMerge", workflow)
                self.assertIn("pending-base-merge-tree", workflow)
                self.assertIn("multiple-current-base-update-merges", workflow)
                self.assertIn("git merge-tree --write-tree", workflow)
                self.assertIn("verify_review_only_merge_resolution.py", workflow)
                self.assertIn("current-base-merge-resolution-not-mydocs", workflow)
                self.assertIn("persist-credentials: false", workflow)
                self.assertIn("ExecutionPath(file.filename)", workflow)

    def test_worker_preflights_reuse_a_green_review_only_candidate_head(
        self,
    ) -> None:
        code_candidate = "c" * 40
        green_review_head = "g" * 40
        trailing_review = "r" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "mydocs/pr/archives/pr_5832_review.md", "status": "added"},
            {"filename": "pdf/pr_5832_reference.pdf", "status": "added"},
            {"filename": "mydocs/orders/20260821.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "b" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": green_review_head,
                "parents": [{"sha": code_candidate}],
                "files": [
                    {"filename": "mydocs/pr/archives/pr_5832_review.md", "status": "added"},
                    {"filename": "pdf/pr_5832_reference.pdf", "status": "added"},
                ],
            },
            {
                "sha": trailing_review,
                "parents": [{"sha": green_review_head}],
                "files": [{"filename": "mydocs/orders/20260821.md", "status": "modified"}],
            },
        ]
        runs = [
            {
                "event": "pull_request",
                "head_sha": trailing_review,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "in_progress",
                "conclusion": None,
                "created_at": "2026-08-20T12:01:00Z",
            },
            {
                "event": "pull_request",
                "head_sha": green_review_head,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-08-20T12:00:00Z",
            },
        ]
        for name, (_, _, output_name, skip_value, _) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(
                    name, files=files, commits=commits, runs=runs
                )
                self.assertEqual(output[output_name], skip_value)

    def test_worker_preflights_do_not_bypass_a_failed_newer_candidate(
        self,
    ) -> None:
        code_candidate = "c" * 40
        failed_review_head = "f" * 40
        trailing_review = "r" * 40
        files = [
            {"filename": "src/renderer/layout.rs", "status": "modified"},
            {"filename": "mydocs/pr/archives/pr_5834_review.md", "status": "added"},
            {"filename": "mydocs/orders/20260821.md", "status": "modified"},
        ]
        commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "b" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": failed_review_head,
                "parents": [{"sha": code_candidate}],
                "files": [{"filename": "mydocs/pr/archives/pr_5834_review.md", "status": "added"}],
            },
            {
                "sha": trailing_review,
                "parents": [{"sha": failed_review_head}],
                "files": [{"filename": "mydocs/orders/20260821.md", "status": "modified"}],
            },
        ]
        runs = [
            {
                "event": "pull_request",
                "head_sha": failed_review_head,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "failure",
                "created_at": "2026-08-20T12:00:00Z",
            },
            {
                "event": "pull_request",
                "head_sha": code_candidate,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-08-20T11:59:00Z",
            },
        ]
        for name, (_, _, output_name, _, full_value) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name):
                output = self._run_worker_preflight(
                    name, files=files, commits=commits, runs=runs
                )
                self.assertEqual(output[output_name], full_value)

    def test_worker_preflights_reuse_modified_pdf_tail_and_reject_wrong_fork_candidate(self) -> None:
        code_candidate = "c" * 40
        modified_pdf = "m" * 40
        modified_pdf_commits = [
            {
                "sha": code_candidate,
                "parents": [{"sha": "b" * 40}],
                "files": [{"filename": "src/renderer/layout.rs", "status": "modified"}],
            },
            {
                "sha": modified_pdf,
                "parents": [{"sha": code_candidate}],
                "files": [{"filename": "pdf/existing_reference.pdf", "status": "modified"}],
            },
        ]
        matching_run = {
            "event": "pull_request",
            "head_sha": code_candidate,
            "head_branch": "fix/bughunt-batch-r3",
            "head_repository": {"id": 7},
            "status": "completed",
            "conclusion": "success",
            "created_at": "2026-08-20T12:00:00Z",
        }
        trailing_review = "r" * 40
        trusted_tail = [
            modified_pdf_commits[0],
            {
                "sha": trailing_review,
                "parents": [{"sha": code_candidate}],
                "files": [{"filename": "mydocs/orders/20260820.md", "status": "modified"}],
            },
        ]
        wrong_fork_run = {
            "event": "pull_request",
            "head_sha": code_candidate,
            "head_branch": "fix/bughunt-batch-r3",
            "head_repository": {"id": 999},
            "status": "completed",
            "conclusion": "success",
            "created_at": "2026-08-20T12:00:00Z",
        }
        for name, (_, _, output_name, _, full_value) in WORKER_PREFLIGHTS.items():
            with self.subTest(workflow=name, case="modified-pdf"):
                output = self._run_worker_preflight(
                    name,
                    files=[
                        {"filename": "src/renderer/layout.rs", "status": "modified"},
                        {"filename": "pdf/existing_reference.pdf", "status": "modified"},
                    ],
                    commits=modified_pdf_commits,
                    runs=[matching_run],
                )
                self.assertEqual(output[output_name], WORKER_PREFLIGHTS[name][3])
            with self.subTest(workflow=name, case="wrong-fork"):
                output = self._run_worker_preflight(
                    name,
                    files=[
                        {"filename": "src/renderer/layout.rs", "status": "modified"},
                        {"filename": "mydocs/orders/20260820.md", "status": "modified"},
                    ],
                    commits=trusted_tail,
                    runs=[wrong_fork_run],
                )
                self.assertEqual(output[output_name], full_value)

    def _run_worker_preflight(
        self,
        name: str,
        *,
        files: list[dict[str, object]],
        commits: list[dict[str, object]] | None = None,
        runs: list[dict[str, object]] | None = None,
        base_sha: str = "b" * 40,
    ) -> dict[str, str]:
        workflow_path, end_marker, _, _, _ = WORKER_PREFLIGHTS[name]
        workflow = workflow_path.read_text(encoding="utf-8")
        script = workflow.split("script: |\n", maxsplit=1)[1].split(end_marker, maxsplit=1)[0]
        script = "\n".join(
            line.removeprefix("            ") for line in script.splitlines()
        )
        fixture = {
            "files": files,
            "commits": commits or [],
            "runs": runs or [],
        }
        harness = textwrap.dedent(
            """
            const fixture = %(fixture)s;
            const outputs = {};
            const listFiles = Symbol('pulls.listFiles');
            const listCommits = Symbol('pulls.listCommits');
            const listWorkflowRuns = Symbol('actions.listWorkflowRuns');
            const commits = new Map(fixture.commits.map((commit) => [commit.sha, commit]));
            const github = {
              rest: {
                pulls: { listFiles, listCommits },
                repos: {
                  getCommit: async ({ ref }) => ({ data: commits.get(ref) }),
                },
                actions: { listWorkflowRuns },
              },
              paginate: async (endpoint) => {
                if (endpoint === listFiles) return fixture.files;
                if (endpoint === listCommits) return fixture.commits;
                if (endpoint === listWorkflowRuns) return fixture.runs;
                throw new Error('unexpected paginate endpoint');
              },
            };
            const context = {
              eventName: 'pull_request',
              repo: { owner: 'edwardkim', repo: 'rhwp' },
              payload: {
                pull_request: {
                  number: 5772,
                  created_at: '2026-08-20T00:00:00Z',
                  base: { ref: 'devel', sha: %(base_sha)s },
                  head: { ref: 'fix/bughunt-batch-r3', repo: { id: 7 } },
                },
              },
            };
            const core = {
              setOutput: (key, value) => { outputs[key] = String(value); },
              info: () => {},
            };
            (async () => {
            %(script)s
            })().then(
              () => process.stdout.write(JSON.stringify(outputs)),
              (error) => { process.stderr.write(String(error.stack || error)); process.exitCode = 1; },
            );
            """
        ) % {
            "fixture": json.dumps(fixture),
            "script": textwrap.indent(script, "  "),
            "base_sha": json.dumps(base_sha),
        }
        completed = subprocess.run(
            ["node"],
            input=harness,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        return json.loads(completed.stdout)

    @staticmethod
    def _render_diff_green_fixture(
        candidate_sha: str,
        *,
        identity_base_sha: str,
    ) -> tuple[list[dict[str, object]], dict[str, list[dict[str, object]]]]:
        runs = [
            {
                "id": 101,
                "path": ".github/workflows/render-diff.yml",
                "event": "pull_request",
                "head_sha": candidate_sha,
                "head_branch": "fix/bughunt-batch-r3",
                "head_repository": {"id": 7},
                "status": "completed",
                "conclusion": "success",
                "created_at": "2026-08-20T12:00:00Z",
                "completed_at": "2026-08-20T12:05:00Z",
            }
        ]
        jobs = {
            "101": [
                {
                    "name": "Canvas visual diff",
                    "status": "completed",
                    "conclusion": "success",
                    "completed_at": "2026-08-20T12:05:00Z",
                    "steps": [
                        {
                            "name": f"Render Diff identity PR #5772 base {identity_base_sha}",
                            "status": "completed",
                            "conclusion": "success",
                        }
                    ],
                }
            ]
        }
        return runs, jobs

    def _run_render_diff_preflight(
        self,
        *,
        files: list[dict[str, object]],
        commits: list[dict[str, object]],
        runs: list[dict[str, object]],
        jobs: dict[str, list[dict[str, object]]],
        base_sha: str,
        head_sha: str,
    ) -> dict[str, str]:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        script = workflow.split("script: |\n", maxsplit=1)[1].split(
            "\n      # 기준선 병합을 fast-pass bridge로",
            maxsplit=1,
        )[0]
        script = "\n".join(
            line.removeprefix("            ") for line in script.splitlines()
        )
        fixture = {
            "files": files,
            "commits": commits,
            "runs": runs,
            "jobs": jobs,
        }
        harness = textwrap.dedent(
            """
            const fixture = %(fixture)s;
            const outputs = {};
            const listFiles = Symbol('pulls.listFiles');
            const listCommits = Symbol('pulls.listCommits');
            const listWorkflowRuns = Symbol('actions.listWorkflowRuns');
            const listJobsForWorkflowRun = Symbol('actions.listJobsForWorkflowRun');
            const listCommitStatusesForRef = Symbol('repos.listCommitStatusesForRef');
            const commits = new Map(fixture.commits.map((commit) => [commit.sha, commit]));
            const github = {
              rest: {
                pulls: { listFiles, listCommits },
                repos: {
                  getCommit: async ({ ref }) => ({ data: commits.get(ref) }),
                  listCommitStatusesForRef,
                },
                actions: {
                  listWorkflowRuns,
                  listJobsForWorkflowRun,
                  getWorkflowRun: async () => { throw new Error('unexpected controller lookup'); },
                },
              },
              paginate: async (endpoint, parameters) => {
                if (endpoint === listFiles) return fixture.files;
                if (endpoint === listCommits) return fixture.commits;
                if (endpoint === listWorkflowRuns) {
                  return fixture.runs.filter((run) => (
                    !parameters?.head_sha || run.head_sha === parameters.head_sha
                  ));
                }
                if (endpoint === listJobsForWorkflowRun) {
                  return fixture.jobs[String(parameters.run_id)] || [];
                }
                if (endpoint === listCommitStatusesForRef) return [];
                throw new Error('unexpected paginate endpoint');
              },
            };
            const context = {
              eventName: 'pull_request',
              repo: { owner: 'edwardkim', repo: 'rhwp' },
              payload: {
                pull_request: {
                  number: 5772,
                  created_at: '2026-08-20T00:00:00Z',
                  base: { ref: 'devel', sha: %(base_sha)s },
                  head: { ref: 'fix/bughunt-batch-r3', sha: %(head_sha)s, repo: { id: 7 } },
                },
              },
            };
            const core = {
              setOutput: (key, value) => { outputs[key] = String(value); },
              info: () => {},
              warning: () => {},
            };
            (async () => {
            %(script)s
            })().then(
              () => process.stdout.write(JSON.stringify(outputs)),
              (error) => { process.stderr.write(String(error.stack || error)); process.exitCode = 1; },
            );
            """
        ) % {
            "fixture": json.dumps(fixture),
            "script": textwrap.indent(script, "  "),
            "base_sha": json.dumps(base_sha),
            "head_sha": json.dumps(head_sha),
        }
        completed = subprocess.run(
            ["node"],
            input=harness,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        return json.loads(completed.stdout)

    def _run_resolution_check(
        self,
        conflict_path: str,
        expected_base_sha: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            self._git(repository, "init", "--initial-branch=main")
            self._git(repository, "config", "user.email", "review@example.invalid")
            self._git(repository, "config", "user.name", "review")
            (repository / "README.md").write_text("root\n", encoding="utf-8")
            self._git(repository, "add", "README.md")
            self._git(repository, "commit", "-m", "root")

            self._git(repository, "switch", "-c", "feature")
            feature_file = repository / conflict_path
            feature_file.parent.mkdir(parents=True, exist_ok=True)
            feature_file.write_text("feature\n", encoding="utf-8")
            self._git(repository, "add", conflict_path)
            self._git(repository, "commit", "-m", "feature")

            self._git(repository, "switch", "main")
            base_file = repository / conflict_path
            base_file.parent.mkdir(parents=True, exist_ok=True)
            base_file.write_text("base\n", encoding="utf-8")
            self._git(repository, "add", conflict_path)
            self._git(repository, "commit", "-m", "base")
            base_sha = self._git_output(repository, "rev-parse", "HEAD")

            self._git(repository, "switch", "feature")
            merge = subprocess.run(
                ["git", "merge", "main"],
                cwd=repository,
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            self.assertNotEqual(merge.returncode, 0)
            feature_file.write_text("base\nfeature\n", encoding="utf-8")
            self._git(repository, "add", conflict_path)
            self._git(repository, "commit", "-m", "resolve mydocs conflict")

            return subprocess.run(
                [
                    sys.executable,
                    str(RESOLUTION_CHECK),
                    "--repository",
                    str(repository),
                    "--base-sha",
                    expected_base_sha or base_sha,
                    "HEAD",
                ],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

    @staticmethod
    def _git(repository: Path, *arguments: str) -> None:
        subprocess.run(
            ["git", *arguments],
            cwd=repository,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    @staticmethod
    def _git_output(repository: Path, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=repository,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        ).stdout.strip()

    def test_render_diff_keeps_its_existing_pr_identity_guard(self) -> None:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        self.assertIn("render-diff-workflow-pr-identity-mismatch", workflow)
        self.assertIn("renderDiffRun.head_branch !== pr.head.ref", workflow)
        self.assertIn("renderDiffRun.head_repository?.id !== pr.head.repo?.id", workflow)

    def test_render_diff_preflight_keeps_candidate_lookup_outside_commit_loop(self) -> None:
        workflow = WORKFLOWS["render-diff"].read_text(encoding="utf-8")
        self.assertIn(
            "codeCandidateSha = sha;\n"
            "              break;\n"
            "            }\n\n"
            "            if (reviewOnlyCandidates.length === 0)",
            workflow,
        )

        for name, workflow_path in WORKFLOWS.items():
            with self.subTest(workflow=name):
                script = workflow_path.read_text(encoding="utf-8").split(
                    "script: |\n", maxsplit=1
                )[1].split(
                    "\n      # 기준선 병합을 fast-pass bridge로", maxsplit=1
                )[0]
                script = "\n".join(
                    line.removeprefix("            ") for line in script.splitlines()
                )
                syntax = subprocess.run(
                    ["node", "--check"],
                    input=f"(async () => {{\n{script}\n}})();\n",
                    check=False,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                self.assertEqual(syntax.returncode, 0, syntax.stderr)


if __name__ == "__main__":
    unittest.main()
