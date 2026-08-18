"""Documented and synthetic table specs used by the case catalog.

Synthetic shapes exist only to exercise occupancy and dimension contracts.
They are not claimed to be binary dumps. Documented samples keep the
paths and sizes from the skill catalog.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .catalog import MERGE_PATTERNS, MergePattern
from .occupancy import Anchor, Occupancy, anchors_from_spans, build_occupancy


@dataclass
class TableSpec:
    spec_id: str
    sample: str
    index: int
    occupancy: Occupancy
    container: list[dict[str, Any]] | None = None
    nested: bool = False
    paragraph: int = 0
    control: int = 0
    documented: bool = False
    notes: str = ""

    @property
    def rows(self) -> int:
        return self.occupancy.rows

    @property
    def cols(self) -> int:
        return self.occupancy.cols

    @property
    def csv_roundtrip(self) -> str:
        from .occupancy import csv_roundtrip_for

        kind = None
        if self.container:
            kind = self.container[0].get("kind")
        return csv_roundtrip_for(self.occupancy, kind, self.nested)


def _texts(pairs: dict[tuple[int, int], str]) -> dict[tuple[int, int], str]:
    return pairs


def recipe02_table() -> TableSpec:
    texts = {
        (0, 0): "제목",
        (0, 1): "담당자",
        (0, 2): "세부 내용",
    }
    anchors = anchors_from_spans(4, 3, (), texts=texts)
    return TableSpec(
        "hwp_table_test_t0",
        "samples/hwp_table_test.hwp",
        0,
        build_occupancy(4, 3, anchors),
        paragraph=3,
        documented=True,
        notes="레시피 02. 3열×4행, 병합 0.",
    )


def issue2007_t0() -> TableSpec:
    texts = {(0, 0): "개요", (1, 0): "본문"}
    return TableSpec(
        "issue2007_t0",
        "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
        0,
        build_occupancy(2, 1, anchors_from_spans(2, 1, (), texts=texts)),
        documented=True,
        notes="1열×2행 개요.",
    )


def issue2007_t1() -> TableSpec:
    texts = {
        (0, 0): "항목",
        (0, 1): "값",
        (0, 2): "비고",
        (1, 0): "A",
        (1, 1): "1",
        (1, 2): "",
    }
    return TableSpec(
        "issue2007_t1",
        "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
        1,
        build_occupancy(2, 3, anchors_from_spans(2, 3, (), texts=texts)),
        documented=True,
        notes="코덱스 20장. --table 1.",
    )


def table001() -> TableSpec:
    texts = {
        (0, 0): "구 분",
        (0, 1): "5월",
        (0, 4): "6월",
        (0, 7): "비고",
    }
    headers = {(0, 1), (0, 4), (0, 7)}
    spans = ((0, 1, 1, 3), (0, 4, 1, 3), (0, 7, 3, 1))
    return TableSpec(
        "table_001",
        "samples/table-001.hwp",
        0,
        build_occupancy(
            19, 9, anchors_from_spans(19, 9, spans, texts=texts, headers=headers)
        ),
        paragraph=1,
        documented=True,
        notes="문서화된 헤더 병합만 모델. 전체 병합 20은 스킬 행렬 수치.",
    )


def wrapper_1x1() -> TableSpec:
    return TableSpec(
        "wrapper_1x1",
        "samples/복학원서.hwp",
        0,
        build_occupancy(1, 1, [Anchor(0, 0, 1, 1, "본문 래퍼", False)]),
        documented=True,
        notes="1×1 래퍼. csvRoundtrip=skip.",
    )


def inner_outer() -> TableSpec:
    anchors = anchors_from_spans(4, 4, (), texts={(0, 0): "바깥"})
    # mark one cell nested without inventing an inner --table
    rebuilt = []
    for anchor in anchors:
        if anchor.row == 1 and anchor.col == 1:
            rebuilt.append(Anchor(1, 1, 1, 1, "", False, nested=True))
        else:
            rebuilt.append(anchor)
    return TableSpec(
        "inner_table_outer",
        "samples/inner-table-01.hwp",
        0,
        build_occupancy(4, 4, rebuilt),
        nested=True,
        documented=True,
        notes="바깥 격자만 --table 0. 중첩은 v1 밖.",
    )


def jichi_header() -> TableSpec:
    return TableSpec(
        "jichi_header_zero",
        "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx",
        0,
        build_occupancy(2, 3, anchors_from_spans(2, 3, (), texts={(0, 0): "머리말"})),
        container=[{"kind": "header", "control": 0}],
        documented=True,
        notes="index 0 이 머리말. --table 후보 아님.",
    )


def jichi_body() -> TableSpec:
    return TableSpec(
        "jichi_body_12",
        "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx",
        12,
        build_occupancy(
            5,
            4,
            anchors_from_spans(
                5,
                4,
                (),
                texts={(0, 0): "연번", (0, 1): "항목", (0, 2): "수량", (0, 3): "비고"},
            ),
        ),
        documented=True,
        notes="본문 최상위 예시 index 12. 형식 hwpx.",
    )


def treatise_header() -> TableSpec:
    return TableSpec(
        "treatise_header",
        "samples/basic/treatise sample.hwp",
        2,
        build_occupancy(2, 2, anchors_from_spans(2, 2, ())),
        container=[{"kind": "header", "control": 0}],
        documented=True,
        notes="info 보다 넓은 수집. CSV 왕복 금지.",
    )


def treatise_body() -> TableSpec:
    return TableSpec(
        "treatise_body",
        "samples/basic/treatise sample.hwp",
        0,
        build_occupancy(
            3,
            3,
            anchors_from_spans(3, 3, (), texts={(0, 0): "장", (0, 1): "절", (0, 2): "쪽"}),
        ),
        documented=True,
        notes="본문 최상위. info 가 세는 그 표.",
    )


def chujin() -> TableSpec:
    texts = {
        (0, 0): "단계",
        (0, 1): "일정",
        (0, 2): "담당",
        (1, 0): "착수",
        (1, 1): "1월",
        (1, 2): "기획",
        (2, 0): "중간",
        (2, 1): "3월",
        (2, 2): "실무",
        (3, 0): "완료",
        (3, 1): "6월",
        (3, 2): "총괄",
    }
    return TableSpec(
        "chujin",
        "samples/추진일정.hwp",
        0,
        build_occupancy(4, 3, anchors_from_spans(4, 3, (), texts=texts)),
        documented=True,
        notes="싼 왕복. 4×3 병합 0.",
    )


def hwpx_basic() -> TableSpec:
    texts = {(0, 0): "이름", (0, 1): "점수", (1, 0): "가", (1, 1): "90"}
    return TableSpec(
        "hwpx_basic_01",
        "samples/hwpx/basic-table-01.hwpx",
        0,
        build_occupancy(2, 2, anchors_from_spans(2, 2, (), texts=texts)),
        documented=True,
        notes="HWPX 입력은 outputFormat=hwpx.",
    )


def from_pattern(
    pattern: MergePattern,
    *,
    sample: str = "synthetic/merge-pattern.hwpx",
    index: int = 0,
    texts: dict[tuple[int, int], str] | None = None,
    documented: bool = False,
) -> TableSpec:
    header_cells = {(row, col) for row, col, _, _ in pattern.spans if row == 0}
    return TableSpec(
        pattern.pattern_id,
        sample,
        index,
        build_occupancy(
            pattern.rows,
            pattern.cols,
            anchors_from_spans(
                pattern.rows, pattern.cols, pattern.spans, texts=texts, headers=header_cells
            ),
        ),
        documented=documented,
        notes=pattern.note,
    )


def labeled_grid(rows: int, cols: int, prefix: str) -> dict[tuple[int, int], str]:
    texts: dict[tuple[int, int], str] = {}
    for r in range(rows):
        for c in range(cols):
            if r == 0:
                texts[(0, c)] = f"{prefix}H{c}"
            else:
                texts[(r, c)] = f"{prefix}{r}_{c}"
    return texts


def unmerged(spec_id: str, rows: int, cols: int, sample: str, index: int = 0) -> TableSpec:
    texts = labeled_grid(rows, cols, spec_id.split("_")[0][:3])
    return TableSpec(
        spec_id,
        sample,
        index,
        build_occupancy(rows, cols, anchors_from_spans(rows, cols, (), texts=texts)),
        notes=f"합성 {rows}×{cols} 병합 0.",
    )


def documented_tables() -> list[TableSpec]:
    return [
        recipe02_table(),
        issue2007_t0(),
        issue2007_t1(),
        table001(),
        wrapper_1x1(),
        inner_outer(),
        jichi_header(),
        jichi_body(),
        treatise_header(),
        treatise_body(),
        chujin(),
        hwpx_basic(),
    ]


def pattern_tables() -> list[TableSpec]:
    tables: list[TableSpec] = []
    for pattern in MERGE_PATTERNS:
        texts = labeled_grid(pattern.rows, pattern.cols, pattern.pattern_id[:4])
        # keep documented header labels on table001 header pattern
        if pattern.pattern_id == "table001_header":
            texts.update({(0, 0): "구 분", (0, 1): "5월", (0, 4): "6월", (0, 7): "비고"})
            tables.append(
                from_pattern(
                    pattern,
                    sample="samples/table-001.hwp",
                    documented=True,
                    texts=texts,
                )
            )
        elif pattern.pattern_id == "none_4x3":
            tables.append(recipe02_table())
        elif pattern.pattern_id == "none_2x3":
            tables.append(issue2007_t1())
        elif pattern.pattern_id == "none_2x1":
            tables.append(issue2007_t0())
        else:
            tables.append(from_pattern(pattern, texts=texts))
    return tables


def unmerged_shape_tables() -> list[TableSpec]:
    specs = (
        ("shape_2x2_m0", 2, 2),
        ("shape_3x4_m0", 3, 4),
        ("shape_4x3_m0", 4, 3),
        ("shape_5x5_m0", 5, 5),
        ("shape_7x7_m0", 7, 7),
        ("shape_8x4_m0", 8, 4),
        ("shape_9x2_m0", 9, 2),
        ("shape_16x3_m0", 16, 3),
        ("shape_20x5_m0", 20, 5),
    )
    return [
        unmerged(spec_id, rows, cols, f"synthetic/{spec_id}.hwp")
        for spec_id, rows, cols in specs
    ]


def all_base_tables() -> list[TableSpec]:
    seen: set[str] = set()
    out: list[TableSpec] = []
    for table in documented_tables() + pattern_tables() + unmerged_shape_tables():
        if table.spec_id in seen:
            continue
        seen.add(table.spec_id)
        out.append(table)
    return out
