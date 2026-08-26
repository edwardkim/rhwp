"""HWP5 저장 계약 인벤토리·diff·table-probe 픽스처 (M-hwp5 / #5469).

이 패키지는 `rhwp hwp5-inventory` / `hwp5-inventory-diff` / `hwp5-table-probe`
가 쓰는 레코드 언어를 Python 으로 다시 적어, oracle/generated 픽스처와
리포트 전사를 디스크에 고정한다. 시리얼라이저 페이지 수 로직은 읽거나
쓰지 않는다. `src/serializer` · `canvaskit_policy` · `pdf` ·
`layout-anomaly` · `oracle_public` · `render_backend` · `proptest` ·
`fidelity_compare` · `gym/` 는 범위 밖이다.
"""

from __future__ import annotations

CLAIM_ID = "M-hwp5"
SCHEMA_VERSION = "1.0"
GENERATOR = "tools/hwp5_inventory/fatten_catalog.py"
KIND = "hwp5InventoryFattenCatalog"
ISSUE = 5469

__all__ = [
    "CLAIM_ID",
    "GENERATOR",
    "ISSUE",
    "KIND",
    "SCHEMA_VERSION",
]
