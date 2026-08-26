"""[#4726] 에이전트 프레임 메타 가드 계약 — 틀의 불변식을 지킨다.

핵심 불변식: 프레임에 등재된 모든 하위체계는 가드를 하나 이상 가진다. 통과만
검사하면 가드가 "무엇이든 통과"로 썩으므로, **가드 없는 하위체계**도 음성 회귀로
고정한다.
"""

from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
FRAME = REPO_ROOT / "tools" / "frame_guard.py"


def load_guard():
    spec = importlib.util.spec_from_file_location("frame_guard", FRAME)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AgentFrameTests(unittest.TestCase):
    def test_frame_is_intact_on_the_committed_repo(self):
        guard = load_guard()
        problems = guard.check()
        self.assertEqual(problems, [], "\n".join(problems))

    def test_every_subsystem_declares_a_guard(self):
        guard = load_guard()
        frame = json.loads(guard.FRAME.read_text(encoding="utf-8"))
        for sub in frame["subsystems"]:
            self.assertTrue(sub.get("guards"), f"{sub['id']}: 가드 없음(불변식 위반)")

    def test_guardless_subsystem_is_caught(self):
        """가드를 뺀 하위체계는 반드시 걸린다 — 틀의 불변식 음성 회귀."""
        guard = load_guard()
        frame = json.loads(guard.FRAME.read_text(encoding="utf-8"))
        broken = copy.deepcopy(frame)
        broken["subsystems"].append(
            {"id": "rogue", "name": "무단 하위체계", "status": "merged",
             "keyFile": "gym/README.md", "guards": []})
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "frame.json"
            p.write_text(json.dumps(broken, ensure_ascii=False), encoding="utf-8")
            original = guard.FRAME
            guard.FRAME = p
            try:
                problems = guard.check()
            finally:
                guard.FRAME = original
        self.assertTrue(any("가드가 없다" in x for x in problems), problems)

    def test_merged_subsystems_point_to_real_files(self):
        guard = load_guard()
        frame = json.loads(guard.FRAME.read_text(encoding="utf-8"))
        for sub in frame["subsystems"]:
            if sub.get("status") == "merged":
                self.assertTrue((REPO_ROOT / sub["keyFile"]).exists(),
                                f"{sub['id']}: keyFile 부재 {sub['keyFile']}")


if __name__ == "__main__":
    unittest.main()
