"""[#5392] M06-4 adapter-diff.yml 계약 — 싼 전어댑터 상호 diff job.

이 파일명은 `test_*workflow*.py` 패턴이라 test_workflow_contract_wiring.py 가
ci.yml 배선을 강제한다(#4080). 배선을 잊으면 그 테스트가 실패한다.
"""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WF = REPO_ROOT / ".github/workflows/adapter-diff.yml"
RUNNER = REPO_ROOT / "scripts/run-adapter-diff.mjs"
HARNESS = REPO_ROOT / "tools/adapter_diff/harness.py"
CI_SCENE = REPO_ROOT / "tools/adapter_diff/fixtures/ci-scene.json"
CI_WF = REPO_ROOT / ".github/workflows/ci.yml"
RUN_ARCHIVE = REPO_ROOT / ".github/workflows/run-nextest-archives.yml"


class AdapterDiffWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.wf = WF.read_text(encoding="utf-8")
        cls.ci = CI_WF.read_text(encoding="utf-8")

    def test_workflow_and_runner_exist(self) -> None:
        self.assertTrue(WF.is_file(), "adapter-diff.yml 이 없다")
        self.assertTrue(RUNNER.is_file(), "run-adapter-diff.mjs 가 없다")
        self.assertTrue(HARNESS.is_file(), "adapter_diff harness 가 없다")
        self.assertTrue(CI_SCENE.is_file(), "CI 픽스처가 없다")

    def test_runs_on_pull_request(self) -> None:
        """PR마다 stable check를 남기되 문서 전용은 본 검증을 건너뛴다."""
        self.assertIn("pull_request:", self.wf)
        self.assertIn("branches: [main, devel]", self.wf)

    def test_review_only_or_trusted_trailing_tail_skips_adapter_job_but_keeps_a_stable_check(self) -> None:
        self.assertIn("adapter-impact:", self.wf)
        self.assertIn("name: adapter inter-diff preflight", self.wf)
        self.assertIn("uses: actions/github-script", self.wf)
        self.assertIn("function isAllowedReviewPath(file)", self.wf)
        self.assertIn("function isPdfReferencePath(filename)", self.wf)
        self.assertIn("function isSampleReferencePath(filename)", self.wf)
        self.assertIn("file.status !== 'added'", self.wf)
        self.assertIn("file.status === 'added' || file.status === 'modified'", self.wf)
        self.assertIn("const pdfPrefixes = ['pdf/', 'pdf-2020/', 'pdf-large/'];", self.wf)
        self.assertIn("filename.endsWith('.pdf')", self.wf)
        self.assertIn("github.rest.pulls.listCommits", self.wf)
        self.assertIn("github.rest.repos.getCommit", self.wf)
        self.assertIn("workflow_id: 'adapter-diff.yml'", self.wf)
        self.assertIn("github.rest.actions.listWorkflowRuns", self.wf)
        self.assertIn("run.head_branch === pr.head.ref", self.wf)
        self.assertIn("run.head_repository?.id === pr.head.repo?.id", self.wf)
        self.assertNotIn("run.pull_requests", self.wf)
        self.assertIn("core.setOutput('adapter_required', required ? 'true' : 'false')", self.wf)
        self.assertIn("skip-trusted-mydocs-tail:", self.wf)
        self.assertIn("review-tail-candidate-run-unavailable", self.wf)
        self.assertIn("needs: adapter-impact", self.wf)
        self.assertIn(
            "needs.adapter-impact.outputs.adapter_required == 'true'", self.wf
        )
        self.assertNotIn("paths-ignore:", self.wf)

    def test_review_only_preflight_script_parses(self) -> None:
        script = self.wf.split("script: |\n", maxsplit=1)[1].split(
            "\n\n      # 기준선 병합을 fast-pass bridge로", maxsplit=1
        )[0]
        script = "\n".join(
            line.removeprefix("            ") for line in script.splitlines()
        )
        completed = subprocess.run(
            ["node", "--check"],
            input=f"(async () => {{\n{script}\n}})();\n",
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)

    def test_is_cheap(self) -> None:
        worker = self.wf.split("  adapter-diff:\n", maxsplit=1)[1]
        self.assertNotIn("--features native-skia", worker)
        self.assertNotIn("samples/", worker)
        self.assertNotIn("cargo-fuzz", worker)
        self.assertIn("timeout-minutes: 25", worker)
        self.assertIn("tools/adapter_diff/harness.py --ci --strict", worker)

    def test_uses_debug_cargo_test_not_release_archive(self) -> None:
        self.assertIn("run-adapter-diff.mjs --cargo-test", self.wf)
        self.assertNotIn("--release", self.wf)
        self.assertNotIn("release-test", self.wf)
        self.assertNotIn("cargo nextest archive", self.wf)
        self.assertNotIn("--archive-file", self.wf)

    def test_prepares_suites_before_running(self) -> None:
        prepare_at = self.wf.index("rust-test-suite-manifest.mjs --prepare")
        run_at = self.wf.index("run-adapter-diff.mjs --cargo-test")
        self.assertLess(prepare_at, run_at, "prepare 가 실행보다 먼저여야 한다")

    def test_keeps_nextest_workers_at_four_balanced_archives(self) -> None:
        """A/B/C/D archive의 worker 네 개 합 = runnable. 다섯 번째 worker는 깨진다."""
        self.assertNotRegex(self.ci, r"(?m)^  adapter-diff:")
        self.assertNotIn("adapter_diff", RUN_ARCHIVE.read_text(encoding="utf-8"))
        self.assertNotIn('partition: "hash:1/2"', self.ci)
        self.assertNotIn('partition: "hash:2/2"', self.ci)
        self.assertEqual(4, self.ci.count('partition: "hash:1/1"'))
        for count_label in ("a-1", "b-1", "c-1", "d-1"):
            with self.subTest(count_label=count_label):
                self.assertIn(f"shard-count-{count_label}", self.ci)

    def test_read_only_permissions(self) -> None:
        self.assertIn("actions: read", self.wf)
        self.assertIn("contents: read", self.wf)
        self.assertIn("pull-requests: read", self.wf)

    def test_skips_missing_adapters_honestly(self) -> None:
        self.assertIn("png", RUNNER.read_text(encoding="utf-8"))
        self.assertIn("skia", RUNNER.read_text(encoding="utf-8"))
        self.assertIn("skip", RUNNER.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
