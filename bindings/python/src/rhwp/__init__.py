"""rhwp — HWP/HWPX 문서 엔진의 파이썬 바인딩.

**바인딩은 새 표면이 아니라 기존 계약의 재포장이다** (`bindings_foundation.md`).
CLI `--json` 봉투와 `mcp-serve` 세션 도구가 이미 증명한 계약 위에만 서고,
파이썬 쪽에서 판정 로직을 새로 만들지 않는다. 그래서 rhwp 본체에 명령이 늘면
바인딩은 자동으로 따라온다.

## 1층 — 무상태 (호출 하나 = 작업 하나)

```python
import rhwp

meta = rhwp.info("보고서.hwp")
print(meta.page_count, meta.format)

hits = rhwp.search("보고서.hwp", "예산")
for m in hits.matches:
    print(m.page, m.snippet)
```

## 2층 — 세션 (같은 문서를 반복해서 만질 때)

```python
with rhwp.open("서식.hwp") as doc:
    doc.fill_fields({"성명": "홍길동"})
    result = doc.save("제출본.hwp", verify=True)
    assert result.verify.identical
```

## 3층 — 계획 (의도를 선언하면 안전은 도구가 보장)

```python
plan = (rhwp.Plan("서식.hwp", "제출본.hwp")
        .fill_fields({"성명": "홍길동"})
        .set_checkbox(1)
        .verify())

if plan.check().ok:      # 디스크 무변경 검사
    plan.run()
```

## 판정 vs 고장

`--verify` 불일치나 시각 회귀는 **예외가 아니다** — 도구는 정상 동작했고 문서에
대한 단언이 실패한 것이다. 판정은 반환값(`result.verify.identical`)으로 읽는다.
예외를 원하면 `raise_on_verdict=True` 를 명시한다.
"""

from __future__ import annotations

from ._binary import ENV_VAR, clear_cache, find_binary
from ._naming import to_camel, to_snake
from ._process import CompletedRun, run_json, run_ndjson, run_raw
from .commands import (
    batch,
    build_from_ingest,
    capabilities,
    convert,
    csv_to_table,
    digest,
    explain,
    export_agent_manifest,
    export_plan_schema,
    export_provenance_map,
    export_doclang,
    export_hml,
    export_hwpx,
    export_markdown,
    export_ontology,
    export_pdf,
    export_structure,
    export_svg,
    export_tables,
    export_text,
    extract_data,
    extract_pages,
    fields,
    fill_fields,
    info,
    inspect,
    ir_diff,
    render_diff,
    replace_text,
    scan,
    search,
    set_cell,
    table_to_csv,
    thumbnail,
)
from .errors import (
    EXIT_OK,
    EXIT_RUNTIME,
    EXIT_USAGE,
    EXIT_VERIFY,
    EXIT_VERIFY_PAGES,
    BinaryNotFoundError,
    ProtocolError,
    RhwpError,
    RhwpRuntimeError,
    SessionClosedError,
    RhwpTimeoutError,
    UsageError,
    VerdictFailed,
)
from .models import Envelope, VerifyReport
from .plan import Plan, PlanResult, run_plan
from .schema import (
    FieldDef,
    IrSchema,
    TypeDef,
    capabilities_schema,
    capabilities_schema_envelope,
    ir_schema,
    ir_schema_envelope,
)
from .session import Document, Session, open

__version__ = "0.1.0"

#: 이 바인딩이 검증한 봉투 스키마 버전. rhwp 본체가 major 를 올리면 여기도 올린다.
SUPPORTED_SCHEMA_VERSION = "1.0"

__all__ = [
    # 메타
    "__version__",
    "SUPPORTED_SCHEMA_VERSION",
    "ENV_VAR",
    "find_binary",
    "clear_cache",
    # 1층 — 조회
    "info",
    "export_text",
    "export_structure",
    "export_tables",
    "table_to_csv",
    "fields",
    "search",
    "digest",
    "explain",
    "extract_data",
    "inspect",
    "export_provenance_map",
    "export_plan_schema",
    "export_agent_manifest",
    "export_ontology",
    "capabilities",
    # 1층 — 산출
    "export_svg",
    "export_pdf",
    "export_markdown",
    "export_hml",
    "export_doclang",
    "thumbnail",
    "extract_pages",
    "build_from_ingest",
    # 1층 — 변환·비교
    "export_hwpx",
    "convert",
    "ir_diff",
    "render_diff",
    # 1층 — 편집
    "fill_fields",
    "replace_text",
    "set_cell",
    "csv_to_table",
    # 1층 — 대량
    "scan",
    "batch",
    # 2층 — 세션
    "open",
    "Session",
    "Document",
    # 3층 — 계획
    "Plan",
    "PlanResult",
    "run_plan",
    # JSON Schema
    "ir_schema",
    "ir_schema_envelope",
    "capabilities_schema",
    "capabilities_schema_envelope",
    "IrSchema",
    "TypeDef",
    "FieldDef",
    # 모델
    "Envelope",
    "VerifyReport",
    # 저수준
    "run_json",
    "run_ndjson",
    "run_raw",
    "CompletedRun",
    "to_snake",
    "to_camel",
    # 예외
    "RhwpError",
    "BinaryNotFoundError",
    "UsageError",
    "RhwpRuntimeError",
    "VerdictFailed",
    "ProtocolError",
    "SessionClosedError",
    "RhwpTimeoutError",
    # 종료 코드 사전
    "EXIT_OK",
    "EXIT_RUNTIME",
    "EXIT_USAGE",
    "EXIT_VERIFY",
    "EXIT_VERIFY_PAGES",
]
