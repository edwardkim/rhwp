from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("verify_r1_report_consistency.py")
SPEC = importlib.util.spec_from_file_location("verify_r1_report_consistency", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"검증기 모듈을 불러올 수 없습니다: {MODULE_PATH}")
CHECKER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECKER
SPEC.loader.exec_module(CHECKER)


class VerificationTableTests(unittest.TestCase):
    def test_parse_legacy_three_outcome_table(self) -> None:
        parsed = CHECKER.parse_tables(
            "\n## 4. 재현 검증\n"
            "| 전체 실행 판정 | 재실행 다름 | 재실행 동일 | 재실행 실패 |\n"
            "| --- | --- | --- | --- |\n"
            "| 다름 254건 | **247 (98.8%)** | 3 | 4 |\n"
            "\n## 5. 결과\n"
        )["verif"]

        self.assertEqual(
            parsed,
            {"raw": 254, "conf": 247, "pct": 98.8, "fp": 3, "fail": 4, "initial_err": 0},
        )

    def test_parse_table_with_initial_execution_error(self) -> None:
        parsed = CHECKER.parse_tables(
            "\n## 4. 재현 검증\n"
            "| 전체 실행 판정 | 재실행 다름 | 재실행 동일 | 재실행 실패 | 전체 실행 ERR |\n"
            "| --- | --- | --- | --- | --- |\n"
            "| 다름 254건 | **247 (98.8%)** | 2 | 4 | 1 |\n"
            "\n## 5. 결과\n"
        )["verif"]

        self.assertEqual(
            parsed,
            {"raw": 254, "conf": 247, "pct": 98.8, "fp": 2, "fail": 4, "initial_err": 1},
        )

    def test_parse_html_code_document_cell(self) -> None:
        parsed = CHECKER.parse_tables(
            "\n### 5.1 PAGE_DELTA\n"
            "| 문서 | 2022 | 2024 | 차이 |\n"
            "| --- | --- | --- | --- |\n"
            "| <code>admrul_downloads\\대법원\\3190263_&#91;별지 3&#93;.hwp</code> | 9 | 8 | -1 |\n"
            "\n### 5.2 BREAK_DIFF\n"
        )["page_delta"]

        self.assertEqual(
            parsed,
            [{"doc": "admrul_downloads\\대법원\\3190263_[별지 3].hwp", "p2022": 9, "p2024": 8, "delta": -1}],
        )


if __name__ == "__main__":
    unittest.main()
