"""봉투 키(camelCase) ↔ 파이썬 속성(snake_case) **기계** 변환.

수기 개명을 금지하는 것이 이 모듈의 요점이다 (`bindings_foundation.md` §3).
사람이 이름을 다시 붙이기 시작하면 봉투에 필드가 하나 늘 때마다 바인딩이
뒤처지고, 어느 쪽이 맞는지 알 수 없게 된다. 규칙을 코드로 고정하면 새 필드는
자동으로 따라온다.
"""

from __future__ import annotations

import re
from typing import Any, Dict, List, Mapping

__all__ = ["to_snake", "to_camel", "snake_keys", "camel_keys"]

# 연속 대문자(약어) 경계를 살린다: "pageCountA" → "page_count_a",
# "irDiff" → "ir_diff", "sourceB" → "source_b", "HTMLPage" → "html_page".
_ACRONYM_BOUNDARY = re.compile(r"([A-Z]+)([A-Z][a-z])")
_WORD_BOUNDARY = re.compile(r"([a-z0-9])([A-Z])")


def to_snake(name: str) -> str:
    """camelCase → snake_case.

    >>> to_snake("pageCount")
    'page_count'
    >>> to_snake("changedPages")
    'changed_pages'
    >>> to_snake("sourceA")
    'source_a'
    >>> to_snake("irSchemaVersion")
    'ir_schema_version'
    >>> to_snake("already_snake")
    'already_snake'
    """
    if not name:
        return name
    step = _ACRONYM_BOUNDARY.sub(r"\1_\2", name)
    step = _WORD_BOUNDARY.sub(r"\1_\2", step)
    return step.lower()


def to_camel(name: str) -> str:
    """snake_case → camelCase (봉투로 되돌려 보낼 때).

    >>> to_camel("page_count")
    'pageCount'
    >>> to_camel("dry_run")
    'dryRun'
    >>> to_camel("alreadyCamel")
    'alreadyCamel'
    """
    if not name or "_" not in name:
        return name
    head, *rest = name.split("_")
    return head + "".join(part[:1].upper() + part[1:] for part in rest if part)


def snake_keys(value: Any) -> Any:
    """중첩 구조 전체의 키를 snake_case 로 변환한다 (리스트 내부까지).

    값은 건드리지 않는다 — 필드 *이름*만 규약을 따르고, 내용은 봉투 그대로다.
    """
    if isinstance(value, Mapping):
        out: Dict[str, Any] = {}
        for key, item in value.items():
            out[to_snake(key) if isinstance(key, str) else key] = snake_keys(item)
        return out
    if isinstance(value, list):
        return [snake_keys(item) for item in value]
    return value


def camel_keys(value: Any) -> Any:
    """중첩 구조 전체의 키를 camelCase 로 되돌린다 (계획서를 보낼 때)."""
    if isinstance(value, Mapping):
        out: Dict[str, Any] = {}
        for key, item in value.items():
            out[to_camel(key) if isinstance(key, str) else key] = camel_keys(item)
        return out
    if isinstance(value, list):
        return [camel_keys(item) for item in value]
    return value


def reserved_safe(name: str) -> str:
    """파이썬 예약어와 충돌하면 뒤에 밑줄을 붙인다 (``from`` → ``from_``)."""
    import keyword

    return f"{name}_" if keyword.iskeyword(name) else name


def field_names(envelope: Mapping[str, Any]) -> List[str]:
    """봉투 최상위 키를 파이썬 속성명으로 (정렬)."""
    return sorted(reserved_safe(to_snake(k)) for k in envelope if isinstance(k, str))
