"""[#5392] M06-4 adapter-diff.yml 계약 — 싼 전어댑터 상호 diff job.

이 파일명은 `test_*workflow*.py` 패턴이라 test_workflow_contract_wiring.py 가
ci.yml 배선을 강제한다(#4080). 배선을 잊으면 그 테스트가 실패한다.
"""

from __future__ import annotations

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

    def test_mydocs_only_pr_skips_adapter_job_but_keeps_a_stable_check(self) -> None:
        self.assertIn("adapter-impact:", self.wf)
        self.assertIn("name: adapter inter-diff preflight", self.wf)
        self.assertIn("fetch-depth: 0", self.wf)
        self.assertIn('git diff --name-only "${BASE_SHA}" "${HEAD_SHA}"', self.wf)
        self.assertIn("grep -qv '^mydocs/'", self.wf)
        self.assertIn("adapter_required=false", self.wf)
        self.assertIn("reason=fail-closed-pr-diff-unavailable", self.wf)
        self.assertIn("needs: adapter-impact", self.wf)
        self.assertIn(
            "needs.adapter-impact.outputs.adapter_required == 'true'", self.wf
        )
        self.assertNotIn("paths-ignore:", self.wf)

    def test_is_cheap(self) -> None:
        self.assertNotIn("--features native-skia", self.wf)
        self.assertNotIn("samples/", self.wf)
        self.assertNotIn("cargo-fuzz", self.wf)
        self.assertIn("timeout-minutes: 25", self.wf)
        self.assertIn("tools/adapter_diff/harness.py --ci --strict", self.wf)

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

    def test_does_not_add_a_fifth_nextest_shard(self) -> None:
        """정규 archive 집계는 shard 4개 합 = runnable. 5번째 worker 는 깨진다."""
        self.assertNotRegex(self.ci, r"(?m)^  adapter-diff:")
        self.assertNotIn("adapter_diff", RUN_ARCHIVE.read_text(encoding="utf-8"))
        self.assertIn("expected 4 shard count files", self.ci)

    def test_read_only_permissions(self) -> None:
        self.assertIn("contents: read", self.wf)

    def test_skips_missing_adapters_honestly(self) -> None:
        self.assertIn("png", RUNNER.read_text(encoding="utf-8"))
        self.assertIn("skia", RUNNER.read_text(encoding="utf-8"))
        self.assertIn("skip", RUNNER.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
