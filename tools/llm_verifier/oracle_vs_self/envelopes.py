"""Lift producer JSON/TSV envelopes into DecisionInputs without importing them."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterable, Mapping

from .contracts import (
    BoundSignals,
    bind_fidelity_page_count_ledger,
    bind_multiver_entry,
    bind_page_smoke_row,
    bind_resolver_pair,
    bind_visual_sweep_manifest,
)
from .decide import Decision, decide


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_tsv(text: str) -> list[dict[str, str]]:
    lines = [line for line in text.splitlines() if line.strip()]
    if not lines:
        return []
    header = lines[0].split("\t")
    rows: list[dict[str, str]] = []
    for line in lines[1:]:
        cells = line.split("\t")
        row = {header[i]: (cells[i] if i < len(cells) else "") for i in range(len(header))}
        rows.append(row)
    return rows


def decide_bound(signals: BoundSignals) -> Decision:
    return decide(
        signals.has_hangul_pdf,
        signals.versions,
        signals.page_count_match,
        signals.render_self_pass,
        signals.cheap_ok,
    )


def from_page_smoke_report(report: Mapping[str, Any]) -> list[tuple[BoundSignals, Decision]]:
    out: list[tuple[BoundSignals, Decision]] = []
    for row in report.get("rows") or ():
        signals = bind_page_smoke_row(row)
        out.append((signals, decide_bound(signals)))
    return out


def from_resolver_manifest(manifest: Mapping[str, Any]) -> list[tuple[BoundSignals, Decision]]:
    out: list[tuple[BoundSignals, Decision]] = []
    for pair in manifest.get("pairs") or ():
        signals = bind_resolver_pair(pair, unmatched=False)
        out.append((signals, decide_bound(signals)))
    for _item in manifest.get("unmatched") or ():
        signals = bind_resolver_pair(None, unmatched=True)
        out.append((signals, decide_bound(signals)))
    return out


def from_multiver_report(report: Mapping[str, Any]) -> list[tuple[BoundSignals, Decision]]:
    out: list[tuple[BoundSignals, Decision]] = []
    for entry in report.get("disagreements") or ():
        signals = bind_multiver_entry(entry)
        out.append((signals, decide_bound(signals)))
    return out


def from_fidelity_ledger(
    rows: Iterable[Mapping[str, Any]],
    *,
    versions: str,
    render_self_pass: bool = True,
) -> tuple[BoundSignals, Decision]:
    signals = bind_fidelity_page_count_ledger(
        list(rows), versions=versions, render_self_pass=render_self_pass
    )
    return signals, decide_bound(signals)


def from_visual_sweep(manifest: Mapping[str, Any]) -> tuple[BoundSignals, Decision]:
    signals = bind_visual_sweep_manifest(manifest)
    return signals, decide_bound(signals)


def load_envelope(path: Path) -> list[tuple[BoundSignals, Decision]]:
    suffix = path.suffix.lower()
    if suffix == ".tsv":
        rows = parse_tsv(path.read_text(encoding="utf-8"))
        if rows and rows[0].get("measure"):
            signals, decision = from_fidelity_ledger(rows, versions="2022")
            return [(signals, decision)]
        return [(bind_page_smoke_row(row), decide_bound(bind_page_smoke_row(row))) for row in rows]
    blob = load_json(path)
    if isinstance(blob, dict) and blob.get("kind") == "pageSmokeReport":
        return from_page_smoke_report(blob)
    if isinstance(blob, dict) and "pairs" in blob and "unmatched" in blob:
        return from_resolver_manifest(blob)
    if isinstance(blob, dict) and blob.get("claim") == "M01-5":
        return from_multiver_report(blob)
    if isinstance(blob, dict) and "schema_version" in blob and "provenance" in blob:
        signals, decision = from_visual_sweep(blob)
        return [(signals, decision)]
    raise ValueError(f"unrecognized envelope: {path}")
