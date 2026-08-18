from __future__ import annotations
import unittest
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1].parent))
from w2_edit_merge_table.decide import decide, RULES, COMMAND, FAMILY

class DecideTests(unittest.TestCase):
    def test_rule_count(self) -> None:
        self.assertGreaterEqual(len(RULES), 40)
        self.assertEqual(COMMAND, 'edit-merge-table')
        self.assertEqual(FAMILY, 'mutate')

    def test_happy_path_is_code(self) -> None:
        env = {
            "pageCount": 3, "paraCount": 3, "itemCount": 3, "declaredCount": 3,
            "arrayLen": 3, "exitCode": 0, "requestedPage": 0, "emittedCount": 1,
            "maxChars": 10, "textLen": 4, "truncated": False, "empty": False,
            "rows": 2, "cols": 2, "rowSpan": 1, "colSpan": 1, "bytes": 12,
            "width": 1, "height": 1, "emptyOutput": False, "matchCount": 1,
            "page": 0, "offset": 0, "kind": "all", "count": 1, "inputN": 2,
            "okN": 2, "failN": 0, "neighborChanged": False, "findingCount": 0,
            "hasSignal": False, "overflow": 0, "overlap": 0, "before": 4,
            "after": 4, "sameFormat": True, "diffCount": 1, "identical": False,
            "pxDelta": 0, "threshold": 2, "structMismatch": False, "emptyPage": 0,
            "verify": True, "written": 1, "reread": 1, "applied": True,
            "beforeCount": 1, "afterCount": 0, "present": True, "extraKey": False,
            "expectedSha": "a" * 64, "actualSha": "a" * 64, "available": True,
            "requiresFeature": "", "rpcError": False, "isError": False,
            "ok": 1, "total": 1, "level": "L1", "inputSha": "b" * 64,
            "planSha": "c" * 64, "outputSha": "d" * 64, "hangulYear": 2022,
            "nols": False, "usedAsOracle": False, "escaped": False,
            "outsideWorkspace": False, "sizeBytes": 10, "capBytes": 100,
            "accepted": True, "hasSpace": False, "parsed": True,
            "rowsIn": 2, "colsIn": 2, "rowsOut": 2, "colsOut": 2,
        }
        verdict = decide(env)
        self.assertIsInstance(verdict, str)
        self.assertNotEqual(verdict, "")

    def test_negative_count_is_usage(self) -> None:
        if FAMILY != "count_eq":
            return
        self.assertEqual(decide({"declaredCount": -1, "arrayLen": 0}), "USAGE")

if __name__ == "__main__":
    unittest.main()
