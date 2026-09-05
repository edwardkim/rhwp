#!/usr/bin/env python3
"""Normalize-path and trigger rules for precommit_skill_gate.py."""

from __future__ import annotations

import unittest

from tools.skill_router.precommit_skill_gate import _is_trigger, _normalize_path


class NormalizePathTests(unittest.TestCase):
    def test_dot_claude_keeps_leading_dot(self) -> None:
        self.assertEqual(
            _normalize_path(".agents/skills/rhwp-skill-author/SKILL.md"),
            ".agents/skills/rhwp-skill-author/SKILL.md",
        )

    def test_dot_agents_keeps_leading_dot(self) -> None:
        self.assertEqual(
            _normalize_path(".agents/skills/bug-hunter/SKILL.md"),
            ".agents/skills/bug-hunter/SKILL.md",
        )

    def test_strips_dot_slash_prefix_only(self) -> None:
        self.assertEqual(
            _normalize_path("./tools/skill_router/gate_new_skill.py"),
            "tools/skill_router/gate_new_skill.py",
        )

    def test_backslash_to_slash(self) -> None:
        self.assertEqual(
            _normalize_path(".agents\\skills\\x\\SKILL.md"),
            ".agents/skills/x/SKILL.md",
        )


class TriggerTests(unittest.TestCase):
    def test_claude_skill_triggers(self) -> None:
        self.assertTrue(_is_trigger(".agents/skills/rhwp-skill-author/SKILL.md"))

    def test_agents_skill_triggers(self) -> None:
        self.assertTrue(_is_trigger(".agents/skills/bug-hunter/SKILL.md"))

    def test_router_triggers(self) -> None:
        self.assertTrue(_is_trigger("tools/skill_router/gate_new_skill.py"))

    def test_unrelated_skips(self) -> None:
        self.assertFalse(_is_trigger("mydocs/report/task_m100_5706/x.md"))


if __name__ == "__main__":
    unittest.main()
