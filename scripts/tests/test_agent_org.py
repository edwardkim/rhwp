"""[#4720] 에이전트 노동 운영체제 계약 — 조직표·배치기·접수 스키마 정합.

부서·직급·배치가 실재 자산(gym pack·로드맵 트랙·과제) 위에서만 돌게 지킨다.
조직표가 실재하지 않는 pack 을 가리키거나 배치기가 유효한 배정을 못 내면
"시스템이 스스로 돈다"는 거짓말이 된다 — 그것을 매 CI 로 막는다. 바이너리 불요.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
ORG_DIR = REPO_ROOT / "mydocs" / "tech" / "agent_org"
DEPARTMENTS = ORG_DIR / "departments.json"
INTAKE_SCHEMA = ORG_DIR / "intake_manifest.schema.json"
PACKS = REPO_ROOT / "gym" / "packs"
TRACKS_DIR = REPO_ROOT / "mydocs" / "tech" / "agent_roadmap"
AWS_LEVELS = ["AW-L1", "AW-L2", "AW-L3", "AW-L4", "AW-L5"]


def load_dispatch():
    spec = importlib.util.spec_from_file_location(
        "agent_dispatch", REPO_ROOT / "tools" / "agent_dispatch.py")
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def org():
    return json.loads(DEPARTMENTS.read_text(encoding="utf-8"))


class OrgStructureTests(unittest.TestCase):
    def test_departments_json_is_well_formed(self):
        o = org()
        self.assertEqual(o["kind"], "agentOrg")
        self.assertTrue(o["departments"], "부서가 비었다")
        self.assertTrue(o.get("legitimacy"), "합법성 조항이 없다")

    def test_career_ladder_maps_to_aws_levels(self):
        ladder = org()["careerLadder"]
        self.assertEqual([r["aws"] for r in ladder], AWS_LEVELS)
        for rung in ladder:
            self.assertTrue(rung.get("level") and rung.get("promoteWhen"))

    def test_every_department_maps_to_real_assets(self):
        """부서의 pack·트랙·입사 과제가 저장소에 실재한다 — 지어낸 조직 금지."""
        problems = []
        for d in org()["departments"]:
            for pid in d.get("packs", []):
                if not (PACKS / pid / "pack.json").is_file():
                    problems.append(f"{d['id']}: 없는 pack {pid}")
            for tr in d.get("tracks", []):
                hits = list(TRACKS_DIR.glob(f"track_{tr.lower()}_*.md"))
                if not hits:
                    problems.append(f"{d['id']}: 없는 트랙 {tr}")
            task = d.get("entryTask")
            if task:
                pid, _, tid = task.partition("/")
                if not (PACKS / pid / "tasks" / f"{tid}.json").is_file():
                    problems.append(f"{d['id']}: 없는 입사 과제 {task}")
            # 서비스 부서가 아니면 일감(pack)이 있어야 배치가 성립한다.
            if not d.get("service") and not d.get("packs"):
                problems.append(f"{d['id']}: 서비스 부서가 아닌데 소유 pack 이 없다")
        self.assertEqual(problems, [], "\n".join(problems))

    def test_intake_schema_is_valid_json_schema(self):
        schema = json.loads(INTAKE_SCHEMA.read_text(encoding="utf-8"))
        self.assertIn("agent", schema["required"])
        dept_ids = {d["id"] for d in org()["departments"]}
        enum = set(schema["properties"]["targetDepartment"]["enum"]) - {"any"}
        self.assertEqual(enum, dept_ids, "접수 스키마의 부서 enum 이 조직표와 어긋난다")


class DispatchTests(unittest.TestCase):
    def test_dispatch_routes_every_department_to_a_valid_assignment(self):
        dispatch = load_dispatch()
        for d in org()["departments"]:
            with self.subTest(department=d["id"]):
                a = dispatch.dispatch({"agent": "t", "targetDepartment": d["id"]})
                self.assertEqual(a["department"]["id"], d["id"])
                if not d.get("service"):
                    self.assertTrue(a["startTask"], f"{d['id']}: 입사 과제 배정 실패")

    def test_unspecified_agent_goes_to_reception(self):
        dispatch = load_dispatch()
        a = dispatch.dispatch({"agent": "newbie"})
        self.assertEqual(a["department"]["id"], "reception")
        self.assertEqual(a["currentLevel"], "지원자")

    def test_promotion_path_advances_with_level(self):
        dispatch = load_dispatch()
        a = dispatch.dispatch({"agent": "t", "targetDepartment": "editing", "awsLevel": "AW-L1"})
        self.assertEqual(a["currentLevel"], "신입")
        self.assertEqual(a["promotion"]["toAws"], "AW-L2")


if __name__ == "__main__":
    unittest.main()
