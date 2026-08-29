"""[#5388] M04-4 proptest-roundtrip.yml 계약 — 싼 왕복 property job.

이 파일명은 `test_*workflow*.py` 패턴이라 test_workflow_contract_wiring.py 가
ci.yml 배선을 강제한다(#4080). 배선을 잊으면 그 테스트가 실패한다.
"""

from __future__ import annotations

import subprocess
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

    def test_review_only_or_trusted_trailing_tail_skips_worker_without_hiding_check(self) -> None:
        self.assertIn("name: Proptest preflight", self.wf)
        self.assertIn("pr.base.ref !== 'devel'", self.wf)
        self.assertIn("function isAllowedReviewPath(file)", self.wf)
        self.assertIn("filename.startsWith('mydocs/')", self.wf)
        self.assertIn("function isPdfReferencePath(filename)", self.wf)
        self.assertIn("function isSampleReferencePath(filename)", self.wf)
        self.assertIn("file.status !== 'added'", self.wf)
        self.assertIn("file.status === 'added' || file.status === 'modified'", self.wf)
        self.assertIn("const pdfPrefixes = ['pdf/', 'pdf-2020/', 'pdf-large/'];", self.wf)
        self.assertIn("filename.endsWith('.pdf')", self.wf)
        self.assertIn("github.rest.pulls.listCommits", self.wf)
        self.assertIn("github.rest.repos.getCommit", self.wf)
        self.assertIn("workflow_id: 'proptest-roundtrip.yml'", self.wf)
        self.assertIn("github.rest.actions.listWorkflowRuns", self.wf)
        self.assertIn("run.head_branch === pr.head.ref", self.wf)
        self.assertIn("run.head_repository?.id === pr.head.repo?.id", self.wf)
        self.assertNotIn("run.pull_requests", self.wf)
        self.assertIn("trusted-mydocs-tail:", self.wf)
        self.assertIn("review-tail-candidate-run-unavailable", self.wf)
        self.assertIn("name: prop roundtrip", self.wf)
        self.assertIn("needs: preflight", self.wf)
        self.assertIn("needs.preflight.outputs.fast_pass != 'true'", self.wf)
        self.assertNotRegex(self.wf, r"(?m)^\\s{2,4}paths-ignore:")

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

    def test_keeps_nextest_workers_at_four_balanced_archives(self) -> None:
        """A/B/C/D archive의 worker 네 개 합 = runnable. 다섯 번째 worker는 깨진다."""
        self.assertNotRegex(self.ci, r"(?m)^  proptest-roundtrip:")
        self.assertNotIn("prop_hwpx_roundtrip", RUN_ARCHIVE.read_text(encoding="utf-8"))
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
