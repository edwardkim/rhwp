"""Closed verdict tree for the V-lineage hash chain.

The tree does not re-run `rhwp lineage` or `rhwp replay`. It reads the
fields those commands already publish:

    parent receipt.outputSha256   (parent_out)
    child  receipt.inputSha256    (child_in)
    links[].parentOk
    links[].lineageOk
    brokenAt

The definition of a chain is `parent_out == child_in` (64 hex, case-folded).
`parentOk` is parent-file integrity, not that equality.
`reproduced` (`--deep`) is V-replay and is never an input to this tree.
Implementer narrative is a `source=prose` observation. It never becomes
CHAIN_ACCEPTED.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

try:
    from .hexutil import normalize_sha256_hex, sha_defect
    from .schema import (
        LineageObservation,
        LineageSource,
        ParentState,
        parse_optional_bool,
        parse_parent_state,
        parse_source,
    )
except ImportError:  # python verify_corpus.py
    from hexutil import normalize_sha256_hex, sha_defect
    from schema import (
        LineageObservation,
        LineageSource,
        ParentState,
        parse_optional_bool,
        parse_parent_state,
        parse_source,
    )

CLAIM_ID = "V-lineage"
SCHEMA_VERSION = "1.0"
KIND = "lineageChainDecision"

CHAIN_ACCEPTED = "CHAIN_ACCEPTED"
LINEAGE_BROKEN = "LINEAGE_BROKEN"
PARENT_TAMPERED = "PARENT_TAMPERED"
ROOT_ONLY = "ROOT_ONLY"
PROSE_NOT_EVIDENCE = "PROSE_NOT_EVIDENCE"
HEAD_MISSING = "HEAD_MISSING"
USAGE = "USAGE"
PARENT_SHA_MISSING = "PARENT_SHA_MISSING"
PARENT_FIELD_MISSING = "PARENT_FIELD_MISSING"
KIND_NOT_CAPSULE = "KIND_NOT_CAPSULE"
HASH_DEFECT = "HASH_DEFECT"
ENVELOPE_CONTRADICTS = "ENVELOPE_CONTRADICTS"

VERDICT_CLASSES: tuple[str, ...] = (
    CHAIN_ACCEPTED,
    LINEAGE_BROKEN,
    PARENT_TAMPERED,
    ROOT_ONLY,
    PROSE_NOT_EVIDENCE,
    HEAD_MISSING,
    USAGE,
    PARENT_SHA_MISSING,
    PARENT_FIELD_MISSING,
    KIND_NOT_CAPSULE,
    HASH_DEFECT,
    ENVELOPE_CONTRADICTS,
)

# Existing contract surfaces this wrapper may *read*. Not rewritten here.
CONSUMED_CONTRACTS: tuple[str, ...] = (
    "rhwp lineage --json parentOk",
    "rhwp lineage --json lineageOk",
    "rhwp lineage --json brokenAt",
    "workCapsule.receipt.outputSha256",
    "workCapsule.receipt.inputSha256",
    "workCapsule.parent.sha256",
)

EVIDENCE_LINEAGE = "lineage_hash_eq"
EVIDENCE_NONE = "none"

HONEST_CLAIMS: dict[str, str] = {
    CHAIN_ACCEPTED: (
        "부모 `outputSha256` 이 자식 `inputSha256` 과 같고 "
        "`parentOk=true` · `lineageOk=true` · `brokenAt=null` 이다. "
        "연대기를 인정한다. `--deep`/`reproduced` 는 이 축이 아니다."
    ),
    LINEAGE_BROKEN: (
        "부모 산출 해시가 자식 입력 해시와 다르거나 봉투 `lineageOk=false` 이다. "
        "`brokenAt` 이 깨진 캡슐을 가리킨다 (exit 3)."
    ),
    PARENT_TAMPERED: (
        "봉투 `parentOk=false` 이다. 부모 캡슐 파일이 발급 당시 바이트가 아니다. "
        "해시 등식과 무관하게 체인이 깨졌다 (exit 3)."
    ),
    ROOT_ONLY: (
        "뿌리 캡슐이다. `parentOk` 와 `lineageOk` 가 null 이다. "
        "비교할 부모가 없어 사슬을 주장하지 않는다."
    ),
    PROSE_NOT_EVIDENCE: (
        "lineage 봉투가 없고 구현자 산문만 있다. 산문은 증거가 아니다."
    ),
    HEAD_MISSING: (
        "머리 캡슐을 읽을 수 없다. `rhwp lineage` 는 봉투 없이 exit 1 이다."
    ),
    USAGE: (
        "인자가 없거나 사용법 오류다. `rhwp lineage` 는 봉투 없이 exit 2 이다."
    ),
    PARENT_SHA_MISSING: (
        "`parent.sha256` 이 없거나 64 hex 가 아니다. 생략이 아니라 fail-closed (exit 3)."
    ),
    PARENT_FIELD_MISSING: (
        "`parent` 키 자체가 없다. 합법 뿌리(`parent: null`)와 다르다 (exit 3)."
    ),
    KIND_NOT_CAPSULE: (
        "`kind != workCapsule` 이다. 계보 머리가 작업 캡슐이 아니다 (exit 3)."
    ),
    HASH_DEFECT: (
        "부모 산출 또는 자식 입력이 64 hex 가 아니거나, 사슬 주장에 해시가 없다. "
        "등식을 계산할 수 없다."
    ),
    ENVELOPE_CONTRADICTS: (
        "봉투의 `lineageOk`/`brokenAt` 이 해시 등식과 모순된다. "
        "연대기의 정의는 부모 산출 == 자식 입력이고, 필드는 그 정의를 따라야 한다."
    ),
}

EXIT_FOR_VERDICT: dict[str, str] = {
    CHAIN_ACCEPTED: "0",
    LINEAGE_BROKEN: "3",
    PARENT_TAMPERED: "3",
    ROOT_ONLY: "0",
    PROSE_NOT_EVIDENCE: "2",
    HEAD_MISSING: "1",
    USAGE: "2",
    PARENT_SHA_MISSING: "3",
    PARENT_FIELD_MISSING: "3",
    KIND_NOT_CAPSULE: "3",
    HASH_DEFECT: "2",
    ENVELOPE_CONTRADICTS: "3",
}

_CAPSULE_KINDS = frozenset({"", "workCapsule", "lineage"})


@dataclass(frozen=True)
class Decision:
    verdict: str
    chain_accepted: bool
    evidence_kind: str
    exit_class: str
    honest_claim: str
    reason_codes: tuple[str, ...]
    consumed: tuple[str, ...]

    @property
    def verdict_class(self) -> str:
        return self.verdict

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM_ID,
            "kind": KIND,
            "verdict": self.verdict,
            "chainAccepted": self.chain_accepted,
            "evidenceKind": self.evidence_kind,
            "exitClass": self.exit_class,
            "honestClaim": self.honest_claim,
            "reasonCodes": list(self.reason_codes),
            "consumedContracts": list(self.consumed),
        }


def _decision(verdict: str, *reasons: str, evidence: str = EVIDENCE_NONE) -> Decision:
    if verdict not in VERDICT_CLASSES:
        raise ValueError(f"unknown verdict {verdict}")
    return Decision(
        verdict=verdict,
        chain_accepted=verdict == CHAIN_ACCEPTED,
        evidence_kind=evidence,
        exit_class=EXIT_FOR_VERDICT[verdict],
        honest_claim=HONEST_CLAIMS[verdict],
        reason_codes=reasons,
        consumed=CONSUMED_CONTRACTS,
    )


def _hashes(obs: LineageObservation) -> tuple[str, str, bool, bool, bool]:
    parent_out = str(obs.parent_out or "")
    child_in = str(obs.child_in or "")
    parent_stripped = parent_out.strip()
    child_stripped = child_in.strip()
    both_present = bool(parent_stripped) and bool(child_stripped)
    # Defects are classified on the raw token. Stripping a leading space
    # must not turn a HASH_DEFECT into a false hash equality.
    parent_ok_hex = sha_defect(parent_out) is None if parent_stripped else False
    child_ok_hex = sha_defect(child_in) is None if child_stripped else False
    hashes_valid = bool(parent_ok_hex and child_ok_hex)
    hashes_match = hashes_valid and normalize_sha256_hex(parent_out) == normalize_sha256_hex(
        child_in
    )
    return parent_out, child_in, both_present, hashes_valid, hashes_match


def decide_observation(obs: LineageObservation) -> Decision:
    """Classify one observation. Source and missing fields beat hash equality."""
    if obs.source is LineageSource.PROSE:
        return _decision(PROSE_NOT_EVIDENCE, "source_not_lineage_envelope")
    if obs.source is LineageSource.USAGE:
        return _decision(USAGE, "lineage_usage_exit_2")
    if obs.source is LineageSource.IO:
        return _decision(HEAD_MISSING, "head_capsule_io_exit_1")

    kind = str(obs.kind or "").strip()
    if obs.source is LineageSource.CAPSULE and kind not in {"", "workCapsule"}:
        return _decision(KIND_NOT_CAPSULE, "kind_ne_workCapsule")
    if kind and kind not in _CAPSULE_KINDS:
        return _decision(KIND_NOT_CAPSULE, "kind_ne_workCapsule")

    if obs.parent_state is ParentState.FIELD_MISSING:
        return _decision(PARENT_FIELD_MISSING, "parent_key_absent")
    if obs.parent_state is ParentState.SHA_MISSING:
        return _decision(PARENT_SHA_MISSING, "parent_sha256_missing_or_nonhex")

    parent_ok = obs.parent_ok
    lineage_ok = obs.lineage_ok
    broken = str(obs.broken_at or "").strip()

    if parent_ok is None and lineage_ok is None:
        if broken:
            return _decision(ENVELOPE_CONTRADICTS, "root_has_broken_at")
        return _decision(ROOT_ONLY, "root_axes_null")

    if parent_ok is False:
        if not broken:
            return _decision(ENVELOPE_CONTRADICTS, "parent_tamper_without_broken_at")
        return _decision(PARENT_TAMPERED, "parent_ok_false")

    parent_out, child_in, both_present, hashes_valid, hashes_match = _hashes(obs)

    if both_present and not hashes_valid:
        defect = sha_defect(parent_out) or sha_defect(child_in) or "invalid"
        return _decision(HASH_DEFECT, f"hash_defect_{defect}")

    if (parent_ok is True or lineage_ok is not None) and not both_present:
        return _decision(HASH_DEFECT, "parent_out_or_child_in_missing")

    if hashes_valid and both_present:
        if hashes_match:
            if lineage_ok is False:
                return _decision(ENVELOPE_CONTRADICTS, "lineage_ok_false_but_hashes_equal")
            if broken:
                return _decision(ENVELOPE_CONTRADICTS, "valid_chain_has_broken_at")
            if lineage_ok is True:
                return _decision(
                    CHAIN_ACCEPTED,
                    "parent_output_eq_child_input",
                    evidence=EVIDENCE_LINEAGE,
                )
            return _decision(ENVELOPE_CONTRADICTS, "parent_ok_true_lineage_ok_null")
        if lineage_ok is True:
            return _decision(ENVELOPE_CONTRADICTS, "lineage_ok_true_but_hashes_differ")
        if not broken:
            return _decision(ENVELOPE_CONTRADICTS, "lineage_break_without_broken_at")
        if lineage_ok is False:
            return _decision(
                LINEAGE_BROKEN,
                "parent_output_ne_child_input",
                evidence=EVIDENCE_LINEAGE,
            )
        return _decision(ENVELOPE_CONTRADICTS, "hashes_differ_lineage_ok_null")

    if lineage_ok is False:
        if not broken:
            return _decision(ENVELOPE_CONTRADICTS, "lineage_ok_false_without_broken_at")
        return _decision(LINEAGE_BROKEN, "lineage_ok_false", evidence=EVIDENCE_LINEAGE)

    if lineage_ok is True and parent_ok is True and not broken:
        return _decision(HASH_DEFECT, "chain_claim_without_hashes")

    return _decision(HASH_DEFECT, "hashes_unavailable")


def decide(
    parent_out: str,
    child_in: str,
    parent_ok: bool | None,
    lineage_ok: bool | None,
    broken_at: str,
    *,
    source: str = LineageSource.LINEAGE.value,
    kind: str = "lineage",
    parent_state: str = ParentState.OK.value,
    valid: bool | None = None,
    reproduced: bool | None = None,
) -> Decision:
    # reproduced is accepted so callers can pass --deep envelopes.
    # It is intentionally unused: V-replay is a different claim.
    del reproduced
    obs = LineageObservation(
        parent_out=parent_out or "",
        child_in=child_in or "",
        parent_ok=parent_ok,
        lineage_ok=lineage_ok,
        broken_at=broken_at or "",
        source=parse_source(source),
        kind=kind or "",
        parent_state=parse_parent_state(parent_state),
        valid=valid,
    )
    return decide_observation(obs)


def decide_row(row: Mapping[str, Any]) -> Decision:
    return decide(
        str(row.get("parent_out", row.get("parentOut", ""))),
        str(row.get("child_in", row.get("childIn", ""))),
        parse_optional_bool(row.get("parent_ok", row.get("parentOk"))),
        parse_optional_bool(row.get("lineage_ok", row.get("lineageOk"))),
        str(row.get("broken_at", row.get("brokenAt", ""))),
        source=str(row.get("source", LineageSource.LINEAGE.value)),
        kind=str(row.get("kind", "lineage")),
        parent_state=str(row.get("parent_state", ParentState.OK.value)),
        valid=parse_optional_bool(row.get("valid")),
        reproduced=parse_optional_bool(row.get("reproduced")),
    )
