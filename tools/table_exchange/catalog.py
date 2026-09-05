"""Existing CLI contract constants for table CSV round-trip.

Values come from devel `cli_commands.md` §export-tables · §table-to-csv ·
§csv-to-table · §종료 코드 #2707, `tests/table_csv_contract.rs`, and
`.agents/skills/rhwp-table-exchange/`. Nothing here invents a new flag
or edit verb.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from . import ISSUE, SCHEMA_VERSION, SKILL, SKILL_ISSUE

COMMANDS = (
    "export-tables",
    "table-to-csv",
    "csv-to-table",
)

# edit set-cell is the documented fallback, not a new writer.
FALLBACK_COMMAND = "edit set-cell"

FLAGS: dict[str, tuple[str, ...]] = {
    "export-tables": ("--json", "-o"),
    "table-to-csv": ("--table", "-o", "--bom", "--json"),
    "csv-to-table": ("--csv", "--table", "-o", "--dry-run", "--verify", "--json"),
}

EXIT_CODES = (0, 1, 2, 3)

EXIT_KIND = {
    0: "success",
    1: "runtime-silent-stdout",
    2: "usage-or-contract-reject",
    3: "verify-judgment",
}

INVALID_REASONS = (
    "rowCountMismatch",
    "colCountMismatch",
    "coveredCellNotEmpty",
    "controlCharacter",
    "csvParse",
)

REASON_FIELDS = {
    "rowCountMismatch": ("reason", "expected", "actual", "message"),
    "colCountMismatch": ("reason", "expected", "actual", "row", "message"),
    "coveredCellNotEmpty": ("reason", "row", "col", "anchorRow", "anchorCol", "message"),
    "controlCharacter": ("reason", "row", "col", "message"),
    "csvParse": ("reason", "message"),
}

REASON_WRITES = {reason: False for reason in INVALID_REASONS}

REASON_EXIT = {reason: 2 for reason in INVALID_REASONS}

CSV_ROUNDTRIP = (
    "allowed",
    "extract-only",
    "forbidden",
    "outer-only",
    "skip",
    "empty-slot",
    "pick-first",
)

FAMILIES = (
    "dimension",
    "covered",
    "dry-run",
    "verify",
    "export-tables",
    "table-to-csv",
)

MODES = (
    "scan",
    "extract",
    "dry-run",
    "write",
    "verify",
    "usage",
)

OUTPUT_FORMATS = {
    ".hwp": "hwp5",
    ".hwpx": "hwpx",
}

UNTRUSTED_FIELDS = {
    "export-tables": ("tables[].cells[].text", "tables[].cells[].nested[]"),
    "table-to-csv": ("tables[].csv",),
    "csv-to-table": ("changed[].oldText",),
}

CONTROL_CHARS = ("\n", "\r", "\t")

BOM_BYTES = (0xEF, 0xBB, 0xBF)
BOM_CHAR = "\ufeff"

ROW_MISMATCH_MESSAGE = (
    "CSV 행 수 {actual} 가 표 {table} 의 행 수 {expected} 와 다릅니다"
    " — 표 크기는 바꾸지 않습니다."
)
COL_MISMATCH_MESSAGE = (
    "CSV {row}행 필드 수 {actual} 가 표 {table} 의 열 수 {expected} 와 다릅니다"
    " — 표 크기는 바꾸지 않습니다."
)
COVERED_MESSAGE = (
    "({row},{col}) 는 병합으로 덮인 칸입니다 — 앵커 ({anchor_row},{anchor_col}) 를 지정하세요."
)
CONTROL_MESSAGE = "셀 값에 줄바꿈·탭은 v1 에서 허용하지 않습니다."
CSV_PARSE_MESSAGE = "CSV 를 읽지 못했습니다 — 닫히지 않은 따옴표."

USAGE_SILENT = "인자 누락은 exit 2 · stdout 0바이트. 치수 실패와 다르다."
RUNTIME_SILENT = "표 없음·파일 없음은 exit 1 · stdout 0바이트. 원본 불변."


@dataclass(frozen=True)
class SampleSpec:
    path: str
    role: str
    table_count: int | None = None
    focus_index: int | None = None
    focus_rows: int | None = None
    focus_cols: int | None = None
    focus_merged: int | None = None
    focus_cell_count: int | None = None
    notes: str = ""


SAMPLES: tuple[SampleSpec, ...] = (
    SampleSpec(
        path="samples/hwp_table_test.hwp",
        role="레시피 02 정본 왕복",
        table_count=10,
        focus_index=0,
        focus_rows=4,
        focus_cols=3,
        focus_merged=0,
        focus_cell_count=12,
        notes="3열×4행. 헤더 제목,담당자,세부 내용. 본문 칸 공란.",
    ),
    SampleSpec(
        path="samples/table-001.hwp",
        role="병합 보존·치수 거부",
        table_count=1,
        focus_index=0,
        focus_rows=19,
        focus_cols=9,
        focus_merged=20,
        focus_cell_count=131,
        notes="19×9=171 이 아니라 cellCount 131. 가로 colSpan=3 · 세로 rowSpan=3.",
    ),
    SampleSpec(
        path="samples/inner-table-01.hwp",
        role="중첩 표 v1 밖",
        table_count=1,
        focus_index=0,
        notes="바깥 14칸 중 1칸이 중첩 24칸. nested 는 --table 이 아니다.",
    ),
    SampleSpec(
        path="samples/basic/treatise sample.hwp",
        role="컨테이너 표 vs info",
        table_count=3,
        notes="info 표 열거 1, export-tables 3. 머리말 표에 containerPath.",
    ),
    SampleSpec(
        path="samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx",
        role="index 0 = 머리말, 표 53",
        table_count=53,
        focus_index=12,
        notes="index 0 은 header. 본문 최상위는 더 큰 index. 형식 hwpx.",
    ),
    SampleSpec(
        path="samples/basic/issue2007_nested_cell_pagination_42065.hwp",
        role="코덱스 20장 실측",
        table_count=5,
        focus_index=1,
        focus_rows=2,
        focus_cols=3,
        focus_merged=0,
        notes="t0 는 2×1 개요, t1 이 데이터 표.",
    ),
    SampleSpec(
        path="samples/multi-table-001.hwp",
        role="--table 지목",
        table_count=6,
        notes="표 6개. --table 없이 table-to-csv -o 는 폴더.",
    ),
    SampleSpec(
        path="samples/hwpx/basic-table-01.hwpx",
        role="cli_commands 사용 예",
        notes="HWPX 왕복은 outputFormat=hwpx.",
    ),
    SampleSpec(
        path="samples/복학원서.hwp",
        role="누름틀 0 → set-cell",
        table_count=3,
        notes="필드 0. 표 셀은 set-cell / csv-to-table.",
    ),
    SampleSpec(
        path="samples/추진일정.hwp",
        role="싼 왕복",
        table_count=1,
    ),
)


@dataclass(frozen=True)
class ShapeSpec:
    shape_id: str
    rows: int
    cols: int
    merged_count: int
    csv_roundtrip: str
    reason: str
    next_action: str
    sample: str | None = None
    index: int | None = None
    container: str | None = None


SHAPES: tuple[ShapeSpec, ...] = (
    ShapeSpec("hwp_table_test_t0", 4, 3, 0, "allowed", "모든 셀 span=1. 레시피 02.", "table-to-csv --table 0", "samples/hwp_table_test.hwp", 0),
    ShapeSpec("issue2007_t0", 2, 1, 0, "allowed", "1열×2행 개요 표.", "table-to-csv --table 0", "samples/basic/issue2007_nested_cell_pagination_42065.hwp", 0),
    ShapeSpec("issue2007_t1", 2, 3, 0, "allowed", "코덱스 20장 실측 표본.", "table-to-csv --table 1", "samples/basic/issue2007_nested_cell_pagination_42065.hwp", 1),
    ShapeSpec("table_001", 19, 9, 20, "extract-only", "병합 20. 되돌리면 coveredCellNotEmpty 위험.", "edit set-cell --table 0", "samples/table-001.hwp", 0),
    ShapeSpec("wrapper_1x1", 1, 1, 0, "skip", "1×1 래퍼는 데이터 표가 아니다.", "다음 index", None, None),
    ShapeSpec("shape_2x2_m0", 2, 2, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_2x1_m0", 2, 1, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_3x4_m0", 3, 4, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_4x3_m0", 4, 3, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_5x5_m0", 5, 5, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_6x3_m2", 6, 3, 2, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_7x7_m0", 7, 7, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_8x4_m0", 8, 4, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_9x2_m0", 9, 2, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_10x6_m1", 10, 6, 1, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_11x11_m6", 11, 11, 6, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_12x8_m4", 12, 8, 4, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_15x4_m3", 15, 4, 3, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_16x3_m0", 16, 3, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_20x5_m0", 20, 5, 0, "allowed", "병합 0 → CSV 왕복", "table-to-csv --table N"),
    ShapeSpec("shape_4x8_m1", 4, 8, 1, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_24x6_m12", 24, 6, 12, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
    ShapeSpec("shape_30x10_m8", 30, 10, 8, "extract-only", "병합 있음 → 추출만", "edit set-cell"),
)


@dataclass(frozen=True)
class MergePattern:
    pattern_id: str
    rows: int
    cols: int
    spans: tuple[tuple[int, int, int, int], ...]
    note: str


# (row, col, rowSpan, colSpan) — documented or synthetic occupancy, not a writer.
MERGE_PATTERNS: tuple[MergePattern, ...] = (
    MergePattern("none_4x3", 4, 3, (), "레시피 02. 덮인 칸 없음."),
    MergePattern("none_2x3", 2, 3, (), "issue2007 t1."),
    MergePattern("none_2x1", 2, 1, (), "issue2007 t0."),
    MergePattern("colspan2_r0c0", 3, 4, ((0, 0, 1, 2),), "헤더 두 칸 가로 병합."),
    MergePattern("colspan2_r0c1", 3, 4, ((0, 1, 1, 2),), "헤더 중간 가로 병합."),
    MergePattern("colspan2_last_row", 4, 3, ((3, 0, 1, 2),), "마지막 행 가로 병합."),
    MergePattern("colspan3_header", 4, 4, ((0, 0, 1, 3),), "헤더 세 칸."),
    MergePattern("colspan3_mid", 5, 5, ((2, 1, 1, 3),), "본문 중간 가로 병합."),
    MergePattern("colspan4_full_row", 3, 4, ((0, 0, 1, 4),), "한 행 전체 가로 병합."),
    MergePattern("rowspan2_r0c0", 4, 3, ((0, 0, 2, 1),), "첫 열 세로 두 칸."),
    MergePattern("rowspan2_r1c2", 4, 3, ((1, 2, 2, 1),), "마지막 열 중간 세로."),
    MergePattern("rowspan3_note", 5, 4, ((0, 3, 3, 1),), "table-001 비고 열과 같은 형태."),
    MergePattern("block_2x2", 4, 4, ((1, 1, 2, 2),), "2×2 블록 병합."),
    MergePattern("block_2x3", 5, 5, ((1, 1, 2, 3),), "2×3 블록."),
    MergePattern("block_3x2", 5, 4, ((1, 1, 3, 2),), "3×2 블록."),
    MergePattern("two_colspan3", 4, 7, ((0, 1, 1, 3), (0, 4, 1, 3)), "table-001 5월/6월 헤더."),
    MergePattern("header_plus_note", 4, 5, ((0, 1, 1, 3), (0, 4, 3, 1)), "가로 헤더 + 세로 비고."),
    MergePattern("first_col_stack", 6, 3, ((0, 0, 3, 1), (3, 0, 3, 1)), "첫 열을 두 덩어리로."),
    MergePattern("checker_safe", 4, 4, ((0, 0, 1, 2), (2, 2, 2, 1)), "겹치지 않는 두 병합."),
    MergePattern("triple_header", 3, 6, ((0, 0, 1, 2), (0, 2, 1, 2), (0, 4, 1, 2)), "헤더 세 쌍."),
    MergePattern("table001_header", 19, 9, ((0, 1, 1, 3), (0, 4, 1, 3), (0, 7, 3, 1)), "table-001 문서화된 헤더 병합만."),
    MergePattern("wide_row0", 6, 8, ((0, 0, 1, 4), (0, 4, 1, 4)), "넓은 표 헤더 둘."),
    MergePattern("tall_col0", 8, 4, ((0, 0, 4, 1), (4, 0, 4, 1)), "키 큰 첫 열."),
    MergePattern("corner_l", 5, 5, ((0, 0, 1, 3), (1, 0, 2, 1)), "ㄱ자처럼 보이지만 두 앵커."),
    MergePattern("last_cell_span", 3, 3, ((1, 1, 2, 2),), "우측 하단 블록."),
    MergePattern("many_small", 8, 6, ((0, 0, 1, 2), (0, 2, 1, 2), (2, 4, 2, 1), (4, 0, 1, 3), (6, 3, 2, 2)), "작은 병합 여러 개."),
)


SOURCE_REFS = (
    "mydocs/manual/cli_commands.md §export-tables",
    "mydocs/manual/cli_commands.md §table-to-csv",
    "mydocs/manual/cli_commands.md §csv-to-table",
    "mydocs/manual/cli_commands.md §종료 코드 #2707",
    "mydocs/manual/recipes/02_table_csv_roundtrip.md",
    "tests/table_csv_contract.rs",
    "tests/table_extract_json_contract.rs",
    ".agents/skills/rhwp-table-exchange/references/csv_to_table_contract.md",
    ".agents/skills/rhwp-table-exchange/references/dry_run_verify.md",
    ".agents/skills/rhwp-table-exchange/references/export_tables_matrix.md",
    ".agents/skills/rhwp-table-exchange/references/table_to_csv_envelopes.md",
)


def output_format_for(path: str) -> str:
    lower = path.lower()
    if lower.endswith(".hwpx"):
        return "hwpx"
    return "hwp5"


def sample_by_path(path: str) -> SampleSpec:
    for sample in SAMPLES:
        if sample.path == path:
            return sample
    raise KeyError(path)


def cli_contract_public() -> dict[str, Any]:
    return {
        "schemaVersion": SCHEMA_VERSION,
        "skill": SKILL,
        "skillIssue": SKILL_ISSUE,
        "issue": ISSUE,
        "commands": list(COMMANDS),
        "fallbackCommand": FALLBACK_COMMAND,
        "flags": {name: list(flags) for name, flags in FLAGS.items()},
        "exits": list(EXIT_CODES),
        "exitKind": dict(EXIT_KIND),
        "invalidReasons": list(INVALID_REASONS),
        "reasonFields": {k: list(v) for k, v in REASON_FIELDS.items()},
        "reasonWrites": dict(REASON_WRITES),
        "csvRoundtrip": list(CSV_ROUNDTRIP),
        "families": list(FAMILIES),
        "modes": list(MODES),
        "untrustedFields": {k: list(v) for k, v in UNTRUSTED_FIELDS.items()},
        "note": "기존 CLI 만. 새 명령·편집 로직·gym 없음.",
    }
