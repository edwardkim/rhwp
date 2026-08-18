"""HWPX→HWP 저장 계약 단위 픽스처.

각 케이스는 한 개의 HWPX construct 와 한컴 oracle / rhwp generated
HWP5 튜플 차이를 고정한다. 페이지 수 로직(#4882) 은 다루지 않는다.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable

from .catalog import TAGS, ctrl_id
from .model import (
    InventoryItem,
    make_item,
    pack_table_ctrl_payload,
    pack_table_payload,
)


DOCINFO = "/DocInfo"
BODY0 = "/BodyText/Section0"
BODY1 = "/BodyText/Section1"


@dataclass
class ContractCase:
    case_id: str
    sample: str
    construct: str
    family: str
    failure_class: str
    hancom_judgment: str
    align_preferred: str
    focus: str
    probe_axes: tuple[str, ...]
    next_probe: str
    lowering_contract: str
    contract_status: str
    oracle_path: str
    generated_path: str
    notes: tuple[str, ...]
    build: Callable[[], tuple[list[InventoryItem], list[InventoryItem]]]
    report_modes: tuple[str, ...] = (
        "diff",
        "hints",
        "bundles",
        "table-fields",
        "table-probe-plan",
    )


def _para_tuple(
    sample: str,
    source: str,
    start: int,
    *,
    text: bytes,
    char_shape: bytes,
    line_seg: bytes,
    char_count_hint: bytes | None = None,
    include_char_shape: bool = True,
    include_line_seg: bool = True,
    extra_range: bytes | None = None,
    body_order: int = 0,
) -> list[InventoryItem]:
    header = char_count_hint if char_count_hint is not None else bytes([len(text) // 2, 0, 0, 0])
    items = [
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start,
            tag_name="PARA_HEADER",
            level=0,
            payload=header + b"\x00\x00\x01\x00\x00\x00\x00\x00",
            body_order=body_order,
            tuple_index=body_order,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 1,
            tag_name="PARA_TEXT",
            level=1,
            payload=text,
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"PARA_HEADER#{start}@lv0",
            body_order=body_order,
            tuple_index=body_order,
        ),
    ]
    next_index = start + 2
    if include_char_shape:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="PARA_CHAR_SHAPE",
                level=1,
                payload=char_shape,
                parent_uid=f"BodyText.Section0#{start}",
                parent_scope=f"PARA_HEADER#{start}@lv0",
                body_order=body_order,
                tuple_index=body_order,
            )
        )
        next_index += 1
    if include_line_seg:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="PARA_LINE_SEG",
                level=1,
                payload=line_seg,
                parent_uid=f"BodyText.Section0#{start}",
                parent_scope=f"PARA_HEADER#{start}@lv0",
                body_order=body_order,
                tuple_index=body_order,
            )
        )
        next_index += 1
    if extra_range is not None:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="PARA_RANGE_TAG",
                level=1,
                payload=extra_range,
                parent_uid=f"BodyText.Section0#{start}",
                parent_scope=f"PARA_HEADER#{start}@lv0",
                body_order=body_order,
                tuple_index=body_order,
            )
        )
    return items


def _docinfo_base(sample: str, source: str, *, section_count: int = 1, bin_count: int = 0) -> list[InventoryItem]:
    mappings = bytes(
        [
            bin_count,
            0,
            1,
            0,
            2,
            0,
            1,
            0,
            1,
            0,
            0,
            0,
            0,
            0,
            1,
            0,
            1,
            0,
        ]
    )
    items = [
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=0,
            tag_name="DOCUMENT_PROPERTIES",
            level=0,
            payload=bytes([section_count, 0, 0, 0, 0, 0, 0, 0]),
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=1,
            tag_name="ID_MAPPINGS",
            level=0,
            payload=mappings,
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=2,
            tag_name="FACE_NAME",
            level=1,
            payload=b"Batang\x00",
            parent_uid="DocInfo#1",
            parent_scope="ID_MAPPINGS#1@lv0",
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=3,
            tag_name="BORDER_FILL",
            level=1,
            payload=b"\x00\x00\x00\x00\x01\x00",
            parent_uid="DocInfo#1",
            parent_scope="ID_MAPPINGS#1@lv0",
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=4,
            tag_name="CHAR_SHAPE",
            level=1,
            payload=b"\x00\x00\x14\x00\x00\x00",
            parent_uid="DocInfo#1",
            parent_scope="ID_MAPPINGS#1@lv0",
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=5,
            tag_name="PARA_SHAPE",
            level=1,
            payload=b"\x00\x00\x00\x00\x00\x00\x00\x00",
            parent_uid="DocInfo#1",
            parent_scope="ID_MAPPINGS#1@lv0",
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=DOCINFO,
            record_index=6,
            tag_name="STYLE",
            level=1,
            payload=b"Normal\x00",
            parent_uid="DocInfo#1",
            parent_scope="ID_MAPPINGS#1@lv0",
            tuple_index=0,
        ),
    ]
    if bin_count:
        items.insert(
            2,
            make_item(
                sample=sample,
                source_path=source,
                stream_path=DOCINFO,
                record_index=2,
                tag_name="BIN_DATA",
                level=1,
                payload=b"\x01\x00BIN0001.jpg\x00",
                parent_uid="DocInfo#1",
                parent_scope="ID_MAPPINGS#1@lv0",
                tuple_index=0,
            ),
        )
        for index, item in enumerate(items):
            if item.stream_path == DOCINFO:
                item.record_index = index
                item.record_uid = f"DocInfo#{index}"
    return items


def _section_def(sample: str, source: str, start: int) -> list[InventoryItem]:
    return [
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start,
            tag_name="CTRL_HEADER",
            level=0,
            payload=b"dces" + b"\x00" * 8,
            control_fourcc="secd",
            body_order=0,
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 1,
            tag_name="PAGE_DEF",
            level=1,
            payload=b"\x70\x17\x00\x00\x20\x1c\x00\x00",
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"CTRL_HEADER#{start}@lv0",
            body_order=0,
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 2,
            tag_name="FOOTNOTE_SHAPE",
            level=1,
            payload=b"\x00\x00\x00\x00",
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"CTRL_HEADER#{start}@lv0",
            body_order=0,
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 3,
            tag_name="PAGE_BORDER_FILL",
            level=1,
            payload=b"\x00\x00\x00\x00\x00\x00",
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"CTRL_HEADER#{start}@lv0",
            body_order=0,
            tuple_index=0,
        ),
    ]


def _table_tuple(
    sample: str,
    source: str,
    start: int,
    *,
    ctrl_payload: bytes,
    table_payload: bytes,
    list_payload: bytes = b"\x01\x00\x00\x00",
    include_list: bool = True,
    include_cell_para: bool = True,
    body_order: int = 1,
    tuple_index: int = 0,
) -> list[InventoryItem]:
    items = [
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start,
            tag_name="CTRL_HEADER",
            level=0,
            payload=ctrl_payload,
            control_fourcc="tbl ",
            body_order=body_order,
            tuple_index=tuple_index,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 1,
            tag_name="TABLE",
            level=1,
            payload=table_payload,
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"CTRL_HEADER#{start}@lv0",
            body_order=body_order,
            tuple_index=tuple_index,
        ),
    ]
    next_index = start + 2
    if include_list:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="LIST_HEADER",
                level=1,
                payload=list_payload,
                parent_uid=f"BodyText.Section0#{start}",
                parent_scope=f"CTRL_HEADER#{start}@lv0",
                body_order=body_order,
                tuple_index=tuple_index,
            )
        )
        next_index += 1
        if include_cell_para:
            items.extend(
                _para_tuple(
                    sample,
                    source,
                    next_index,
                    text="가".encode("utf-16-le"),
                    char_shape=b"\x00\x00\x00\x00",
                    line_seg=b"\x00\x00\x20\x00\x10\x00",
                    body_order=body_order + 1,
                )
            )
    return items


def _shape_tuple(
    sample: str,
    source: str,
    start: int,
    *,
    matrix: bytes,
    include_picture: bool = True,
    include_ctrl_data: bool = True,
    body_order: int = 1,
) -> list[InventoryItem]:
    items = [
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start,
            tag_name="CTRL_HEADER",
            level=0,
            payload=b" osg" + b"\x00" * 12,
            control_fourcc="gso ",
            body_order=body_order,
            tuple_index=0,
        ),
        make_item(
            sample=sample,
            source_path=source,
            stream_path=BODY0,
            record_index=start + 1,
            tag_name="SHAPE_COMPONENT",
            level=1,
            payload=b"$pic" + matrix,
            parent_uid=f"BodyText.Section0#{start}",
            parent_scope=f"CTRL_HEADER#{start}@lv0",
            body_order=body_order,
            tuple_index=0,
        ),
    ]
    next_index = start + 2
    if include_picture:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="SHAPE_PICTURE",
                level=2,
                payload=b"\x01\x00\x00\x00BIN0001\x00",
                parent_uid=f"BodyText.Section0#{start + 1}",
                parent_scope=f"SHAPE_COMPONENT#{start + 1}@lv1",
                body_order=body_order,
                tuple_index=0,
            )
        )
        next_index += 1
    if include_ctrl_data:
        items.append(
            make_item(
                sample=sample,
                source_path=source,
                stream_path=BODY0,
                record_index=next_index,
                tag_name="CTRL_DATA",
                level=1,
                payload=b"\x01\x00shape\x00",
                parent_uid=f"BodyText.Section0#{start}",
                parent_scope=f"CTRL_HEADER#{start}@lv0",
                body_order=body_order,
                tuple_index=0,
            )
        )
    return items


def _clone_items(items: list[InventoryItem], source_path: str) -> list[InventoryItem]:
    cloned = []
    for item in items:
        copy = InventoryItem(**{**item.__dict__, "source_path": source_path})
        cloned.append(copy)
    return cloned


def _replace_payload(item: InventoryItem, payload: bytes, **kwargs: object) -> InventoryItem:
    data = item.__dict__.copy()
    data.update(kwargs)
    rebuilt = make_item(
        sample=item.sample,
        source_path=str(data.get("source_path", item.source_path)),
        stream_path=item.stream_path,
        record_index=item.record_index,
        tag_name=item.tag_name,
        level=item.level,
        payload=payload,
        control_fourcc=None,
        control_name=item.control_name,
        parent_uid=item.parent_uid,
        parent_scope=item.parent_scope,
        body_order=item.body_order,
        tuple_index=item.tuple_index,
        note=item.note,
    )
    if item.control_id:
        rebuilt.control_id = item.control_id
        rebuilt.control_name = item.control_name
    return rebuilt


ORACLE_CTRL = pack_table_ctrl_payload(
    common_attr=0x00000001,
    x=1000,
    y=2000,
    width=8000,
    height=4000,
    out_margin=(80, 80, 40, 40),
    tail=b"\x11\x22",
)
GENERATED_CTRL_ZERO_MARGIN = pack_table_ctrl_payload(
    common_attr=0x00000001,
    x=1000,
    y=2000,
    width=8000,
    height=4000,
    out_margin=(0, 0, 0, 0),
    tail=b"\x11\x22",
)
GENERATED_CTRL_ATTR = pack_table_ctrl_payload(
    common_attr=0x00000000,
    x=1000,
    y=2000,
    width=8000,
    height=4000,
    out_margin=(80, 80, 40, 40),
    tail=b"\x11\x22",
)
ORACLE_TABLE = pack_table_payload(table_attr=0x00000003, rows=2, cols=3, tail=b"\xab\xcd")
GENERATED_TABLE_ATTR = pack_table_payload(table_attr=0x00000000, rows=2, cols=3, tail=b"\xab\xcd")
GENERATED_TABLE_TAIL = pack_table_payload(table_attr=0x00000003, rows=2, cols=3, tail=b"\x00\x00")
GENERATED_TABLE_COUNT = pack_table_payload(table_attr=0x00000003, rows=1, cols=3, tail=b"\xab\xcd")


def _paths(sample: str) -> tuple[str, str]:
    return (
        f"samples/hwpx/hancom-hwp/{sample}.hwp",
        f"output/poc/hwpx2hwp/inventory/{sample}.hwp",
    )


def _case(
    case_id: str,
    sample: str,
    construct: str,
    family: str,
    failure_class: str,
    hancom_judgment: str,
    align_preferred: str,
    focus: str,
    probe_axes: tuple[str, ...],
    next_probe: str,
    lowering_contract: str,
    contract_status: str,
    notes: tuple[str, ...],
    builder: Callable[[str, str, str], tuple[list[InventoryItem], list[InventoryItem]]],
) -> ContractCase:
    oracle_path, generated_path = _paths(sample)

    def build() -> tuple[list[InventoryItem], list[InventoryItem]]:
        return builder(sample, oracle_path, generated_path)

    return ContractCase(
        case_id=case_id,
        sample=sample,
        construct=construct,
        family=family,
        failure_class=failure_class,
        hancom_judgment=hancom_judgment,
        align_preferred=align_preferred,
        focus=focus,
        probe_axes=probe_axes,
        next_probe=next_probe,
        lowering_contract=lowering_contract,
        contract_status=contract_status,
        oracle_path=oracle_path,
        generated_path=generated_path,
        notes=notes,
        build=build,
    )


def _table_axis_builder(
    oracle_ctrl: bytes,
    oracle_table: bytes,
    generated_ctrl: bytes,
    generated_table: bytes,
    *,
    include_list_oracle: bool = True,
    include_list_generated: bool = True,
    extra_generated: Callable[[str, str, int], list[InventoryItem]] | None = None,
):
    def build(sample: str, oracle_path: str, generated_path: str):
        oracle = _docinfo_base(sample, oracle_path) + _section_def(sample, oracle_path, 0)
        generated = _docinfo_base(sample, generated_path) + _section_def(sample, generated_path, 0)
        oracle.extend(
            _table_tuple(
                sample,
                oracle_path,
                4,
                ctrl_payload=oracle_ctrl,
                table_payload=oracle_table,
                include_list=include_list_oracle,
            )
        )
        generated.extend(
            _table_tuple(
                sample,
                generated_path,
                4,
                ctrl_payload=generated_ctrl,
                table_payload=generated_table,
                include_list=include_list_generated,
            )
        )
        if extra_generated:
            generated.extend(extra_generated(sample, generated_path, 20))
        return oracle, generated

    return build


def _simple_delta(
    mutate_oracle: Callable[[list[InventoryItem]], None] | None = None,
    mutate_generated: Callable[[list[InventoryItem]], None] | None = None,
    *,
    bin_count: int = 0,
    with_shape: bool = False,
    with_table: bool = False,
    section_count_oracle: int = 1,
    section_count_generated: int = 1,
):
    def build(sample: str, oracle_path: str, generated_path: str):
        oracle = _docinfo_base(
            sample, oracle_path, section_count=section_count_oracle, bin_count=bin_count
        ) + _section_def(sample, oracle_path, 0)
        generated = _docinfo_base(
            sample,
            generated_path,
            section_count=section_count_generated,
            bin_count=bin_count if mutate_generated is None else bin_count,
        ) + _section_def(sample, generated_path, 0)
        if with_table:
            oracle.extend(
                _table_tuple(
                    sample,
                    oracle_path,
                    4,
                    ctrl_payload=ORACLE_CTRL,
                    table_payload=ORACLE_TABLE,
                )
            )
            generated.extend(
                _table_tuple(
                    sample,
                    generated_path,
                    4,
                    ctrl_payload=ORACLE_CTRL,
                    table_payload=ORACLE_TABLE,
                )
            )
        if with_shape:
            matrix = b"\x00\x00\x80\x3f" * 6
            oracle.extend(_shape_tuple(sample, oracle_path, 20, matrix=matrix))
            generated.extend(_shape_tuple(sample, generated_path, 20, matrix=matrix))
        if mutate_oracle:
            mutate_oracle(oracle)
        if mutate_generated:
            mutate_generated(generated)
        return oracle, generated

    return build


def _drop_tag(tag_name: str):
    def mutate(items: list[InventoryItem]) -> None:
        items[:] = [item for item in items if item.tag_name != tag_name]

    return mutate


def _rewrite_tag(tag_name: str, payload: bytes):
    def mutate(items: list[InventoryItem]) -> None:
        for index, item in enumerate(items):
            if item.tag_name == tag_name:
                items[index] = _replace_payload(item, payload)
                return

    return mutate


def _append_item(factory: Callable[[str, str], InventoryItem]):
    def mutate(items: list[InventoryItem]) -> None:
        if not items:
            return
        items.append(factory(items[0].sample, items[0].source_path))

    return mutate


CASES: list[ContractCase] = [
    _case(
        "T01",
        "hwpx-h-01-outer-margin",
        "hp:tbl / CTRL_HEADER outer margin",
        "table",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "table",
        ("ctrl_outer_margin",),
        "01_ctrl_outer_margin_only",
        "HWPX 표 바깥 여백이 없으면 한컴 oracle CTRL_HEADER 0x1c..0x23 값을 채운다.",
        "violated",
        (
            "표가 종이 왼쪽에 붙는 관찰과 같은 축이다.",
            "페이지 수는 이 케이스가 재지 않는다.",
        ),
        _table_axis_builder(ORACLE_CTRL, ORACLE_TABLE, GENERATED_CTRL_ZERO_MARGIN, ORACLE_TABLE),
    ),
    _case(
        "T02",
        "hwpx-h-01-table-attr",
        "hp:tbl / TABLE.table_attr",
        "table",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "table",
        ("table_attr",),
        "02_table_attr_only",
        "TABLE 첫 4바이트는 HWPX 속성 복사가 아니라 oracle attr 비트다.",
        "violated",
        (
            "Stage 58 시각 후보다. 규칙 승격은 이 inventory 대조 뒤다.",
        ),
        _table_axis_builder(ORACLE_CTRL, ORACLE_TABLE, ORACLE_CTRL, GENERATED_TABLE_ATTR),
    ),
    _case(
        "T03",
        "hwpx-h-01-table-tail",
        "hp:tbl / TABLE tail after 0x16",
        "table",
        "C",
        "파일 손상",
        "lcs",
        "table",
        ("table_tail",),
        "03_table_tail_only",
        "TABLE 0x16 이후 tail 은 관찰 필드다. 길이/바이트가 oracle 과 같아야 한다.",
        "violated",
        (
            "tail_after_0x16 은 확정 계약 이름이 아니다.",
        ),
        _table_axis_builder(ORACLE_CTRL, ORACLE_TABLE, ORACLE_CTRL, GENERATED_TABLE_TAIL),
    ),
    _case(
        "T04",
        "hwpx-h-01-common-attr",
        "hp:tbl / CTRL_HEADER.common_attr",
        "table",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "table",
        ("ctrl_common_attr",),
        "04_ctrl_common_attr_only",
        "common_attr 비트는 표 흐름/배치 후보. 한 축만 이식해 확인한다.",
        "violated",
        ("outer margin 과 결합하면 05 variant.",),
        _table_axis_builder(ORACLE_CTRL, ORACLE_TABLE, GENERATED_CTRL_ATTR, ORACLE_TABLE),
    ),
    _case(
        "T05",
        "hwpx-h-01-missing-list-header",
        "hp:tbl / LIST_HEADER subtree",
        "table",
        "B",
        "파일 손상",
        "lcs",
        "missing",
        (),
        "oracle LIST_HEADER graft",
        "CTRL_HEADER(Table)+TABLE 다음에는 LIST_HEADER 와 셀 문단 튜플이 와야 한다.",
        "violated",
        ("마지막 출력 위치는 첫 표 직후.",),
        _table_axis_builder(
            ORACLE_CTRL,
            ORACLE_TABLE,
            ORACLE_CTRL,
            ORACLE_TABLE,
            include_list_generated=False,
        ),
    ),
    _case(
        "T06",
        "hwpx-h-02-row-count",
        "hp:tbl / TABLE.rows vs LIST_HEADER count",
        "table",
        "C",
        "파일 손상",
        "index",
        "table",
        ("table_attr",),
        "row/col count 필드 좁히기",
        "TABLE.rows 와 실제 LIST_HEADER 개수가 같아야 한다.",
        "violated",
        ("count 불일치는 다음 셀에서 밀린 듯 보인다.",),
        _table_axis_builder(ORACLE_CTRL, ORACLE_TABLE, ORACLE_CTRL, GENERATED_TABLE_COUNT),
    ),
    _case(
        "T07",
        "hwpx-h-02-extra-cell",
        "hp:tbl / extra cell paragraph",
        "table",
        "B",
        "파일 손상",
        "lcs",
        "table",
        (),
        "extra PARA_HEADER 제거",
        "oracle 에 없는 셀 문단을 넣으면 extra 다. 한컴은 범위 밖으로 읽는다.",
        "violated",
        ("LCS 가 중간 삽입을 분리한다. index 는 uid 가 밀린다.",),
        _table_axis_builder(
            ORACLE_CTRL,
            ORACLE_TABLE,
            ORACLE_CTRL,
            ORACLE_TABLE,
            extra_generated=lambda sample, path, start: _para_tuple(
                sample,
                path,
                start,
                text="추가".encode("utf-16-le"),
                char_shape=b"\x00\x00",
                line_seg=b"\x00\x00\x10\x00",
                body_order=9,
            ),
        ),
    ),
    _case(
        "T08",
        "hwpx-h-01-all-table-axes",
        "hp:tbl / four table-probe axes together",
        "table",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "table",
        ("ctrl_outer_margin", "ctrl_common_attr", "table_attr", "table_tail"),
        "08_all_table_axes",
        "네 축을 한 번에 맞추는 것은 positive guard 이지 원인 분리가 아니다.",
        "violated",
        ("단독 variant 없이 08 만 성공하면 승격하지 않는다.",),
        _table_axis_builder(
            ORACLE_CTRL,
            ORACLE_TABLE,
            GENERATED_CTRL_ZERO_MARGIN.replace(b"\x01\x00\x00\x00", b"\x00\x00\x00\x00", 1)
            if False
            else pack_table_ctrl_payload(
                common_attr=0,
                x=1000,
                y=2000,
                width=8000,
                height=4000,
                out_margin=(0, 0, 0, 0),
                tail=b"\x11\x22",
            ),
            pack_table_payload(table_attr=0, rows=2, cols=3, tail=b"\x00\x00"),
        ),
    ),
    _case(
        "T09",
        "hwpx-h-03-nested-table",
        "hp:tbl inside hp:tbl",
        "table",
        "B",
        "파일 손상",
        "lcs",
        "table",
        ("table_attr",),
        "중첩 표 LIST_HEADER 범위",
        "안쪽 표는 바깥 LIST_HEADER 자식이지 형제 CTRL_HEADER 가 아니다.",
        "violated",
        ("scope_path 가 바깥 TABLE 아래로 내려가야 한다.",),
        _simple_delta(with_table=True, mutate_generated=_drop_tag("LIST_HEADER")),
    ),
    _case(
        "T10",
        "hwpx-h-header-table",
        "hp:header / hp:tbl",
        "table",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        ("ctrl_outer_margin",),
        "머리말 안 표는 Header LIST_HEADER 범위 안",
        "머리말 목록 밖의 표 CTRL_HEADER 는 트리 계약 위반이다.",
        "violated",
        ("Header 와 Table 을 한 튜플로 묶는다.",),
        _simple_delta(
            with_table=True,
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=30,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"daeh" + b"\x00" * 8,
                    control_fourcc="head",
                    body_order=8,
                )
            ),
        ),
    ),
    _case(
        "S01",
        "hwpx-h-01-bindata-missing",
        "hp:pic / DocInfo BIN_DATA",
        "shape",
        "D",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "BIN_DATA + SHAPE_PICTURE 동시 대조",
        "그림은 BIN_DATA record 와 CFB BIN0001 스트림과 SHAPE_PICTURE.bin_data_id 가 한 튜플.",
        "violated",
        ("이미지 미출력. 그림 경로 찾기 대화상자가 신호다.",),
        _simple_delta(
            bin_count=1,
            with_shape=True,
            mutate_generated=_drop_tag("BIN_DATA"),
        ),
    ),
    _case(
        "S02",
        "hwpx-h-03-shape-matrix-f32",
        "hp:pic / SHAPE_COMPONENT rendering matrix",
        "shape",
        "C",
        "파일 손상",
        "lcs",
        "shape",
        (),
        "f32→f64 양자화 payload 대조",
        "소수 matrix 는 f32 로 양자화한 뒤 f64 슬롯에 넣는다. 표시값이 같아도 hash 가 다르다.",
        "violated",
        ("필드 디코더만 보면 같은 값으로 보인다. payload_hash 를 본다.",),
        _simple_delta(
            bin_count=1,
            with_shape=True,
            mutate_generated=_rewrite_tag(
                "SHAPE_COMPONENT",
                b"$pic" + b"\x00\x00\x80\x3f\x01\x00\x00\x00" + b"\x00" * 16,
            ),
        ),
    ),
    _case(
        "S03",
        "hwpx-h-03-missing-shape-component",
        "hp:pic / SHAPE_COMPONENT after GenShape",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "shape",
        (),
        "CTRL_HEADER(GenShape) 다음 SHAPE_COMPONENT graft",
        "gso 컨트롤 다음은 SHAPE_COMPONENT 다. 빠지면 그림 직후 손상.",
        "violated",
        ("마지막 출력 위치는 그림 개체 묶기 전.",),
        _simple_delta(bin_count=1, with_shape=True, mutate_generated=_drop_tag("SHAPE_COMPONENT")),
    ),
    _case(
        "S04",
        "hwpx-h-03-missing-ctrl-data",
        "hp:pic / CTRL_DATA ParameterSet",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "hwp5-ctrl-data-trace",
        "그림/도형 ParameterSet 은 CTRL_DATA 로 내려간다. 위치/레벨/크기를 본다.",
        "violated",
        ("ctrl-data-trace 가 다음 도구다. 이 파동은 inventory 만.",),
        _simple_delta(bin_count=1, with_shape=True, mutate_generated=_drop_tag("CTRL_DATA")),
    ),
    _case(
        "S05",
        "hwpx-h-02-picture-in-cell",
        "hp:tbl cell / hp:pic",
        "shape",
        "D",
        "열림 + 조판 실패",
        "lcs",
        "shape",
        (),
        "셀 안 그림은 TABLE 튜플 + BIN_DATA 동시",
        "셀 안 그림은 표 트리와 BinData 를 분리해서 보지 않는다.",
        "violated",
        ("hwpx-h-01 Stage 53 관찰과 같은 결합 축.",),
        _simple_delta(bin_count=1, with_table=True, with_shape=True, mutate_generated=_drop_tag("SHAPE_PICTURE")),
    ),
    _case(
        "S06",
        "hwpx-ole-shape",
        "hp:ole / SHAPE_OLE",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "shape",
        (),
        "SHAPE_OLE + BIN_DATA",
        "OLE 는 $ole 와 SHAPE_OLE 레코드가 짝. 차트 데이터와 섞지 않는다.",
        "violated",
        ("#4669 축과 인접하지만 페이지 수와 무관.",),
        _simple_delta(
            bin_count=1,
            with_shape=True,
            mutate_generated=_drop_tag("SHAPE_PICTURE"),
        ),
    ),
    _case(
        "S07",
        "hwpx-container-group",
        "hp:container / SHAPE_CONTAINER",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "shape",
        (),
        "컨테이너 자식 개수",
        "묶음 컨테이너의 자식 SHAPE_COMPONENT 수가 payload 와 같아야 한다.",
        "violated",
        ("그룹 도형 직후 손상이 신호다.",),
        _simple_delta(
            with_shape=True,
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=40,
                    tag_name="SHAPE_CONTAINER",
                    level=1,
                    payload=b"$con\x02\x00",
                    body_order=4,
                )
            ),
        ),
    ),
    _case(
        "P01",
        "hwpx-lineseg-copy",
        "hp:p / lineSegArray copied into PARA_LINE_SEG",
        "para",
        "F",
        "열림 + 조판 실패",
        "index",
        "all",
        (),
        "oracle PARA_LINE_SEG 만 정답",
        "HWPX lineSegArray 는 PARA_LINE_SEG 정답이 아니다. 직접 복사 금지.",
        "violated",
        (
            "페이지 나눔 결과가 달라도 이 파동은 쪽수 로직을 고치지 않는다.",
            "inventory 는 payload_hash 만 남긴다.",
        ),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="줄".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x28\x00\x14\x00\x00\x00",
                    body_order=2,
                )
            ),
            mutate_generated=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="줄".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\xff\xff\x10\x00\x08\x00\x00\x00",
                    body_order=2,
                )
            ),
        ),
    ),
    _case(
        "P02",
        "hwpx-para-char-count",
        "hp:p / PARA_HEADER.char_count",
        "para",
        "C",
        "파일 손상",
        "index",
        "all",
        (),
        "char_count == PARA_TEXT code units",
        "char_count 와 PARA_TEXT UTF-16 code unit 수가 같아야 한다.",
        "violated",
        ("문단 직후 손상이 신호다.",),
        _simple_delta(
            mutate_generated=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="한글".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x18\x00",
                    char_count_hint=b"\x01\x00\x00\x00",
                    body_order=2,
                )
            )
            or items.extend(
                []
            ),
        ),
    ),
    _case(
        "P03",
        "hwpx-missing-char-shape",
        "hp:p / PARA_CHAR_SHAPE",
        "para",
        "B",
        "파일 손상",
        "lcs",
        "missing",
        (),
        "PARA_CHAR_SHAPE graft",
        "PARA_HEADER 튜플은 TEXT + CHAR_SHAPE + LINE_SEG 를 함께 둔다.",
        "violated",
        ("CHAR_SHAPE 누락은 문단 직후 손상.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="본문".encode("utf-16-le"),
                    char_shape=b"\x00\x00\x00\x00",
                    line_seg=b"\x00\x00\x18\x00",
                    body_order=2,
                )
            ),
            mutate_generated=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="본문".encode("utf-16-le"),
                    char_shape=b"\x00\x00\x00\x00",
                    line_seg=b"\x00\x00\x18\x00",
                    include_char_shape=False,
                    body_order=2,
                )
            ),
        ),
    ),
    _case(
        "P04",
        "hwpx-extra-range-tag",
        "hp:p / extra PARA_RANGE_TAG",
        "para",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "all",
        (),
        "과잉 RANGE_TAG 제거",
        "oracle 에 없는 RANGE_TAG 는 extra. 의미 없는 삽입을 금지한다.",
        "violated",
        ("LCS 가 extra 한 줄을 남긴다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="범위".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=2,
                )
            ),
            mutate_generated=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="범위".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    extra_range=b"\x01\x00\x02\x00",
                    body_order=2,
                )
            ),
        ),
    ),
    _case(
        "P05",
        "hwpx-lineseg-tac-mix",
        "hp:p + TAC table mix / PARA_LINE_SEG",
        "para",
        "F",
        "열림 + 조판 실패",
        "lcs",
        "all",
        (),
        "혼합 문단은 oracle lineSeg 우선",
        "텍스트+TAC 표 혼합 문단에서 lineSegArray 보다 oracle tuple 을 우선한다.",
        "violated",
        ("reflow 값을 HWP5 contract 로 쓰지 않는다.",),
        _simple_delta(
            with_table=True,
            mutate_generated=_rewrite_tag("TABLE", GENERATED_TABLE_ATTR),
        ),
    ),
    _case(
        "D01",
        "hwpx-section-count",
        "hsz:secPr / DOCUMENT_PROPERTIES.section_count",
        "docinfo",
        "A",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "section_count == BodyText section streams",
        "DocProperties.section_count 는 실제 섹션 수와 맞아야 한다. 쪽수 계산기를 고치지 않는다.",
        "violated",
        (
            "마지막 페이지 미출력 후보다. #4882 가 쪽수 로직을 맡는다.",
            "이 케이스는 inventory 의 section_count 필드만 고정한다.",
        ),
        _simple_delta(section_count_oracle=2, section_count_generated=1),
    ),
    _case(
        "D02",
        "hwpx-id-mappings-count",
        "DocInfo / ID_MAPPINGS counts",
        "docinfo",
        "C",
        "파일 읽기 오류",
        "index",
        "docinfo",
        (),
        "ID_MAPPINGS count == 자식 record 수",
        "CharShape/ParaShape/BinData 개수가 매핑 표와 같아야 한다.",
        "violated",
        ("초기 로딩 실패가 신호다.",),
        _simple_delta(
            mutate_generated=_rewrite_tag("ID_MAPPINGS", b"\xff" * 18),
        ),
    ),
    _case(
        "D03",
        "hwpx-missing-face-name",
        "DocInfo / FACE_NAME",
        "docinfo",
        "D",
        "파일 읽기 오류",
        "index",
        "docinfo",
        (),
        "FACE_NAME graft",
        "CharShape 가 가리키는 face 인덱스는 FACE_NAME 표에 있어야 한다.",
        "violated",
        ("서체 표 누락은 초기 로딩에서 터진다.",),
        _simple_delta(mutate_generated=_drop_tag("FACE_NAME")),
    ),
    _case(
        "D04",
        "hwpx-extra-char-shape",
        "DocInfo / extra CHAR_SHAPE",
        "docinfo",
        "D",
        "열림 + 조판 실패",
        "lcs",
        "docinfo",
        (),
        "과잉 CHAR_SHAPE 제거 또는 매핑 수정",
        "oracle 에 없는 CharShape 를 추가하면 인덱스가 밀린다.",
        "violated",
        ("ID_MAPPINGS 와 같이 본다.",),
        _simple_delta(
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=20,
                    tag_name="CHAR_SHAPE",
                    level=1,
                    payload=b"\x02\x00\x18\x00\x00\x00",
                    parent_uid="DocInfo#1",
                    parent_scope="ID_MAPPINGS#1@lv0",
                    tuple_index=1,
                )
            )
        ),
    ),
    _case(
        "D05",
        "hwpx-compatible-document",
        "DocInfo / COMPATIBLE_DOCUMENT",
        "docinfo",
        "A",
        "파일 읽기 오류",
        "index",
        "docinfo",
        (),
        "호환 문서 표지 보존",
        "한컴 oracle 이 쓰는 CompatibleDocument 표지를 버려서는 안 된다.",
        "violated",
        ("구버전 로더 경로를 가른다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=DOCINFO,
                        record_index=30,
                        tag_name="COMPATIBLE_DOCUMENT",
                        level=0,
                        payload=b"\x01\x00\x00\x00",
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=DOCINFO,
                        record_index=31,
                        tag_name="LAYOUT_COMPATIBILITY",
                        level=1,
                        payload=b"\x00" * 20,
                        parent_uid="DocInfo#30",
                        parent_scope="COMPATIBLE_DOCUMENT#30@lv0",
                    ),
                ]
            )
        ),
    ),
    _case(
        "D06",
        "hwpx-distribute-doc",
        "DocInfo / DISTRIBUTE_DOC_DATA",
        "docinfo",
        "A",
        "파일 읽기 오류",
        "index",
        "docinfo",
        (),
        "배포용 문서 플래그와 데이터 동시",
        "distribution 파일이면 ViewText 스트림과 DISTRIBUTE_DOC_DATA 를 같이 본다.",
        "violated",
        ("inventory 는 stream_path 로 ViewText 를 표시한다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=32,
                    tag_name="DISTRIBUTE_DOC_DATA",
                    level=0,
                    payload=b"\x01\x00dist\x00",
                )
            )
        ),
    ),
    _case(
        "D07",
        "hwpx-numbering-bullet",
        "DocInfo / NUMBERING + BULLET",
        "docinfo",
        "D",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "번호/글머리표 표 개수",
        "문단 번호와 글머리표는 ID_MAPPINGS 자식이다.",
        "violated",
        ("목록 문단이 깨지면 이 표를 먼저 본다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=DOCINFO,
                        record_index=33,
                        tag_name="NUMBERING",
                        level=1,
                        payload=b"\x01\x00\x01\x00",
                        parent_uid="DocInfo#1",
                        parent_scope="ID_MAPPINGS#1@lv0",
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=DOCINFO,
                        record_index=34,
                        tag_name="BULLET",
                        level=1,
                        payload=b"\x95\x00",
                        parent_uid="DocInfo#1",
                        parent_scope="ID_MAPPINGS#1@lv0",
                    ),
                ]
            )
        ),
    ),
    _case(
        "D08",
        "hwpx-parashape-valign",
        "DocInfo / PARA_SHAPE.attr1 valign",
        "docinfo",
        "E",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "셀 클리핑이면 ParaShape valign bits",
        "셀 텍스트가 위로 붙으면 margin 보다 ParaShape attr1 세로 정렬 비트를 본다.",
        "violated",
        ("#903 Stage 50-51.",),
        _simple_delta(
            mutate_generated=_rewrite_tag("PARA_SHAPE", b"\x30\x00\x00\x00\x00\x00\x00\x00"),
        ),
    ),
    _case(
        "C01",
        "hwpx-equation-missing-eqedit",
        "hp:equation / EQEDIT",
        "equation",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "CTRL_HEADER(Equation)+EQEDIT",
        "수식 컨트롤 다음은 EQEDIT 다.",
        "violated",
        ("수식 직후 손상.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=50,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"deqe" + b"\x00" * 8,
                        control_fourcc="eqed",
                        body_order=5,
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=51,
                        tag_name="EQEDIT",
                        level=1,
                        payload=b"sum_{i=1}^{n} i",
                        parent_uid="BodyText.Section0#50",
                        parent_scope="CTRL_HEADER#50@lv0",
                        body_order=5,
                    ),
                ]
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=50,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"deqe" + b"\x00" * 8,
                    control_fourcc="eqed",
                    body_order=5,
                )
            ),
        ),
    ),
    _case(
        "C02",
        "hwpx-footnote-list-count",
        "hp:fn / LIST_HEADER paragraph count",
        "note",
        "C",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "각주 문단 수",
        "각주 LIST_HEADER 의 paragraph count 와 자식 PARA_HEADER 수가 같아야 한다.",
        "violated",
        ("각주 본문 중간 손상이 신호다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=60,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"  nf" + b"\x00" * 8,
                        control_fourcc="fn  ",
                        body_order=6,
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=61,
                        tag_name="LIST_HEADER",
                        level=1,
                        payload=b"\x01\x00\x00\x00",
                        parent_uid="BodyText.Section0#60",
                        parent_scope="CTRL_HEADER#60@lv0",
                        body_order=6,
                    ),
                ]
            ),
            mutate_generated=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=60,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"  nf" + b"\x00" * 8,
                        control_fourcc="fn  ",
                        body_order=6,
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=61,
                        tag_name="LIST_HEADER",
                        level=1,
                        payload=b"\x03\x00\x00\x00",
                        parent_uid="BodyText.Section0#60",
                        parent_scope="CTRL_HEADER#60@lv0",
                        body_order=6,
                    ),
                ]
            ),
        ),
    ),
    _case(
        "C03",
        "hwpx-endnote-missing",
        "hp:en / Endnote control",
        "note",
        "B",
        "파일 손상",
        "lcs",
        "missing",
        (),
        "미주 컨트롤 graft",
        "미주는 fn 과 다른 fourcc(en) 다. 각주 경로로 낮추지 않는다.",
        "violated",
        ("컨트롤 ID 가 바뀌면 control_changed.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=70,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"  ne" + b"\x00" * 8,
                    control_fourcc="en  ",
                    body_order=7,
                )
            )
        ),
    ),
    _case(
        "C04",
        "hwpx-header-footer",
        "hp:header + hp:footer",
        "note",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "머리말/꼬리말 목록 범위",
        "head/foot 는 각각 LIST_HEADER 를 가진다. 한쪽만 넣으면 missing.",
        "violated",
        ("쪽 머리글 직후 손상.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=80,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"daeh" + b"\x00" * 8,
                        control_fourcc="head",
                        body_order=8,
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=81,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"toof" + b"\x00" * 8,
                        control_fourcc="foot",
                        body_order=9,
                    ),
                ]
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=80,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"daeh" + b"\x00" * 8,
                    control_fourcc="head",
                    body_order=8,
                )
            ),
        ),
    ),
    _case(
        "C05",
        "hwpx-form-object",
        "hp:form / FORM_OBJECT",
        "form",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "FORM_OBJECT graft",
        "양식 개체는 CTRL_HEADER(form) + FORM_OBJECT. 누름틀 필드와 섞지 않는다.",
        "violated",
        ("양식 직후 손상.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=90,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b"mrof" + b"\x00" * 8,
                        control_fourcc="form",
                        body_order=10,
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=91,
                        tag_name="FORM_OBJECT",
                        level=1,
                        payload=b"\x01\x00name\x00",
                        parent_uid="BodyText.Section0#90",
                        parent_scope="CTRL_HEADER#90@lv0",
                        body_order=10,
                    ),
                ]
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=90,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"mrof" + b"\x00" * 8,
                    control_fourcc="form",
                    body_order=10,
                )
            ),
        ),
    ),
    _case(
        "C06",
        "hwpx-memo-shape",
        "hp:memo / MEMO_SHAPE + MEMO_LIST",
        "note",
        "B",
        "파일 손상",
        "lcs",
        "docinfo",
        (),
        "메모 모양은 DocInfo, 목록은 BodyText",
        "메모 필드는 MEMO_SHAPE(DocInfo) 와 MEMO_LIST(본문) 가 짝.",
        "violated",
        ("한쪽만 있으면 메모 클릭이 죽는다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=DOCINFO,
                        record_index=40,
                        tag_name="MEMO_SHAPE",
                        level=0,
                        payload=b"\x01\x00memo\x00",
                    ),
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=100,
                        tag_name="MEMO_LIST",
                        level=0,
                        payload=b"\x01\x00",
                        body_order=11,
                    ),
                ]
            )
        ),
    ),
    _case(
        "C07",
        "hwpx-chart-data-extra",
        "hp:chart / extra CHART_DATA",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "all",
        (),
        "oracle 에 없는 CHART_DATA 제거",
        "차트는 OLE/SHAPE_OLE 경로와 CHART_DATA 를 섞어 넣지 않는다.",
        "violated",
        ("과잉 차트 레코드는 extra.",),
        _simple_delta(
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=110,
                    tag_name="CHART_DATA",
                    level=1,
                    payload=b"chart-extra",
                    body_order=12,
                )
            )
        ),
    ),
    _case(
        "C08",
        "hwpx-bookmark",
        "hp:bookmark / CTRL_HEADER bokm",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "책갈피 컨트롤 보존",
        "책갈피는 필드 %bmk 와 컨트롤 bokm 이 다르다. 낮출 때 섞지 않는다.",
        "violated",
        ("찾아보기/책갈피 점프 실패.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=120,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"mkob" + b"\x00" * 8,
                    control_fourcc="bokm",
                    body_order=13,
                )
            )
        ),
    ),
    _case(
        "F01",
        "hwpx-field-clickhere",
        "hp:fieldBegin type=CLICKHERE",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "누름틀 %clk 정체성 보존",
        "누름틀은 %clk. Unknown 으로 붕괴시키면 안 된다(이름은 Unknown 이어도 fourcc 는 유지).",
        "violated",
        ("ctrl_name 이 Unknown 인 것은 필드 fourcc 계약이다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=130,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"klc%" + b"\x00" * 8,
                    control_fourcc="%clk",
                    body_order=14,
                )
            )
        ),
    ),
    _case(
        "F02",
        "hwpx-field-hyperlink",
        "hp:fieldBegin type=HYPERLINK",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "하이퍼링크 %hlk",
        "하이퍼링크 필드 fourcc 는 %hlk.",
        "violated",
        ("클릭 링크 소실.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=140,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"klh%" + b"\x00" * 8,
                    control_fourcc="%hlk",
                    body_order=15,
                )
            )
        ),
    ),
    _case(
        "F03",
        "hwpx-field-mailmerge",
        "hp:fieldBegin type=MAILMERGE",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "메일머지 %mmg",
        "메일머지 필드는 %mmg. 누름틀로 치환하지 않는다.",
        "violated",
        ("서식 채우기 경로와 별개로 저장 정체성만 본다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=150,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"gmm%" + b"\x00" * 8,
                    control_fourcc="%mmg",
                    body_order=16,
                )
            )
        ),
    ),
    _case(
        "F04",
        "hwpx-field-proofreading",
        "hp:fieldBegin type=PROOFREADING_MARKS_DELETE",
        "field",
        "C",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "교정부호는 %%*d / command $RevisionDelete",
        "HWP5 는 이 종류를 ctrl_id 가 아니라 command 로 든다. %unk 로 굳히지 않는다.",
        "violated",
        ("#4896 실측. inventory 는 fourcc 와 key_payload 를 남긴다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=160,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"d*%%" + b"$RevisionDelete;\x00",
                    control_fourcc="%%*d",
                    body_order=17,
                )
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=160,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"knu%" + b"\x00" * 8,
                    control_fourcc="%unk",
                    body_order=17,
                )
            ),
        ),
    ),
    _case(
        "G01",
        "hwpx-columndef-defaults",
        "hp:colPr / ColumnDef defaults",
        "page",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "다단 기본값 합성",
        "HWPX 가 생략한 단 간격/구분선은 oracle ColumnDef payload 로 채운다.",
        "violated",
        ("다단 조판이 한쪽으로 붙음.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=170,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"dloc" + b"\x02\x00\x80\x00",
                    control_fourcc="cold",
                    body_order=18,
                )
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=170,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"dloc" + b"\x00\x00\x00\x00",
                    control_fourcc="cold",
                    body_order=18,
                )
            ),
        ),
    ),
    _case(
        "G02",
        "hwpx-pagedef-missing",
        "hp:secPr / PAGE_DEF",
        "page",
        "B",
        "파일 읽기 오류",
        "lcs",
        "missing",
        (),
        "PAGE_DEF graft",
        "SectionDef 튜플에서 PAGE_DEF 는 필수.",
        "violated",
        ("초기 로딩 또는 첫 쪽 전 손상.",),
        _simple_delta(mutate_generated=_drop_tag("PAGE_DEF")),
    ),
    _case(
        "G03",
        "hwpx-page-border-fill",
        "hp:pagePr / PAGE_BORDER_FILL defaults",
        "page",
        "E",
        "열림 + 조판 실패",
        "index",
        "all",
        (),
        "쪽 테두리 기본값",
        "쪽 테두리/배경 기본값이 빠지면 조판만 틀린다.",
        "violated",
        ("열리지만 배경이 없다.",),
        _simple_delta(
            mutate_generated=_rewrite_tag("PAGE_BORDER_FILL", b"\xff\xff\xff\xff\xff\xff"),
        ),
    ),
    _case(
        "G04",
        "hwpx-pagenum-pos",
        "hp:pageNum / PageNumPos",
        "page",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "쪽번호 위치 기본값",
        "쪽번호 위치 컨트롤은 페이지 수 계산과 별개다. inventory 만 대조한다.",
        "violated",
        ("#4882 쪽수 로직을 여기서 고치지 않는다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=180,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"pngp" + b"\x01\x00",
                    control_fourcc="pgnp",
                    body_order=19,
                )
            )
        ),
    ),
    _case(
        "G05",
        "hwpx-pagehide",
        "hp:pageHide / PageHide",
        "page",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "감추기 비트 보존",
        "첫 쪽 머리글 감추기는 pghd 컨트롤이다.",
        "violated",
        ("머리글이 첫 쪽에 다시 나타남.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=190,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"dhgp" + b"\x01\x00",
                    control_fourcc="pghd",
                    body_order=20,
                )
            )
        ),
    ),
    _case(
        "X01",
        "hwpx-index-vs-lcs-insert",
        "중간 삽입에서 index vs lcs",
        "para",
        "B",
        "파일 손상",
        "lcs",
        "all",
        (),
        "중간 삽입은 lcs 우선",
        "중간 레코드가 끼면 index uid 정렬은 뒤가 전부 changed 로 보인다. lcs 를 쓴다.",
        "violated",
        ("align=index 와 align=lcs 전사를 둘 다 남긴다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="하나".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=2,
                )
                + _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    14,
                    text="둘".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=3,
                )
            ),
            mutate_generated=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="삽입".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=2,
                )
                + _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    14,
                    text="하나".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=3,
                )
                + _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    18,
                    text="둘".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=4,
                )
            ),
        ),
    ),
    _case(
        "X02",
        "hwpx-scope-changed",
        "같은 uid / 다른 scope_path",
        "table",
        "B",
        "파일 손상",
        "index",
        "table",
        (),
        "scope_path 재배치",
        "레코드가 살아 있어도 부모가 바뀌면 scope_changed 다.",
        "violated",
        ("index 모드만 scope_changed 를 낸다.",),
        _simple_delta(
            with_table=True,
            mutate_generated=lambda items: _rewrite_scope(items, "TABLE", "CTRL_HEADER#0@lv0"),
        ),
    ),
    _case(
        "X03",
        "hwpx-control-remapped",
        "CTRL_HEADER fourcc remapped",
        "ctrl",
        "C",
        "파일 손상",
        "index",
        "ctrl",
        (),
        "컨트롤 ID 보존",
        "표 자리에 도형 fourcc 를 넣으면 control_changed.",
        "violated",
        ("다음 concrete record 도 같이 틀린다.",),
        _simple_delta(
            with_table=True,
            mutate_generated=lambda items: _remap_first_table_ctrl(items),
        ),
    ),
    _case(
        "X04",
        "hwpx-tag-changed",
        "같은 uid / 다른 tag",
        "para",
        "B",
        "파일 손상",
        "index",
        "all",
        (),
        "tag_changed 는 트리 재작성 신호",
        "같은 인덱스에 다른 태그가 오면 튜플이 깨진 것이다.",
        "violated",
        ("index 모드가 tag_changed 를 낸다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.extend(
                _para_tuple(
                    items[0].sample,
                    items[0].source_path,
                    10,
                    text="태그".encode("utf-16-le"),
                    char_shape=b"\x00\x00",
                    line_seg=b"\x00\x00\x10\x00",
                    body_order=2,
                )
            ),
            mutate_generated=lambda items: items.extend(
                [
                    make_item(
                        sample=items[0].sample,
                        source_path=items[0].source_path,
                        stream_path=BODY0,
                        record_index=10,
                        tag_name="CTRL_HEADER",
                        level=0,
                        payload=b" osg" + b"\x00" * 8,
                        control_fourcc="gso ",
                        body_order=2,
                    )
                ]
            ),
        ),
    ),
    _case(
        "X05",
        "hwpx-trackchange-extra",
        "DocInfo / extra TRACKCHANGE",
        "docinfo",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "docinfo",
        (),
        "변경 추적 메타 과잉 삽입 금지",
        "oracle 에 없는 TRACKCHANGE 는 extra.",
        "violated",
        ("교정부호 필드와 별개.",),
        _simple_delta(
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=50,
                    tag_name="TRACKCHANGE",
                    level=0,
                    payload=b"\x01\x00track\x00",
                )
            )
        ),
    ),
    _case(
        "X06",
        "hwpx-forbidden-char",
        "DocInfo / FORBIDDEN_CHAR defaults",
        "docinfo",
        "E",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "금칙 문자 기본값",
        "금칙 문자는 로더 경고에 가깝다. payload 만 대조한다.",
        "violated",
        ("조판보다 금칙 처리 차이.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=51,
                    tag_name="FORBIDDEN_CHAR",
                    level=0,
                    payload="。，".encode("utf-16-le"),
                )
            )
        ),
    ),
    _case(
        "X07",
        "hwpx-tab-def",
        "DocInfo / TAB_DEF",
        "docinfo",
        "D",
        "열림 + 조판 실패",
        "index",
        "docinfo",
        (),
        "탭 정의 표",
        "ParaShape 가 참조하는 TAB_DEF 가 빠지면 탭 위치가 무너진다.",
        "violated",
        ("표 안 탭과 본문 탭을 같은 표로 본다.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=52,
                    tag_name="TAB_DEF",
                    level=1,
                    payload=b"\x01\x00\x00\x10\x00\x00",
                    parent_uid="DocInfo#1",
                    parent_scope="ID_MAPPINGS#1@lv0",
                )
            )
        ),
    ),
    _case(
        "X08",
        "hwpx-doc-data",
        "DocInfo / DOC_DATA ParameterSet",
        "docinfo",
        "A",
        "파일 읽기 오류",
        "index",
        "docinfo",
        (),
        "문서 부가 데이터 보존",
        "DOC_DATA ParameterSet 은 한컴이 기대하는 키가 있다.",
        "violated",
        ("초기 로딩 경고.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=DOCINFO,
                    record_index=53,
                    tag_name="DOC_DATA",
                    level=0,
                    payload=b"\x01\x00kset\x00",
                )
            )
        ),
    ),
    _case(
        "X09",
        "hwpx-autonumber",
        "hp:autoNum / AutoNumber",
        "field",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "자동번호 기본 속성",
        "자동번호 컨트롤 기본 속성은 oracle payload 다.",
        "violated",
        ("번호가 1로 리셋됨.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=200,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"onta" + b"\x01\x00\x00\x00",
                    control_fourcc="atno",
                    body_order=21,
                )
            ),
            mutate_generated=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=200,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"onta" + b"\x00\x00\x00\x00",
                    control_fourcc="atno",
                    body_order=21,
                )
            ),
        ),
    ),
    _case(
        "X10",
        "hwpx-newnumber",
        "hp:newNum / NewNumber",
        "field",
        "E",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "새 번호 시작값",
        "새 번호 컨트롤은 시작값을 oracle 에서 가져온다.",
        "violated",
        ("구역마다 번호가 이어짐.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=210,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"onwn" + b"\x05\x00",
                    control_fourcc="nwno",
                    body_order=22,
                )
            )
        ),
    ),
    _case(
        "X11",
        "hwpx-index-mark",
        "hp:indexmark / IndexMark",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "찾아보기 표식 보존",
        "찾아보기 표식 idxm 은 본문 자리와 짝.",
        "violated",
        ("색인 생성 실패.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=220,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"mxdi" + b"key\x00",
                    control_fourcc="idxm",
                    body_order=23,
                )
            )
        ),
    ),
    _case(
        "X12",
        "hwpx-hidden-comment",
        "hp:hiddenComment / HiddenComment",
        "note",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "숨은 설명 컨트롤",
        "숨은 설명 tcmt 는 메모와 다른 컨트롤이다.",
        "violated",
        ("숨은 설명 창이 비어 있음.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=230,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"tmct" + b"\x00" * 8,
                    control_fourcc="tcmt",
                    body_order=24,
                )
            )
        ),
    ),
    _case(
        "X13",
        "hwpx-char-overlap",
        "hp:dutmal / CharOverlap",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "덧말 컨트롤",
        "덧말은 tdut. 글자겹침 tcps 와 섞지 않는다.",
        "violated",
        ("덧말 위치 소실.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=240,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"tudt" + b"\x00" * 8,
                    control_fourcc="tdut",
                    body_order=25,
                )
            )
        ),
    ),
    _case(
        "X14",
        "hwpx-tcps",
        "hp:compose / Tcps",
        "field",
        "B",
        "열림 + 조판 실패",
        "lcs",
        "ctrl",
        (),
        "글자겹침 컨트롤",
        "글자겹침은 tcps.",
        "violated",
        ("겹친 글자가 가로로 풀림.",),
        _simple_delta(
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=250,
                    tag_name="CTRL_HEADER",
                    level=0,
                    payload=b"spct" + b"\x00" * 8,
                    control_fourcc="tcps",
                    body_order=26,
                )
            )
        ),
    ),
    _case(
        "X15",
        "hwpx-textbox-list-header",
        "hp:textbox / LIST_HEADER",
        "shape",
        "B",
        "파일 손상",
        "lcs",
        "ctrl",
        (),
        "글상자 목록 범위",
        "글상자는 GenShape + LIST_HEADER 문단 튜플이다.",
        "violated",
        ("글상자 직후 손상. #1058 계열.",),
        _simple_delta(
            with_shape=True,
            mutate_oracle=lambda items: items.append(
                make_item(
                    sample=items[0].sample,
                    source_path=items[0].source_path,
                    stream_path=BODY0,
                    record_index=260,
                    tag_name="LIST_HEADER",
                    level=1,
                    payload=b"\x01\x00\x00\x00\x10\x00",
                    parent_uid="BodyText.Section0#20",
                    parent_scope="CTRL_HEADER#20@lv0",
                    body_order=1,
                )
            ),
        ),
    ),
    _case(
        "X16",
        "hwpx-identical-roundtrip",
        "oracle == generated sentinel",
        "para",
        "A",
        "성공",
        "lcs",
        "all",
        (),
        "차이 없음 — 다음 샘플로",
        "차이가 없으면 matched 만 남고 probe 를 만들지 않는다.",
        "satisfied",
        ("sentinel. 구현 후보가 아니라 회귀 바닥.",),
        _simple_delta(),
    ),
]


def _rewrite_scope(items: list[InventoryItem], tag_name: str, new_parent_scope: str) -> None:
    for item in items:
        if item.tag_name == tag_name:
            item.parent_scope = new_parent_scope
            parts = item.scope_path.split("/")
            if len(parts) >= 2:
                parts[-2] = new_parent_scope.split("@lv")[0]
                item.scope_path = "/".join(parts)


def _remap_first_table_ctrl(items: list[InventoryItem]) -> None:
    for index, item in enumerate(items):
        if item.tag_name == "CTRL_HEADER" and item.control_name == "Table":
            items[index] = make_item(
                sample=item.sample,
                source_path=item.source_path,
                stream_path=item.stream_path,
                record_index=item.record_index,
                tag_name="CTRL_HEADER",
                level=item.level,
                payload=b" osg" + item.payload_bytes[4:],
                control_fourcc="gso ",
                parent_uid=item.parent_uid,
                parent_scope=item.parent_scope,
                body_order=item.body_order,
                tuple_index=item.tuple_index,
            )
            return


def cases_by_id() -> dict[str, ContractCase]:
    return {case.case_id: case for case in CASES}


def assert_catalog_coverage() -> None:
    used_tags = {item.tag_name for item in TAGS}
    seen_tags: set[str] = set()
    families = {case.family for case in CASES}
    classes = {case.failure_class for case in CASES}
    if families < {"table", "shape", "para", "docinfo", "field", "note", "page", "equation", "form", "ctrl"}:
        missing = {"table", "shape", "para", "docinfo", "field", "note", "page", "equation", "form"} - families
        raise AssertionError(f"family coverage hole: {missing}")
    if classes != {"A", "B", "C", "D", "E", "F"}:
        raise AssertionError(f"failure class hole: {classes}")
    ids = [case.case_id for case in CASES]
    if len(ids) != len(set(ids)):
        raise AssertionError("duplicate case id")
    samples = [case.sample for case in CASES]
    if len(samples) != len(set(samples)):
        raise AssertionError("duplicate sample stem")
    _ = used_tags, seen_tags
