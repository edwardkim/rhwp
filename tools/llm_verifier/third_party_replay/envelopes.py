"""Lift existing replay / workCapsule envelopes into ReplayObservation.

Field names are the devel contract (`schemaVersion`, `mode`, `reproduced`,
`expectedOutputSha256`, `toolVersion`, `planText`, `receipt`). This module
does not invent a new CLI and does not rewrite the work-receipt skill.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping

from .decide import Decision, decide_observation
from .schema import ReplayMode, ReplayObservation, ReplaySource, parse_mode, parse_reproduced


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _text(value: Any) -> str:
    if value is None:
        return ""
    return str(value)


def _plan_from_mapping(blob: Mapping[str, Any]) -> str:
    for key in ("planText", "plan_text", "planJson", "plan"):
        value = blob.get(key)
        if isinstance(value, str) and value.strip():
            return value
        if isinstance(value, Mapping) and value:
            return json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    return ""


def observation_from_replay(
    envelope: Mapping[str, Any],
    *,
    plan: str | None = None,
    expected_tool_version: str = "",
) -> ReplayObservation:
    """Wrap a `rhwp replay --json` envelope."""
    expect = envelope.get("expectedOutputSha256")
    mode_raw = envelope.get("mode") or ReplayMode.ABSENT.value
    return ReplayObservation(
        plan=plan if plan is not None else _plan_from_mapping(envelope),
        expect_sha="" if expect is None else _text(expect),
        reproduced=parse_reproduced(envelope.get("reproduced")),
        tool_version=_text(envelope.get("toolVersion")),
        mode=parse_mode(mode_raw),
        source=ReplaySource.REPLAY,
        expected_tool_version=expected_tool_version,
    )


def observation_from_capsule(
    capsule: Mapping[str, Any],
    *,
    expected_tool_version: str = "",
) -> ReplayObservation:
    """Wrap a workCapsule file. Only `receipt.reproduced` counts."""
    receipt = capsule.get("receipt")
    if not isinstance(receipt, Mapping):
        return ReplayObservation(
            plan=_plan_from_mapping(capsule),
            expect_sha="",
            reproduced=None,
            tool_version="",
            mode=ReplayMode.ABSENT,
            source=ReplaySource.PROSE,
            expected_tool_version=expected_tool_version,
        )
    expect = receipt.get("expectedOutputSha256")
    plan = _plan_from_mapping(capsule)
    if not plan:
        plan = _plan_from_mapping(receipt)
    return ReplayObservation(
        plan=plan,
        expect_sha="" if expect is None else _text(expect),
        reproduced=parse_reproduced(receipt.get("reproduced")),
        tool_version=_text(receipt.get("toolVersion")),
        mode=parse_mode(receipt.get("mode") or ReplayMode.ABSENT.value),
        source=ReplaySource.CAPSULE,
        expected_tool_version=expected_tool_version,
    )


def observation_from_prose(text: str) -> ReplayObservation:
    return ReplayObservation(
        plan=text,
        expect_sha="",
        reproduced=None,
        tool_version="",
        mode=ReplayMode.ABSENT,
        source=ReplaySource.PROSE,
    )


def decide_envelope(blob: Mapping[str, Any], *, expected_tool_version: str = "") -> Decision:
    if blob.get("kind") == "workCapsule" or "receipt" in blob:
        return decide_observation(
            observation_from_capsule(blob, expected_tool_version=expected_tool_version)
        )
    if "mode" in blob or "reproduced" in blob or "expectedOutputSha256" in blob:
        return decide_observation(
            observation_from_replay(blob, expected_tool_version=expected_tool_version)
        )
    return decide_observation(observation_from_prose(_text(blob.get("claim") or blob.get("text"))))
