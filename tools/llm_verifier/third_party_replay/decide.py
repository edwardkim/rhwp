"""Closed verdict tree for third-party replay labor.

The tree does not re-run `rhwp replay`. It reads the fields that command
already publishes:

    plan / planSha256
    expectedOutputSha256   (--expect-output-sha256)
    reproduced             (verify true/false; attest null)
    toolVersion

Implementer narrative is a `source=prose` observation. It never becomes
LABOR_ACCEPTED. Attest receipts (`reproduced=null`, no expect sha) are
issuance, not third-party verification.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

try:
    from .hexutil import expect_sha_defect, is_sha256_hex
    from .schema import ReplayMode, ReplayObservation, ReplaySource, parse_mode, parse_reproduced, parse_source
except ImportError:  # python verify_corpus.py
    from hexutil import expect_sha_defect, is_sha256_hex
    from schema import ReplayMode, ReplayObservation, ReplaySource, parse_mode, parse_reproduced, parse_source

CLAIM_ID = "V-replay"
SCHEMA_VERSION = "1.0"
KIND = "thirdPartyReplayDecision"

LABOR_ACCEPTED = "LABOR_ACCEPTED"
LABOR_REJECTED = "LABOR_REJECTED"
ATTEST_NOT_THIRD_PARTY = "ATTEST_NOT_THIRD_PARTY"
PROSE_NOT_EVIDENCE = "PROSE_NOT_EVIDENCE"
NO_EXPECT_SHA = "NO_EXPECT_SHA"
INVALID_EXPECT_SHA = "INVALID_EXPECT_SHA"
TOOL_VERSION_MISSING = "TOOL_VERSION_MISSING"
TOOL_VERSION_MISMATCH = "TOOL_VERSION_MISMATCH"
NO_PLAN = "NO_PLAN"

VERDICT_CLASSES: tuple[str, ...] = (
    LABOR_ACCEPTED,
    LABOR_REJECTED,
    ATTEST_NOT_THIRD_PARTY,
    PROSE_NOT_EVIDENCE,
    NO_EXPECT_SHA,
    INVALID_EXPECT_SHA,
    TOOL_VERSION_MISSING,
    TOOL_VERSION_MISMATCH,
    NO_PLAN,
)

# Existing contract surfaces this wrapper may *read*. Not rewritten here.
CONSUMED_CONTRACTS: tuple[str, ...] = (
    "rhwp replay --expect-output-sha256",
    "rhwp replay --json reproduced",
    "workCapsule.receipt.reproduced",
    "workCapsule.receipt.expectedOutputSha256",
    "workCapsule.receipt.toolVersion",
    "workCapsule.planText",
)

EVIDENCE_REPRODUCED = "reproduced_field"
EVIDENCE_NONE = "none"

HONEST_CLAIMS: dict[str, str] = {
    LABOR_ACCEPTED: (
        "제3자가 같은 계획으로 `rhwp replay --expect-output-sha256` 를 돌렸고 "
        "봉투 `reproduced=true` 이다. 노동을 인정한다."
    ),
    LABOR_REJECTED: (
        "제3자 재실행 봉투 `reproduced=false` 이다. 구현자 산문과 무관하게 "
        "주장을 기각한다 (exit 3)."
    ),
    ATTEST_NOT_THIRD_PARTY: (
        "`mode=attest` 이거나 `reproduced=null` 이다. 영수증 발급일 뿐 "
        "제3자 검증이 아니다."
    ),
    PROSE_NOT_EVIDENCE: (
        "replay/capsule 봉투가 없고 구현자 산문만 있다. 산문은 증거가 아니다."
    ),
    NO_EXPECT_SHA: (
        "verify 경로인데 `--expect-output-sha256` / `expectedOutputSha256` 이 없다. "
        "재현 대조를 할 수 없다."
    ),
    INVALID_EXPECT_SHA: (
        "기대 산출 해시가 64 hex 계약이 아니다. CLI 는 봉투 없이 exit 2 다."
    ),
    TOOL_VERSION_MISSING: (
        "영수증 `toolVersion` 이 비었다. 재현 조건을 선대조할 수 없다."
    ),
    TOOL_VERSION_MISMATCH: (
        "주장 toolVersion 과 영수증 toolVersion 이 다르다. "
        "해시 대조 전에 기각한다."
    ),
    NO_PLAN: (
        "계획 원문(plan / planText)이 없다. 제3자가 같은 바이트를 재실행할 수 없다."
    ),
}

EXIT_FOR_VERDICT: dict[str, str] = {
    LABOR_ACCEPTED: "0",
    LABOR_REJECTED: "3",
    ATTEST_NOT_THIRD_PARTY: "0",
    PROSE_NOT_EVIDENCE: "2",
    NO_EXPECT_SHA: "2",
    INVALID_EXPECT_SHA: "2",
    TOOL_VERSION_MISSING: "2",
    TOOL_VERSION_MISMATCH: "3",
    NO_PLAN: "2",
}


@dataclass(frozen=True)
class Decision:
    verdict: str
    labor_accepted: bool
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
            "laborAccepted": self.labor_accepted,
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
        labor_accepted=verdict == LABOR_ACCEPTED,
        evidence_kind=evidence,
        exit_class=EXIT_FOR_VERDICT[verdict],
        honest_claim=HONEST_CLAIMS[verdict],
        reason_codes=reasons,
        consumed=CONSUMED_CONTRACTS,
    )


def decide_observation(obs: ReplayObservation) -> Decision:
    """Classify one observation. Prose and missing fields beat reproduced."""
    if obs.source is ReplaySource.PROSE or obs.mode is ReplayMode.ABSENT:
        return _decision(PROSE_NOT_EVIDENCE, "source_not_replay_envelope")

    if not str(obs.plan).strip():
        return _decision(NO_PLAN, "plan_text_missing")

    if not str(obs.tool_version).strip():
        return _decision(TOOL_VERSION_MISSING, "tool_version_empty")

    expected = str(obs.expected_tool_version or "").strip()
    actual = str(obs.tool_version).strip()
    if expected and expected != actual:
        return _decision(
            TOOL_VERSION_MISMATCH,
            "tool_version_ne_expected",
            evidence=EVIDENCE_REPRODUCED,
        )

    if obs.mode is ReplayMode.ATTEST:
        return _decision(ATTEST_NOT_THIRD_PARTY, "mode_attest_reproduced_null")

    expect = str(obs.expect_sha or "").strip()
    defect = expect_sha_defect(obs.expect_sha)
    if defect == "missing":
        return _decision(NO_EXPECT_SHA, "expected_output_sha256_missing")
    if defect is not None or not is_sha256_hex(expect):
        return _decision(INVALID_EXPECT_SHA, f"expected_output_sha256_{defect or 'invalid'}")

    if obs.reproduced is True:
        return _decision(
            LABOR_ACCEPTED,
            "reproduced_true",
            evidence=EVIDENCE_REPRODUCED,
        )
    if obs.reproduced is False:
        return _decision(
            LABOR_REJECTED,
            "reproduced_false_exit_3",
            evidence=EVIDENCE_REPRODUCED,
        )
    return _decision(ATTEST_NOT_THIRD_PARTY, "reproduced_null_without_verify")


def decide(
    plan: str,
    expect_sha: str | None,
    reproduced: bool | None,
    tool_version: str,
    *,
    mode: str = ReplayMode.VERIFY.value,
    source: str = ReplaySource.REPLAY.value,
    expected_tool_version: str = "",
) -> Decision:
    obs = ReplayObservation(
        plan=plan or "",
        expect_sha="" if expect_sha is None else str(expect_sha),
        reproduced=reproduced,
        tool_version=tool_version or "",
        mode=parse_mode(mode),
        source=parse_source(source),
        expected_tool_version=expected_tool_version or "",
    )
    return decide_observation(obs)


def decide_row(row: Mapping[str, Any]) -> Decision:
    return decide(
        str(row.get("plan", "")),
        row.get("expect_sha", row.get("expectedOutputSha256")),
        parse_reproduced(row.get("reproduced")),
        str(row.get("tool_version", row.get("toolVersion", ""))),
        mode=str(row.get("mode", ReplayMode.VERIFY.value)),
        source=str(row.get("source", ReplaySource.REPLAY.value)),
        expected_tool_version=str(row.get("expected_tool_version", "")),
    )
