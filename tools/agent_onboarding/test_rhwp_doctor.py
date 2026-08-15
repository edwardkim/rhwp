#!/usr/bin/env python3
"""rhwp_doctor.py 의 순수 로직 가드 테스트 — 바이너리 불요.

.mcp.json 방출기와 리포트 집계(종료 코드), 레시피 지도 실존 검증, 샘플 선택을
스텁 경로로 검증한다. rhwp 바이너리 없이도 돌므로 CI 의 바이너리 불요 게이트에 맞는다.

실행:
    python -m unittest tools/agent_onboarding/test_rhwp_doctor.py
"""

import os
import sys
import tempfile
import unittest
from pathlib import Path

# CWD 와 무관하게 대상 모듈을 import 한다(CI 는 저장소 루트에서 파일 경로로 호출).
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rhwp_doctor as doc  # noqa: E402


class TestMcpSnippet(unittest.TestCase):
    def test_path_case_uses_bare_command(self):
        snip = doc.build_mcp_snippet("rhwp")
        self.assertEqual(snip["mcpServers"]["rhwp"]["command"], "rhwp")
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_absolute_path_case(self):
        abspath = r"C:\repo\target\release\rhwp.exe"
        snip = doc.build_mcp_snippet(abspath)
        self.assertEqual(snip["mcpServers"]["rhwp"]["command"], abspath)
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_snippet_is_json_roundtrippable(self):
        import json

        snip = doc.build_mcp_snippet("rhwp")
        again = json.loads(json.dumps(snip, ensure_ascii=False))
        self.assertEqual(again["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_args_are_copied_not_aliased(self):
        shared = ["mcp-serve"]
        snip = doc.build_mcp_snippet("rhwp", shared)
        shared.append("--boom")
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])


class TestAggregate(unittest.TestCase):
    def _chk(self, status, critical=True):
        return {"id": "x", "status": status, "critical": critical}

    def test_all_pass_is_zero(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.PASS)], binary_found=True)
        self.assertTrue(ok)
        self.assertEqual(code, 0)

    def test_critical_fail_is_one(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.FAIL)], binary_found=True)
        self.assertFalse(ok)
        self.assertEqual(code, 1)

    def test_critical_skip_is_not_ok(self):
        ok, code = doc.aggregate([self._chk(doc.SKIP)], binary_found=True)
        self.assertFalse(ok)
        self.assertEqual(code, 1)

    def test_binary_missing_is_three(self):
        # 바이너리가 없으면 검사 목록이 비어 있어도 종료 코드 3(빌드 필요).
        ok, code = doc.aggregate([], binary_found=False)
        self.assertFalse(ok)
        self.assertEqual(code, 3)

    def test_noncritical_fail_does_not_sink_health(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.FAIL, critical=False)], binary_found=True)
        self.assertTrue(ok)
        self.assertEqual(code, 0)


class TestRecipeMap(unittest.TestCase):
    def test_missing_repo_marks_everything_absent(self):
        with tempfile.TemporaryDirectory() as d:
            rows = doc.resolve_recipe_map(Path(d))
            self.assertEqual(len(rows), len(doc.RECIPES))
            for r in rows:
                self.assertFalse(r["skillExists"])
                self.assertFalse(r["recipeExists"])

    def test_detects_existing_skill_and_recipe(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            # 첫 레시피의 스킬 SKILL.md 를 만들어 실존 검출을 확인.
            skill = doc.RECIPES[0]["skill"]
            (root / ".claude" / "skills" / skill).mkdir(parents=True)
            (root / ".claude" / "skills" / skill / "SKILL.md").write_text("x", encoding="utf-8")
            # recipe 가 있는 항목 하나를 골라 파일 생성.
            with_recipe = next(r for r in doc.RECIPES if r["recipe"])
            rp = root / with_recipe["recipe"]
            rp.parent.mkdir(parents=True, exist_ok=True)
            rp.write_text("x", encoding="utf-8")

            rows = doc.resolve_recipe_map(root)
            by_skill = {r["skill"]: r for r in rows}
            self.assertTrue(by_skill[skill]["skillExists"])
            self.assertTrue(next(r for r in rows if r["recipe"] == with_recipe["recipe"])["recipeExists"])

    def test_recipe_none_is_never_marked_existing(self):
        # recipe 가 None 인 항목은 recipeExists 가 항상 False 여야 한다(빈 인용 방지).
        rows = doc.resolve_recipe_map(doc.default_repo_root())
        for r in rows:
            if r["recipe"] is None:
                self.assertFalse(r["recipeExists"])


class TestPickSample(unittest.TestCase):
    def test_none_when_absent(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(doc.pick_sample(Path(d), None))

    def test_override_wins_when_present(self):
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "my.hwp"
            f.write_text("x", encoding="utf-8")
            self.assertEqual(doc.pick_sample(Path(d), str(f)), f)

    def test_override_absent_returns_none(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(doc.pick_sample(Path(d), str(Path(d) / "nope.hwp")))

    def test_finds_candidate_in_tree(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            rel = doc.SAMPLE_CANDIDATES[0]
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text("x", encoding="utf-8")
            self.assertEqual(doc.pick_sample(root, None), p)


if __name__ == "__main__":
    unittest.main()
