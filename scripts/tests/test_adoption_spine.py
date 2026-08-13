"""[#4715] 채택 척추 가드 계약 — 에이전트 작업 표준(AWS)이 전 표면에서 정합한가.

이 테스트가 지키는 것: 표면(규약 파일·gym·로드맵·표준 정본)이 늘어도 표준을
일관되게 가리키는 척추가 끊기지 않는다. 통과만 검사하면 가드가 "무엇이든
통과" 로 썩으므로, **반드시 걸려야 하는 깨진 표준**도 음성 회귀로 고정한다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SPINE = REPO_ROOT / "tools" / "adoption_spine.py"


def load_spine():
    spec = importlib.util.spec_from_file_location("adoption_spine", SPINE)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class AdoptionSpineTests(unittest.TestCase):
    def test_spine_is_intact_on_the_committed_repo(self):
        """실제 저장소에서 척추가 온전 — 표준이 전 표면에서 가리켜진다."""
        spine = load_spine()
        problems = spine.check()
        self.assertEqual(problems, [], "\n".join(problems))

    def test_machine_spec_has_five_named_levels(self):
        spine = load_spine()
        spec = json.loads(spine.STD_JSON.read_text(encoding="utf-8"))
        self.assertEqual([lvl["id"] for lvl in spec["levels"]], spine.LEVEL_IDS)
        self.assertEqual(spec["version"], "1.0")

    def test_broken_machine_spec_is_caught(self):
        """레벨을 하나 빠뜨린 기계용 정본은 반드시 걸린다(음성 회귀)."""
        spine = load_spine()
        real = json.loads(spine.STD_JSON.read_text(encoding="utf-8"))
        real["levels"] = real["levels"][:4]  # AW-L5 제거
        with tempfile.TemporaryDirectory() as d:
            broken = Path(d) / "broken.json"
            broken.write_text(json.dumps(real, ensure_ascii=False), encoding="utf-8")
            original = spine.STD_JSON
            spine.STD_JSON = broken
            try:
                problems = spine.check()
            finally:
                spine.STD_JSON = original
        self.assertTrue(any("레벨 id" in p for p in problems), problems)

    def test_surface_pointer_loss_is_caught(self):
        """뿌리(AGENTS.md)가 표준을 안 가리키면 걸린다 — 척추 끊김 감지."""
        spine = load_spine()
        with tempfile.TemporaryDirectory() as d:
            empty = Path(d) / "AGENTS.md"
            empty.write_text("# 표준을 안 가리키는 파일\n", encoding="utf-8")
            original = spine.AGENTS
            spine.AGENTS = empty
            try:
                problems = spine.check()
            finally:
                spine.AGENTS = original
        self.assertTrue(any("AGENTS.md" in p for p in problems), problems)


if __name__ == "__main__":
    unittest.main()
