from __future__ import annotations

import unittest

from untrusted_sandbox.slot import (
    ALLOWED_SLOTS,
    INSTRUCTION_SLOTS,
    SLOT_VALUES,
    Slot,
    parse_slot,
)


class SlotTests(unittest.TestCase):
    def test_closed_set(self) -> None:
        self.assertEqual(len(SLOT_VALUES), 10)
        self.assertEqual(len(ALLOWED_SLOTS), 2)
        self.assertEqual(len(INSTRUCTION_SLOTS), 8)
        self.assertTrue(ALLOWED_SLOTS.isdisjoint(INSTRUCTION_SLOTS))

    def test_parse(self) -> None:
        self.assertEqual(parse_slot("criteria"), Slot.CRITERIA)
        with self.assertRaises(ValueError):
            parse_slot("reward")

    def test_criteria_is_instruction(self) -> None:
        self.assertTrue(Slot.CRITERIA.is_instruction)
        self.assertTrue(Slot.CRITERIA.is_criteria)
        self.assertFalse(Slot.CRITERIA.is_allowed)
        self.assertTrue(Slot.LLM_DATA_BLOCK.is_allowed)
        self.assertTrue(Slot.USER_DISPLAY.is_allowed)
