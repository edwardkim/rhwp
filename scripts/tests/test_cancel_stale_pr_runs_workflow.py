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

    def test_force_cancel_error_polls_bounded_window_for_completed_target(self):
        """오류 직후 상태 반영이 늦어도 유한 polling 안의 완료만 정상 처리한다."""
        body = self.workflow
        self.assertNotIn("if (error.status !== 409) throw error;", body)

        delays = body.index(
            "const completionPollDelaysMs = [0, 500, 1_000, 2_000];"
        )
        status_helper = body.index("async function getWorkflowRun(runId)", delays)
        poll_helper = body.index("async function waitForCompletedRun(runId)", status_helper)
        poll_loop = body.index("for (const delay of completionPollDelaysMs)", poll_helper)
        poll_sleep = body.index("await sleep(delay);", poll_loop)
        reread = body.index("await getWorkflowRun(runId)", poll_sleep)
        completed_return = body.index(
            "if (currentRun.status === 'completed') return currentRun;", reread
        )
        exhausted = body.index("return null;", completed_return)

        cancel_call = body.index("await forceCancelWithRetry(run.id);")
        catch_start = body.index("} catch (error) {", cancel_call)
        poll_call = body.index("await waitForCompletedRun(run.id)", catch_start)
        active_failure = body.index("if (!completedRun) throw error;", poll_call)
        completed_notice = body.index("already completed after force-cancel error", active_failure)

        self.assertLess(delays, status_helper)
        self.assertLess(status_helper, poll_helper)
        self.assertLess(poll_loop, poll_sleep)
        self.assertLess(poll_sleep, reread)
        self.assertLess(reread, completed_return)
        self.assertLess(completed_return, exhausted)
        self.assertLess(catch_start, poll_call)
        self.assertLess(poll_call, active_failure)
        self.assertLess(active_failure, completed_notice)

    def test_status_poll_error_preserves_force_cancel_failure(self):
        """상태 API 자체가 실패하면 취소 실패를 성공으로 바꾸지 않는다."""
        body = self.workflow
        cancel_call = body.index("await forceCancelWithRetry(run.id);")
        cancel_catch = body.index("} catch (error) {", cancel_call)
        poll_call = body.index("await waitForCompletedRun(run.id)", cancel_catch)
        poll_catch = body.index("} catch (statusError) {", poll_call)
        warning = body.index("Failed to poll stale run", poll_catch)
        original_error = body.index("throw error;", warning)
        active_failure = body.index("if (!completedRun) throw error;", original_error)

        self.assertLess(poll_call, poll_catch)
        self.assertLess(poll_catch, warning)
        self.assertLess(warning, original_error)
        self.assertLess(original_error, active_failure)

    def test_force_cancel_retries_transient_github_api_errors(self):
        """일시적인 GitHub API 장애는 제한된 backoff 뒤에만 재시도한다."""
        body = self.workflow
        transient_statuses = body.index(
            "const retryableForceCancelStatuses = new Set([502, 503, 504]);"
        )
        delays = body.index("const forceCancelRetryDelaysMs = [1_000, 2_000, 4_000];")
        helper = body.index("async function forceCancelWithRetry(runId)")
        retry_guard = body.index(
            "if (!retryableForceCancelStatuses.has(error.status) || delay === undefined)",
            helper,
        )
        retry_notice = body.index("Retry force-cancel for stale run", retry_guard)
        call_site = body.index("await forceCancelWithRetry(run.id);")

        self.assertLess(transient_statuses, helper)
        self.assertLess(delays, helper)
        self.assertLess(helper, retry_guard)
        self.assertLess(retry_guard, retry_notice)
        self.assertLess(retry_notice, call_site)

    def test_github_script_uses_system_ca_without_disabling_tls_verification(self):
        """runner/프록시 CA를 신뢰하되 인증서 검증 자체를 끄지 않는다."""
        body = self.workflow
        action = body.index("uses: actions/github-script@")
        system_ca = body.index("NODE_OPTIONS: --use-system-ca", action)
        script = body.index("script: |", system_ca)

        self.assertLess(action, system_ca)
        self.assertLess(system_ca, script)
        self.assertNotIn("NODE_TLS_REJECT_UNAUTHORIZED", body)


if __name__ == "__main__":
    unittest.main()
