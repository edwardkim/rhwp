"""[#5959] undo depth 측정 게이트(#5769)의 CI 배선 계약.

스냅샷 예산 축출 회귀는 실 wasm + 실 브라우저로만 잡힌다. 그래서 이 게이트는
frontend-package-gates(실 wasm 빌드) 안에서 `npm run e2e:undo-depth` 로 단다.
이 테스트는 그 배선이 사라지지 않게 고정한다. 파일명이 `test_*workflow*.py`
패턴이라 test_workflow_contract_wiring.py 가 ci.yml 배선을 강제한다(#4080).
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CI_WF = REPO_ROOT / ".github/workflows/ci.yml"
STUDIO = REPO_ROOT / "rhwp-studio"
RUNNER = STUDIO / "e2e/run-with-vite.mjs"
SERVER_HELPER = STUDIO / "e2e/vite-server.mjs"
GATE_TEST = STUDIO / "e2e/undo-depth-issue5769.test.mjs"


def package_gates_section(ci_text: str) -> str:
    """ci.yml 에서 frontend-package-gates job 본문만 떼어 온다."""
    starts = [m.start() for m in re.finditer(r"(?m)^  [a-z0-9-]+:$", ci_text)]
    begin = ci_text.index("  frontend-package-gates:")
    end = next((pos for pos in starts if pos > begin), len(ci_text))
    return ci_text[begin:end]


class UndoDepthE2EGateWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WF.read_text(encoding="utf-8")
        cls.package_gates = package_gates_section(cls.ci)

    def test_runner_and_gate_files_exist(self) -> None:
        self.assertTrue(RUNNER.is_file(), "run-with-vite.mjs 러너가 없다")
        self.assertTrue(SERVER_HELPER.is_file(), "vite-server.mjs 공용 헬퍼가 없다")
        self.assertTrue(GATE_TEST.is_file(), "undo-depth 게이트 테스트 파일이 없다")

    def test_gate_is_wired_in_frontend_package_gates(self) -> None:
        """게이트 단계는 package 모드 잡 안에서 실제로 실행돼야 한다."""
        self.assertIn(
            "name: Run undo depth measurement gate (#5769)",
            self.package_gates,
            "frontend-package-gates 에 게이트 단계가 없다",
        )
        self.assertIn("npm run e2e:undo-depth", self.package_gates)
        self.assertIn(
            "name: Install headless Chrome",
            self.package_gates,
            "게이트에 필요한 Chrome 설치 단계가 없다 — CHROME_PATH 없이는 구동 불가다",
        )

    def test_gate_step_runs_after_real_wasm_build(self) -> None:
        """unit 모드용 스텁 wasm 이 아니라 실 wasm 빌드 뒤에 있어야 한다."""
        wasm_build = self.package_gates.index("Build fresh WASM package for frontend gates")
        gate = self.package_gates.index("Run undo depth measurement gate (#5769)")
        self.assertLess(wasm_build, gate)

    def test_npm_script_routes_through_vite_runner(self) -> None:
        """npm script 는 run-with-vite 러너 경유여야 한다 — CI 에 서버 기동이 필요하기 때문."""
        scripts = json.loads((STUDIO / "package.json").read_text(encoding="utf-8"))["scripts"]
        script = scripts["e2e:undo-depth"]
        self.assertIn("run-with-vite.mjs", script)
        self.assertIn("undo-depth-issue5769.test.mjs", script)
        self.assertIn("--mode=headless", script)

    def test_gate_test_declares_no_snapshot_contract(self) -> None:
        """게이트 본문은 무축출 계약(슬롯 0·전량 소진)을 단언해야 한다."""
        body = GATE_TEST.read_text(encoding="utf-8")
        self.assertIn("snapshotResourceCount", body)
        self.assertIn("E2E_DEPTH_ROUNDS", body, "스모크용 라운드 조절 환경변수가 없다")


if __name__ == "__main__":
    unittest.main()
