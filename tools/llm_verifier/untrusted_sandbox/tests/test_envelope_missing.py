from __future__ import annotations

import json
import unittest

from support import PKG
from untrusted_sandbox.envelope import extract_slices, provenance_present, untrusted_content
from untrusted_sandbox.host import isolate_envelope


class MissingProvenanceTests(unittest.TestCase):
    def test_missing_keys_are_not_clean(self) -> None:
        envelope = json.loads(
            (PKG / "fixtures" / "envelopes" / "missing_keys.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertFalse(provenance_present(envelope))
        self.assertIsNone(untrusted_content(envelope))
        slices = extract_slices(envelope, "export-text")
        self.assertEqual(slices[0].path, "<missing-provenance>")
        report = isolate_envelope(
            envelope,
            command="export-text",
            source_label="samples/기안문/legacy.hwp",
        )
        self.assertTrue(report.blocked)
        self.assertIn("missing_provenance_keys", report.errors)
