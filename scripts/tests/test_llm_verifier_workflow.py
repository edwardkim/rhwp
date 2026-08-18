"""LLM verifier 패키지가 CI에서 실제로 검증되는지 고정한다."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"


class LlmVerifierWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WORKFLOW.read_text(encoding="utf-8")

    def test_lint_runs_all_standalone_verifier_contracts(self) -> None:
        lint_job = re.search(
            r"(?ms)^  lint:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)", self.ci
        )
        self.assertIsNotNone(lint_job, "ci.yml 에 lint job 이 없다")
        body = lint_job.group(0) if lint_job else ""
        self.assertIn("- name: Validate LLM verifier tool contracts", body)
        self.assertIn("export PYTHONPATH=tools/llm_verifier", body)
        for command in [
            "tools/llm_verifier/third_party_replay/tests",
            "tools/llm_verifier/shadow_agree/tests",
            "tools/llm_verifier/untrusted_sandbox/tests",
            "tools/llm_verifier/lineage_chain/tests",
            "cargo test --manifest-path tools/llm_verifier/repeat_eval/Cargo.toml",
        ]:
            self.assertIn(command, body)


if __name__ == "__main__":
    unittest.main()
