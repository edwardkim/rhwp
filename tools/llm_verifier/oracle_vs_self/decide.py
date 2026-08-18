"""WHEN to trust a Hangul-official PDF vs when only self-consistency is honest.

Decision axis (issue #5487):

    (has_hangul_pdf, versions, page_count_match, render_self_pass, cheap_ok)
        -> expected_verdict_class

The tree does not run fidelity_compare / oracle_public / visual_sweep.
It reads the *signals those tools already publish* and refuses to upgrade
a self-check into an independent Hangul-oracle claim.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Mapping

from .versions import ParsedVersions, parse_versions

CLAIM_ID = "V-oracle"
SCHEMA_VERSION = "1.0"
KIND = "oracleVsSelfDecision"

# Honest verdict classes. Each class names the strongest claim that is true.
NO_ORACLE_SELF_CONSISTENT = "NO_ORACLE_SELF_CONSISTENT"
NO_ORACLE_SELF_RENDER_FAIL = "NO_ORACLE_SELF_RENDER_FAIL"
NO_ORACLE_SELF_CHEAP_FAIL = "NO_ORACLE_SELF_CHEAP_FAIL"
NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF = "NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF"
ORACLE_UNVERSIONED = "ORACLE_UNVERSIONED"
ORACLE_YEAR_OUT_OF_CONTRACT = "ORACLE_YEAR_OUT_OF_CONTRACT"
ORACLE_MULTIVER_DISAGREE = "ORACLE_MULTIVER_DISAGREE"
ORACLE_PAGECOUNT_MISMATCH = "ORACLE_PAGECOUNT_MISMATCH"
ORACLE_CHEAP_FAIL = "ORACLE_CHEAP_FAIL"
ORACLE_BLOCKED_BY_SELF = "ORACLE_BLOCKED_BY_SELF"
ORACLE_TRUSTED = "ORACLE_TRUSTED"

VERDICT_CLASSES: tuple[str, ...] = (
    NO_ORACLE_SELF_CONSISTENT,
    NO_ORACLE_SELF_RENDER_FAIL,
    NO_ORACLE_SELF_CHEAP_FAIL,
    NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF,
    ORACLE_UNVERSIONED,
    ORACLE_YEAR_OUT_OF_CONTRACT,
    ORACLE_MULTIVER_DISAGREE,
    ORACLE_PAGECOUNT_MISMATCH,
    ORACLE_CHEAP_FAIL,
    ORACLE_BLOCKED_BY_SELF,
    ORACLE_TRUSTED,
)

# Tools whose published envelopes this tree may *read* as data.
CONSUMED_CONTRACTS: tuple[str, ...] = (
    "tools/oracle_public/oracle_resolver.py",
    "tools/oracle_public/page_smoke.py",
    "tools/oracle_public/multiver_index.py",
    "tools/fidelity_compare/fidelity_compare.py",
    "scripts/visual_sweep.py",
    "rhwp render-diff",
)

# Independent Hangul-oracle visual tools. Allowed only on ORACLE_TRUSTED.
INDEPENDENT_ORACLE_TOOLS: tuple[str, ...] = (
    "tools/fidelity_compare",
    "scripts/visual_sweep.py",
)

# Self-consistency tools. Always an honest next step when no oracle is trusted.
SELF_CONSISTENCY_TOOLS: tuple[str, ...] = (
    "rhwp render-diff A==A",
    "rhwp dump-pages --json",
)


HONEST_CLAIMS: dict[str, str] = {
    NO_ORACLE_SELF_CONSISTENT: (
        "한컴 공식 PDF가 없다. 정직한 주장은 render-diff A==A 와 "
        "자기 쪽수(dump-pages) 자기일관성뿐이다."
    ),
    NO_ORACLE_SELF_RENDER_FAIL: (
        "한컴 공식 PDF가 없고 render-diff A==A 가 실패했다. "
        "독립 오라클 주장은 불가하고 자기 렌더가 불안정하다."
    ),
    NO_ORACLE_SELF_CHEAP_FAIL: (
        "한컴 공식 PDF가 없고 값싼 자기 쪽수/전처리 게이트가 실패했다. "
        "자기일관성조차 아직 주장할 수 없다."
    ),
    NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF: (
        "한컴 PDF가 없는데 버전 토큰이 붙어 있다. "
        "oracle_resolver unmatched 행에 공식 연도를 심을 수 없다."
    ),
    ORACLE_UNVERSIONED: (
        "PDF 경로는 있으나 한컴 공식 연도로 묶이지 않는다. "
        "한컴 공식 오라클로 신뢰하지 않는다."
    ),
    ORACLE_YEAR_OUT_OF_CONTRACT: (
        "PDF 는 있으나 연도가 oracle_public 계약(2010/2018/2020/2022/2024) "
        "밖이다. 한컴 공식 오라클로 쓰지 않는다."
    ),
    ORACLE_MULTIVER_DISAGREE: (
        "같은 stem 의 한컴 버전 PDF 쪽수가 갈린다(multiver_index "
        "page_count_disagree). 한 연도를 정답지로 고르지 않는다."
    ),
    ORACLE_PAGECOUNT_MISMATCH: (
        "버전 있는 한컴 PDF 와 rhwp dump-pages 쪽수가 다르다. "
        "이것은 독립 오라클의 값싼 주장이다(page_smoke MISMATCH)."
    ),
    ORACLE_CHEAP_FAIL: (
        "한컴 PDF 와 연도는 있으나 page_smoke ERROR / LFS / run-state 누락 "
        "등 값싼 게이트가 실패했다. 시각 오라클 대조를 열지 않는다."
    ),
    ORACLE_BLOCKED_BY_SELF: (
        "한컴 PDF 는 쓸 수 있으나 render-diff A==A 가 실패했다. "
        "자기 렌더가 불안정한 채 공식 PDF 시각 일치를 주장하지 않는다."
    ),
    ORACLE_TRUSTED: (
        "한컴 공식 PDF · 계약 연도 · 쪽수 일치 · 값싼 게이트 · A==A 가 "
        "모두 통과했다. fidelity_compare / visual_sweep 후보 대조를 열 수 있다. "
        "픽셀·문자 보고는 여전히 candidate 이지 최종 결함이 아니다."
    ),
}


TREE_STEPS: tuple[str, ...] = (
    "has_hangul_pdf?",
    "versions.bindable?",
    "versions.multiver_agree?",
    "page_count_match?",
    "cheap_ok?",
    "render_self_pass?",
)


@dataclass(frozen=True)
class DecisionInputs:
    has_hangul_pdf: bool
    versions: str
    page_count_match: bool
    render_self_pass: bool
    cheap_ok: bool

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "DecisionInputs":
        from .schema import parse_bool

        return cls(
            has_hangul_pdf=parse_bool(row["has_hangul_pdf"]),
            versions=str(row.get("versions", "")),
            page_count_match=parse_bool(row["page_count_match"]),
            render_self_pass=parse_bool(row["render_self_pass"]),
            cheap_ok=parse_bool(row["cheap_ok"]),
        )


@dataclass(frozen=True)
class Decision:
    verdict_class: str
    honest_claim: str
    allowed_tools: tuple[str, ...]
    blocked_tools: tuple[str, ...]
    parsed_versions: ParsedVersions
    tree_path: tuple[str, ...]
    independent_oracle: bool
    self_only: bool
    notes: tuple[str, ...] = field(default_factory=tuple)

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "kind": KIND,
            "claim": CLAIM_ID,
            "verdictClass": self.verdict_class,
            "honestClaim": self.honest_claim,
            "independentOracle": self.independent_oracle,
            "selfOnly": self.self_only,
            "allowedTools": list(self.allowed_tools),
            "blockedTools": list(self.blocked_tools),
            "versions": {
                "raw": self.parsed_versions.raw,
                "kind": self.parsed_versions.kind,
                "years": list(self.parsed_versions.years),
                "agree": self.parsed_versions.agree,
                "canonical": self.parsed_versions.canonical,
                "outOfContract": list(self.parsed_versions.out_of_contract),
            },
            "treePath": list(self.tree_path),
            "notes": list(self.notes),
        }


def _finish(
    verdict: str,
    parsed: ParsedVersions,
    path: list[str],
    *notes: str,
) -> Decision:
    independent = verdict == ORACLE_TRUSTED
    pagecount_oracle = verdict == ORACLE_PAGECOUNT_MISMATCH
    self_only = verdict.startswith("NO_ORACLE_") or verdict in {
        ORACLE_UNVERSIONED,
        ORACLE_YEAR_OUT_OF_CONTRACT,
    }
    if independent:
        allowed = SELF_CONSISTENCY_TOOLS + INDEPENDENT_ORACLE_TOOLS
        blocked: tuple[str, ...] = ()
    elif pagecount_oracle:
        allowed = ("tools/oracle_public/page_smoke.py",) + SELF_CONSISTENCY_TOOLS
        blocked = INDEPENDENT_ORACLE_TOOLS
    elif verdict == ORACLE_MULTIVER_DISAGREE:
        allowed = ("tools/oracle_public/multiver_index.py",)
        blocked = INDEPENDENT_ORACLE_TOOLS + ("pin-year-without-evidence",)
    elif verdict == ORACLE_BLOCKED_BY_SELF:
        allowed = SELF_CONSISTENCY_TOOLS
        blocked = INDEPENDENT_ORACLE_TOOLS
    elif verdict == ORACLE_CHEAP_FAIL:
        allowed = ("tools/oracle_public/page_smoke.py",)
        blocked = INDEPENDENT_ORACLE_TOOLS
    else:
        allowed = SELF_CONSISTENCY_TOOLS
        blocked = INDEPENDENT_ORACLE_TOOLS
    return Decision(
        verdict_class=verdict,
        honest_claim=HONEST_CLAIMS[verdict],
        allowed_tools=allowed,
        blocked_tools=blocked,
        parsed_versions=parsed,
        tree_path=tuple(path),
        independent_oracle=independent,
        self_only=self_only,
        notes=notes,
    )


def decide(
    has_hangul_pdf: bool,
    versions: str,
    page_count_match: bool,
    render_self_pass: bool,
    cheap_ok: bool,
) -> Decision:
    """Pure selection tree. Inputs are already-published tool signals."""
    parsed = parse_versions(versions)
    path: list[str] = []

    if not has_hangul_pdf:
        path.append("has_hangul_pdf=false")
        if parsed.kind != "none":
            path.append("versions!=none")
            return _finish(
                NO_ORACLE_VERSION_TOKEN_WITHOUT_PDF,
                parsed,
                path,
                "unmatched sample cannot carry an official Hangul year",
            )
        path.append("versions=none")
        if not cheap_ok:
            path.append("cheap_ok=false")
            return _finish(NO_ORACLE_SELF_CHEAP_FAIL, parsed, path)
        path.append("cheap_ok=true")
        if not render_self_pass:
            path.append("render_self_pass=false")
            return _finish(NO_ORACLE_SELF_RENDER_FAIL, parsed, path)
        path.append("render_self_pass=true")
        return _finish(NO_ORACLE_SELF_CONSISTENT, parsed, path)

    path.append("has_hangul_pdf=true")
    if parsed.kind in {"none", "unknown"}:
        path.append(f"versions.kind={parsed.kind}")
        return _finish(ORACLE_UNVERSIONED, parsed, path)
    if parsed.kind == "invalid":
        path.append("versions.kind=invalid")
        return _finish(
            ORACLE_YEAR_OUT_OF_CONTRACT,
            parsed,
            path,
            f"out_of_contract={','.join(parsed.out_of_contract)}",
        )
    if parsed.kind == "disagree":
        path.append("versions.kind=disagree")
        return _finish(ORACLE_MULTIVER_DISAGREE, parsed, path)

    path.append(f"versions.kind=agree:{parsed.canonical}")
    if not page_count_match:
        path.append("page_count_match=false")
        return _finish(ORACLE_PAGECOUNT_MISMATCH, parsed, path)
    path.append("page_count_match=true")
    if not cheap_ok:
        path.append("cheap_ok=false")
        return _finish(ORACLE_CHEAP_FAIL, parsed, path)
    path.append("cheap_ok=true")
    if not render_self_pass:
        path.append("render_self_pass=false")
        return _finish(ORACLE_BLOCKED_BY_SELF, parsed, path)
    path.append("render_self_pass=true")
    return _finish(ORACLE_TRUSTED, parsed, path)


def decide_inputs(inputs: DecisionInputs) -> Decision:
    return decide(
        inputs.has_hangul_pdf,
        inputs.versions,
        inputs.page_count_match,
        inputs.render_self_pass,
        inputs.cheap_ok,
    )


def decide_row(row: Mapping[str, Any]) -> Decision:
    return decide_inputs(DecisionInputs.from_mapping(row))
