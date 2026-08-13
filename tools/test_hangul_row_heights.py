"""hangul_row_heights의 COM 비의존 gate 단위 검증."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


MODULE_PATH = Path(__file__).with_name("hangul_row_heights.py")
SPEC = importlib.util.spec_from_file_location("hangul_row_heights", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class TotalDiffGateTest(unittest.TestCase):
    def test_gate_is_opt_in_and_uses_absolute_difference(self) -> None:
        self.assertFalse(MODULE.total_diff_exceeds(-706.89, None))
        self.assertFalse(MODULE.total_diff_exceeds(-100.0, 100.0))
        self.assertTrue(MODULE.total_diff_exceeds(-100.01, 100.0))
        self.assertTrue(MODULE.total_diff_exceeds(100.01, 100.0))


if __name__ == "__main__":
    unittest.main()
