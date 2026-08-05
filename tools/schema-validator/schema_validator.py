#!/usr/bin/env python3
"""rhwp ingest JSON 스키마 검증기.

`rhwp build-from-ingest` 에 넘기기 전에 ingest JSON 을
`tools/rhwp-ingest/schema/ingest_schema_v1.json` (JSON Schema draft-07 부분집합)
기준으로 선검사한다. 표준 라이브러리만 사용한다.

판정 규약
---------
- ERROR   : 스키마 위반. 종료 코드 1, `--json` 출력의 `valid` 는 false.
- WARNING : 스키마상 허용이지만 주의가 필요한 신호(예: 스키마 `properties` 에
            정의되지 않은 필드 — Rust 측 `build-from-ingest` 는
            `deny_unknown_fields` 로 이런 필드를 거부한다).
            종료 코드와 `valid` 판정에는 영향을 주지 않는다.

핵심 계약 (#4044 리뷰 반영)
---------------------------
`_validate()` 의 반환 bool 은 "이 값(과 그 모든 하위 값)이 스키마에 부합하는가"다.
중첩 검증(`properties`/`items`/`oneOf` 대안) 실패는 반드시 상위 반환값으로
전파된다. `oneOf` 는 draft-07 의미론 그대로 **정확히 한 대안**과 일치해야 하며,
0개 일치·2개 이상 일치는 모두 ERROR 다.

지원 키워드: type, const, enum, required, properties, additionalProperties,
items(단일 스키마), oneOf, minimum, maximum, minLength, maxLength,
$ref(#/definitions/* 한정). 그 외 키워드(default, description 등)는 주석으로
간주하고 무시한다.

종료 코드: 0 = 유효(ERROR 0건) / 1 = 스키마 위반 / 2 = 사용법·환경 오류.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

EXIT_OK = 0
EXIT_INVALID = 1
EXIT_USAGE = 2


class ErrorLevel(Enum):
    """검증 메시지 심각도."""

    ERROR = "ERROR"
    WARNING = "WARNING"


@dataclass
class ValidationError:
    """검증 실패 1건 — JSON 경로와 오류 코드로 위치를 특정한다."""

    level: ErrorLevel
    message: str
    path: str  # 예: "questions[0].choices[1]"
    code: str  # 예: "TYPE_MISMATCH"
    position: Optional[str] = None  # JSON 구문 오류에만 "Line L, Column C"

    def __str__(self) -> str:
        pos = f" ({self.position})" if self.position else ""
        return f"[{self.level.value}]{pos} {self.path}: {self.message}"


def _type_name(value: Any) -> str:
    """파이썬 값의 JSON 타입명."""
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, (int, float)):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, list):
        return "array"
    if isinstance(value, dict):
        return "object"
    return type(value).__name__


def _check_single_type(value: Any, type_name: str) -> bool:
    if type_name == "object":
        return isinstance(value, dict)
    if type_name == "array":
        return isinstance(value, list)
    if type_name == "string":
        return isinstance(value, str)
    if type_name == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if type_name == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if type_name == "boolean":
        return isinstance(value, bool)
    if type_name == "null":
        return value is None
    return False


class IngestSchemaValidator:
    """ingest JSON 을 스키마와 대조한다.

    사용:
        validator = IngestSchemaValidator(schema_path)
        errors = validator.validate_file(json_path)   # List[ValidationError]
        ok = not any(e.level is ErrorLevel.ERROR for e in errors)
    """

    def __init__(self, schema_path: Union[str, Path]):
        with open(schema_path, encoding="utf-8") as f:
            self.schema: Dict[str, Any] = json.load(f)

    # ── 공개 API ──────────────────────────────────────────────────────────

    def validate_file(self, file_path: Union[str, Path]) -> List[ValidationError]:
        """파일을 읽고 파싱한 뒤 검증한다. 구문 오류는 줄/칸 위치를 담는다."""
        errors: List[ValidationError] = []
        try:
            content = Path(file_path).read_text(encoding="utf-8")
        except OSError as e:
            errors.append(
                ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"파일을 읽을 수 없습니다: {e}",
                    path="$",
                    code="FILE_READ_ERROR",
                )
            )
            return errors

        try:
            data = json.loads(content)
        except json.JSONDecodeError as e:
            errors.append(
                ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"올바른 JSON 이 아닙니다: {e.msg}",
                    path="$",
                    code="INVALID_JSON",
                    position=f"Line {e.lineno}, Column {e.colno}",
                )
            )
            return errors

        return self.validate(data)

    def validate(self, data: Any) -> List[ValidationError]:
        """이미 파싱된 값을 검증한다. 스키마 자체 결함은 ValueError 로 던진다."""
        errors: List[ValidationError] = []
        self._validate(data, self.schema, "$", errors)
        return errors

    # ── 내부 구현 ─────────────────────────────────────────────────────────

    def _deref(self, schema: Dict[str, Any]) -> Dict[str, Any]:
        """`$ref` 를 실제 정의로 치환한다. 못 찾으면 조용히 통과시키지 않고 실패한다."""
        seen = 0
        while "$ref" in schema:
            ref = schema["$ref"]
            if not isinstance(ref, str) or not ref.startswith("#/definitions/"):
                raise ValueError(f"스키마 오류: 지원하지 않는 $ref 형식입니다: {ref!r}")
            name = ref[len("#/definitions/") :]
            target = self.schema.get("definitions", {}).get(name)
            if target is None:
                raise ValueError(f"스키마 오류: $ref 대상 정의가 없습니다: {ref}")
            schema = target
            seen += 1
            if seen > 32:
                raise ValueError(f"스키마 오류: $ref 순환 참조 의심: {ref}")
        return schema

    def _validate(
        self,
        value: Any,
        schema: Dict[str, Any],
        path: str,
        errors: List[ValidationError],
    ) -> bool:
        """값 하나를 스키마와 대조한다.

        반환 bool == "value 와 그 모든 하위 값이 schema 에 부합"이다.
        하위 검증(_validate_object/_validate_array/oneOf 대안) 실패는 이 반환값에
        반드시 반영된다 — 이전 구현은 하위 결과를 버려서 oneOf 판정이 전부
        참으로 계산되는 버그가 있었다.
        """
        schema = self._deref(schema)

        # type — 불일치면 이후 검사는 의미가 없으므로 즉시 종료
        if "type" in schema:
            expected = schema["type"]
            allowed = expected if isinstance(expected, list) else [expected]
            if not any(_check_single_type(value, t) for t in allowed):
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=(
                            f"타입이 {' 또는 '.join(allowed)} 이어야 하는데 "
                            f"{_type_name(value)} 입니다"
                        ),
                        path=path,
                        code="TYPE_MISMATCH",
                    )
                )
                return False

        if "const" in schema and value != schema["const"]:
            errors.append(
                ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"값이 {schema['const']!r} 이어야 하는데 {value!r} 입니다",
                    path=path,
                    code="CONST_MISMATCH",
                )
            )
            return False

        if "enum" in schema and value not in schema["enum"]:
            errors.append(
                ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"값이 {schema['enum']} 중 하나여야 하는데 {value!r} 입니다",
                    path=path,
                    code="ENUM_MISMATCH",
                )
            )
            return False

        ok = True

        if "oneOf" in schema:
            ok = self._validate_one_of(value, schema["oneOf"], path, errors) and ok

        if isinstance(value, dict) and (
            schema.get("type") == "object" or "properties" in schema or "required" in schema
        ):
            ok = self._validate_object(value, schema, path, errors) and ok

        if isinstance(value, list) and (schema.get("type") == "array" or "items" in schema):
            ok = self._validate_array(value, schema, path, errors) and ok

        if isinstance(value, (int, float)) and not isinstance(value, bool):
            if "minimum" in schema and value < schema["minimum"]:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=f"값 {value} 이(가) 최솟값 {schema['minimum']} 미만입니다",
                        path=path,
                        code="BELOW_MINIMUM",
                    )
                )
                ok = False
            if "maximum" in schema and value > schema["maximum"]:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=f"값 {value} 이(가) 최댓값 {schema['maximum']} 초과입니다",
                        path=path,
                        code="ABOVE_MAXIMUM",
                    )
                )
                ok = False

        if isinstance(value, str):
            if "minLength" in schema and len(value) < schema["minLength"]:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=(
                            f"문자열 길이 {len(value)} 이(가) 최소 {schema['minLength']} 미만입니다"
                        ),
                        path=path,
                        code="STRING_TOO_SHORT",
                    )
                )
                ok = False
            if "maxLength" in schema and len(value) > schema["maxLength"]:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=(
                            f"문자열 길이 {len(value)} 이(가) 최대 {schema['maxLength']} 초과입니다"
                        ),
                        path=path,
                        code="STRING_TOO_LONG",
                    )
                )
                ok = False

        return ok

    def _validate_one_of(
        self,
        value: Any,
        alternatives: List[Dict[str, Any]],
        path: str,
        errors: List[ValidationError],
    ) -> bool:
        """draft-07 `oneOf`: 정확히 한 대안과 일치해야 통과.

        각 대안은 스크래치 오류 목록으로 격리 검증하고 **반환 bool** 로 일치를
        판정한다(스크래치의 WARNING 은 일치 여부에 영향 없음). 0개 일치는
        ONEOF_FAILED, 2개 이상 일치는 ONEOF_AMBIGUOUS — 둘 다 ERROR 다.
        """
        matched: List[int] = []
        probe_errors: List[List[ValidationError]] = []
        for i, alt in enumerate(alternatives):
            scratch: List[ValidationError] = []
            if self._validate(value, alt, path, scratch):
                matched.append(i)
            probe_errors.append(scratch)

        if len(matched) == 1:
            return True

        if not matched:
            hints = []
            for i, scratch in enumerate(probe_errors):
                first = next((e for e in scratch if e.level is ErrorLevel.ERROR), None)
                if first is not None:
                    hints.append(f"대안[{i}]: {first.message}")
            hint_str = f" ({'; '.join(hints)})" if hints else ""
            errors.append(
                ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"oneOf 의 어느 대안과도 일치하지 않습니다{hint_str}",
                    path=path,
                    code="ONEOF_FAILED",
                )
            )
            return False

        errors.append(
            ValidationError(
                level=ErrorLevel.ERROR,
                message=(
                    f"oneOf 는 정확히 한 대안과 일치해야 하는데 "
                    f"{len(matched)}개 대안(인덱스 {matched})과 일치합니다"
                ),
                path=path,
                code="ONEOF_AMBIGUOUS",
            )
        )
        return False

    def _validate_object(
        self,
        obj: Dict[str, Any],
        schema: Dict[str, Any],
        path: str,
        errors: List[ValidationError],
    ) -> bool:
        ok = True

        for req in schema.get("required", []):
            if req not in obj:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=f"필수 필드 '{req}' 가 없습니다",
                        path=path,
                        code="MISSING_REQUIRED_FIELD",
                    )
                )
                ok = False

        properties = schema.get("properties", {})
        additional = schema.get("additionalProperties", True)

        for key, val in obj.items():
            child_path = key if path == "$" else f"{path}.{key}"
            if key in properties:
                ok = self._validate(val, properties[key], child_path, errors) and ok
            elif additional is False:
                errors.append(
                    ValidationError(
                        level=ErrorLevel.ERROR,
                        message=f"허용되지 않는 필드 '{key}' 입니다",
                        path=child_path,
                        code="UNKNOWN_FIELD",
                    )
                )
                ok = False
            elif isinstance(additional, dict):
                ok = self._validate(val, additional, child_path, errors) and ok
            elif properties:
                # 스키마상 허용(additionalProperties 기본 true)이지만 Rust 측
                # build-from-ingest 는 deny_unknown_fields 로 거부한다. 오탈자
                # 조기 발견용 경고 — valid 판정·종료 코드에는 영향 없음.
                errors.append(
                    ValidationError(
                        level=ErrorLevel.WARNING,
                        message=(
                            f"스키마에 정의되지 않은 필드 '{key}' — "
                            "rhwp build-from-ingest 는 미지 필드를 거부합니다"
                        ),
                        path=child_path,
                        code="UNKNOWN_FIELD",
                    )
                )

        return ok

    def _validate_array(
        self,
        arr: List[Any],
        schema: Dict[str, Any],
        path: str,
        errors: List[ValidationError],
    ) -> bool:
        items_schema = schema.get("items")
        if not isinstance(items_schema, dict):
            return True  # items 미지정 또는 튜플 형식(미지원)은 검사하지 않는다

        ok = True
        for i, item in enumerate(arr):
            ok = self._validate(item, items_schema, f"{path}[{i}]", errors) and ok
        return ok


# ── CLI ──────────────────────────────────────────────────────────────────────


def format_errors(errors: List[ValidationError]) -> str:
    """사람용 출력."""
    if not errors:
        return "검증 통과: 오류 0건, 경고 0건"

    lines = ["검증 결과:", "=" * 70]
    for i, error in enumerate(errors, 1):
        lines.append(f"{i}. {error}")
    lines.append("=" * 70)

    error_count = sum(1 for e in errors if e.level is ErrorLevel.ERROR)
    warning_count = sum(1 for e in errors if e.level is ErrorLevel.WARNING)
    lines.append(f"오류 {error_count}건, 경고 {warning_count}건")
    return "\n".join(lines)


def default_schema_path() -> Path:
    """저장소 canonical 스키마 위치 — 별도 사본을 두지 않는다."""
    return (
        Path(__file__).resolve().parent.parent
        / "rhwp-ingest"
        / "schema"
        / "ingest_schema_v1.json"
    )


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="rhwp ingest JSON 을 ingest_schema_v1.json 과 대조 검증한다"
    )
    parser.add_argument("file", help="검증할 ingest JSON 파일")
    parser.add_argument(
        "--schema",
        default=None,
        help="스키마 파일 경로 (기본: tools/rhwp-ingest/schema/ingest_schema_v1.json)",
    )
    parser.add_argument("--json", action="store_true", help="결과를 JSON 으로 출력")
    parser.add_argument(
        "--quiet", action="store_true", help="출력 없이 종료 코드만 반환"
    )
    args = parser.parse_args(argv)

    # Windows 콘솔(cp949)에서도 한글·원문자 출력이 깨지지 않게 한다.
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            try:
                stream.reconfigure(encoding="utf-8")
            except (OSError, ValueError):
                pass

    schema_path = Path(args.schema) if args.schema else default_schema_path()
    if not schema_path.exists():
        print(f"오류: 스키마 파일이 없습니다: {schema_path}", file=sys.stderr)
        return EXIT_USAGE

    try:
        validator = IngestSchemaValidator(schema_path)
    except json.JSONDecodeError as e:
        print(f"오류: 스키마 파일이 올바른 JSON 이 아닙니다: {e}", file=sys.stderr)
        return EXIT_USAGE

    try:
        errors = validator.validate_file(Path(args.file))
    except ValueError as e:
        # _deref 가 던지는 스키마 자체 결함 — 입력이 아니라 도구 설정 문제다.
        print(f"오류: {e}", file=sys.stderr)
        return EXIT_USAGE

    error_count = sum(1 for e in errors if e.level is ErrorLevel.ERROR)
    warning_count = sum(1 for e in errors if e.level is ErrorLevel.WARNING)

    if args.json:
        # `valid` 는 종료 코드와 동일 기준(ERROR 0건)이다. 이전 구현은 경고까지
        # 세어 valid=false 를 내는 비일관이 있었다(#4044 리뷰 3번).
        result = {
            "valid": error_count == 0,
            "error_count": error_count,
            "warning_count": warning_count,
            "errors": [
                {
                    "level": e.level.value,
                    "path": e.path,
                    "message": e.message,
                    "code": e.code,
                    "position": e.position,
                }
                for e in errors
            ],
        }
        print(json.dumps(result, indent=2, ensure_ascii=False))
    elif not args.quiet:
        print(format_errors(errors))

    return EXIT_OK if error_count == 0 else EXIT_INVALID


if __name__ == "__main__":
    sys.exit(main())
