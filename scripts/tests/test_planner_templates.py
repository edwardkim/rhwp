"""[#4718] 계획 템플릿 계약 — mydocs/manual/planner_templates/*.plan.json.

템플릿은 에이전트가 복사해 쓰는 시작점이다. 형식이 깨진 템플릿은 조용히
에이전트를 헛돌게 하므로, 매 CI 마다 **계획 스키마 형식**을 지킨다. 바이너리
없이 커밋된 JSON 만 본다 — `rhwp export-plan-schema` 의 형식과 대조한다.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATES = REPO_ROOT / "mydocs" / "manual" / "planner_templates"

#: 계획 스텝의 action 과 그 필수 필드(export-plan-schema $defs 와 일치).
STEP_REQUIRED = {
    "set_cell": ("table", "row", "col", "text"),
    "replace_text": ("find", "replace"),
    "fill_fields": ("data",),
    "set_checkbox": ("occurrence",),
}


def template_files():
    return sorted(TEMPLATES.glob("*.plan.json"))


class PlannerTemplateTests(unittest.TestCase):
    def test_templates_exist(self):
        self.assertGreaterEqual(len(template_files()), 4,
                                "계획 템플릿이 줄었다면 이관 사고를 의심하라")

    def test_every_template_is_a_valid_plan(self):
        for path in template_files():
            with self.subTest(template=path.name):
                plan = json.loads(path.read_text(encoding="utf-8"))
                # 계획서 필수 키
                for key in ("planVersion", "input", "output", "steps"):
                    self.assertIn(key, plan, f"{path.name}: 필수 키 {key} 없음")
                self.assertEqual(plan["planVersion"], "1.0",
                                 f"{path.name}: planVersion 은 '1.0'")
                self.assertTrue(plan["steps"], f"{path.name}: steps 가 비었다")
                for i, step in enumerate(plan["steps"]):
                    action = step.get("action")
                    self.assertIn(action, STEP_REQUIRED,
                                  f"{path.name} step{i}: 미지 action {action}")
                    for field in STEP_REQUIRED[action]:
                        self.assertIn(field, step,
                                      f"{path.name} step{i}: {action} 에 {field} 없음")

    def test_every_template_carries_a_self_verification_assertion(self):
        """계획이 스스로 검증 조건을 들고 다니게 — 템플릿은 단언을 뺀 채 배포하지 않는다."""
        for path in template_files():
            with self.subTest(template=path.name):
                plan = json.loads(path.read_text(encoding="utf-8"))
                assertions = plan.get("assertions", {})
                self.assertTrue(
                    assertions.get("notFoundEmpty") or assertions.get("verify"),
                    f"{path.name}: assertions.notFoundEmpty(또는 verify)가 없다")


if __name__ == "__main__":
    unittest.main()
