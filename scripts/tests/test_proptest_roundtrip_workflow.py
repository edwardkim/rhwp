"""[#5388] M04-4 proptest-roundtrip.yml 계약 — 싼 왕복 property job.

이 파일명은 `test_*workflow*.py` 패턴이라 test_workflow_contract_wiring.py 가
ci.yml 배선을 강제한다(#4080). 배선을 잊으면 그 테스트가 실패한다.
"""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WF = REPO_ROOT / ".github/workflows/proptest-roundtrip.yml"
RUNNER = REPO_ROOT / "scripts/run-prop-roundtrip.mjs"
CI_WF = REPO_ROOT / ".github/workflows/ci.yml"
RUN_ARCHIVE = REPO_ROOT / ".github/workflows/run-nextest-archives.yml"


class ProptestRoundtripWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.wf = WF.read_text(encoding="utf-8")
        cls.ci = CI_WF.read_text(encoding="utf-8")

    def test_workflow_and_runner_exist(self) -> None:
        self.assertTrue(WF.is_file(), "proptest-roundtrip.yml 이 없다")
        self.assertTrue(RUNNER.is_file(), "run-prop-roundtrip.mjs 가 없다")

    def test_runs_on_pull_request(self) -> None:
        """PR 마다 preflight를 거치며 nightly 전용 퍼지가 아니다."""
        self.assertIn("pull_request:", self.wf)
        self.assertIn("branches: [main, devel]", self.wf)

    def test_mydocs_only_or_trusted_trailing_tail_skips_worker_without_hiding_check(self) -> None:
        self.assertIn("name: Proptest preflight", self.wf)
        self.assertIn("pr.base.ref !== 'devel'", self.wf)
        self.assertIn("file.filename.startsWith('mydocs/')", self.wf)
        self.assertIn("file.previous_filename.startsWith('mydocs/')", self.wf)
        self.assertIn("github.rest.pulls.listCommits", self.wf)
        self.assertIn("github.rest.repos.getCommit", self.wf)
        self.assertIn("workflow_id: 'proptest-roundtrip.yml'", self.wf)
        self.assertIn("github.rest.actions.listWorkflowRuns", self.wf)
        self.assertIn("trusted-mydocs-tail:", self.wf)
        self.assertIn("review-tail-candidate-run-unavailable", self.wf)
        self.assertIn("name: prop roundtrip", self.wf)
        self.assertIn("needs: preflight", self.wf)
        self.assertIn("needs.preflight.outputs.fast_pass != 'true'", self.wf)
        self.assertNotRegex(self.wf, r"(?m)^\\s{2,4}paths-ignore:")

    def test_is_not_a_long_fuzz(self) -> None:
        self.assertNotIn("cargo-fuzz", self.wf)
        self.assertNotIn("fuzz run", self.wf)
        self.assertNotIn("max_total_time", self.wf)
        self.assertNotIn("nightly", self.wf)
        self.assertIn("timeout-minutes: 20", self.wf)

    def test_uses_debug_cargo_test_not_release_archive(self) -> None:
        self.assertIn("run-prop-roundtrip.mjs --cargo-test", self.wf)
        self.assertNotIn("--release", self.wf)
        self.assertNotIn("release-test", self.wf)
        self.assertNotIn("cargo nextest archive", self.wf)
        self.assertNotIn("--archive-file", self.wf)

    def test_prepares_suites_before_running(self) -> None:
        prepare_at = self.wf.index("rust-test-suite-manifest.mjs --prepare")
        run_at = self.wf.index("run-prop-roundtrip.mjs --cargo-test")
        self.assertLess(prepare_at, run_at, "prepare 가 실행보다 먼저여야 한다")

    def test_does_not_add_a_fifth_nextest_shard(self) -> None:
        """정규 archive 집계는 shard 4개 합 = runnable. 5번째 worker 는 깨진다."""
        self.assertNotRegex(self.ci, r"(?m)^  proptest-roundtrip:")
        self.assertNotIn("prop_hwpx_roundtrip", RUN_ARCHIVE.read_text(encoding="utf-8"))
        self.assertIn("expected 4 shard count files", self.ci)

    def test_read_only_permissions(self) -> None:
        self.assertIn("actions: read", self.wf)
        self.assertIn("contents: read", self.wf)
        self.assertIn("pull-requests: read", self.wf)

    def test_m04f_catalog_and_generator_exist(self) -> None:
        root = REPO_ROOT
        self.assertTrue((root / "tools" / "proptest_roundtrip" / "gen_m04f_catalogs.py").is_file())
        self.assertTrue((root / "tests" / "fixtures" / "proptest_m04f" / "catalogs" / "skip_catalog.jsonl").is_file())
        self.assertTrue((root / "tests" / "cases" / "prop_m04f_catalog.rs").is_file())
        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("prop_m04f_catalog", runner)
        self.assertIn("prop_m04f_skip", runner)


if __name__ == "__main__":
    unittest.main()
