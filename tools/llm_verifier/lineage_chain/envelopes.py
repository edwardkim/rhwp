"""Lift existing `rhwp lineage` / workCapsule envelopes into LineageObservation.

Field names are the devel contract (`parentOk`, `lineageOk`, `brokenAt`,
`inputSha256`, `outputSha256`, `parent.sha256`, `kind`). This module does
not invent a new CLI and does not rewrite the work-receipt skill.

`--deep` `reproduced` is lifted onto the observation so tests can prove
`decide()` ignores it. That field belongs to V-replay.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping

from .decide import Decision, decide_observation
from .schema import (
    LineageObservation,
    LineageSource,
    ParentState,
    parse_optional_bool,
)


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _text(value: Any) -> str:
    if value is None:
        return ""
    return str(value)


def _link_sha(link: Mapping[str, Any], key: str) -> str:
    return _text(link.get(key))


def observation_from_lineage(envelope: Mapping[str, Any]) -> LineageObservation:
    """Wrap a `rhwp lineage --json` envelope."""
    meta = envelope.get("_skillMeta")
    if isinstance(meta, Mapping):
        exit_code = meta.get("exit")
        if exit_code == 1:
            return LineageObservation(
                parent_out="",
                child_in="",
                parent_ok=None,
                lineage_ok=None,
                broken_at="",
                source=LineageSource.IO,
                kind="",
                parent_state=ParentState.ABSENT,
            )
        if exit_code == 2:
            return LineageObservation(
                parent_out="",
                child_in="",
                parent_ok=None,
                lineage_ok=None,
                broken_at="",
                source=LineageSource.USAGE,
                kind="",
                parent_state=ParentState.ABSENT,
            )

    links = envelope.get("links")
    if not isinstance(links, list) or not links:
        if "stderr" in envelope and "head" not in envelope:
            text = _text(envelope.get("stderr")).lower()
            if "사용법" in text or "usage" in text:
                source = LineageSource.USAGE
            else:
                source = LineageSource.IO
            return LineageObservation(
                parent_out="",
                child_in="",
                parent_ok=None,
                lineage_ok=None,
                broken_at="",
                source=source,
                kind="",
                parent_state=ParentState.ABSENT,
            )
        return LineageObservation(
            parent_out="",
            child_in="",
            parent_ok=None,
            lineage_ok=None,
            broken_at=_text(envelope.get("brokenAt")),
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.ABSENT,
        )

    child = links[0] if isinstance(links[0], Mapping) else {}
    parent_link = links[1] if len(links) > 1 and isinstance(links[1], Mapping) else {}

    error = _text(child.get("error"))
    parent_state = ParentState.OK
    if "parent.sha256" in error or "64자리" in error:
        parent_state = ParentState.SHA_MISSING
    elif "parent" in error and "없" in error:
        parent_state = ParentState.FIELD_MISSING

    parent_out = _link_sha(parent_link, "outputSha256")
    if not parent_out:
        parent_out = _link_sha(child, "parentOutputSha256")
    child_in = _link_sha(child, "inputSha256")

    return LineageObservation(
        parent_out=parent_out,
        child_in=child_in,
        parent_ok=parse_optional_bool(child.get("parentOk")),
        lineage_ok=parse_optional_bool(child.get("lineageOk")),
        broken_at=_text(envelope.get("brokenAt")),
        source=LineageSource.LINEAGE,
        kind="lineage",
        parent_state=parent_state,
        valid=parse_optional_bool(envelope.get("valid")),
        reproduced=parse_optional_bool(child.get("reproduced")),
    )


def observation_from_capsule_pair(
    child: Mapping[str, Any],
    parent: Mapping[str, Any] | None = None,
    *,
    parent_ok: bool | None = True,
    broken_at: str = "",
) -> LineageObservation:
    """Wrap a child workCapsule plus optional parent receipt hashes."""
    kind = _text(child.get("kind"))
    if kind and kind != "workCapsule":
        return LineageObservation(
            parent_out="",
            child_in="",
            parent_ok=None,
            lineage_ok=None,
            broken_at=broken_at,
            source=LineageSource.CAPSULE,
            kind=kind,
            parent_state=ParentState.ABSENT,
        )

    parent_field = child.get("parent")
    if "parent" not in child:
        return LineageObservation(
            parent_out="",
            child_in=_text((child.get("receipt") or {}).get("inputSha256"))
            if isinstance(child.get("receipt"), Mapping)
            else "",
            parent_ok=None,
            lineage_ok=None,
            broken_at=broken_at,
            source=LineageSource.CAPSULE,
            kind="workCapsule",
            parent_state=ParentState.FIELD_MISSING,
        )
    if parent_field is None:
        receipt = child.get("receipt") if isinstance(child.get("receipt"), Mapping) else {}
        return LineageObservation(
            parent_out=_text(receipt.get("outputSha256")),
            child_in=_text(receipt.get("inputSha256")),
            parent_ok=None,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.CAPSULE,
            kind="workCapsule",
            parent_state=ParentState.ROOT,
        )
    if not isinstance(parent_field, Mapping):
        return LineageObservation(
            parent_out="",
            child_in="",
            parent_ok=None,
            lineage_ok=None,
            broken_at=broken_at,
            source=LineageSource.CAPSULE,
            kind="workCapsule",
            parent_state=ParentState.SHA_MISSING,
        )
    parent_sha = _text(parent_field.get("sha256"))
    if sha_missing(parent_sha):
        return LineageObservation(
            parent_out="",
            child_in="",
            parent_ok=False,
            lineage_ok=None,
            broken_at=broken_at,
            source=LineageSource.CAPSULE,
            kind="workCapsule",
            parent_state=ParentState.SHA_MISSING,
        )

    child_receipt = child.get("receipt") if isinstance(child.get("receipt"), Mapping) else {}
    parent_receipt: Mapping[str, Any] = {}
    if isinstance(parent, Mapping):
        maybe = parent.get("receipt")
        if isinstance(maybe, Mapping):
            parent_receipt = maybe
    parent_out = _text(parent_receipt.get("outputSha256"))
    child_in = _text(child_receipt.get("inputSha256"))
    lineage_ok: bool | None
    if parent_out and child_in:
        lineage_ok = parent_out.strip().lower() == child_in.strip().lower()
    else:
        lineage_ok = None
    return LineageObservation(
        parent_out=parent_out,
        child_in=child_in,
        parent_ok=parent_ok,
        lineage_ok=lineage_ok,
        broken_at=broken_at,
        source=LineageSource.CAPSULE,
        kind="workCapsule",
        parent_state=ParentState.OK,
        reproduced=parse_optional_bool(child_receipt.get("reproduced")),
    )


def sha_missing(value: str) -> bool:
    from .hexutil import sha_defect

    return sha_defect(value) is not None


def observation_from_prose(text: str) -> LineageObservation:
    return LineageObservation(
        parent_out="",
        child_in="",
        parent_ok=None,
        lineage_ok=None,
        broken_at="",
        source=LineageSource.PROSE,
        kind="",
        parent_state=ParentState.ABSENT,
    )


def decide_envelope(blob: Mapping[str, Any]) -> Decision:
    if blob.get("kind") == "workCapsule" or "receipt" in blob:
        return decide_observation(observation_from_capsule_pair(blob))
    if (
        "links" in blob
        or "brokenAt" in blob
        or "lineageOk" in blob
        or (isinstance(blob.get("_skillMeta"), Mapping) and blob["_skillMeta"].get("command") == "lineage")
    ):
        return decide_observation(observation_from_lineage(blob))
    return decide_observation(observation_from_prose(_text(blob.get("claim") or blob.get("text"))))
