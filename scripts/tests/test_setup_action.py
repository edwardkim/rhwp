"""Setup action repeat-invocation contract."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class SetupActionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.action = (ROOT / "action.yml").read_text(encoding="utf-8")
        cls.selftest = (
            ROOT / ".github/workflows/action-selftest.yml"
        ).read_text(encoding="utf-8")

    def test_each_invocation_uses_a_unique_install_directory(self) -> None:
        self.assertIn(
            'DEST="$(mktemp -d "${DEST_ROOT%/}/rhwp-setup.XXXXXX")"',
            self.action,
        )
        self.assertNotIn('DEST="${RUNNER_TEMP}/rhwp-setup"', self.action)
        self.assertIn('DEST_ROOT="$(cygpath -u "$DEST_ROOT")"', self.action)
        self.assertIn('BIN_DIR="$(cygpath -w "$BIN_DIR")"', self.action)

    def test_selftest_invokes_the_action_twice_in_one_job(self) -> None:
        self.assertEqual(self.selftest.count("        uses: ./\n"), 2)
        self.assertIn("steps.setup-first.outputs.version", self.selftest)
        self.assertIn("steps.setup-second.outputs.version", self.selftest)


if __name__ == "__main__":
    unittest.main()
