"""[#4662] gym-release-gate.yml 계약 — 릴리스 게이트 워크플로가 규약대로인지.

이 파일명은 `test_*workflow*.py` 패턴이라 test_workflow_contract_wiring.py 가
ci.yml 배선을 강제한다(#4080). 배선을 잊으면 그 테스트가 실패한다.
"""

from __future__ import annotations

import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GATE_WF = REPO_ROOT / ".github/workflows/gym-release-gate.yml"
RELEASE_WF = REPO_ROOT / ".github/workflows/release-binary.yml"
GATE_RUNNER = REPO_ROOT / "gym/tools/release_gate.py"


class ReleaseGateWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.wf = GATE_WF.read_text(encoding="utf-8")

    def test_workflow_exists(self):
        self.assertTrue(GATE_WF.is_file(), "gym-release-gate.yml 이 없다")

    def test_does_not_touch_the_release_binary_workflow(self):
        """게이트는 릴리스 본체를 건드리지 않는다 — 독립 워크플로여야 한다."""
        release = RELEASE_WF.read_text(encoding="utf-8")
        self.assertNotIn("release_gate.py", release,
                         "릴리스 본체 워크플로에 게이트가 침습했다")
        self.assertNotIn("gym-release-gate", release)

    def test_invokes_the_gate_runner(self):
        self.assertIn("gym/tools/release_gate.py", self.wf)
        self.assertTrue(GATE_RUNNER.is_file(), "게이트 러너 스크립트가 없다")

    def test_manual_dispatch_and_tag_trigger(self):
        # 수동 실행 + 태그 관찰 둘 다 있어야 한다.
        self.assertIn("workflow_dispatch:", self.wf)
        self.assertIn("tags:", self.wf)

    def test_old_binary_is_optional(self):
        """직전 태그 미빌드 시 차등을 생략하는 분기가 있어야 한다(부재≠실패)."""
        self.assertIn("if [ -f ./rhwp-old-bin ]", self.wf)
        self.assertIn("old_ref", self.wf)

    def test_writes_github_summary(self):
        self.assertIn("--github-summary", self.wf)

    def test_runs_discrimination_audit_before_the_gate(self):
        """벤치마크 무결성 전제 — 약한 오라클(false-pass)이 있으면 릴리스 차단.

        판별 감사는 old/new 차등 이전에 돌아야 한다(벤치마크 자체가 성립하는지
        먼저 본다). exit 1 로 워크플로를 실패시켜 릴리스를 막는다.
        """
        self.assertIn("gym/tools/discriminate.py", self.wf)
        audit_at = self.wf.index("discriminate.py")
        gate_at = self.wf.index("release_gate.py")
        self.assertLess(audit_at, gate_at, "판별 감사가 게이트보다 먼저 배선돼야 한다")

    def test_read_only_permissions(self):
        """게이트는 판정만 한다 — 쓰기 권한이 필요 없다."""
        self.assertIn("contents: read", self.wf)

    def test_runs_trajectory_necessity_audit_before_the_gate(self):
        """트라젝토리 무결성 전제 — 무의미한 마지막 스텝(연극)이 있으면 릴리스 차단.

        다단계 과제의 마지막 스텝이 채점에 무의미하면(부분 트라젝토리가 통과)
        exit 1 로 워크플로를 실패시킨다. 게이트 차등 이전에 배선돼야 한다.
        """
        self.assertIn("gym/tools/trajectory.py", self.wf)
        audit_at = self.wf.index("trajectory.py")
        gate_at = self.wf.index("release_gate.py")
        self.assertLess(audit_at, gate_at, "트라젝토리 감사가 게이트보다 먼저 배선돼야 한다")

    def test_discriminate_runs_on_the_new_binary_only(self):
        """판별 감사는 현재 릴리스 바이너리만 본다. --old 를 넘기지 않는다.

        약한 오라클은 '지금 벤치가 일을 거부하는가' 이지 '두 바이너리가
        같은가' 가 아니다. 구 바이너리를 넣으면 차등과 겹친다.
        """
        # 감사 스텝 본문에 --old 가 없어야 한다. 게이트 스텝은 --old 를 쓴다.
        audit_at = self.wf.index("Discrimination audit")
        gate_at = self.wf.index("Run release gate")
        audit_block = self.wf[audit_at:gate_at]
        self.assertIn("discriminate.py", audit_block)
        self.assertIn("target/debug/rhwp", audit_block)
        self.assertNotIn("--old", audit_block)

    def test_discriminate_failure_fails_the_job_before_gate(self):
        """판별 실패는 워크플로 기본 동작으로 잡을 닫는다.

        continue-on-error 가 있으면 exit 1 이 게이트까지 흘러가지 않고,
        약한 오라클이 있는 릴리스가 차등만 통과해 나갈 수 있다.
        """
        audit_at = self.wf.index("Discrimination audit")
        gate_at = self.wf.index("Run release gate")
        audit_block = self.wf[audit_at:gate_at]
        self.assertNotIn("continue-on-error", audit_block)
        self.assertNotIn("|| true", audit_block)

    def test_gate_step_does_not_swallow_nonzero(self):
        """게이트 스텝도 종료 코드를 삼키지 않는다. review(2)/block(3) 이
        워크플로 실패로 보여야 사람이 본다. fail(1) 도 같다.
        """
        gate_at = self.wf.index("Run release gate")
        upload_at = self.wf.index("Upload verdict")
        gate_block = self.wf[gate_at:upload_at]
        self.assertNotIn("continue-on-error", gate_block)
        self.assertNotIn("|| true", gate_block)

    def test_upload_verdict_runs_even_when_gate_fails(self):
        """판정이 fail/block 이어도 봉투를 올려야 리뷰어가 이유를 본다."""
        self.assertIn("if: always()", self.wf)
        self.assertIn("gate-verdict.json", self.wf)
        self.assertIn("gym-release-gate-verdict", self.wf)

    def test_old_ref_empty_skips_old_build_not_the_gate(self):
        """old_ref 가 비면 구 바이너리 빌드만 생략한다. 게이트는 --new 만으로 돈다."""
        self.assertIn("if: ${{ github.event.inputs.old_ref != '' }}", self.wf)
        self.assertIn("--new target/debug/rhwp", self.wf)

    def test_concurrency_cancels_in_progress(self):
        self.assertIn("cancel-in-progress: true", self.wf)
        self.assertIn("gym-release-gate-${{ github.ref }}", self.wf)

    def test_does_not_add_write_permissions(self):
        """게이트는 판정만 한다. pull-requests: write 같은 권한이 생기면 침습이다."""
        self.assertNotIn("pull-requests: write", self.wf)
        self.assertNotIn("contents: write", self.wf)
        self.assertNotIn("id-token: write", self.wf)

    def test_runner_docs_exist(self):
        """#5259 문서 계약 — 규약과 작업 기록이 커밋되어 있어야 한다."""
        docs = REPO_ROOT / "gym/docs/release_gate.md"
        working = REPO_ROOT / "mydocs/working/gym_release_gate.md"
        self.assertTrue(docs.is_file(), "gym/docs/release_gate.md 이 없다")
        self.assertTrue(working.is_file(), "mydocs/working/gym_release_gate.md 이 없다")
        text = docs.read_text(encoding="utf-8")
        for needle in ("missing-old-bin", "missing-new-bin", "discriminate-fail",
                       "surface-changed", "regression"):
            self.assertIn(needle, text, needle)

    def test_workflow_comment_states_honesty_clause(self):
        self.assertIn("어느 쪽이 옳은가", self.wf)
        self.assertIn("regression 만 차단", self.wf)


class GateRunnerContractTests(unittest.TestCase):
    """러너의 판정 계약 — 종료 코드가 게이트 의미론과 일치하는지."""

    def setUp(self):
        import importlib.util
        import sys
        if str(REPO_ROOT) not in sys.path:
            sys.path.insert(0, str(REPO_ROOT))
        spec = importlib.util.spec_from_file_location("gym_release_gate_test", GATE_RUNNER)
        self.rg = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = self.rg
        spec.loader.exec_module(self.rg)

    def _gate_with_diff(self, classification, divergences, surface):
        import io
        import json
        from unittest import mock

        def fake(script, args):
            out = args[args.index("-o") + 1]
            with io.open(out, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({"classification": classification,
                                     "divergences": divergences,
                                     "surfaceChanged": surface, "tasksCompared": 91}))
            return (0, "")

        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", side_effect=fake):
            return self.rg.gate("old", "new", "agent", ["core-cli"], verify_board=False)

    def test_stable_passes(self):
        v = self._gate_with_diff("stable", 0, False)
        self.assertEqual((v["verdict"], v["exit"]), ("pass", 0))

    def test_regression_blocks(self):
        v = self._gate_with_diff("regression", 4, False)
        self.assertEqual((v["verdict"], v["exit"]), ("block", 3))

    def test_surface_changed_is_review_not_block(self):
        """정직 조항 — 표면 변경은 차단이 아니라 리뷰 신호다."""
        v = self._gate_with_diff("surface-changed", 70, True)
        self.assertEqual((v["verdict"], v["exit"]), ("review", 2))

    def test_broken_leaderboard_blocks(self):
        from unittest import mock
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", return_value=(3, "체인 파손")):
            v = self.rg.gate(None, "new", "agent", None, verify_board=True)
        self.assertEqual(v["exit"], 3)

    def test_leaderboard_uses_the_selected_new_binary(self):
        """기본 PATH가 아니라 게이트의 새 바이너리로 원장을 검증한다."""
        from unittest import mock
        with mock.patch("os.path.exists", return_value=True), \
                mock.patch.object(self.rg, "run_tool", return_value=(0, "무결")) as run_tool:
            v = self.rg.gate(None, "/tmp/rhwp-new", "agent", None, verify_board=True)
        self.assertEqual(v["verdict"], "pass")
        run_tool.assert_called_once_with(
            "leaderboard.py", ["--bin", "/tmp/rhwp-new", "verify"])

    def test_missing_old_binary_skips_diff_not_fail(self):
        from unittest import mock
        # 구 바이너리 부재는 차등 생략이지 실패가 아니다.
        # 신 바이너리까지 없다고 목하면 그 경로가 이긴다 — 이 시험은
        # missing-old 만 분리한다. 신 경로는 있다고 본다.
        def exists(path):
            text = str(path).replace("\\", "/")
            if "ledger.ndjson" in text:
                return False
            return True

        with mock.patch("os.path.exists", side_effect=exists):
            v = self.rg.gate(None, "new", "agent", None, verify_board=False)
        self.assertEqual(v["diff"]["classification"], "skipped")
        self.assertEqual(v["verdict"], "pass")
        self.assertEqual(v["reason"], "missing-old-bin")


if __name__ == "__main__":
    unittest.main()
