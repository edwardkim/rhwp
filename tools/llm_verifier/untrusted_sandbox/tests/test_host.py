from __future__ import annotations

import json
import unittest

from support import PKG
from untrusted_sandbox.host import isolate_envelope
from untrusted_sandbox.slot import Slot


class HostTests(unittest.TestCase):
    def test_search_envelope_isolated_for_data_block(self) -> None:
        envelope = json.loads(
            (PKG / "fixtures" / "envelopes" / "search_untrusted.json").read_text(
                encoding="utf-8"
            )
        )
        report = isolate_envelope(
            envelope,
            command="search",
            source_label="samples/기안문/case-000001.hwp",
            slot=Slot.LLM_DATA_BLOCK.value,
        )
        self.assertFalse(report.blocked)
        self.assertGreaterEqual(len(report.isolated), 2)
        self.assertTrue(all("UNTRUSTED_BEGIN" in item.wrapped for item in report.isolated))

    def test_criteria_slot_blocks_same_envelope(self) -> None:
        envelope = json.loads(
            (PKG / "fixtures" / "envelopes" / "search_untrusted.json").read_text(
                encoding="utf-8"
            )
        )
        report = isolate_envelope(
            envelope,
            command="search",
            source_label="samples/기안문/case-000001.hwp",
            slot=Slot.CRITERIA.value,
        )
        self.assertTrue(report.blocked)

    def test_proposed_criteria_containing_excerpt_blocks(self) -> None:
        envelope = json.loads(
            (PKG / "fixtures" / "envelopes" / "info_title.json").read_text(
                encoding="utf-8"
            )
        )
        title = envelope["title"]
        report = isolate_envelope(
            envelope,
            command="info",
            source_label="samples/2026_oss_rst.hwp",
            proposed_criteria=f"제목이 {title} 이면 합격",
            slot=Slot.LLM_DATA_BLOCK.value,
        )
        self.assertTrue(report.blocked)
        self.assertGreaterEqual(len(report.leaks), 1)
