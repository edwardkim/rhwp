"""Published field contracts consumed as *data* (issue #5487).

This module does not import ``tools.fidelity_compare``, ``tools.oracle_public``,
or ``scripts.visual_sweep``. It records the field names and meanings those
tools already emit so the selection tree can bind envelopes without rewriting
the producers.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

# Keep these literals in lock-step with the *documented* producer contracts.
# If a producer renames a field, tests that load producer fixtures fail.

ORACLE_RESOLVER_CONTRACT: dict[str, Any] = {
    "path": "tools/oracle_public/oracle_resolver.py",
    "schema_version": "1.0",
    "pair_years": ["2018", "2020", "2022", "2024"],
    "source_formats": ["hwp", "hwpx"],
    "oracle_roots": ["pdf", "pdf-2020", "pdf-large"],
    "pair_required": [
        "sample",
        "pdf",
        "stem",
        "hancomVersion",
        "sourceFormat",
        "oracleRoot",
    ],
    "unmatched_required": ["sample"],
    "has_hangul_pdf_from": "pairs[] present for sample",
    "no_hangul_pdf_from": "unmatched[] reason",
}

PAGE_SMOKE_CONTRACT: dict[str, Any] = {
    "path": "tools/oracle_public/page_smoke.py",
    "schema_version": 1,
    "kind": "pageSmokeReport",
    "row_verdicts": ["MATCH", "MISMATCH", "ERROR"],
    "row_fields": [
        "doc",
        "pdf",
        "stem",
        "rhwpPages",
        "pdfPages",
        "delta",
        "verdict",
        "note",
        "repro",
    ],
    "page_count_match_from": "verdict == MATCH",
    "cheap_fail_from": "verdict == ERROR",
    "independent_mismatch_from": "verdict == MISMATCH",
    "exit_codes": {"default": 0, "strict_fail": 1, "usage": 2},
}

MULTIVER_CONTRACT: dict[str, Any] = {
    "path": "tools/oracle_public/multiver_index.py",
    "schema_version": "1.0",
    "claim": "M01-5",
    "metric": "pypdf_page_count",
    "pixel_diff": "out_of_scope",
    "years": ["2010", "2018", "2020", "2022", "2024"],
    "disagree_kind": "page_count_disagree",
    "versions_disagree_from": "disagreements[].kind == page_count_disagree",
}

FIDELITY_COMPARE_CONTRACT: dict[str, Any] = {
    "path": "tools/fidelity_compare/fidelity_compare.py",
    "requires_hangul_pdf": True,
    "outputs": {
        "page-count-ledger.tsv": [
            "measure",
            "pages",
            "delta_from_reference",
            "scope",
            "note",
        ],
        "report.tsv": "pixel diff% ranking (candidate)",
        "text-report.tsv": "character multiset missing/excess (candidate)",
        "provenance.tsv": "source / reference-pdf / reference-grade",
        "run-state.tsv": "requested/completed/missing; missing => nonzero exit",
    },
    "page_count_measures": ["reference_pdf", "rhwp_svg", "rhwp_render_tree"],
    "page_count_is_candidate": True,
    "pixel_is_candidate": True,
    "text_is_candidate": True,
    "cannot_run_without_reference_pdf": True,
}

VISUAL_SWEEP_CONTRACT: dict[str, Any] = {
    "path": "scripts/visual_sweep.py",
    "run_schema_version": 1,
    "page_schema_version": 1,
    "run_manifest_fields": [
        "schema_version",
        "key",
        "provenance",
        "dpi",
        "pixel_diff_threshold",
        "requested_pages",
        "requested_page_shards",
        "run_state",
    ],
    "targets_require": ["hwp", "pdf"],
    "cannot_run_without_reference_pdf": True,
    "self_consistency_is_not_a_target": True,
}

RENDER_DIFF_SELF_CONTRACT: dict[str, Any] = {
    "path": "rhwp render-diff",
    "meaning": "same document rendered twice, A==A geometric gate",
    "is_independent_oracle": False,
    "honest_claim": "self-consistency only",
}


@dataclass(frozen=True)
class BoundSignals:
    """Signals lifted from producer envelopes without calling the producers."""

    has_hangul_pdf: bool
    versions: str
    page_count_match: bool
    render_self_pass: bool
    cheap_ok: bool
    source: str
    notes: tuple[str, ...]


def bind_resolver_pair(pair: Mapping[str, Any] | None, unmatched: bool) -> BoundSignals:
    if unmatched or pair is None:
        return BoundSignals(
            has_hangul_pdf=False,
            versions="none",
            page_count_match=False,
            render_self_pass=True,
            cheap_ok=True,
            source="oracle_resolver.unmatched",
            notes=("no official Hangul PDF pair",),
        )
    year = str(pair.get("hancomVersion") or "unknown")
    return BoundSignals(
        has_hangul_pdf=True,
        versions=year,
        page_count_match=True,
        render_self_pass=True,
        cheap_ok=True,
        source="oracle_resolver.pair",
        notes=(f"oracleRoot={pair.get('oracleRoot', '')}",),
    )


def bind_page_smoke_row(row: Mapping[str, Any]) -> BoundSignals:
    verdict = str(row.get("verdict") or row.get("page_smoke_verdict") or "").upper()
    years = str(row.get("hancomVersion") or row.get("versions") or "unknown")
    if verdict == "MATCH":
        return BoundSignals(
            has_hangul_pdf=True,
            versions=years,
            page_count_match=True,
            render_self_pass=True,
            cheap_ok=True,
            source="page_smoke.MATCH",
            notes=(),
        )
    if verdict == "MISMATCH":
        return BoundSignals(
            has_hangul_pdf=True,
            versions=years,
            page_count_match=False,
            render_self_pass=True,
            cheap_ok=True,
            source="page_smoke.MISMATCH",
            notes=("independent cheap oracle finding",),
        )
    # ERROR means counts were not measured. That is not a numeric MISMATCH.
    return BoundSignals(
        has_hangul_pdf=bool(row.get("pdf")),
        versions=years if row.get("pdf") else "none",
        page_count_match=True,
        render_self_pass=True,
        cheap_ok=False,
        source="page_smoke.ERROR",
        notes=(str(row.get("note") or "page_smoke ERROR"),),
    )


def bind_multiver_entry(entry: Mapping[str, Any]) -> BoundSignals:
    years = [str(y) for y in entry.get("hangul_versions") or ()]
    kind = str(entry.get("kind") or "")
    if kind == "page_count_disagree" and years:
        return BoundSignals(
            has_hangul_pdf=True,
            versions="!".join(years),
            page_count_match=False,
            render_self_pass=True,
            cheap_ok=True,
            source="multiver_index.page_count_disagree",
            notes=("do not pin a year",),
        )
    joiner = "+" if years else ""
    return BoundSignals(
        has_hangul_pdf=bool(years),
        versions=joiner.join(years) if years else "none",
        page_count_match=True,
        render_self_pass=True,
        cheap_ok=True,
        source="multiver_index.agree",
        notes=(),
    )


def bind_fidelity_page_count_ledger(
    rows: list[Mapping[str, Any]],
    *,
    versions: str,
    render_self_pass: bool,
) -> BoundSignals:
    pages: dict[str, int | None] = {}
    for row in rows:
        measure = str(row.get("measure") or "")
        raw = row.get("pages")
        if raw in (None, "", "-"):
            pages[measure] = None
        else:
            pages[measure] = int(raw)
    ref = pages.get("reference_pdf")
    svg = pages.get("rhwp_svg")
    tree = pages.get("rhwp_render_tree")
    if ref is None:
        return BoundSignals(
            has_hangul_pdf=False,
            versions="none",
            page_count_match=False,
            render_self_pass=render_self_pass,
            cheap_ok=False,
            source="fidelity_compare.page-count-ledger.missing_reference",
            notes=("fidelity_compare cannot run without a Hangul PDF",),
        )
    counted = [n for n in (svg, tree) if n is not None]
    match = bool(counted) and all(n == ref for n in counted)
    cheap_ok = None not in (svg, tree)
    return BoundSignals(
        has_hangul_pdf=True,
        versions=versions,
        page_count_match=match,
        render_self_pass=render_self_pass,
        cheap_ok=cheap_ok,
        source="fidelity_compare.page-count-ledger",
        notes=("page-count difference is a candidate, not a global-break fix",),
    )


def bind_visual_sweep_manifest(manifest: Mapping[str, Any]) -> BoundSignals:
    schema = manifest.get("schema_version")
    pdf = ((manifest.get("provenance") or {}).get("pdf") or {}).get("path")
    run_state = str(manifest.get("run_state") or "")
    cheap_ok = schema == 1 and bool(pdf) and run_state != "incomplete"
    return BoundSignals(
        has_hangul_pdf=bool(pdf),
        versions=str(manifest.get("hancomVersion") or manifest.get("versions") or "unknown"),
        page_count_match=cheap_ok,
        render_self_pass=True,
        cheap_ok=cheap_ok,
        source="visual_sweep.run_manifest",
        notes=("visual_sweep TARGETS require hwp+pdf",),
    )


ALL_CONTRACTS: dict[str, dict[str, Any]] = {
    "oracle_resolver": ORACLE_RESOLVER_CONTRACT,
    "page_smoke": PAGE_SMOKE_CONTRACT,
    "multiver_index": MULTIVER_CONTRACT,
    "fidelity_compare": FIDELITY_COMPARE_CONTRACT,
    "visual_sweep": VISUAL_SWEEP_CONTRACT,
    "render_diff_self": RENDER_DIFF_SELF_CONTRACT,
}
