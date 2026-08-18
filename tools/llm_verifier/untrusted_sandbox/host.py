"""Verifier-side host: isolate untrusted envelope fields before judging.

The host never copies an excerpt into a criteria string. It wraps allowed
LLM data blocks with a nonce and reports a block when a caller still
tries to treat the excerpt as a criterion or instruction.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Mapping

from .decide import Decision, decide
from .envelope import UntrustedSlice, extract_slices, provenance_present, untrusted_content
from .leak import LeakReport, detect_leak
from .nonce import WrapError, derive_nonce_avoiding, wrap_untrusted
from .slot import Slot


@dataclass(frozen=True)
class IsolatedField:
    path: str
    excerpt: str
    nonce: str
    wrapped: str
    source_label: str


@dataclass
class SandboxReport:
    blocked: bool
    decisions: list[Decision] = field(default_factory=list)
    isolated: list[IsolatedField] = field(default_factory=list)
    leaks: list[LeakReport] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)

    def as_dict(self) -> dict[str, Any]:
        return {
            "blocked": self.blocked,
            "isolatedCount": len(self.isolated),
            "leakCount": len(self.leaks),
            "errorCount": len(self.errors),
            "decisions": [d.verdict for d in self.decisions],
            "failKinds": [d.fail_kinds_cell() for d in self.decisions if d.fail_kinds],
            "errors": list(self.errors),
        }


def isolate_envelope(
    envelope: Mapping[str, Any],
    *,
    command: str,
    source_label: str,
    proposed_criteria: str = "",
    slot: str = Slot.LLM_DATA_BLOCK.value,
) -> SandboxReport:
    report = SandboxReport(blocked=False)
    if not provenance_present(envelope):
        report.blocked = True
        report.errors.append("missing_provenance_keys")
        report.decisions.append(
            decide(slot, False, "", "", "caller_path", "unmarked", False)
        )
        return report

    flag = untrusted_content(envelope)
    if flag is False:
        return report

    for index, slice_ in enumerate(extract_slices(envelope, command)):
        _isolate_slice(
            report,
            slice_,
            source_label=source_label,
            seed=f"{command}|{source_label}|{index}|{slice_.path}",
            proposed_criteria=proposed_criteria,
            slot=slot,
        )
    report.blocked = report.blocked or any(d.expected_block for d in report.decisions)
    return report


def _isolate_slice(
    report: SandboxReport,
    slice_: UntrustedSlice,
    *,
    source_label: str,
    seed: str,
    proposed_criteria: str,
    slot: str,
) -> None:
    try:
        nonce = derive_nonce_avoiding(seed, slice_.excerpt)
        wrapped = wrap_untrusted(slice_.excerpt, nonce, source_label)
    except WrapError as exc:
        report.errors.append(f"{slice_.path}:{exc.kind}")
        report.decisions.append(
            decide(slot, True, "", slice_.excerpt, "caller_path", "nonce_collision", True)
        )
        return

    leaked = False
    if proposed_criteria:
        leak = detect_leak(
            criteria=proposed_criteria,
            excerpt=slice_.excerpt,
            nonce=nonce,
            wrapped=wrapped.text,
        )
        if leak.leaked:
            leaked = True
            report.leaks.append(leak)

    decision = decide(
        slot=slot,
        leaked_into_criteria=leaked or slot == Slot.CRITERIA.value,
        nonce=nonce,
        excerpt=slice_.excerpt,
        source_label_kind="caller_path",
        wrap_state="wrapped_ok",
        untrusted_content=True,
    )
    report.decisions.append(decision)
    if not decision.expected_block:
        report.isolated.append(
            IsolatedField(
                path=slice_.path,
                excerpt=slice_.excerpt,
                nonce=nonce,
                wrapped=wrapped.text,
                source_label=source_label,
            )
        )
