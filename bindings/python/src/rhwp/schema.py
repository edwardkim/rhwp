"""JSON Schema 소비 — IR·명령 표면의 타입 정보를 노출한다.

바인딩이 IR 모양을 **하드코딩하지 않는** 이유가 여기 있다. rhwp 가 IR 에 필드를
더하면 스키마가 먼저 알려주고, 코드 생성기(`tools/gen_models.py`)가 그걸 읽어
모델을 다시 만든다. 수기 목록을 두면 반드시 뒤처진다.
"""

from __future__ import annotations

from typing import Any, Dict, Iterator, List, Mapping, Optional, Tuple

from ._process import DEFAULT_TIMEOUT, run_json
from .models import Envelope

__all__ = [
    "capabilities_schema",
    "capabilities_schema_envelope",
    "ir_schema",
    "ir_schema_envelope",
    "IrSchema",
    "TypeDef",
    "FieldDef",
]


class FieldDef:
    """스키마가 서술하는 필드 하나."""

    __slots__ = ("name", "raw", "required")

    def __init__(self, name: str, raw: Mapping[str, Any], required: bool) -> None:
        self.name = name
        self.raw = dict(raw)
        self.required = required

    @property
    def description(self) -> str:
        """설명 — 생성된 바인딩의 docstring 원천."""
        return str(self.raw.get("description", ""))

    @property
    def json_type(self) -> Optional[str]:
        """JSON 타입 (``object``/``array``/``string``/``integer``/``boolean``)."""
        value = self.raw.get("type")
        return value if isinstance(value, str) else None

    @property
    def ref(self) -> Optional[str]:
        """다른 정의를 가리키면 그 이름. 아니면 ``None``."""
        return _ref_name(self.raw)

    @property
    def item_ref(self) -> Optional[str]:
        """배열이면 항목이 가리키는 정의 이름."""
        items = self.raw.get("items")
        return _ref_name(items) if isinstance(items, Mapping) else None

    @property
    def enum_values(self) -> Optional[List[str]]:
        """열거형이면 허용 값 목록."""
        values = self.raw.get("enum")
        if isinstance(values, list):
            return [str(v) for v in values]
        return None

    @property
    def python_type(self) -> str:
        """파이썬 타입 힌트 문자열 — 코드 생성기가 그대로 쓴다."""
        if self.ref:
            return self.ref
        if self.enum_values:
            return "str"
        mapping = {
            "string": "str",
            "integer": "int",
            "number": "float",
            "boolean": "bool",
            "object": "Dict[str, Any]",
        }
        json_type = self.json_type
        if json_type == "array":
            inner = self.item_ref or _scalar_hint(self.raw.get("items"))
            return f"List[{inner}]"
        if json_type in mapping:
            return mapping[json_type]
        # oneOf 로 null 을 허용하는 형태.
        one_of = self.raw.get("oneOf")
        if isinstance(one_of, list):
            names = [_ref_name(o) for o in one_of if isinstance(o, Mapping)]
            concrete = next((n for n in names if n), None)
            if concrete:
                return f"Optional[{concrete}]"
        return "Any"

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        mark = "" if self.required else "?"
        return f"FieldDef({self.name}{mark}: {self.python_type})"


class TypeDef:
    """스키마 정의(`$defs` 항목) 하나."""

    __slots__ = ("name", "raw")

    def __init__(self, name: str, raw: Mapping[str, Any]) -> None:
        self.name = name
        self.raw = dict(raw)

    @property
    def description(self) -> str:
        return str(self.raw.get("description", ""))

    @property
    def is_object(self) -> bool:
        return self.raw.get("type") == "object"

    @property
    def is_union(self) -> bool:
        """``oneOf`` 태그 유니온인지 (예: Control)."""
        return isinstance(self.raw.get("oneOf"), list)

    @property
    def variants(self) -> List[str]:
        """유니온이면 변형 정의 이름 목록."""
        one_of = self.raw.get("oneOf")
        if not isinstance(one_of, list):
            return []
        return [n for n in (_ref_name(o) for o in one_of if isinstance(o, Mapping)) if n]

    @property
    def fields(self) -> List[FieldDef]:
        """필드 목록 (필수가 앞, 그 안에서 이름순)."""
        props = self.raw.get("properties")
        if not isinstance(props, Mapping):
            return []
        required = set(self.raw.get("required") or [])
        defs = [FieldDef(name, spec, name in required) for name, spec in props.items()]
        defs.sort(key=lambda f: (not f.required, f.name))
        return defs

    def field(self, name: str) -> FieldDef:
        """이름으로 필드 하나. 없으면 :class:`KeyError`."""
        for f in self.fields:
            if f.name == name:
                return f
        raise KeyError(f"{self.name} 에 '{name}' 필드가 없습니다")

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        return f"TypeDef({self.name}, {len(self.fields)} fields)"


class IrSchema:
    """`export-ir-schema` 또는 `export-capabilities-schema` 결과를 순회 가능한 형태로."""

    def __init__(self, envelope: Mapping[str, Any]) -> None:
        self._envelope = dict(envelope)
        body = self._envelope.get("schema", self._envelope)
        if not isinstance(body, Mapping):
            raise TypeError("스키마 본문이 객체가 아닙니다")
        self._body = dict(body)
        defs = self._body.get("$defs")
        self._defs: Dict[str, Any] = dict(defs) if isinstance(defs, Mapping) else {}

    @property
    def version(self) -> str:
        """IR 또는 명령 표면 스키마 버전 — 봉투 schemaVersion 과 별개다."""
        value = (
            self._envelope.get("irSchemaVersion")
            or self._envelope.get("capabilitiesSchemaVersion")
            or self._body.get("irSchemaVersion")
            or self._body.get("capabilitiesSchemaVersion")
            or "unknown"
        )
        return value if isinstance(value, str) else "unknown"

    @property
    def dialect(self) -> str:
        """JSON Schema 방언 URI."""
        return str(self._envelope.get("dialect") or self._body.get("$schema") or "")

    @property
    def root(self) -> TypeDef:
        """루트 타입 (Document)."""
        ref = _ref_name(self._body) or "Document"
        return self[ref]

    def __getitem__(self, name: str) -> TypeDef:
        if name not in self._defs:
            raise KeyError(
                f"스키마에 '{name}' 정의가 없습니다. 있는 정의: {', '.join(sorted(self._defs))}"
            )
        return TypeDef(name, self._defs[name])

    def __contains__(self, name: object) -> bool:
        return name in self._defs

    def __iter__(self) -> Iterator[TypeDef]:
        for name in sorted(self._defs):
            yield TypeDef(name, self._defs[name])

    def __len__(self) -> int:
        return len(self._defs)

    def names(self) -> List[str]:
        """정의 이름 목록 (정렬)."""
        return sorted(self._defs)

    def dangling_references(self) -> List[Tuple[str, str]]:
        """끊어진 ``$ref`` 를 (참조한 곳, 없는 이름) 으로 돌려준다.

        코드 생성 전에 이걸 확인하면 생성기가 절반쯤 만들다 죽는 일을 막는다.
        """
        broken: List[Tuple[str, str]] = []
        for type_def in self:
            for field in type_def.fields:
                for target in (field.ref, field.item_ref):
                    if target and target not in self._defs:
                        broken.append((f"{type_def.name}.{field.name}", target))
            for variant in type_def.variants:
                if variant not in self._defs:
                    broken.append((type_def.name, variant))
        return broken

    @property
    def raw(self) -> Dict[str, Any]:
        """원문 스키마 본문."""
        return dict(self._body)

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        return f"IrSchema(v{self.version}, {len(self._defs)} defs)"


def ir_schema(*, timeout: Optional[float] = DEFAULT_TIMEOUT) -> IrSchema:
    """rhwp 에서 IR JSON Schema 를 읽어 온다.

    문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
    속성이 아니다.
    """
    envelope = run_json(["export-ir-schema", "--json"], timeout=timeout)
    return IrSchema(envelope)


def capabilities_schema(*, timeout: Optional[float] = DEFAULT_TIMEOUT) -> IrSchema:
    """rhwp 명령 표면의 JSON Schema 를 읽어 온다.

    문서를 입력으로 받지 않는다. ``capabilities`` 가 명령·플래그·봉투를 설명한다면,
    이 스키마는 그 설명을 기계적으로 소비할 수 있는 모양을 제공한다.
    """
    envelope = run_json(["export-capabilities-schema", "--json"], timeout=timeout)
    return IrSchema(envelope)


def _ref_name(spec: Any) -> Optional[str]:
    """``{"$ref": "#/$defs/X"}`` 에서 ``X`` 를 꺼낸다."""
    if not isinstance(spec, Mapping):
        return None
    ref = spec.get("$ref")
    if isinstance(ref, str) and ref.startswith("#/$defs/"):
        return ref[len("#/$defs/") :]
    return None


def _scalar_hint(spec: Any) -> str:
    """배열 항목이 원시 타입일 때의 힌트."""
    if not isinstance(spec, Mapping):
        return "Any"
    mapping = {"string": "str", "integer": "int", "number": "float", "boolean": "bool"}
    json_type = spec.get("type")
    if isinstance(json_type, str) and json_type in mapping:
        return mapping[json_type]
    return "Any"


# Envelope 를 쓰는 소비자를 위한 편의 — 봉투 그대로 받고 싶을 때.
def ir_schema_envelope(*, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """봉투를 그대로 돌려준다 (definitionCount 등 메타 포함)."""
    return Envelope(run_json(["export-ir-schema", "--json"], timeout=timeout))


def capabilities_schema_envelope(*, timeout: Optional[float] = DEFAULT_TIMEOUT) -> Envelope:
    """명령 표면 스키마 봉투를 그대로 돌려준다 (MCP 스키마 메타 포함)."""
    return Envelope(run_json(["export-capabilities-schema", "--json"], timeout=timeout))
