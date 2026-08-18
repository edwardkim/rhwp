from __future__ import annotations

import unittest

from support import REPO
from shadow_agree.checks import (
    CHECKS,
    INVENTED_COMMANDS,
    check_by_id,
    command_key,
    iter_distinct_pairs,
    same_command,
)


class CheckCatalogTests(unittest.TestCase):
    def test_ids_are_unique(self) -> None:
        ids = [item.check_id for item in CHECKS]
        self.assertEqual(len(ids), len(set(ids)))
        keys = [item.command_key for item in CHECKS]
        self.assertEqual(len(keys), len(set(keys)))

    def test_commands_are_existing_rhwp_verbs(self) -> None:
        src = (REPO / "src" / "main.rs").read_text(encoding="utf-8")
        for item in CHECKS:
            self.assertTrue(item.command.startswith("rhwp "), item.check_id)
            verb = item.command.split()[1]
            self.assertIn(f'Some("{verb}")', src, item.command)
            if item.pass_field not in {"pageCount"}:
                self.assertIn(item.pass_field.split(".")[-1], src)

    def test_no_invented_cli(self) -> None:
        catalog = " ".join(item.command for item in CHECKS)
        for invented in INVENTED_COMMANDS:
            self.assertNotIn(invented, catalog)

    def test_unknown_check_rejected(self) -> None:
        with self.assertRaises(ValueError):
            check_by_id("holistic-score")

    def test_distinct_pairs_never_share_command(self) -> None:
        pairs = iter_distinct_pairs()
        self.assertGreaterEqual(len(pairs), 16 * 15)
        for left, right in pairs:
            self.assertNotEqual(left.command_key, right.command_key)
            self.assertFalse(same_command(left.check_id, right.check_id))

    def test_command_key_lookup(self) -> None:
        self.assertEqual(command_key("ir-diff"), "ir-diff")
        self.assertEqual(command_key("verify-pages"), "verify")
        self.assertEqual(command_key("fill-verify"), "fill-fields")


if __name__ == "__main__":
    unittest.main()
