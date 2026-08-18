"""Lift two existing command envelopes into a shadow-agreement decision.

The envelopes are data. This module does not import rhwp producers and
does not invent fields. A single envelope with two fighting fields is
V-abstain and is rejected here.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping

try:
    from .checks import MechanicalCheck, check_by_id
    from .decide import Decision, decide
except ImportError:
    from checks import MechanicalCheck, check_by_id
    from decide import Decision, decide


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def read_dotted(blob: Mapping[str, Any], path: str) -> Any:
    cur: Any = blob
    for part in path.split("."):
        if not isinstance(cur, Mapping) or part not in cur:
            raise KeyError(path)
        cur = cur[part]
    return cur


def field_matches(check: MechanicalCheck, value: Any) -> bool:
    if check.pass_equals == "equal":
        return str(value) not in {"", "mismatch", "0"} and str(value) != check.fail_example
    text = str(value).strip().lower()
    expected = check.pass_equals.strip().lower()
    if expected in {"true", "false"}:
        return text == expected
    return str(value) == check.pass_equals


def pass_from_envelope(check_id: str, envelope: Mapping[str, Any]) -> bool:
    check = check_by_id(check_id)
    if check.pass_field == "pageCount" and check.pass_equals == "equal":
        observed = envelope.get("pageCount")
        expected = envelope.get("expectedPageCount", observed)
        return observed is not None and observed == expected
    value = read_dotted(envelope, check.pass_field)
    return field_matches(check, value)


def decide_envelopes(
    check_a: str,
    envelope_a: Mapping[str, Any],
    check_b: str,
    envelope_b: Mapping[str, Any],
) -> Decision:
    if envelope_a is envelope_b:
        raise ValueError("V-shadow reads two command envelopes, not one (that is V-abstain)")
    return decide(
        check_a,
        check_b,
        pass_from_envelope(check_a, envelope_a),
        pass_from_envelope(check_b, envelope_b),
    )


def load_pair_fixture(path: Path) -> Decision:
    blob = load_json(path)
    return decide_envelopes(
        str(blob["check_a"]),
        blob["envelope_a"],
        str(blob["check_b"]),
        blob["envelope_b"],
    )
