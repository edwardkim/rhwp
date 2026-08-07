"""[#4106] 큐 규율 잠금 판별 회귀 테스트.

네트워크 없이 `gh` 응답을 고정해, 회고나 인용의 "착수"라는 단어가 실제 잠금을
흉내 내지 못하고 protocol §5-1의 명시 형식만 잠금으로 인정하는지 검증한다.
"""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace


REPO_ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = REPO_ROOT / "tools" / "agent_preflight.py"


def load_module():
    spec = importlib.util.spec_from_file_location("agent_preflight_queue_test", MODULE_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class QueueDisciplineTests(unittest.TestCase):
    def run_check(self, comment: str):
        module = load_module()

        def fake_run(command, cwd=None, stdin_data=None):
            del cwd, stdin_data
            if command[:3] == ["gh", "auth", "status"]:
                return SimpleNamespace(returncode=0, stdout="", stderr="")
            if command[:3] == ["gh", "pr", "list"]:
                return SimpleNamespace(
                    returncode=0,
                    stdout=json.dumps([{"number": 4106, "title": "큐 규율", "body": ""}]),
                    stderr="",
                )
            if command[:4] == ["git", "symbolic-ref", "--short", "-q"]:
                return SimpleNamespace(returncode=0, stdout="task/3914-queue\n", stderr="")
            if command[:3] == ["gh", "issue", "view"]:
                return SimpleNamespace(
                    returncode=0,
                    stdout=json.dumps({"assignees": [], "comments": [{"body": comment}]}),
                    stderr="",
                )
            raise AssertionError(f"unexpected command: {command}")

        module.run = fake_run
        report = module.Report()
        module.check_queue_discipline(REPO_ROOT, report)
        return report

    def test_plain_word_mention_does_not_unlock_issue(self):
        report = self.run_check("아직 착수하지 않습니다. 이전 착수 규약만 검토했습니다.")
        self.assertEqual(len(report.warnings), 1)
        self.assertIn("assignee 도 착수 코멘트도 없다", report.warnings[0][1])

    def test_explicit_claim_format_unlocks_issue(self):
        report = self.run_check("착수합니다 — 큐 규율 검사 구현")
        self.assertEqual(report.warnings, [])
        self.assertTrue(any("잠금=착수 코멘트" in item for item in report.passed))


if __name__ == "__main__":
    unittest.main()
