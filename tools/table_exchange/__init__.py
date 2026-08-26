"""표 CSV 왕복 치수·병합 픽스처 (M-tbl / #5485).

devel 의 `export-tables` / `table-to-csv` / `csv-to-table` 계약을
Python 으로 다시 적어, 치수·coveredCellNotEmpty·dry-run/verify 픽스처를
디스크에 고정한다.

새 CLI 를 만들지 않는다. DocumentCore 편집 로직을 발명하지 않는다.
병합 풀기·표 리사이즈를 구현하지 않는다. gym/ 과 다른 진행 석
(`fidelity_compare`, `hwp5_inventory`, `form_fill`, `work_receipt`,
inspect, provenance) 은 범위 밖이다.

정본: `.claude/skills/rhwp-table-exchange/`,
`tests/table_csv_contract.rs`, `mydocs/manual/cli_commands.md`.
"""

from __future__ import annotations

CLAIM_ID = "M-tbl"
SCHEMA_VERSION = "1.0"
GENERATOR = "tools/table_exchange/fatten_catalog.py"
KIND = "tableCsvRoundtripFattenCatalog"
ISSUE = 5485
SKILL = "rhwp-table-exchange"
SKILL_ISSUE = 5306

__all__ = [
    "CLAIM_ID",
    "GENERATOR",
    "ISSUE",
    "KIND",
    "SCHEMA_VERSION",
    "SKILL",
    "SKILL_ISSUE",
]
