#!/usr/bin/env python3
"""Run the bounded, local-only rank-8 Q3 qualification projection.

The private document ledger and render trees never enter the public result.  The
candidate changes only the normalized layout advance used for the target face;
the product registry, generated metric DB, paint and supply remain untouched.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import math
import os
import statistics
import subprocess
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from fontTools.ttLib import TTFont

from font_rank8_metric_hypothesis import (
    EXACT_TTF_SHA256,
    HWPUNIT_PER_PX,
    TARGET_FACE,
    reject_absolute_paths,
)
from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_json,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


Q0_SHA256 = "5284eea2541f88bab8635be9107e9331b1bef738e00815ebacbaf36494fc3003"
Q2_SHA256 = "377b2a60ae1332e97d966791201d888f3c441d7c04a74f03e56a6e66f8f53f05"
Q0_CANONICAL = "23536673cc94f1fcfa617adbd4adf4858116a029adc63403c5939b76c5293618"
Q2_CANONICAL = "392f1a756fc3974f522f895ce14253ebe9f1487805da522a3907acbb5d22b0f5"
MAX_DOCUMENT_BYTES = 512 * 1024 * 1024
MAX_JSON_BYTES = 64 * 1024 * 1024
MAX_TRACE_CHARACTERS = 4096
OVERFLOW_TOLERANCE_PX = 0.5
DELTA_TOLERANCE_PX = 0.1
HWPUNIT_PER_INCH = 7200.0
LAYOUT_DPI = 96.0


class Rank8PrivateQualificationError(OracleStage2Error):
    """A fail-closed W8-Q3 contract violation."""


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise Rank8PrivateQualificationError(f"{label} mismatch")


def _canonical_valid(value: dict[str, Any], expected: str, label: str) -> None:
    require_equal(value.get("canonicalSha256"), expected, f"{label} canonical field")
    clone = dict(value)
    clone.pop("canonicalSha256", None)
    require_equal(sha256_bytes(canonical_json_bytes(clone)), expected, f"{label} canonical")


def infer_font_size_hwpunit(records: list[dict[str, Any]]) -> int:
    """Recover the unrounded run em from the trace's heuristic base widths."""

    inferred: list[int] = []
    for record in records:
        metric = record.get("layoutMetric", {})
        base = metric.get("baseAdvanceHwpunit")
        source = metric.get("widthSource")
        if not isinstance(base, int) or base < 0:
            continue
        if source == "heuristicFullwidth":
            inferred.append(base)
        elif source == "heuristicHalfwidth":
            inferred.append(base * 2)
        elif source == "heuristicNarrow":
            inferred.append(round(base / 0.3))
    if not inferred:
        raise Rank8PrivateQualificationError("target run has no recoverable font size")
    result = round(statistics.median(inferred))
    # The trace stores integer HWPUNIT advances after the 0.3/0.5 heuristic.
    # Reversing those ratios can differ by a few units for non-round em sizes.
    if max(abs(value - result) for value in inferred) > 5:
        raise Rank8PrivateQualificationError("target run font-size inference diverged")
    return result


def resolve_font_size_hwpunit(records: list[dict[str, Any]]) -> int:
    """Choose the nearby integer em that reproduces every current advance."""

    approximate = infer_font_size_hwpunit(records)
    candidates = range(max(1, approximate - 8), approximate + 9)
    scored = []
    for candidate in candidates:
        mismatches = sum(
            apply_metric_transform_precise(
                record,
                heuristic_base_px(record, candidate),
                font_size_hwpunit=candidate,
            )
            != record["layoutMetric"]["finalAdvanceHwpunit"]
            for record in records
        )
        scored.append((mismatches, abs(candidate - approximate), candidate))
    best = min(scored)
    if best[0] != 0:
        failing = next(
            record
            for record in records
            if apply_metric_transform_precise(
                record,
                heuristic_base_px(record, best[2]),
                font_size_hwpunit=best[2],
            )
            != record["layoutMetric"]["finalAdvanceHwpunit"]
        )
        metric = failing["layoutMetric"]
        raise Rank8PrivateQualificationError(
            "current run transform cannot be reproduced: "
            f"approximate={approximate} best={best[0]} source={metric['widthSource']} "
            f"base={metric['baseAdvanceHwpunit']} final={metric['finalAdvanceHwpunit']} "
            f"transforms={metric.get('transforms', [])}"
        )
    return best[2]


def heuristic_base_px(record: dict[str, Any], font_size_hwpunit: int) -> float:
    font_size_px = font_size_hwpunit * LAYOUT_DPI / HWPUNIT_PER_INCH
    source = record["layoutMetric"].get("widthSource")
    if source == "heuristicFullwidth":
        return font_size_px
    if source == "heuristicHalfwidth":
        return font_size_px * 0.5
    if source == "heuristicNarrow":
        return font_size_px * 0.3
    raise Rank8PrivateQualificationError("record is not a heuristic metric candidate")


def apply_metric_transform_precise(
    record: dict[str, Any], base_px: float, *, font_size_hwpunit: int
) -> int:
    """Replay the engine in px precision and quantize only the final result."""

    if not math.isfinite(base_px) or base_px < 0 or font_size_hwpunit <= 0:
        raise Rank8PrivateQualificationError("invalid precise metric input")
    ratio = 1.0
    letter_spacing = 0.0
    additive = 0.0
    clamp = False
    for step in record["layoutMetric"].get("transforms", []):
        kind = step.get("kind")
        if kind == "ratio":
            ratio = float(step["output"])
        elif kind == "letterSpacing":
            letter_spacing = float(step["input"])
        elif kind in {"extraCharacterSpacing", "extraWordSpacing", "extraDashAdvance"}:
            additive += float(step["input"])
        elif kind == "negativeSpacingClamp":
            clamp = True
        elif kind == "boldFallback":
            continue
        else:
            raise Rank8PrivateQualificationError(f"unsupported precise transform: {kind}")
    scaled = base_px * ratio
    font_size_px = font_size_hwpunit * LAYOUT_DPI / HWPUNIT_PER_INCH
    final_px = scaled + letter_spacing * (scaled / font_size_px) + additive
    if clamp:
        final_px = max(final_px, scaled * 0.5)
    if not math.isfinite(final_px) or final_px < 0:
        raise Rank8PrivateQualificationError("invalid precise metric result")
    # Match renderer::px_to_hwpunit operation order exactly. Replacing this
    # with `final_px * 75` changes a few boundary casts around 0.3em.
    return int(final_px * HWPUNIT_PER_INCH / LAYOUT_DPI)


def line_disposition(current_overflow_px: float, candidate_overflow_px: float) -> str:
    current = current_overflow_px > OVERFLOW_TOLERANCE_PX
    candidate = candidate_overflow_px > OVERFLOW_TOLERANCE_PX
    if current and not candidate:
        return "overflow-removed"
    if not current and candidate:
        return "overflow-introduced"
    if current and candidate:
        if candidate_overflow_px < current_overflow_px - DELTA_TOLERANCE_PX:
            return "overflow-reduced"
        if candidate_overflow_px > current_overflow_px + DELTA_TOLERANCE_PX:
            return "overflow-increased"
        return "overflow-unchanged"
    if candidate_overflow_px < current_overflow_px - DELTA_TOLERANCE_PX:
        return "slack-increased"
    if candidate_overflow_px > current_overflow_px + DELTA_TOLERANCE_PX:
        return "slack-decreased"
    return "unchanged"


def classify_document(dispositions: Counter[str]) -> str:
    if dispositions["overflow-introduced"] or dispositions["overflow-increased"]:
        return "worsened"
    if dispositions["overflow-removed"] or dispositions["overflow-reduced"]:
        return "improved"
    return "unchanged"


def qualification_status(
    *, improved: bool, decisive_modelled_regressions: int, unmodelled_characters: int
) -> str:
    """A proven modelled regression rejects the candidate even with open gaps."""

    if decisive_modelled_regressions:
        return "no-change"
    if unmodelled_characters:
        return "blocked"
    if improved:
        return "qualified-for-q4"
    return "no-change"


def _run_private(command: list[str], label: str, timeout_seconds: int) -> bytes:
    try:
        completed = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        raise Rank8PrivateQualificationError(f"{label} timed out") from error
    if completed.returncode != 0:
        raise Rank8PrivateQualificationError(f"{label} failed")
    if len(completed.stdout) + len(completed.stderr) > MAX_JSON_BYTES:
        raise Rank8PrivateQualificationError(f"{label} output limit exceeded")
    return completed.stdout


def _evidence_page(
    binary: Path, source: Path, page: int, cohort_id: str
) -> tuple[int, dict[str, Any]]:
    payload = _run_private(
        [
            str(binary),
            str(source),
            "--page",
            str(page),
            "--max-characters",
            str(MAX_TRACE_CHARACTERS),
            "--json",
        ],
        f"{cohort_id} page same-snapshot evidence",
        120,
    )
    try:
        envelope = json.loads(payload)
    except json.JSONDecodeError as error:
        raise Rank8PrivateQualificationError(
            f"{cohort_id} page same-snapshot evidence JSON failed"
        ) from error
    evidence = envelope.get("evidence")
    if envelope.get("tool") != "rhwp-q-font-layout-evidence" or not isinstance(
        evidence, dict
    ):
        raise Rank8PrivateQualificationError(f"{cohort_id} page evidence identity mismatch")
    if (
        evidence.get("status") != "complete"
        or evidence.get("scope", {}).get("pageTreeBuilds") != 1
        or evidence.get("scope", {}).get("sameSnapshot") is not True
        or evidence.get("counts", {}).get("unframedRuns") != 0
    ):
        raise Rank8PrivateQualificationError(f"{cohort_id} page evidence is incomplete")
    trace = evidence.get("trace", {})
    counts = trace.get("counts", {})
    if (
        trace.get("status") != "complete"
        or counts.get("recordsOmitted") != 0
        or counts.get("charactersSeen") != counts.get("recordsEmitted")
    ):
        raise Rank8PrivateQualificationError(f"{cohort_id} page trace is incomplete")
    return page, evidence


def _exact_font(path: Path) -> tuple[int, dict[int, int]]:
    require_equal(sha256_file(path), EXACT_TTF_SHA256, "exact TTF")
    try:
        font = TTFont(path, lazy=False, recalcBBoxes=False, recalcTimestamp=False)
    except Exception as error:
        raise Rank8PrivateQualificationError("exact TTF parse failed") from error
    try:
        for table in ("head", "cmap", "hmtx"):
            if table not in font:
                raise Rank8PrivateQualificationError(f"exact TTF lacks {table}")
        units_per_em = int(font["head"].unitsPerEm)
        cmap = font.getBestCmap() or {}
        metrics = font["hmtx"].metrics
        advances = {codepoint: int(metrics[glyph][0]) for codepoint, glyph in cmap.items()}
    finally:
        font.close()
    return units_per_em, advances


def _document_input(
    corpus_root: Path, raw_source: str, expected_format: str, cohort_id: str
) -> Path:
    source = Path(raw_source)
    try:
        resolved = source.resolve(strict=True)
    except FileNotFoundError as error:
        raise Rank8PrivateQualificationError(f"{cohort_id} source is missing") from error
    if corpus_root != resolved.parent and corpus_root not in resolved.parents:
        raise Rank8PrivateQualificationError(f"{cohort_id} source escaped corpus root")
    if source.is_symlink() or not resolved.is_file():
        raise Rank8PrivateQualificationError(f"{cohort_id} source is not a regular file")
    if resolved.suffix.lower() != f".{expected_format}":
        raise Rank8PrivateQualificationError(f"{cohort_id} source format mismatch")
    size = resolved.stat().st_size
    if size <= 0 or size > MAX_DOCUMENT_BYTES:
        raise Rank8PrivateQualificationError(f"{cohort_id} source size limit exceeded")
    return resolved


def _candidate_advances(
    trace: dict[str, Any], units_per_em: int, exact_advances: dict[int, int]
) -> tuple[dict[int, int], Counter[str], Counter[str], int]:
    by_run: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for record in trace["records"]:
        by_run[record["source"]["runIndex"]].append(record)
    run_delta: dict[int, int] = defaultdict(int)
    signs: Counter[str] = Counter()
    coverage: Counter[str] = Counter()
    replay_mismatches = 0
    for run_index, records in by_run.items():
        target = [record for record in records if record["document"]["face"] == TARGET_FACE]
        if not target:
            continue
        eligible = [
            record
            for record in target
            if record["layoutMetric"].get("widthSource")
            in {"heuristicFullwidth", "heuristicHalfwidth", "heuristicNarrow"}
            and all(
                step.get("kind")
                not in {"tabContextAdvance", "zeroAdvance", "dashLeaderClamp"}
                for step in record["layoutMetric"].get("transforms", [])
            )
        ]
        if not eligible:
            signs["equal"] += len(target)
            coverage["special-preserved"] += len(target)
            continue
        applicable = [
            record
            for record in eligible
            if record["source"]["codePoint"] in exact_advances
        ]
        if not applicable:
            signs["equal"] += len(target)
            coverage["cmap-miss-preserved"] += len(eligible)
            coverage["special-preserved"] += len(target) - len(eligible)
            continue
        font_size_hwpunit = resolve_font_size_hwpunit(applicable)
        for record in target:
            metric = record["layoutMetric"]
            current = metric["finalAdvanceHwpunit"]
            if record not in eligible:
                signs["equal"] += 1
                coverage["special-preserved"] += 1
                continue
            codepoint = record["source"]["codePoint"]
            if codepoint not in exact_advances:
                signs["equal"] += 1
                coverage["cmap-miss-preserved"] += 1
                continue
            replay = apply_metric_transform_precise(
                record,
                heuristic_base_px(record, font_size_hwpunit),
                font_size_hwpunit=font_size_hwpunit,
            )
            replay_mismatches += replay != current
            candidate_base_px = (
                exact_advances[codepoint]
                / units_per_em
                * (font_size_hwpunit * LAYOUT_DPI / HWPUNIT_PER_INCH)
            )
            candidate = apply_metric_transform_precise(
                record, candidate_base_px, font_size_hwpunit=font_size_hwpunit
            )
            delta = candidate - current
            run_delta[run_index] += delta
            signs["narrower" if delta < 0 else "wider" if delta > 0 else "equal"] += 1
            coverage["exact-metric-applied"] += 1
    return dict(run_delta), signs, coverage, replay_mismatches


def _page_projection(
    evidence: dict[str, Any],
    units_per_em: int,
    exact_advances: dict[int, int],
    page: int,
) -> dict[str, Any]:
    trace = evidence["trace"]
    lines = evidence["lines"]
    require_equal(
        evidence["counts"]["runs"], trace["counts"]["runsSeen"], "trace/evidence run count"
    )
    records_by_run: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for record in trace["records"]:
        records_by_run[record["source"]["runIndex"]].append(record)

    run_delta, signs, coverage, replay_mismatches = _candidate_advances(
        trace, units_per_em, exact_advances
    )
    target_records = [
        record for record in trace["records"] if record["document"]["face"] == TARGET_FACE
    ]
    line_rows = []
    invalid_frame_target_characters = 0
    invalid_frame_target_lines = 0
    first_metric = None
    for record in target_records:
        run_index = record["source"]["runIndex"]
        if run_delta.get(run_index, 0) != 0:
            first_metric = {
                "page": page,
                "runIndex": run_index,
                "characterOffset": record["source"]["charOffset"],
            }
            break
    for line_index, line in enumerate(lines):
        layout_indexes = line["runIndices"]
        target_count = sum(
            1
            for run_index in layout_indexes
            for record in records_by_run[run_index]
            if record["document"]["face"] == TARGET_FACE
        )
        if target_count == 0:
            continue
        bbox = line["frame"]
        if not isinstance(bbox, dict) or bbox.get("width", 0) <= 0:
            invalid_frame_target_lines += 1
            invalid_frame_target_characters += target_count
            continue
        owned_runs = line["runs"]
        current_right = max(run["x"] + run["width"] for run in owned_runs)
        frame_right = bbox["x"] + bbox["width"]
        delta_hwpunit = sum(run_delta.get(index, 0) for index in layout_indexes)
        delta_px = delta_hwpunit / HWPUNIT_PER_PX
        current_overflow = max(0.0, current_right - frame_right)
        candidate_overflow = max(0.0, current_right + delta_px - frame_right)
        disposition = line_disposition(current_overflow, candidate_overflow)
        line_rows.append(
            {
                "page": page,
                "lineOrdinal": line_index,
                "context": line["context"],
                "storedRowDisposition": line["storedRow"]["disposition"],
                "storedRowReason": line["storedRow"]["reason"],
                "targetCharacters": target_count,
                "frameWidthPx": round(bbox["width"], 1),
                "currentOverflowPx": round(current_overflow, 3),
                "candidateOverflowPx": round(candidate_overflow, 3),
                "advanceDeltaHwpunit": delta_hwpunit,
                "disposition": disposition,
            }
        )
    semantic_keys = Counter()
    for record in target_records:
        source = record["source"]
        key = (
            source.get("sectionIndex"),
            source.get("paragraphIndex"),
            json.dumps(source.get("nestedPath"), sort_keys=True, separators=(",", ":")),
            source.get("runIndex"),
            source.get("charOffset"),
            source.get("charShapeId"),
            source.get("codePoint"),
        )
        semantic_keys[key] += 1
    framed_layout_indexes = {
        run_index for line in lines for run_index in line.get("runIndices", [])
    }
    unframed_target = sum(
        1
        for run_index, records in records_by_run.items()
        if run_index not in framed_layout_indexes
        for record in records
        if record["document"]["face"] == TARGET_FACE
    )
    return {
        "targetCharacters": len(target_records),
        "sourceCoordinateAvailable": sum(
            record["source"].get("status") == "complete" for record in target_records
        ),
        "semanticCoordinateKeys": semantic_keys,
        "deltaSigns": signs,
        "coverage": coverage,
        "replayMismatches": replay_mismatches,
        "firstMetricDivergence": first_metric,
        "unframedTargetCharacters": unframed_target,
        "invalidFrameTargetCharacters": invalid_frame_target_characters,
        "invalidFrameTargetLines": invalid_frame_target_lines,
        "lines": line_rows,
    }


def project_document(
    *,
    cohort_id: str,
    document: dict[str, Any],
    source: Path,
    evidence_bin: Path,
    units_per_em: int,
    exact_advances: dict[int, int],
    workers: int,
) -> dict[str, Any]:
    first_page, first_evidence = _evidence_page(evidence_bin, source, 0, cohort_id)
    require_equal(first_page, 0, "first evidence page")
    page_count = first_evidence.get("scope", {}).get("pageCount")
    if not isinstance(page_count, int) or page_count <= 0:
        raise Rank8PrivateQualificationError(f"{cohort_id} page count is invalid")

    def page_query(page: int) -> tuple[int, dict[str, Any]]:
        return _evidence_page(evidence_bin, source, page, cohort_id)

    if page_count > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            remaining = list(pool.map(page_query, range(1, page_count)))
    else:
        remaining = []
    evidences = {page: evidence for page, evidence in [(0, first_evidence), *remaining]}
    require_equal(len(evidences), page_count, "same-snapshot evidence page count")
    pages = [
        _page_projection(evidences[page], units_per_em, exact_advances, page)
        for page in range(page_count)
    ]

    lines = [line for page in pages for line in page["lines"]]
    dispositions = Counter(line["disposition"] for line in lines)
    delta_signs: Counter[str] = Counter()
    coverage: Counter[str] = Counter()
    semantic_keys: Counter[tuple[Any, ...]] = Counter()
    for page in pages:
        delta_signs.update(page["deltaSigns"])
        coverage.update(page["coverage"])
        semantic_keys.update(page["semanticCoordinateKeys"])
    first_metric = next(
        (page["firstMetricDivergence"] for page in pages if page["firstMetricDivergence"]),
        None,
    )
    first_capacity = next(
        (
            {
                "page": line["page"],
                "lineOrdinal": line["lineOrdinal"],
                "context": line["context"],
                "disposition": line["disposition"],
            }
            for line in lines
            if line["disposition"]
            in {
                "overflow-removed",
                "overflow-reduced",
                "overflow-introduced",
                "overflow-increased",
            }
        ),
        None,
    )
    unframed_target_characters = sum(
        page["unframedTargetCharacters"] for page in pages
    )
    invalid_frame_target_characters = sum(
        page["invalidFrameTargetCharacters"] for page in pages
    )
    cache_unmodelled_target_characters = sum(
        line["targetCharacters"]
        for line in lines
        if line["storedRowDisposition"] == "unmodelled"
    )
    cache_target_characters = Counter()
    cache_target_lines = Counter()
    for line in lines:
        cache_target_characters[line["storedRowDisposition"]] += line["targetCharacters"]
        cache_target_lines[line["storedRowDisposition"]] += 1
    classification = (
        "unmodelled"
        if unframed_target_characters
        + invalid_frame_target_characters
        + cache_unmodelled_target_characters
        else classify_document(dispositions)
    )
    return {
        "cohortId": cohort_id,
        "source": str(source),
        "format": document["format"],
        "sourceSha256": sha256_file(source),
        "sourceUsageCharacters": document["summary"]["totalCharacters"],
        "renderPages": len(pages),
        "targetRenderPages": sum(page["targetCharacters"] > 0 for page in pages),
        "renderObservedCharacters": sum(page["targetCharacters"] for page in pages),
        "sourceCoordinateAvailable": sum(page["sourceCoordinateAvailable"] for page in pages),
        "semanticCoordinateCount": len(semantic_keys),
        "repeatedSemanticObservations": sum(count - 1 for count in semantic_keys.values()),
        "currentTransformReplayMismatches": sum(
            page["replayMismatches"] for page in pages
        ),
        "unframedTargetCharacters": unframed_target_characters,
        "invalidFrameTargetCharacters": invalid_frame_target_characters,
        "invalidFrameTargetLines": sum(
            page["invalidFrameTargetLines"] for page in pages
        ),
        "cacheUnmodelledTargetCharacters": cache_unmodelled_target_characters,
        "cacheTargetCharacters": dict(sorted(cache_target_characters.items())),
        "cacheTargetLines": dict(sorted(cache_target_lines.items())),
        "recordDeltaCounts": dict(sorted(delta_signs.items())),
        "candidateCoverage": dict(sorted(coverage.items())),
        "targetLines": len(lines),
        "contextLines": dict(sorted(Counter(line["context"] for line in lines).items())),
        "lineDispositions": dict(sorted(dispositions.items())),
        "classification": classification,
        "firstMetricDivergence": first_metric,
        "firstCapacityDivergence": first_capacity,
        "lines": lines,
    }


def build_public(private: dict[str, Any], q0: dict[str, Any], q2: dict[str, Any]) -> dict[str, Any]:
    documents = private["documents"]
    classifications = Counter(document["classification"] for document in documents)
    line_dispositions: Counter[str] = Counter()
    contexts: Counter[str] = Counter()
    record_deltas: Counter[str] = Counter()
    coverage: Counter[str] = Counter()
    cache_target_characters: Counter[str] = Counter()
    cache_target_lines: Counter[str] = Counter()
    for document in documents:
        line_dispositions.update(document["lineDispositions"])
        contexts.update(document["contextLines"])
        record_deltas.update(document["recordDeltaCounts"])
        coverage.update(document["candidateCoverage"])
        cache_target_characters.update(document["cacheTargetCharacters"])
        cache_target_lines.update(document["cacheTargetLines"])
    improved = bool(
        line_dispositions["overflow-removed"]
        or line_dispositions["overflow-reduced"]
    )
    decisive_modelled_regressions = sum(
        1
        for document in documents
        for line in document["lines"]
        if line["storedRowDisposition"] in {"admitted", "rejected"}
        and line["disposition"] in {"overflow-introduced", "overflow-increased"}
    )
    non_worsening = decisive_modelled_regressions == 0
    unmodelled = sum(
        document["unframedTargetCharacters"]
        + document["invalidFrameTargetCharacters"]
        + document["cacheUnmodelledTargetCharacters"]
        for document in documents
    )
    status = qualification_status(
        improved=improved and non_worsening,
        decisive_modelled_regressions=decisive_modelled_regressions,
        unmodelled_characters=unmodelled,
    )
    result = {
        "schemaVersion": 1,
        "kind": "font-rank8-private-qualification-projection",
        "issue": 4967,
        "stage": "W8-Q3R",
        "target": {"face": TARGET_FACE, "targetDecisionPlane": "layout-metric"},
        "inputs": {
            "q0CanonicalSha256": q0["canonicalSha256"],
            "q2CanonicalSha256": q2["canonicalSha256"],
            "exactTtfSha256": EXACT_TTF_SHA256,
        },
        "accounting": {
            "documents": len(documents),
            "documentsByFormat": dict(
                sorted(Counter(document["format"] for document in documents).items())
            ),
            "sourceUsageCharacters": sum(
                document["sourceUsageCharacters"] for document in documents
            ),
            "renderObservedCharacters": sum(
                document["renderObservedCharacters"] for document in documents
            ),
            "renderObservationDelta": sum(
                document["renderObservedCharacters"]
                - document["sourceUsageCharacters"]
                for document in documents
            ),
            "sourceAndRenderCountersAreDistinct": True,
            "traceTruncatedPages": 0,
            "currentTransformReplayMismatches": sum(
                document["currentTransformReplayMismatches"] for document in documents
            ),
            "unframedTargetCharacters": sum(
                document["unframedTargetCharacters"] for document in documents
            ),
            "invalidFrameTargetCharacters": sum(
                document["invalidFrameTargetCharacters"] for document in documents
            ),
            "invalidFrameTargetLines": sum(
                document["invalidFrameTargetLines"] for document in documents
            ),
            "cacheUnmodelledTargetCharacters": sum(
                document["cacheUnmodelledTargetCharacters"] for document in documents
            ),
        },
        "projection": {
            "recordDeltaCounts": dict(sorted(record_deltas.items())),
            "candidateCoverage": dict(sorted(coverage.items())),
            "targetLines": sum(document["targetLines"] for document in documents),
            "contextLines": dict(sorted(contexts.items())),
            "lineDispositions": dict(sorted(line_dispositions.items())),
            "documentClassifications": dict(sorted(classifications.items())),
            "decisiveModelledRegressionLines": decisive_modelled_regressions,
            "cacheTargetCharacters": dict(sorted(cache_target_characters.items())),
            "cacheTargetLines": dict(sorted(cache_target_lines.items())),
            "firstMetricDivergencePresent": any(
                document["firstMetricDivergence"] is not None for document in documents
            ),
            "firstCapacityDivergencePresent": any(
                document["firstCapacityDivergence"] is not None for document in documents
            ),
        },
        "lineSegFeatureDetection": {
            "sourceUsageLane": "stored",
            "storedRiskCharacters": q0["cohort"]["storedRiskCharacters"],
            "freshRiskCharacters": q0["cohort"]["freshRiskCharacters"],
            "detector": "same-snapshot-production-cache-key-probe",
            "samePartitionProjection": True,
            "cacheAdmissionObserved": True,
            "validityClaim": unmodelled == 0,
            "reason": (
                None
                if unmodelled == 0
                else "at least one target character remains in a fail-closed unmodelled frame"
            ),
        },
        "decision": {
            "status": status,
            "actualDocumentImprovementObserved": improved,
            "allDocumentsNonWorsening": non_worsening,
            "evidenceComplete": unmodelled == 0,
            "terminalReason": (
                "modelled-regression"
                if decisive_modelled_regressions
                else "unmodelled-evidence"
                if unmodelled
                else None
            ),
            "productMutationAuthorized": False,
            "nextStage": "backend-portable-policy" if status == "qualified-for-q4" else None,
        },
        "executionPolicy": {
            "boundedDocuments": len(documents),
            "fullCorpusPass": False,
            "hyperVOracleRerun": False,
            "productSourceMutation": False,
            "registryMutation": False,
        },
        "privacy": {
            "privateDetailedOutputTracked": False,
            "privateCorpusPathIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateDocumentNameIncluded": False,
            "privateDocumentHashIncluded": False,
            "privateTextIncluded": False,
            "fontBytesIncluded": False,
            "absolutePathIncluded": False,
        },
    }
    result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
    reject_absolute_paths(result)
    return result


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus-root", required=True)
    parser.add_argument("--private-cohort", required=True)
    parser.add_argument("--q0", required=True)
    parser.add_argument("--q2", required=True)
    parser.add_argument("--font-root", required=True)
    parser.add_argument("--ttf", required=True)
    parser.add_argument("--font-layout-evidence-bin", required=True)
    parser.add_argument("--private-output", required=True)
    parser.add_argument("--public-output", required=True)
    parser.add_argument("--workers", type=int, default=4)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    if not 1 <= args.workers <= 8:
        raise Rank8PrivateQualificationError("workers must be in 1..=8")
    corpus_root = Path(args.corpus_root).resolve(strict=True)
    if not corpus_root.is_dir() or corpus_root.is_symlink():
        raise Rank8PrivateQualificationError("corpus root must be a real directory")
    cohort_path = regular_input(ROOT, args.private_cohort, MAX_JSON_BYTES)
    if cohort_path.stat().st_mode & 0o077:
        raise Rank8PrivateQualificationError("private cohort must be owner-only")
    q0_path = regular_input(ROOT, args.q0, MAX_JSON_BYTES)
    q2_path = regular_input(ROOT, args.q2, MAX_JSON_BYTES)
    evidence_bin = regular_input(
        ROOT, args.font_layout_evidence_bin, 512 * 1024 * 1024
    )
    font_root = Path(args.font_root).resolve(strict=True)
    ttf_path = regular_input(font_root, args.ttf, 64 * 1024 * 1024)
    require_equal(sha256_file(q0_path), Q0_SHA256, "Q0 raw")
    require_equal(sha256_file(q2_path), Q2_SHA256, "Q2 raw")
    q0 = read_json(q0_path)
    q2 = read_json(q2_path)
    _canonical_valid(q0, Q0_CANONICAL, "Q0")
    _canonical_valid(q2, Q2_CANONICAL, "Q2")
    if q2.get("hypothesis", {}).get("status") != "qualified-for-q3":
        raise Rank8PrivateQualificationError("Q2 did not qualify Q3")
    cohort = read_json(cohort_path)
    if (
        cohort.get("kind") != "font-rank8-private-cohort"
        or cohort.get("issue") != 4967
        or len(cohort.get("documents", [])) != 6
        or cohort.get("privacy", {}).get("localOnly") is not True
    ):
        raise Rank8PrivateQualificationError("private cohort identity mismatch")
    units_per_em, exact_advances = _exact_font(ttf_path)
    documents = []
    for ordinal, document in enumerate(cohort["documents"], 1):
        cohort_id = f"cohort-{ordinal:02d}"
        source = _document_input(
            corpus_root, document["source"], document["format"], cohort_id
        )
        documents.append(
            project_document(
                cohort_id=cohort_id,
                document=document,
                source=source,
                evidence_bin=evidence_bin,
                units_per_em=units_per_em,
                exact_advances=exact_advances,
                workers=args.workers,
            )
        )
    private = {
        "schemaVersion": 1,
        "kind": "font-rank8-private-qualification-detail",
        "issue": 4967,
        "stage": "W8-Q3R",
        "localOnly": True,
        "privateCohortRawSha256": sha256_file(cohort_path),
        "documents": documents,
    }
    public = build_public(private, q0, q2)
    write_json(output_path(ROOT, args.private_output), private, mode=0o600)
    write_json(output_path(ROOT, args.public_output), public, mode=0o644)
    print(
        json.dumps(
            {
                "status": public["decision"]["status"],
                "documents": public["accounting"]["documents"],
                "renderObservedCharacters": public["accounting"][
                    "renderObservedCharacters"
                ],
                "documentClassifications": public["projection"][
                    "documentClassifications"
                ],
                "canonicalSha256": public["canonicalSha256"],
                "hyperVOracleRerun": False,
            },
            ensure_ascii=False,
            separators=(",", ":"),
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
