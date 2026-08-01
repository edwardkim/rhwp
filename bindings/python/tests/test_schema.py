"""IR 스키마 소비 계층 — 하드코딩 없이 타입 정보를 읽는다.

바인딩이 IR 모양을 손으로 적지 않는 이유를 지키는 테스트다. 스키마가 먼저
알려주고 생성기가 따라간다.
"""

from __future__ import annotations

from typing import Any, Dict

import pytest

from rhwp.schema import FieldDef, IrSchema, TypeDef

# 실제 rhwp 없이도 로직을 검증할 수 있게 만든 최소 스키마.
FAKE: Dict[str, Any] = {
    "schemaVersion": "1.0",
    "irSchemaVersion": "1.0",
    "dialect": "https://json-schema.org/draft/2020-12/schema",
    "definitionCount": 4,
    "schema": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "irSchemaVersion": "1.0",
        "$ref": "#/$defs/Document",
        "$defs": {
            "Document": {
                "type": "object",
                "description": "문서 루트",
                "properties": {
                    "sections": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Section"},
                        "description": "구역 목록",
                    },
                    "title": {"type": "string", "description": "제목"},
                    "pageCount": {"type": "integer", "description": "쪽수"},
                    "preview": {
                        "oneOf": [{"$ref": "#/$defs/Preview"}, {"type": "null"}],
                        "description": "미리보기",
                    },
                },
                "required": ["sections"],
                "additionalProperties": True,
            },
            "Section": {
                "type": "object",
                "description": "구역",
                "properties": {
                    "index": {"type": "integer", "description": "구역 번호"},
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "태그",
                    },
                },
                "required": ["index"],
                "additionalProperties": True,
            },
            "Preview": {
                "type": "object",
                "description": "미리보기",
                "properties": {"hasImage": {"type": "boolean", "description": "이미지 여부"}},
                "required": [],
                "additionalProperties": True,
            },
            "Control": {
                "description": "컨트롤 유니온",
                "oneOf": [{"$ref": "#/$defs/Section"}, {"$ref": "#/$defs/Preview"}],
            },
        },
    },
}


@pytest.fixture
def schema() -> IrSchema:
    return IrSchema(FAKE)


def test_version_and_dialect_are_exposed(schema: IrSchema) -> None:
    assert schema.version == "1.0"
    assert "json-schema.org" in schema.dialect


def test_root_resolves_to_document(schema: IrSchema) -> None:
    root = schema.root
    assert isinstance(root, TypeDef)
    assert root.name == "Document"
    assert root.description == "문서 루트"


def test_iteration_yields_every_definition(schema: IrSchema) -> None:
    names = [t.name for t in schema]
    assert names == sorted(names), "정렬돼 나와야 결정론적이다"
    assert set(names) == {"Document", "Section", "Preview", "Control"}
    assert len(schema) == 4


def test_missing_definition_lists_available_ones(schema: IrSchema) -> None:
    with pytest.raises(KeyError) as caught:
        _ = schema["없는타입"]
    assert "Document" in str(caught.value)


def test_required_fields_come_first(schema: IrSchema) -> None:
    fields = schema["Document"].fields
    assert fields[0].name == "sections"
    assert fields[0].required
    assert all(not f.required for f in fields[1:])


def test_python_type_hints_are_generated(schema: IrSchema) -> None:
    doc = schema["Document"]
    assert doc.field("sections").python_type == "List[Section]"
    assert doc.field("title").python_type == "str"
    assert doc.field("pageCount").python_type == "int"
    assert doc.field("preview").python_type == "Optional[Preview]"

    section = schema["Section"]
    assert section.field("tags").python_type == "List[str]"
    assert schema["Preview"].field("hasImage").python_type == "bool"


def test_union_variants_are_listed(schema: IrSchema) -> None:
    control = schema["Control"]
    assert control.is_union
    assert set(control.variants) == {"Section", "Preview"}


def test_object_and_union_are_distinguished(schema: IrSchema) -> None:
    assert schema["Document"].is_object
    assert not schema["Document"].is_union
    assert schema["Control"].is_union
    assert not schema["Control"].is_object


def test_dangling_references_are_reported() -> None:
    """생성기가 절반쯤 만들다 죽지 않도록 미리 잡는다."""
    broken = dict(FAKE)
    broken["schema"] = {
        **FAKE["schema"],
        "$defs": {
            **FAKE["schema"]["$defs"],
            "Document": {
                "type": "object",
                "description": "문서",
                "properties": {"ghost": {"$ref": "#/$defs/없는타입"}},
                "required": [],
                "additionalProperties": True,
            },
        },
    }
    dangling = IrSchema(broken).dangling_references()
    assert ("Document.ghost", "없는타입") in dangling


def test_healthy_schema_has_no_dangling_references(schema: IrSchema) -> None:
    assert schema.dangling_references() == []


def test_field_missing_raises_with_context(schema: IrSchema) -> None:
    with pytest.raises(KeyError) as caught:
        schema["Document"].field("없는필드")
    assert "Document" in str(caught.value)


def test_non_mapping_schema_is_rejected() -> None:
    with pytest.raises(TypeError):
        IrSchema({"schema": "객체가 아님"})


def test_field_repr_marks_optional(schema: IrSchema) -> None:
    required = schema["Document"].field("sections")
    optional = schema["Document"].field("title")
    assert isinstance(required, FieldDef)
    assert "?" not in repr(required)
    assert "?" in repr(optional)


# ── 실제 바이너리 통합 ──────────────────────────────────────────────────


@pytest.mark.integration
def test_real_schema_is_consumable(wired_binary: object) -> None:
    """실물 스키마가 소비 계층으로 온전히 읽히는지."""
    import rhwp

    schema = rhwp.ir_schema()
    assert schema.version, "IR 버전이 없다"
    assert len(schema) >= 25, f"정의가 너무 적다: {len(schema)}"
    assert schema.root.name == "Document"
    assert schema.dangling_references() == [], "끊어진 참조가 있다"

    # 편집 API 가 쓰는 타입이 실제로 서술돼 있어야 한다.
    for required in ("Section", "Paragraph", "TableControl", "TableCell", "FieldRange"):
        assert required in schema, f"{required} 정의 없음"

    # 모든 정의에 설명이 있어야 생성된 바인딩에 docstring 이 붙는다.
    undocumented = [t.name for t in schema if not t.description.strip()]
    assert not undocumented, f"설명 없는 정의: {undocumented}"


@pytest.mark.integration
def test_real_schema_generates_valid_type_hints(wired_binary: object) -> None:
    """모든 필드가 유효한 파이썬 타입 힌트로 환산돼야 코드 생성이 성립한다."""
    import rhwp

    schema = rhwp.ir_schema()
    for type_def in schema:
        for field in type_def.fields:
            hint = field.python_type
            assert hint and not hint.startswith("List[]"), (
                f"{type_def.name}.{field.name} 의 타입 힌트가 이상하다: {hint}"
            )
