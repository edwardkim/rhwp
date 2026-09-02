"""Closed catalog of existing mechanical commands.

Every check_id maps to a real `rhwp` (or already-shipped tool) command
and a published envelope field. New CLI verbs are not invented here.
Two rows that share `command_key` are the *same* command — pairing them
is not shadow agreement.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class MechanicalCheck:
    check_id: str
    command: str
    command_key: str
    pass_field: str
    pass_equals: str
    fail_example: str
    meaning_ko: str
    producer: str


# Existing commands only. Fields are the ones those commands already emit.
CHECKS: tuple[MechanicalCheck, ...] = (
    MechanicalCheck(
        "ir-diff",
        "rhwp ir-diff --json",
        "ir-diff",
        "identical",
        "true",
        "false",
        "두 문서 IR 이 같다(identical).",
        "src/main.rs:ir-diff",
    ),
    MechanicalCheck(
        "verify-pages",
        "rhwp verify --expect-pages --json",
        "verify",
        "failCount",
        "0",
        "1",
        "기대 쪽수 조건이 맞다(failCount=0).",
        "src/main.rs:verify",
    ),
    MechanicalCheck(
        "fill-verify",
        "rhwp fill-fields --verify --json",
        "fill-fields",
        "verify.identical",
        "true",
        "false",
        "채움 후 자기검증이 같다(verify.identical).",
        "src/main.rs:fill-fields",
    ),
    MechanicalCheck(
        "layout-anomaly",
        "rhwp layout-anomaly --json",
        "layout-anomaly",
        "hasSignal",
        "false",
        "true",
        "한 장 기하에 확정 이상 신호가 없다(hasSignal=false).",
        "src/main.rs:layout-anomaly",
    ),
    MechanicalCheck(
        "render-diff",
        "rhwp render-diff --json",
        "render-diff",
        "pageCountMismatch",
        "false",
        "true",
        "두 렌더 쪽수가 같다(pageCountMismatch=false).",
        "src/main.rs:render-diff",
    ),
    MechanicalCheck(
        "dump-pages",
        "rhwp dump-pages --json",
        "dump-pages",
        "pageCount",
        "equal",
        "mismatch",
        "dump-pages 쪽수가 짝 쪽수와 같다.",
        "src/main.rs:dump-pages",
    ),
    MechanicalCheck(
        "info",
        "rhwp info --json",
        "info",
        "pageCount",
        "equal",
        "mismatch",
        "info 쪽수가 짝 쪽수와 같다.",
        "src/main.rs:info",
    ),
    MechanicalCheck(
        "inspect-hidden",
        "rhwp inspect hidden-text --json",
        "inspect-hidden-text",
        "clean",
        "true",
        "false",
        "은닉 텍스트가 없다(clean).",
        "src/main.rs:inspect hidden-text",
    ),
    MechanicalCheck(
        "inspect-injection",
        "rhwp inspect injection --json",
        "inspect-injection",
        "clean",
        "true",
        "false",
        "주입 신호가 없다(clean).",
        "src/main.rs:inspect injection",
    ),
    MechanicalCheck(
        "inspect-unicode",
        "rhwp inspect unicode --json",
        "inspect-unicode",
        "clean",
        "true",
        "false",
        "유니코드 기만이 없다(clean).",
        "src/main.rs:inspect unicode",
    ),
    MechanicalCheck(
        "inspect-watermark",
        "rhwp inspect watermark --json",
        "inspect-watermark",
        "clean",
        "true",
        "false",
        "숨은 마크가 없다(clean).",
        "src/main.rs:inspect watermark",
    ),
    MechanicalCheck(
        "replay",
        "rhwp replay --json",
        "replay",
        "reproduced",
        "true",
        "false",
        "제3자 재실행이 같다(reproduced).",
        "src/main.rs:replay",
    ),
    MechanicalCheck(
        "audit",
        "rhwp audit --json",
        "audit",
        "valid",
        "true",
        "false",
        "캡슐 전수 재현이 유효하다(valid).",
        "src/main.rs:audit",
    ),
    MechanicalCheck(
        "lineage",
        "rhwp lineage --json",
        "lineage",
        "matches",
        "true",
        "false",
        "부모 산출=자식 입력 계보가 맞다(matches).",
        "src/main.rs:lineage",
    ),
    MechanicalCheck(
        "csv-to-table-verify",
        "rhwp csv-to-table --verify --json",
        "csv-to-table",
        "verify.identical",
        "true",
        "false",
        "표 되돌림 자기검증이 같다(verify.identical).",
        "src/main.rs:csv-to-table",
    ),
    MechanicalCheck(
        "sanitize-verify",
        "rhwp sanitize --verify --json",
        "sanitize",
        "verify.identical",
        "true",
        "false",
        "sanitize 자기검증이 같다(verify.identical).",
        "src/main.rs:sanitize",
    ),
)

CHECK_BY_ID: dict[str, MechanicalCheck] = {item.check_id: item for item in CHECKS}


def iter_checks() -> tuple[MechanicalCheck, ...]:
    return CHECKS


def check_by_id(check_id: str) -> MechanicalCheck:
    try:
        return CHECK_BY_ID[check_id]
    except KeyError as exc:
        raise ValueError(f"unknown mechanical check {check_id!r}") from exc


def command_key(check_id: str) -> str:
    return check_by_id(check_id).command_key


def same_command(check_a: str, check_b: str) -> bool:
    return command_key(check_a) == command_key(check_b)


def iter_distinct_pairs() -> tuple[tuple[MechanicalCheck, MechanicalCheck], ...]:
    pairs: list[tuple[MechanicalCheck, MechanicalCheck]] = []
    for left in CHECKS:
        for right in CHECKS:
            if left.command_key == right.command_key:
                continue
            pairs.append((left, right))
    return tuple(pairs)


def iter_same_command_pairs() -> tuple[tuple[MechanicalCheck, MechanicalCheck], ...]:
    return tuple((item, item) for item in CHECKS)


INVENTED_COMMANDS: tuple[str, ...] = (
    "rhwp shadow-agree",
    "rhwp joint-pass",
    "rhwp dual-check",
    "rhwp llm-verify",
)
