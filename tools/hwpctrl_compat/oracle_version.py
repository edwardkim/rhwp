"""한글 COM 오라클 버전 판정 공통 함수."""

from __future__ import annotations

import re


def major_version(value: object) -> int | None:
    """`12, 0, 0, 4547`과 `12.0.0.4547` 모두에서 major를 읽는다."""
    match = re.match(r"\s*(\d+)", str(value or ""))
    return int(match.group(1)) if match else None


def matches_expected_version(actual: object, expected: str | None) -> bool:
    """기대 major가 비어 있으면 허용하고, 아니면 숫자 major만 비교한다."""
    if not expected:
        return True
    actual_major = major_version(actual)
    expected_major = major_version(expected)
    return actual_major is not None and expected_major is not None and actual_major == expected_major
