"""생성된 IR 모델 — 생성기가 만든 코드가 실제로 쓸 만한지.

생성물은 손으로 고치지 않는다. 대신 **생성 규칙**을 여기서 고정한다:
필수/선택 구분, 가변 기본값 회피, 예약어 회피, 이름 변환 일관성.
"""

from __future__ import annotations

import dataclasses
import importlib
import inspect
from typing import Any, get_type_hints

import pytest

ir = pytest.importorskip("rhwp.ir", reason="생성 모델이 아직 없습니다 (gen_models.py 실행 필요)")


def _dataclasses() -> list:
    """생성된 dataclass 전부."""
    return [
        obj
        for _, obj in inspect.getmembers(ir)
        if inspect.isclass(obj) and dataclasses.is_dataclass(obj)
    ]


def test_module_declares_schema_version() -> None:
    """어느 IR 버전에서 생성됐는지 코드가 스스로 말해야 한다."""
    assert hasattr(ir, "IR_SCHEMA_VERSION")
    assert isinstance(ir.IR_SCHEMA_VERSION, str)
    assert ir.IR_SCHEMA_VERSION


def test_document_root_exists() -> None:
    assert hasattr(ir, "Document"), "루트 타입이 생성되지 않았다"
    assert dataclasses.is_dataclass(ir.Document)


def test_core_types_are_generated() -> None:
    """편집 API 가 쓰는 타입이 있어야 바인딩이 표·누름틀을 다룰 수 있다."""
    for name in ("Document", "Section", "Paragraph", "TableControl", "TableCell"):
        assert hasattr(ir, name), f"{name} 이 생성되지 않았다"


def test_all_exports_every_generated_type() -> None:
    exported = set(ir.__all__)
    for cls in _dataclasses():
        assert cls.__name__ in exported, f"{cls.__name__} 이 __all__ 에 없다"


def test_every_dataclass_has_a_docstring() -> None:
    """스키마 description 이 docstring 으로 이어져야 IDE 에서 쓸모가 있다."""
    undocumented = [c.__name__ for c in _dataclasses() if not (c.__doc__ or "").strip()]
    assert not undocumented, f"docstring 없는 생성 클래스: {undocumented}"


def test_optional_fields_have_defaults() -> None:
    """선택 필드에 기본값이 없으면 인스턴스를 만들 수 없다."""
    for cls in _dataclasses():
        fields = dataclasses.fields(cls)
        seen_default = False
        for f in fields:
            has_default = (
                f.default is not dataclasses.MISSING
                or f.default_factory is not dataclasses.MISSING  # type: ignore[misc]
            )
            if has_default:
                seen_default = True
            else:
                # 기본값 있는 필드 뒤에 없는 필드가 오면 파이썬이 거부한다.
                assert not seen_default, (
                    f"{cls.__name__}.{f.name}: 기본값 있는 필드 뒤에 없는 필드가 왔다"
                )


def test_mutable_defaults_use_factory() -> None:
    """리스트·딕셔너리를 기본값으로 공유하면 인스턴스끼리 상태가 새어 나간다."""
    for cls in _dataclasses():
        for f in dataclasses.fields(cls):
            assert not isinstance(f.default, (list, dict, set)), (
                f"{cls.__name__}.{f.name} 이 가변 기본값을 쓴다 — default_factory 여야 한다"
            )


def test_list_fields_default_to_empty_list() -> None:
    for cls in _dataclasses():
        for f in dataclasses.fields(cls):
            hint = str(f.type)
            if hint.startswith("List[") and f.default_factory is not dataclasses.MISSING:  # type: ignore[misc]
                assert f.default_factory() == [], (  # type: ignore[misc]
                    f"{cls.__name__}.{f.name} 의 기본값이 빈 리스트가 아니다"
                )


def test_field_names_are_snake_case() -> None:
    """수기 개명 금지 규약의 결과 — 전부 기계 변환된 이름이어야 한다."""
    for cls in _dataclasses():
        for f in dataclasses.fields(cls):
            assert f.name == f.name.lower(), f"{cls.__name__}.{f.name} 이 snake_case 가 아니다"
            assert not f.name.startswith("_"), f"{cls.__name__}.{f.name} 이 밑줄로 시작한다"


def test_no_python_keyword_collisions() -> None:
    import keyword

    for cls in _dataclasses():
        for f in dataclasses.fields(cls):
            assert not keyword.iskeyword(f.name), (
                f"{cls.__name__}.{f.name} 이 파이썬 예약어와 충돌한다"
            )


def test_types_resolve_without_forward_reference_errors() -> None:
    """전방 참조가 풀려야 런타임 타입 검사·직렬화가 가능하다."""
    for cls in _dataclasses():
        try:
            get_type_hints(cls, vars(ir))
        except NameError as exc:  # pragma: no cover - 실패 시에만
            pytest.fail(f"{cls.__name__} 의 타입 힌트가 풀리지 않는다: {exc}")


def test_instances_can_be_constructed_with_required_fields_only() -> None:
    """선택 필드를 다 채우지 않아도 인스턴스가 만들어져야 실용적이다."""
    made: list[Any] = []
    for cls in _dataclasses():
        required = [
            f
            for f in dataclasses.fields(cls)
            if f.default is dataclasses.MISSING
            and f.default_factory is dataclasses.MISSING  # type: ignore[misc]
        ]
        if required:
            continue  # 필수 필드가 있으면 더미 값을 지어내지 않는다
        made.append(cls())
    # 필수 필드가 없는 타입이 하나라도 있어야 이 테스트가 의미 있다.
    assert made, "필수 필드 없는 생성 타입이 하나도 없다"


def test_module_warns_against_hand_editing() -> None:
    """자동 생성 파일임을 파일 스스로 말해야 한다."""
    source = inspect.getsource(ir)
    assert "자동 생성" in source
    assert "손으로 고치지 마세요" in source


@pytest.mark.integration
def test_generated_models_match_current_schema(wired_binary: object) -> None:
    """생성물이 지금 스키마와 같은 버전인지 — 어긋나면 재생성이 필요하다."""
    import rhwp

    schema = rhwp.ir_schema()
    assert ir.IR_SCHEMA_VERSION == schema.version, (
        f"생성 모델은 IR v{ir.IR_SCHEMA_VERSION}, 현재 스키마는 v{schema.version} — "
        "python tools/gen_models.py -o src/rhwp/ir.py 를 다시 돌리세요"
    )
    # 스키마의 모든 정의가 모델로 나와 있어야 한다.
    generated = {c.__name__ for c in _dataclasses()}
    generated |= {n for n in ir.__all__ if n != "IR_SCHEMA_VERSION"}
    missing = [name for name in schema.names() if name not in generated]
    assert not missing, f"모델로 생성되지 않은 정의: {missing}"
