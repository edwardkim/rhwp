"""stale PR run 취소 workflow의 완료 race 처리 계약을 검증한다."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPO_ROOT / ".github/workflows/cancel-stale-pr-runs.yml"


class CancelStalePrRunsWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_force_cancel_error_rechecks_completed_target(self):
        """500 등 응답 코드가 아니라 대상 run의 실제 완료 상태로 race를 판정한다."""
        body = self.workflow
        self.assertNotIn("if (error.status !== 409) throw error;", body)

        catch_start = body.index("} catch (error) {")
        reread = body.index("github.rest.actions.getWorkflowRun", catch_start)
        active_failure = body.index(
            "if (currentRun.status !== 'completed') throw error;", reread
        )
        completed_notice = body.index("already completed after force-cancel error", active_failure)

        self.assertLess(catch_start, reread)
        self.assertLess(reread, active_failure)
        self.assertLess(active_failure, completed_notice)


if __name__ == "__main__":
    unittest.main()
