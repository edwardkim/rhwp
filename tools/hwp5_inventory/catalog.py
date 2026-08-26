"""HWP5 inventory 언어의 정본 카탈로그.

숫자는 `src/parser/tags.rs` 와 `src/diagnostics/hwp5_inventory.rs` 의
`tuple_role` / `ctrl_name` 을 따른다. 필드 오프셋은
`src/diagnostics/hwp5_inventory_diff.rs` 의 P0 decoder 관찰명이다.
`tail_after_0x16` · `z_order_or_instance` 는 확정 계약 이름이 아니다.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal


HWPTAG_BEGIN = 0x010

TupleRole = Literal[
    "docinfo",
    "para_header",
    "para_text",
    "para_char_shape",
    "para_line_seg",
    "para_range_tag",
    "ctrl_header",
    "list_header",
    "page_control",
    "table",
    "shape_component",
    "pic",
    "ctrl_data",
    "equation",
    "form_object",
    "memo",
    "forbidden_char",
    "chart_data",
    "unknown",
]


@dataclass(frozen=True)
class TagSpec:
    tag_id: int
    tag_name: str
    role: TupleRole
    owner: Literal["DocInfo", "BodyText"]
    stream_hint: str
    required_children: tuple[str, ...]
    inventory_note: str


@dataclass(frozen=True)
class ControlSpec:
    fourcc: str
    ctrl_id: int
    ctrl_name: str
    family: str
    required_tuple: tuple[str, ...]
    inventory_focus: str
    failure_hint: str


@dataclass(frozen=True)
class FieldSpec:
    record_kind: str
    field_name: str
    offset: int
    width: int
    kind: Literal["u16", "u32", "u32_hex", "i32", "hex"]
    probe_axis: str | None
    observation_name: bool
    meaning: str


@dataclass(frozen=True)
class FailureClass:
    code: str
    name: str
    inspect: tuple[str, ...]
    signals: tuple[str, ...]
    inventory_columns: tuple[str, ...]
    typical_diff_kinds: tuple[str, ...]
    typical_focus: str
    next_probe: str


def ctrl_id(fourcc: str) -> int:
    raw = fourcc.encode("ascii")
    if len(raw) != 4:
        raise ValueError(f"fourcc must be 4 bytes: {fourcc!r}")
    return (raw[0] << 24) | (raw[1] << 16) | (raw[2] << 8) | raw[3]


def ctrl_id_hex(fourcc: str) -> str:
    return f"0x{ctrl_id(fourcc):08x}"


def tag_id(offset: int) -> int:
    return HWPTAG_BEGIN + offset


TAGS: tuple[TagSpec, ...] = (
    TagSpec(
        tag_id(0),
        "DOCUMENT_PROPERTIES",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "section_count / 문서 속성. 마지막 쪽 미출력과 묶인다.",
    ),
    TagSpec(
        tag_id(1),
        "ID_MAPPINGS",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (
            "BIN_DATA",
            "FACE_NAME",
            "BORDER_FILL",
            "CHAR_SHAPE",
            "TAB_DEF",
            "NUMBERING",
            "BULLET",
            "PARA_SHAPE",
            "STYLE",
        ),
        "DocInfo 항목 개수 표. count/reference 계약의 입구.",
    ),
    TagSpec(
        tag_id(2),
        "BIN_DATA",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "그림/OLE 바이너리 참조. CFB BinData/BINxxxx 와 쌍.",
    ),
    TagSpec(
        tag_id(3),
        "FACE_NAME",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "서체 이름 표. CharShape face 인덱스가 가리킨다.",
    ),
    TagSpec(
        tag_id(4),
        "BORDER_FILL",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "테두리/배경. 표·셀·문단이 인덱스로 참조.",
    ),
    TagSpec(
        tag_id(5),
        "CHAR_SHAPE",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "글자 모양. PARA_CHAR_SHAPE 가 인덱스로 참조.",
    ),
    TagSpec(
        tag_id(6),
        "TAB_DEF",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "탭 정의. ParaShape 가 참조.",
    ),
    TagSpec(
        tag_id(7),
        "NUMBERING",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "문단 번호 매기기. 개수 불일치는 ID_MAPPINGS 와 같이 본다.",
    ),
    TagSpec(
        tag_id(8),
        "BULLET",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "글머리표. Numbering 과 같은 DocInfo 축.",
    ),
    TagSpec(
        tag_id(9),
        "PARA_SHAPE",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "문단 모양. attr1 세로 정렬 비트는 셀 클리핑 후보.",
    ),
    TagSpec(
        tag_id(10),
        "STYLE",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "스타일 표. 문단 헤더가 인덱스로 참조.",
    ),
    TagSpec(
        tag_id(11),
        "DOC_DATA",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "문서 부가 데이터 ParameterSet.",
    ),
    TagSpec(
        tag_id(12),
        "DISTRIBUTE_DOC_DATA",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "배포용 문서 데이터. distribution 플래그와 함께 본다.",
    ),
    TagSpec(
        tag_id(14),
        "COMPATIBLE_DOCUMENT",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        ("LAYOUT_COMPATIBILITY",),
        "호환 문서 표지. 구버전 로더 경로를 가른다.",
    ),
    TagSpec(
        tag_id(15),
        "LAYOUT_COMPATIBILITY",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "조판 호환 비트. CompatibleDocument 자식.",
    ),
    TagSpec(
        tag_id(16),
        "TRACKCHANGE",
        "docinfo",
        "DocInfo",
        "/DocInfo",
        (),
        "변경 추적 메타. 본문 교정부호 필드와 별개.",
    ),
    TagSpec(
        tag_id(50),
        "PARA_HEADER",
        "para_header",
        "BodyText",
        "/BodyText/Section0",
        ("PARA_TEXT", "PARA_CHAR_SHAPE", "PARA_LINE_SEG"),
        "문단 헤더. char_count 와 PARA_TEXT code unit 수가 같아야 한다.",
    ),
    TagSpec(
        tag_id(51),
        "PARA_TEXT",
        "para_text",
        "BodyText",
        "/BodyText/Section0",
        (),
        "문단 본문. 확장 컨트롤(0x0B) 자리와 CTRL_HEADER 가 짝.",
    ),
    TagSpec(
        tag_id(52),
        "PARA_CHAR_SHAPE",
        "para_char_shape",
        "BodyText",
        "/BodyText/Section0",
        (),
        "문단 글자 모양 런. 누락이면 한컴이 문단 직후 손상 판정.",
    ),
    TagSpec(
        tag_id(53),
        "PARA_LINE_SEG",
        "para_line_seg",
        "BodyText",
        "/BodyText/Section0",
        (),
        "HWP5 line segment. HWPX lineSegArray 를 그대로 복사하면 안 된다.",
    ),
    TagSpec(
        tag_id(54),
        "PARA_RANGE_TAG",
        "para_range_tag",
        "BodyText",
        "/BodyText/Section0",
        (),
        "문단 범위 표지. 과잉 삽입은 extra 로 잡힌다.",
    ),
    TagSpec(
        tag_id(55),
        "CTRL_HEADER",
        "ctrl_header",
        "BodyText",
        "/BodyText/Section0",
        (),
        "컨트롤 헤더. 다음 concrete record 종류가 컨트롤 ID 와 맞아야 한다.",
    ),
    TagSpec(
        tag_id(56),
        "LIST_HEADER",
        "list_header",
        "BodyText",
        "/BodyText/Section0",
        ("PARA_HEADER",),
        "목록 헤더. paragraph count 와 자식 문단 범위가 같아야 한다.",
    ),
    TagSpec(
        tag_id(57),
        "PAGE_DEF",
        "page_control",
        "BodyText",
        "/BodyText/Section0",
        (),
        "용지/여백. SectionDef 튜플의 일부.",
    ),
    TagSpec(
        tag_id(58),
        "FOOTNOTE_SHAPE",
        "page_control",
        "BodyText",
        "/BodyText/Section0",
        (),
        "각주/미주 모양. 구역 정의 자식.",
    ),
    TagSpec(
        tag_id(59),
        "PAGE_BORDER_FILL",
        "page_control",
        "BodyText",
        "/BodyText/Section0",
        (),
        "쪽 테두리/배경. 기본값 미합성은 조판 실패 후보.",
    ),
    TagSpec(
        tag_id(60),
        "SHAPE_COMPONENT",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "도형 공통. rendering matrix 소수값은 f32→f64 양자화.",
    ),
    TagSpec(
        tag_id(61),
        "TABLE",
        "table",
        "BodyText",
        "/BodyText/Section0",
        ("LIST_HEADER",),
        "표 본문. attr/rows/cols/margin/tail 이 table-probe 축.",
    ),
    TagSpec(
        tag_id(62),
        "SHAPE_LINE",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "직선 도형 구체.",
    ),
    TagSpec(
        tag_id(63),
        "SHAPE_RECTANGLE",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "사각형 도형 구체.",
    ),
    TagSpec(
        tag_id(64),
        "SHAPE_ELLIPSE",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "타원 도형 구체.",
    ),
    TagSpec(
        tag_id(65),
        "SHAPE_ARC",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "호 도형 구체.",
    ),
    TagSpec(
        tag_id(66),
        "SHAPE_POLYGON",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "다각형 도형 구체.",
    ),
    TagSpec(
        tag_id(67),
        "SHAPE_CURVE",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "곡선 도형 구체.",
    ),
    TagSpec(
        tag_id(68),
        "SHAPE_OLE",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "OLE 도형 구체. BinData 와 함께 본다.",
    ),
    TagSpec(
        tag_id(69),
        "SHAPE_PICTURE",
        "pic",
        "BodyText",
        "/BodyText/Section0",
        (),
        "그림 구체. bin_data_id 가 DocInfo BIN_DATA 를 가리켜야 한다.",
    ),
    TagSpec(
        tag_id(70),
        "SHAPE_CONTAINER",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "묶음 컨테이너. 자식 SHAPE_COMPONENT 개수 계약.",
    ),
    TagSpec(
        tag_id(71),
        "CTRL_DATA",
        "ctrl_data",
        "BodyText",
        "/BodyText/Section0",
        (),
        "컨트롤 ParameterSet. 그림/도형 특수 payload.",
    ),
    TagSpec(
        tag_id(72),
        "EQEDIT",
        "equation",
        "BodyText",
        "/BodyText/Section0",
        (),
        "수식 본문. CTRL_HEADER(Equation) 다음 필수.",
    ),
    TagSpec(
        tag_id(74),
        "SHAPE_TEXTART",
        "shape_component",
        "BodyText",
        "/BodyText/Section0",
        (),
        "글맵시 구체.",
    ),
    TagSpec(
        tag_id(75),
        "FORM_OBJECT",
        "form_object",
        "BodyText",
        "/BodyText/Section0",
        (),
        "양식 개체. 누름틀과 별개 레코드.",
    ),
    TagSpec(
        tag_id(76),
        "MEMO_SHAPE",
        "memo",
        "DocInfo",
        "/DocInfo",
        (),
        "메모 모양. DocInfo 축.",
    ),
    TagSpec(
        tag_id(77),
        "MEMO_LIST",
        "memo",
        "BodyText",
        "/BodyText/Section0",
        (),
        "메모 목록. 본문 메모 필드와 짝.",
    ),
    TagSpec(
        tag_id(78),
        "FORBIDDEN_CHAR",
        "forbidden_char",
        "DocInfo",
        "/DocInfo",
        (),
        "금칙 문자. 기본값 미합성은 조판보다 로더 경고에 가깝다.",
    ),
    TagSpec(
        tag_id(79),
        "CHART_DATA",
        "chart_data",
        "BodyText",
        "/BodyText/Section0",
        (),
        "차트 데이터. OLE/차트 저장 경로.",
    ),
)


CONTROLS: tuple[ControlSpec, ...] = (
    ControlSpec(
        "secd",
        ctrl_id("secd"),
        "SectionDef",
        "page",
        ("CTRL_HEADER", "PAGE_DEF", "FOOTNOTE_SHAPE", "PAGE_BORDER_FILL"),
        "ctrl",
        "A/B: 구역 스트림과 PAGE_DEF 튜플",
    ),
    ControlSpec(
        "cold",
        ctrl_id("cold"),
        "ColumnDef",
        "page",
        ("CTRL_HEADER",),
        "ctrl",
        "E: 다단 기본값 미합성",
    ),
    ControlSpec(
        "tbl ",
        ctrl_id("tbl "),
        "Table",
        "table",
        ("CTRL_HEADER", "TABLE", "LIST_HEADER", "PARA_HEADER"),
        "table",
        "B/C/E: 표 튜플과 table-probe 축",
    ),
    ControlSpec(
        "eqed",
        ctrl_id("eqed"),
        "Equation",
        "equation",
        ("CTRL_HEADER", "EQEDIT"),
        "ctrl",
        "B: EQEDIT 누락은 수식 직후 손상",
    ),
    ControlSpec(
        "gso ",
        ctrl_id("gso "),
        "GenShape",
        "shape",
        ("CTRL_HEADER", "SHAPE_COMPONENT"),
        "shape",
        "B/D: SHAPE_COMPONENT + BinData",
    ),
    ControlSpec(
        "head",
        ctrl_id("head"),
        "Header",
        "note",
        ("CTRL_HEADER", "LIST_HEADER"),
        "ctrl",
        "B: 머리말 목록 범위",
    ),
    ControlSpec(
        "foot",
        ctrl_id("foot"),
        "Footer",
        "note",
        ("CTRL_HEADER", "LIST_HEADER"),
        "ctrl",
        "B: 꼬리말 목록 범위",
    ),
    ControlSpec(
        "fn  ",
        ctrl_id("fn  "),
        "Footnote",
        "note",
        ("CTRL_HEADER", "LIST_HEADER"),
        "ctrl",
        "C: 각주 문단 수",
    ),
    ControlSpec(
        "en  ",
        ctrl_id("en  "),
        "Endnote",
        "note",
        ("CTRL_HEADER", "LIST_HEADER"),
        "ctrl",
        "C: 미주 문단 수",
    ),
    ControlSpec(
        "atno",
        ctrl_id("atno"),
        "AutoNumber",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "E: 자동번호 기본 속성",
    ),
    ControlSpec(
        "nwno",
        ctrl_id("nwno"),
        "NewNumber",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "E: 새 번호 기본 속성",
    ),
    ControlSpec(
        "pgnp",
        ctrl_id("pgnp"),
        "PageNumPos",
        "page",
        ("CTRL_HEADER",),
        "ctrl",
        "E: 쪽번호 위치 기본값",
    ),
    ControlSpec(
        "pgct",
        ctrl_id("pgct"),
        "PageNumCtrl",
        "page",
        ("CTRL_HEADER",),
        "ctrl",
        "E: pageStartsOn 대응",
    ),
    ControlSpec(
        "pghd",
        ctrl_id("pghd"),
        "PageHide",
        "page",
        ("CTRL_HEADER",),
        "ctrl",
        "E: 감추기 비트",
    ),
    ControlSpec(
        "idxm",
        ctrl_id("idxm"),
        "IndexMark",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "B: 찾아보기 표식 누락",
    ),
    ControlSpec(
        "Mtit",
        ctrl_id("Mtit"),
        "TitleMark(ignore)",
        "field",
        (),
        "ctrl",
        "인라인 0x08 — CTRL_HEADER 를 만들지 않는다",
    ),
    ControlSpec(
        "Mign",
        ctrl_id("Mign"),
        "TitleMark",
        "field",
        (),
        "ctrl",
        "인라인 0x08 — CTRL_HEADER 를 만들지 않는다",
    ),
    ControlSpec(
        "bokm",
        ctrl_id("bokm"),
        "Bookmark",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "B: 책갈피 컨트롤",
    ),
    ControlSpec(
        "tcps",
        ctrl_id("tcps"),
        "Tcps",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "B: 글자겹침",
    ),
    ControlSpec(
        "form",
        ctrl_id("form"),
        "Form",
        "form",
        ("CTRL_HEADER", "FORM_OBJECT"),
        "ctrl",
        "B: FORM_OBJECT 누락",
    ),
    ControlSpec(
        "tdut",
        ctrl_id("tdut"),
        "CharOverlap",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "B: 덧말",
    ),
    ControlSpec(
        "tcmt",
        ctrl_id("tcmt"),
        "HiddenComment",
        "note",
        ("CTRL_HEADER",),
        "ctrl",
        "B: 숨은 설명",
    ),
    ControlSpec(
        "%clk",
        ctrl_id("%clk"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 누름틀. ctrl_name 은 Unknown (필드 fourcc)",
    ),
    ControlSpec(
        "%hlk",
        ctrl_id("%hlk"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 하이퍼링크",
    ),
    ControlSpec(
        "%bmk",
        ctrl_id("%bmk"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 책갈피",
    ),
    ControlSpec(
        "%dte",
        ctrl_id("%dte"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 현재 날짜",
    ),
    ControlSpec(
        "%ddt",
        ctrl_id("%ddt"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 문서 날짜",
    ),
    ControlSpec(
        "%pat",
        ctrl_id("%pat"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 파일 경로",
    ),
    ControlSpec(
        "%mmg",
        ctrl_id("%mmg"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 메일머지",
    ),
    ControlSpec(
        "%xrf",
        ctrl_id("%xrf"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 상호참조",
    ),
    ControlSpec(
        "%fmu",
        ctrl_id("%fmu"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 표 계산식",
    ),
    ControlSpec(
        "%smr",
        ctrl_id("%smr"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 문서 요약",
    ),
    ControlSpec(
        "%usr",
        ctrl_id("%usr"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 사용자 정보",
    ),
    ControlSpec(
        "%%me",
        ctrl_id("%%me"),
        "Unknown",
        "field",
        ("CTRL_HEADER", "MEMO_LIST"),
        "ctrl",
        "필드 메모",
    ),
    ControlSpec(
        "%cpr",
        ctrl_id("%cpr"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 개인정보 보안",
    ),
    ControlSpec(
        "%toc",
        ctrl_id("%toc"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "필드 차례",
    ),
    ControlSpec(
        "%unk",
        ctrl_id("%unk"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "정체성 미모델링 필드. command 문자열을 같이 본다",
    ),
    ControlSpec(
        "%%*d",
        ctrl_id("%%*d"),
        "Unknown",
        "field",
        ("CTRL_HEADER",),
        "ctrl",
        "교정부호(삭제). 한컴은 %%*d 로 센다",
    ),
)


SHAPE_INNER_IDS: tuple[tuple[str, str], ...] = (
    ("$pic", "picture"),
    ("$ole", "ole"),
    ("$rec", "rectangle"),
    ("$lin", "line"),
    ("$ell", "ellipse"),
    ("$pol", "polygon"),
    ("$arc", "arc"),
    ("$cur", "curve"),
    ("$col", "connector"),
    ("$con", "container"),
)


TABLE_CTRL_FIELDS: tuple[FieldSpec, ...] = (
    FieldSpec("CTRL_HEADER", "ctrl_id", 0x00, 4, "u32", None, False, "Table fourcc 0x74626c20"),
    FieldSpec(
        "CTRL_HEADER",
        "common_attr",
        0x04,
        4,
        "u32_hex",
        "ctrl_common_attr",
        False,
        "TABLE control wrapper 공통 속성 비트",
    ),
    FieldSpec("CTRL_HEADER", "x", 0x08, 4, "i32", None, False, "바깥 상자 x (HWPUNIT)"),
    FieldSpec("CTRL_HEADER", "y", 0x0C, 4, "i32", None, False, "바깥 상자 y (HWPUNIT)"),
    FieldSpec("CTRL_HEADER", "width", 0x10, 4, "i32", None, False, "바깥 상자 폭"),
    FieldSpec("CTRL_HEADER", "height", 0x14, 4, "i32", None, False, "바깥 상자 높이"),
    FieldSpec(
        "CTRL_HEADER",
        "z_order_or_instance",
        0x18,
        4,
        "u32",
        None,
        True,
        "관찰명. 확정 계약 이름 아님",
    ),
    FieldSpec(
        "CTRL_HEADER",
        "out_margin_left",
        0x1C,
        2,
        "u16",
        "ctrl_outer_margin",
        False,
        "바깥 여백 왼쪽",
    ),
    FieldSpec(
        "CTRL_HEADER",
        "out_margin_right",
        0x1E,
        2,
        "u16",
        "ctrl_outer_margin",
        False,
        "바깥 여백 오른쪽",
    ),
    FieldSpec(
        "CTRL_HEADER",
        "out_margin_top",
        0x20,
        2,
        "u16",
        "ctrl_outer_margin",
        False,
        "바깥 여백 위",
    ),
    FieldSpec(
        "CTRL_HEADER",
        "out_margin_bottom",
        0x22,
        2,
        "u16",
        "ctrl_outer_margin",
        False,
        "바깥 여백 아래",
    ),
    FieldSpec(
        "CTRL_HEADER",
        "tail_after_0x24",
        0x24,
        16,
        "hex",
        None,
        True,
        "관찰명. 0x24 이후 16바이트",
    ),
)


TABLE_RECORD_FIELDS: tuple[FieldSpec, ...] = (
    FieldSpec(
        "TABLE",
        "table_attr",
        0x00,
        4,
        "u32_hex",
        "table_attr",
        False,
        "TABLE record 첫 4바이트 속성 비트",
    ),
    FieldSpec("TABLE", "rows", 0x04, 2, "u16", None, False, "행 수"),
    FieldSpec("TABLE", "cols", 0x06, 2, "u16", None, False, "열 수"),
    FieldSpec("TABLE", "cell_spacing", 0x08, 2, "u16", None, False, "셀 간격"),
    FieldSpec("TABLE", "in_margin_left", 0x0A, 2, "u16", None, False, "안쪽 여백 왼쪽"),
    FieldSpec("TABLE", "in_margin_right", 0x0C, 2, "u16", None, False, "안쪽 여백 오른쪽"),
    FieldSpec("TABLE", "in_margin_top", 0x0E, 2, "u16", None, False, "안쪽 여백 위"),
    FieldSpec("TABLE", "in_margin_bottom", 0x10, 2, "u16", None, False, "안쪽 여백 아래"),
    FieldSpec("TABLE", "row_count_hint", 0x12, 2, "u16", None, True, "관찰명. 행 힌트"),
    FieldSpec("TABLE", "col_count_hint", 0x14, 2, "u16", None, True, "관찰명. 열 힌트"),
    FieldSpec(
        "TABLE",
        "tail_after_0x16",
        0x16,
        16,
        "hex",
        "table_tail",
        True,
        "관찰명. 0x16 이후 tail. table-probe 는 전체 tail 을 이식",
    ),
)


PROBE_AXES: tuple[tuple[str, str, str, str], ...] = (
    (
        "ctrl_outer_margin",
        "CTRL_HEADER(Table)",
        "out_margin_left,out_margin_right,out_margin_top,out_margin_bottom",
        "TABLE control wrapper의 바깥 여백 4필드",
    ),
    (
        "ctrl_common_attr",
        "CTRL_HEADER(Table)",
        "common_attr",
        "TABLE control wrapper 공통 속성 비트",
    ),
    (
        "table_attr",
        "TABLE",
        "table_attr",
        "TABLE record 첫 4바이트 속성 비트",
    ),
    (
        "table_tail",
        "TABLE",
        "table_tail_full",
        "TABLE record 0x16 이후 tail payload",
    ),
)


PROBE_VARIANTS: tuple[tuple[str, tuple[str, ...], str], ...] = (
    (
        "01_ctrl_outer_margin_only",
        ("ctrl_outer_margin",),
        "TABLE CTRL_HEADER outer margin 단독 효과 확인",
    ),
    ("02_table_attr_only", ("table_attr",), "TABLE attr 단독 효과 확인"),
    ("03_table_tail_only", ("table_tail",), "TABLE payload tail 단독 효과 확인"),
    (
        "04_ctrl_common_attr_only",
        ("ctrl_common_attr",),
        "TABLE CTRL_HEADER common attr 차이의 영향 확인",
    ),
    (
        "05_outer_margin_table_attr",
        ("ctrl_outer_margin", "table_attr"),
        "위치/속성 축 결합 확인",
    ),
    (
        "06_outer_margin_table_tail",
        ("ctrl_outer_margin", "table_tail"),
        "위치/tail 축 결합 확인",
    ),
    (
        "07_table_attr_tail",
        ("table_attr", "table_tail"),
        "TABLE record 내부 축 결합 확인",
    ),
    (
        "08_all_table_axes",
        ("ctrl_outer_margin", "ctrl_common_attr", "table_attr", "table_tail"),
        "TABLE 관련 관찰 축 전체 positive guard",
    ),
)


ALIGN_MODES = ("index", "lcs")
REPORT_MODES = ("diff", "hints", "bundles", "table-fields", "table-probe-plan")
FOCUS_MODES = ("all", "table", "shape", "ctrl", "missing", "docinfo")
OUTPUT_FORMATS = ("jsonl", "md")
DIFF_KINDS = (
    "missing",
    "extra",
    "changed",
    "tag_changed",
    "size_changed",
    "payload_changed",
    "scope_changed",
    "control_changed",
)
CHANGED_FIELDS = (
    "tag",
    "size",
    "payload_hash",
    "scope_path",
    "control",
    "control_name",
)
HANCOM_JUDGMENTS = (
    "파일 읽기 오류",
    "파일 손상",
    "열림 + 조판 실패",
    "성공",
    "rhwp-studio 정상 + 한컴 실패",
)


FAILURE_CLASSES: tuple[FailureClass, ...] = (
    FailureClass(
        "A",
        "Container / Stream Contract",
        (
            "FileHeader",
            "DocInfo stream",
            "BodyText/Section stream",
            "BinData stream",
            "compression flag",
            "stream size",
            "section_count",
        ),
        (
            "파일 읽기 오류",
            "파일 크기가 비정상적으로 작거나 큼",
            "rhwp-studio는 열지만 한컴이 초기 로딩에서 실패",
        ),
        ("stream_path", "section", "owner", "size"),
        ("missing", "extra", "size_changed"),
        "docinfo",
        "DocInfo / FileHeader graft 를 BodyText 보다 먼저 분리한다.",
    ),
    FailureClass(
        "B",
        "Record Tree Contract",
        (
            "CTRL_HEADER -> concrete control record",
            "LIST_HEADER -> child paragraph records",
            "PARA_HEADER -> PARA_TEXT / PARA_CHAR_SHAPE / PARA_LINE_SEG",
            "TABLE -> CELL/LIST_HEADER/TEXT subtree",
            "SHAPE_COMPONENT / SHAPE_PICTURE subtree",
        ),
        (
            "일부 출력 후 파일 손상",
            "특정 표, 그림, 문단 직후 중단",
            "rhwp-studio는 정상 렌더링",
        ),
        ("scope_path", "level", "tuple_role", "parent_uid", "record_index"),
        ("missing", "extra", "tag_changed", "scope_changed"),
        "missing",
        "중단 지점 직후 record 를 oracle 단위로 graft 한다.",
    ),
    FailureClass(
        "C",
        "Count / Size / Reference Contract",
        (
            "PARA_HEADER char_count",
            "control mask",
            "LIST_HEADER paragraph count",
            "TABLE row_count / col_count / cell_count / span",
            "DocInfo ID_MAPPINGS",
            "CharShape / ParaShape / BorderFill / BinData reference id",
        ),
        (
            "파일 손상",
            "일부 셀이나 일부 문단 이후 중단",
            "특정 object 삽입 후 다음 record가 밀린 듯 보임",
        ),
        ("size", "key_payload", "payload_hash", "control_id"),
        ("size_changed", "payload_changed", "control_changed"),
        "ctrl",
        "count/size 필드를 table-fields 또는 payload hex 로 좁힌다.",
    ),
    FailureClass(
        "D",
        "DocInfo / BinData Contract",
        (
            "BIN_DATA record count",
            "BIN_DATA type/path/storage id",
            "CFB BinData/BINxxxx stream 존재 여부",
            "picture control의 bin_data_id",
            "image format 변환 여부",
        ),
        (
            "그림 경로 찾기 대화상자",
            "이미지 미출력",
            "한컴은 열지만 이미지가 빠짐",
        ),
        ("control_id", "payload_hash", "stream_path"),
        ("missing", "payload_changed"),
        "docinfo",
        "BIN_DATA 와 CTRL_HEADER(GenShape)/SHAPE_PICTURE 를 한 튜플로 본다.",
    ),
    FailureClass(
        "E",
        "Missing HWP Defaults",
        (
            "CTRL_HEADER attr",
            "TABLE attr / margin / tail",
            "LIST_HEADER extra",
            "ParaShape attr bits",
            "SectionDef 기본 필드",
            "PageBorderFill / PageDef / ColumnDef 기본값",
        ),
        (
            "한컴에서 열리지만 조판이 다름",
            "셀 텍스트가 위로 올라가거나 클리핑됨",
            "표가 종이 왼쪽에 붙음",
        ),
        ("key_payload", "payload_hash", "size"),
        ("payload_changed", "size_changed"),
        "table",
        "table-probe 한 축만 이식하는 variant 로 기본값 축을 가른다.",
    ),
    FailureClass(
        "F",
        "Layout-computed Values",
        (
            "PARA_LINE_SEG",
            "line height / baseline",
            "object vpos / hpos",
            "table row height",
            "page break result",
        ),
        (
            "rhwp-studio 렌더링과 저장 HWP 재로드가 다름",
            "한컴 저장 정답 HWP와 generated HWP의 lineSegArray가 다름",
            "HWPX lineSegArray를 그대로 쓰면 표/문단 혼합 케이스가 깨짐",
        ),
        ("tuple_role", "payload_hash", "size", "body_order"),
        ("payload_changed", "size_changed", "missing", "extra"),
        "all",
        "PARA_LINE_SEG 를 페이지 수 로직과 섞지 않는다. inventory payload 만 대조한다.",
    ),
)


CLI_EXIT_CODES = (
    {
        "command": "hwp5-inventory",
        "args": [],
        "exit": 2,
        "stdout": "empty",
        "stderr": "usage",
        "note": "인자 없음은 사용법 오류. help 로 취급하면 스크립트가 성공으로 읽는다.",
    },
    {
        "command": "hwp5-inventory",
        "args": ["--help"],
        "exit": 0,
        "stdout": "empty",
        "stderr": "usage",
        "note": "명시적 --help 만 성공.",
    },
    {
        "command": "hwp5-inventory",
        "args": ["does-not-exist.hwp"],
        "exit": 1,
        "stdout": "empty",
        "stderr": "runtime",
        "note": "없는 파일은 런타임 실패.",
    },
    {
        "command": "hwp5-inventory-diff",
        "args": [],
        "exit": 2,
        "stdout": "empty",
        "stderr": "usage",
        "note": "oracle/generated 두 경로가 필요하다.",
    },
    {
        "command": "hwp5-inventory-diff",
        "args": ["--help"],
        "exit": 0,
        "stdout": "empty",
        "stderr": "usage",
        "note": "명시적 --help 만 성공.",
    },
    {
        "command": "hwp5-table-probe",
        "args": [],
        "exit": 2,
        "stdout": "empty",
        "stderr": "usage",
        "note": "oracle/generated/--out-dir 가 필요하다.",
    },
    {
        "command": "hwp5-table-probe",
        "args": ["--help"],
        "exit": 0,
        "stdout": "empty",
        "stderr": "usage",
        "note": "명시적 --help 만 성공.",
    },
)


INVENTORY_COLUMNS = (
    "sample",
    "source_path",
    "stream_path",
    "section",
    "record_index",
    "record_uid",
    "level",
    "tag_id",
    "tag_name",
    "size",
    "owner",
    "parent_uid",
    "parent_scope",
    "scope_path",
    "body_order",
    "control_id",
    "control_name",
    "tuple_role",
    "tuple_index",
    "payload_head_hex",
    "key_payload",
    "payload_hash",
    "note",
)


DIFF_COLUMNS = (
    "align_mode",
    "alignment_status",
    "diff_kind",
    "key",
    "stream_path",
    "section",
    "record_index",
    "oracle_record_index",
    "generated_record_index",
    "oracle_record_uid",
    "generated_record_uid",
    "signature",
    "changed_fields",
    "note",
)


def tag_by_name(name: str) -> TagSpec:
    for tag in TAGS:
        if tag.tag_name == name:
            return tag
    raise KeyError(name)


def control_by_fourcc(fourcc: str) -> ControlSpec:
    for control in CONTROLS:
        if control.fourcc == fourcc:
            return control
    raise KeyError(fourcc)


def failure_class(code: str) -> FailureClass:
    for item in FAILURE_CLASSES:
        if item.code == code:
            return item
    raise KeyError(code)


def fields_for(record_kind: str) -> tuple[FieldSpec, ...]:
    if record_kind == "CTRL_HEADER":
        return TABLE_CTRL_FIELDS
    if record_kind == "TABLE":
        return TABLE_RECORD_FIELDS
    return ()
