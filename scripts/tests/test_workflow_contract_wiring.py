"""[#4080] workflow 계약 테스트가 CI 에 실제로 배선되어 있는지 강제한다.

`scripts/tests/` 아래 workflow 계약 테스트는 workflow YAML 을 파싱해 CI 계약을
단언한다. 그런데 파일만 추가하고 `ci.yml` 에 실행 줄을 넣지 않으면 **한 번도 돌지
않는 회귀 방지 장치**가 된다. 실제로 그런 일이 두 번 있었다.

- `test_review_only_fast_pass_workflows.py` (#4071) — 추가 뒤 미배선
- `test_cache_sweep_workflow.py` (#4080) — 추가 뒤 미배선

이 테스트는 그 구멍을 막는다. 새 계약 테스트를 추가하면 여기서 실패하므로 `ci.yml`
배선을 같이 넣게 된다.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = REPO_ROOT / "scripts/tests"
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"

# workflow YAML 을 읽어 계약을 단언하는 테스트의 파일명 패턴.
CONTRACT_TEST_PATTERN = re.compile(r"^test_.*workflow.*\.py$")


def contract_test_files() -> list[str]:
    return sorted(
        path.name
        for path in TESTS_DIR.glob("test_*.py")
        if CONTRACT_TEST_PATTERN.match(path.name)
    )


class WorkflowContractWiringTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WORKFLOW.read_text(encoding="utf-8")

    def test_discovery_pattern_finds_the_known_contract_tests(self):
        """패턴 자체가 망가지면 이 테스트는 조용히 무의미해진다."""
        found = contract_test_files()
        for expected in [
            "test_ci_impact_workflow.py",
            "test_render_diff_workflow.py",
            "test_cache_sweep_workflow.py",
            "test_codeql_workflow.py",
            "test_release_channel_policy_workflow.py",
            "test_review_only_fast_pass_workflows.py",
            "test_workflow_contract_wiring.py",
        ]:
            self.assertIn(expected, found)

    def test_every_contract_test_is_invoked_by_ci(self):
        missing = [
            name
            for name in contract_test_files()
            if f"scripts/tests/{name}" not in self.ci
        ]
        self.assertEqual(
            missing,
            [],
            "ci.yml 에서 실행되지 않는 workflow 계약 테스트가 있다. Lint job 의 "
            "'Validate workflow contracts' 단계에 한 줄 추가한다.",
        )

    def test_contract_tests_run_in_a_job_that_survives_impact_conditioning(self):
        """#3790 조건화 뒤에도 이 테스트들이 실제로 도는 job 에 있어야 한다.

        `scripts/tests/**` 는 classifier 의 어떤 분류에도 걸리지 않아
        `fail-closed:unclassified-path` 로 `rust_required=true` 가 되고, `.github/**`
        변경은 `fail-closed:workflow-contract` 로 같은 결과가 된다. 따라서 Lint job
        (rust_required 조건) 에 있으면 두 경로 모두에서 실행이 보장된다.

        분류가 바뀌어 이 전제가 깨지면 배선 위치를 다시 판단해야 하므로, 위치를
        계약으로 고정한다.
        """
        lint_job = re.search(
            r"(?ms)^  lint:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)", self.ci
        )
        self.assertIsNotNone(lint_job, "ci.yml 에 lint job 이 없다")
        body = lint_job.group(0) if lint_job else ""
        for name in contract_test_files():
            self.assertIn(
                f"scripts/tests/{name}",
                body,
                f"{name} 이 lint job 밖에서 실행된다 — 조건화로 생략될 수 있다",
            )


if __name__ == "__main__":
    unittest.main()
