"""Lift existing rhwp --json envelopes into EnvelopeFields.

Does not import or rewrite V-proto. Field names are the ones those
commands already publish.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping, Optional

from .decide import Decision, decide
from .schema import EnvelopeFields

HERE = Path(__file__).resolve().parent
FIXTURE_ENV = HERE / "fixtures" / "envelopes"


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _as_bool(value: Any) -> Optional[bool]:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    return None


def _as_int(value: Any) -> Optional[int]:
    if value is None or isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    return None


def _as_str(value: Any) -> Optional[str]:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    return None


def _reproduced_bool(value: Any) -> Optional[bool]:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int) and not isinstance(value, bool):
        return value > 0
    return None


def bind_envelope(raw: Mapping[str, Any], *, command: str, exit_code: int) -> EnvelopeFields:
    verify = raw.get("verify")
    verify_identical = None
    verify_diff = None
    if isinstance(verify, Mapping):
        verify_identical = _as_bool(verify.get("identical"))
        verify_diff = _as_int(verify.get("diffCount"))

    struct_status = _as_str(raw.get("status"))
    if raw.get("hardStructPages") not in (None, 0) and struct_status in (None, "PASS", "OK"):
        struct_status = "STRUCT_MISMATCH"

    struct_node = _as_str(raw.get("structNode")) or _as_str(raw.get("worstNode"))
    if struct_node is None:
        delta = raw.get("structDelta") or raw.get("struct_delta")
        if isinstance(delta, str) and delta:
            struct_node = delta.split(";")[0]
        elif isinstance(raw.get("pages"), list) and raw["pages"]:
            first = raw["pages"][0]
            if isinstance(first, Mapping):
                struct_node = _as_str(first.get("node")) or _as_str(first.get("path"))

    page_node = _as_str(raw.get("pageCountNode"))
    if page_node is None:
        page_node = struct_node

    page_a = _as_int(raw.get("pageCountA"))
    page_b = _as_int(raw.get("pageCountB"))
    if page_a is None:
        page_a = _as_int(raw.get("pageCount"))
    if page_b is None and raw.get("pageCountMismatch") is False and page_a is not None:
        page_b = page_a

    return EnvelopeFields(
        command=command,
        exit=exit_code,
        identical=_as_bool(raw.get("identical")),
        has_signal=_as_bool(raw.get("hasSignal")),
        reproduced=_reproduced_bool(raw.get("reproduced")),
        page_count_a=page_a,
        page_count_b=page_b,
        page_count_mismatch=_as_bool(raw.get("pageCountMismatch")),
        struct_status=struct_status,
        struct_node=struct_node,
        page_count_node=page_node,
        verify_identical=verify_identical,
        verify_diff_count=verify_diff,
        diff_count=_as_int(raw.get("diffCount")),
        fail_count=_as_int(raw.get("failCount")),
        pass_count=_as_int(raw.get("passCount")),
        verdict=_as_str(raw.get("verdict")),
        regression=_as_bool(raw.get("regression")),
        status=_as_str(raw.get("status")),
        clean=_as_bool(raw.get("clean")),
        signal_count=_as_int(raw.get("signalCount")),
        valid=_as_bool(raw.get("valid")),
        finding_count=_as_int(raw.get("findingCount")),
        overflow_count=_as_int(raw.get("overflowCount")),
        overlap_count=_as_int(raw.get("overlapCount")),
        empty_page_count=_as_int(raw.get("emptyPageCount")),
    )


def decide_envelope(raw: Mapping[str, Any], *, command: str, exit_code: int) -> Decision:
    return decide(bind_envelope(raw, command=command, exit_code=exit_code))


def load_fixture(name: str) -> dict[str, Any]:
    return load_json(FIXTURE_ENV / name)
