from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = {
    "ci": ROOT / ".github/workflows/ci.yml",
    "codeql": ROOT / ".github/workflows/codeql.yml",
    "render-diff": ROOT / ".github/workflows/render-diff.yml",
}
RESOLUTION_CHECK = ROOT / "scripts/verify_review_only_merge_resolution.py"


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

    def test_resolution_checker_accepts_only_mydocs_conflicts(self) -> None:
        self.assertEqual(
            self._run_resolution_check("mydocs/orders/20260807.md").returncode,
            0,
        )
        rejected = self._run_resolution_check("src/lib.rs")
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("current-base-merge-resolution-not-mydocs", rejected.stderr)
        wrong_base = self._run_resolution_check(
            "mydocs/orders/20260807.md",
            expected_base_sha="0" * 40,
        )
        self.assertNotEqual(wrong_base.returncode, 0)
        self.assertIn("current-base-merge-resolution-invalid-merge", wrong_base.stderr)

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

        script = workflow.split("script: |\n", maxsplit=1)[1].split(
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
