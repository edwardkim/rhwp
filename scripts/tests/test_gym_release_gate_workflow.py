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

    def test_read_only_permissions(self):
        """게이트는 판정만 한다 — 쓰기 권한이 필요 없다."""
        self.assertIn("contents: read", self.wf)


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

    def test_missing_old_binary_skips_diff_not_fail(self):
        from unittest import mock
        # old 경로가 없으면(find_bin 이 그대로 반환, exists=False) 차등은 skipped.
        with mock.patch("os.path.exists", return_value=False):
            v = self.rg.gate(None, "new", "agent", None, verify_board=False)
        self.assertEqual(v["diff"]["classification"], "skipped")
        self.assertEqual(v["verdict"], "pass")


if __name__ == "__main__":
    unittest.main()
