#!/usr/bin/env python3
"""Qualify the rank-1 virtual layout-name relation without product mutation."""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from fontTools.ttLib import TTFont

from font_rank8_metric_hypothesis import apply_metric_transform, crossing_index
from oracle_stage2_common import (
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_json,
    regular_input,
    run_bounded,
    sha256_bytes,
    sha256_file,
    write_json,
)


TARGET_FACE = "문체부 바탕체"
CANONICAL_FACE = "MBatang"
EXPECTED_FONT_SHA256 = "d10509215d923fef07c1f2dffe8ebf55cbca706476559a861dff6f7cf969ff44"
EXPECTED_FIXTURE_SHA256 = "8ded3aff6f0286ee5ee4ad9c66732026fa627220b529e5d0fa7b9d51bc3ddb3f"
EXPECTED_ENTRY_ID = "font-metric.e6fdb023c2acf414807d"
MAX_FONT_BYTES = 64 * 1024 * 1024
MAX_JSON_BYTES = 64 * 1024 * 1024
FONT_SIZE_HWPUNIT = 1000
FIXED_CONTEXTS = (
    {"context": "table-cell", "parentParaIdx": 19, "contentWidthHwpunit": 28980},
    {"context": "text-box", "parentParaIdx": 20, "contentWidthHwpunit": 29434},
)


class Rank1MetricError(OracleStage2Error):
    """A fail-closed W8-R1-Q2 contract violation."""


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise Rank1MetricError(f"{label} mismatch")


def reject_absolute_paths(value: Any, label: str = "public") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            reject_absolute_paths(child, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            reject_absolute_paths(child, f"{label}[{index}]")
    elif isinstance(value, str):
        if value.startswith(("/", "\\\\")) or (
            len(value) >= 3 and value[0].isalpha() and value[1] == ":" and value[2] in "\\/"
        ):
            raise Rank1MetricError(f"{label} exposes an absolute path")


def _canonical_check(value: dict[str, Any], label: str) -> None:
    body = dict(value)
    claimed = body.pop("canonicalSha256", None)
    if claimed != sha256_bytes(canonical_json_bytes(body)):
        raise Rank1MetricError(f"{label} canonical SHA-256 drifted")


def _integer_array(source: str, symbol: str, width: str) -> list[int]:
    pattern = re.compile(
        rf"static\s+{re.escape(symbol)}:\s*\[{width};\s*(\d+)\]\s*=\s*\[(.*?)\];",
        re.DOTALL,
    )
    matches = list(pattern.finditer(source))
    if len(matches) != 1:
        raise Rank1MetricError(f"metric array {symbol} is missing or duplicated")
    expected = int(matches[0].group(1))
    values = [int(value, 0) for value in re.findall(r"0x[0-9A-Fa-f]+|\d+", matches[0].group(2))]
    require_equal(len(values), expected, f"metric array {symbol} length")
    return values


def parse_generated_metric(source: str, name: str = CANONICAL_FACE) -> dict[str, Any]:
    metric_pattern = re.compile(
        r"FontMetric\s*\{\s*"
        rf'name:\s*"{re.escape(name)}",\s*'
        r"bold:\s*(true|false),\s*italic:\s*(true|false),\s*"
        r"em_size:\s*(\d+),\s*latin_ranges:\s*&([A-Z0-9_]+),\s*"
        r"hangul:\s*(?:Some\(&([A-Z0-9_]+)\)|None),\s*\}",
        re.DOTALL,
    )
    matches = list(metric_pattern.finditer(source))
    if len(matches) != 1:
        raise Rank1MetricError(f"generated metric {name} is missing or duplicated")
    match = matches[0]
    index = source[: match.start()].count("FontMetric {")
    latin_symbol = match.group(4)
    hangul_symbol = match.group(5)
    latin_block = re.search(
        rf"static\s+{re.escape(latin_symbol)}:\s*\[LatinRange;\s*(\d+)\]\s*=\s*\[(.*?)\];",
        source,
        re.DOTALL,
    )
    if latin_block is None:
        raise Rank1MetricError("generated Latin ranges are missing")
    ranges = []
    for range_match in re.finditer(
        r"LatinRange\s*\{\s*start:\s*(0x[0-9A-Fa-f]+|\d+),\s*"
        r"end:\s*(0x[0-9A-Fa-f]+|\d+),\s*widths:\s*&([A-Z0-9_]+),\s*\}",
        latin_block.group(2),
        re.DOTALL,
    ):
        start = int(range_match.group(1), 0)
        end = int(range_match.group(2), 0)
        widths = _integer_array(source, range_match.group(3), "u16")
        require_equal(len(widths), end - start + 1, "Latin range width count")
        ranges.append({"start": start, "end": end, "widths": widths})
    require_equal(len(ranges), int(latin_block.group(1)), "Latin range count")

    hangul = None
    if hangul_symbol is not None:
        hangul_block = re.search(
            rf"static\s+{re.escape(hangul_symbol)}:\s*HangulMetric\s*=\s*(?:HangulMetric\s*)?\{{(.*?)\}};",
            source,
            re.DOTALL,
        )
        if hangul_block is None:
            raise Rank1MetricError("generated Hangul metric is missing")
        body = hangul_block.group(1)

        def scalar(field: str) -> int:
            found = re.search(rf"{field}:\s*(\d+),", body)
            if found is None:
                raise Rank1MetricError(f"Hangul scalar {field} is missing")
            return int(found.group(1))

        def symbol(field: str) -> str:
            found = re.search(rf"{field}:\s*&([A-Z0-9_]+),", body)
            if found is None:
                raise Rank1MetricError(f"Hangul symbol {field} is missing")
            return found.group(1)

        hangul = {
            "choGroups": scalar("cho_groups"),
            "jungGroups": scalar("jung_groups"),
            "jongGroups": scalar("jong_groups"),
            "choMap": _integer_array(source, symbol("cho_map"), "u8"),
            "jungMap": _integer_array(source, symbol("jung_map"), "u8"),
            "jongMap": _integer_array(source, symbol("jong_map"), "u8"),
            "widths": _integer_array(source, symbol("widths"), "u16"),
        }
        require_equal(len(hangul["choMap"]), 19, "Hangul choseong map")
        require_equal(len(hangul["jungMap"]), 21, "Hangul jungseong map")
        require_equal(len(hangul["jongMap"]), 28, "Hangul jongseong map")
        require_equal(
            len(hangul["widths"]),
            hangul["choGroups"] * hangul["jungGroups"] * hangul["jongGroups"],
            "Hangul width grid",
        )
    return {
        "index": index,
        "name": name,
        "bold": match.group(1) == "true",
        "italic": match.group(2) == "true",
        "emSize": int(match.group(3)),
        "latinRanges": ranges,
        "hangul": hangul,
    }


def metric_width(metric: dict[str, Any], codepoint: int) -> int | None:
    if 0xAC00 <= codepoint <= 0xD7A3:
        hangul = metric["hangul"]
        if hangul is None:
            return None
        offset = codepoint - 0xAC00
        cho = offset // (21 * 28)
        jung = (offset % (21 * 28)) // 28
        jong = offset % 28
        group = (
            hangul["choMap"][cho] * hangul["jungGroups"] * hangul["jongGroups"]
            + hangul["jungMap"][jung] * hangul["jongGroups"]
            + hangul["jongMap"][jong]
        )
        return hangul["widths"][group]
    for entry in metric["latinRanges"]:
        if entry["start"] <= codepoint <= entry["end"]:
            width = entry["widths"][codepoint - entry["start"]]
            return width if width > 0 else None
    return None


def exhaustive_equivalence(
    metric: dict[str, Any], exact_cmap: dict[int, str], exact_metrics: dict[str, tuple[int, int]]
) -> dict[str, Any]:
    generated_hangul_widths = Counter(
        metric_width(metric, codepoint) for codepoint in range(0xAC00, 0xD7A4)
    )
    require_equal(generated_hangul_widths, Counter({1000: 11172}), "generated Hangul domain")
    require_equal(metric_width(metric, 0x20), 500, "generated space width")
    exact_layout_codepoints = sorted(
        codepoint for codepoint in exact_cmap if codepoint == 0x20 or 0xAC00 <= codepoint <= 0xD7A3
    )
    exact_mismatches = [
        codepoint
        for codepoint in exact_layout_codepoints
        if metric_width(metric, codepoint) != exact_metrics[exact_cmap[codepoint]][0]
    ]
    require_equal(exact_mismatches, [], "generated/exact layout-bearing advances")
    exact_hangul = sum(0xAC00 <= codepoint <= 0xD7A3 for codepoint in exact_cmap)
    return {
        "currentToVirtual": {
            "hangulCodepoints": 11172,
            "hangulAdvanceHwpunit": 1000,
            "spaceAdvanceHwpunit": 500,
            "otherCodepoints": "generated-metric-miss-preserves-current-heuristic",
        },
        "virtualToExact": {
            "exactLayoutBearingCodepoints": len(exact_layout_codepoints),
            "advanceMismatchCount": len(exact_mismatches),
            "exactHangulCodepoints": exact_hangul,
            "exactNonLayoutControlCodepoints": len(exact_cmap) - len(exact_layout_codepoints),
            "sourceMissingCodepoints": "preserve-virtual-result",
        },
        "generatedHangulCodepoints": 11172,
        "generatedHangulOutsideExactCmap": 11172 - exact_hangul,
        "transformClosure": "equal-base-advance-preserves-ratio-spacing-justification-and-clamp-result",
        "styleDomain": {
            "boldItalicCombinations": 4,
            "availableMetricStyles": ["regular"],
            "missingStyleSelection": "name-first-regular-metric",
            "boldFallbackLayoutAdvance": "unchanged-metadata-only",
        },
        "disposition": "all-layout-bearing-codepoints-observationally-equivalent",
        "boundedCohortCharacterEnumerationRequired": False,
        "boundedCohortReparsePerformed": False,
    }


def project_record(
    record: dict[str, Any], metric: dict[str, Any], exact_cmap: dict[int, str], exact_metrics: dict[str, tuple[int, int]]
) -> dict[str, Any]:
    codepoint = record["source"]["codePoint"]
    current_base = record["layoutMetric"]["baseAdvanceHwpunit"]
    stored_width = metric_width(metric, codepoint)
    if record["source"]["character"] == " ":
        virtual_base = metric["emSize"] // 2
        virtual_source = "metricHalfSpace"
    elif stored_width is not None:
        virtual_base = stored_width * FONT_SIZE_HWPUNIT // metric["emSize"]
        virtual_source = "embeddedMetric"
    else:
        virtual_base = current_base
        virtual_source = record["layoutMetric"]["widthSource"]
    glyph = exact_cmap.get(codepoint)
    exact_applied = glyph is not None
    exact_base = (
        exact_metrics[glyph][0] * FONT_SIZE_HWPUNIT // metric["emSize"]
        if exact_applied
        else virtual_base
    )
    current_final = record["layoutMetric"]["finalAdvanceHwpunit"]
    replay_final = apply_metric_transform(record, current_base)
    virtual_final = apply_metric_transform(record, virtual_base)
    exact_final = apply_metric_transform(record, exact_base)
    return {
        "currentBase": current_base,
        "virtualBase": virtual_base,
        "exactBase": exact_base,
        "currentFinal": current_final,
        "replayFinal": replay_final,
        "virtualFinal": virtual_final,
        "exactFinal": exact_final,
        "virtualWidthSource": virtual_source,
        "generatedMetricHit": stored_width is not None or record["source"]["character"] == " ",
        "exactApplied": exact_applied,
    }


def _read_trace(binary: Path, fixture: Path) -> dict[str, Any]:
    stdout, _ = run_bounded(
        [str(binary), str(fixture), "--page", "0", "--max-characters", "4096", "--json"],
        timeout_seconds=60,
        maximum_output_bytes=MAX_JSON_BYTES,
    )
    try:
        envelope = json.loads(stdout)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Rank1MetricError("font trace output is invalid JSON") from error
    if envelope.get("tool") != "rhwp-q-font-trace":
        raise Rank1MetricError("font trace envelope identity mismatch")
    return envelope["trace"]


def _axis(record: dict[str, Any]) -> tuple[str, str, str, bool]:
    ratio = "1"
    spacing = "0"
    additive = "0"
    clamp = False
    total_additive = 0.0
    for step in record["layoutMetric"].get("transforms", []):
        if step["kind"] == "ratio":
            ratio = str(step["output"])
        elif step["kind"] == "letterSpacing":
            spacing = str(step["input"])
        elif step["kind"] in {"extraCharacterSpacing", "extraWordSpacing", "extraDashAdvance"}:
            total_additive += float(step["input"])
        elif step["kind"] == "negativeSpacingClamp":
            clamp = True
    additive = format(total_additive, ".12g")
    return ratio, spacing, additive, clamp


def build_projection(
    *,
    q0: dict[str, Any],
    q1: dict[str, Any],
    lineage: dict[str, Any],
    trace: dict[str, Any],
    metric: dict[str, Any],
    exact_public: dict[str, Any],
    exact_cmap: dict[int, str],
    exact_metrics: dict[str, tuple[int, int]],
    paths: dict[str, Path],
) -> dict[str, Any]:
    records = trace.get("records")
    if (
        trace.get("schemaVersion") != 1
        or trace.get("status") != "complete"
        or not isinstance(records, list)
        or len(records) != 1556
        or trace.get("counts", {}).get("recordsOmitted") != 0
    ):
        raise Rank1MetricError("current trace is incomplete")
    if not all(record.get("document", {}).get("face") == TARGET_FACE for record in records):
        raise Rank1MetricError("current trace contains another face")

    projected = [project_record(record, metric, exact_cmap, exact_metrics) for record in records]
    require_equal(sum(row["replayFinal"] != row["currentFinal"] for row in projected), 0, "transform replay")
    require_equal(sum(row["virtualFinal"] != row["currentFinal"] for row in projected), 0, "virtual relation final advance")
    require_equal(sum(row["exactFinal"] != row["virtualFinal"] for row in projected), 0, "exact final advance")

    exhaustive = exhaustive_equivalence(metric, exact_cmap, exact_metrics)

    axis_rows: dict[tuple[str, str, str, bool], dict[str, int]] = defaultdict(
        lambda: {"records": 0, "currentAdvanceHwpunit": 0, "virtualAdvanceHwpunit": 0, "exactAdvanceHwpunit": 0}
    )
    transform_kinds: Counter[str] = Counter()
    for record, row in zip(records, projected, strict=True):
        axis = axis_rows[_axis(record)]
        axis["records"] += 1
        axis["currentAdvanceHwpunit"] += row["currentFinal"]
        axis["virtualAdvanceHwpunit"] += row["virtualFinal"]
        axis["exactAdvanceHwpunit"] += row["exactFinal"]
        transform_kinds.update(step["kind"] for step in record["layoutMetric"].get("transforms", []))

    fixed_geometry = []
    for context in FIXED_CONTEXTS:
        for representative_index in range(3):
            indexes = [
                index
                for index, record in enumerate(records)
                if record["source"]["paragraphIndex"] == context["parentParaIdx"]
                and record["source"]["nestedPath"] == [0, 0, representative_index]
            ]
            if not indexes:
                raise Rank1MetricError("fixed-context trace records are missing")
            current = [projected[index]["currentFinal"] for index in indexes]
            virtual = [projected[index]["virtualFinal"] for index in indexes]
            exact = [projected[index]["exactFinal"] for index in indexes]
            frame = context["contentWidthHwpunit"]
            current_crossing = crossing_index(current, frame)
            virtual_crossing = crossing_index(virtual, frame)
            exact_crossing = crossing_index(exact, frame)
            require_equal(virtual_crossing, current_crossing, "virtual frame crossing")
            require_equal(exact_crossing, current_crossing, "exact frame crossing")
            fixed_geometry.append(
                {
                    "context": context["context"],
                    "representativeIndex": representative_index,
                    "contentWidthHwpunit": frame,
                    "records": len(indexes),
                    "currentAdvanceHwpunit": sum(current),
                    "virtualAdvanceHwpunit": sum(virtual),
                    "exactAdvanceHwpunit": sum(exact),
                    "currentFirstFrameCrossingCharacterIndex": current_crossing,
                    "virtualFirstFrameCrossingCharacterIndex": virtual_crossing,
                    "exactFirstFrameCrossingCharacterIndex": exact_crossing,
                    "disposition": "unchanged",
                }
            )

    fixture_codepoints = {record["source"]["codePoint"] for record in records}
    exact_fixture_codepoints = fixture_codepoints & set(exact_cmap)
    current_total = sum(row["currentFinal"] for row in projected)
    virtual_total = sum(row["virtualFinal"] for row in projected)
    exact_total = sum(row["exactFinal"] for row in projected)
    entry = next(
        item for item in lineage.get("entries", []) if item.get("entryId") == EXPECTED_ENTRY_ID
    )
    public = {
        "schemaVersion": 1,
        "kind": "font-rank1-metric-hypothesis",
        "issue": 4967,
        "stage": "W8-R1-Q2",
        "target": {
            "documentFace": TARGET_FACE,
            "virtualCanonicalFace": CANONICAL_FACE,
            "queueRank": 1,
        },
        "inputs": {
            "q0Baseline": {"artifact": "mydocs/tech/investigations/issue-4967/rank1_qualification_baseline.json", "canonicalSha256": q0["canonicalSha256"]},
            "q1Baseline": {"artifact": "mydocs/tech/investigations/issue-4967/rank1_runtime_boundary.json", "canonicalSha256": q1["canonicalSha256"]},
            "fixture": {"artifact": "mydocs/tech/investigations/issue-4963/fixtures/oracle_typesetting_fixture.hwpx", "sha256": sha256_file(paths["fixture"])},
            "metricSource": {"artifact": "src/renderer/font_metrics_generated.rs", "sha256": sha256_file(paths["metricSource"])},
            "metricLookupSource": {"artifact": "src/renderer/font_metrics_data.rs", "sha256": sha256_file(paths["metricLookupSource"])},
            "textMeasurementSource": {"artifact": "src/renderer/layout/text_measurement.rs", "sha256": sha256_file(paths["textMeasurementSource"])},
            "fontDecisionSource": {"artifact": "src/document_core/queries/font_decision.rs", "sha256": sha256_file(paths["fontDecisionSource"])},
            "metricLineage": {"artifact": "mydocs/tech/investigations/issue-4964/font_metric_lineage_manifest.json", "sha256": sha256_file(paths["lineage"])},
            "nativeBinary": {"artifact": "target/debug/rhwp-q-font-trace", "sha256": sha256_file(paths["nativeBinary"])},
        },
        "currentMetric": {
            "entryId": entry["entryId"],
            "currentIndex": metric["index"],
            "name": metric["name"],
            "bold": metric["bold"],
            "italic": metric["italic"],
            "emSize": metric["emSize"],
            "originKind": entry["origin"]["kind"],
            "originStatus": entry["origin"]["status"],
            "metricDataSha256": entry["semanticHashes"]["metricDataSha256"],
            "widthProjectionSha256": entry["semanticHashes"]["widthProjectionSha256"],
        },
        "exactSource": exact_public,
        "metricCompatibility": {
            "exactCmapCodepoints": len(exact_cmap),
            "exactLayoutBearingCodepoints": exhaustive["virtualToExact"]["exactLayoutBearingCodepoints"],
            "exactLayoutAdvanceMismatches": exhaustive["virtualToExact"]["advanceMismatchCount"],
            "exactHangulCodepoints": exhaustive["virtualToExact"]["exactHangulCodepoints"],
            "generatedHangulCodepoints": exhaustive["generatedHangulCodepoints"],
            "generatedHangulOutsideExactCmap": exhaustive["generatedHangulOutsideExactCmap"],
            "fixtureCodepoints": len(fixture_codepoints),
            "fixtureExactCoveredCodepoints": len(exact_fixture_codepoints),
            "fixtureExactMissingCodepoints": len(fixture_codepoints - exact_fixture_codepoints),
            "fixtureRecords": len(records),
            "fixtureExactCoveredRecords": sum(row["exactApplied"] for row in projected),
            "fixtureExactMissingRecords": sum(not row["exactApplied"] for row in projected),
            "fixtureGeneratedMetricHitRecords": sum(row["generatedMetricHit"] for row in projected),
            "fixtureGeneratedMetricMissRecords": sum(not row["generatedMetricHit"] for row in projected),
            "fontIdentityEstablished": False,
            "paintIdentityEstablished": False,
        },
        "exhaustiveLayoutDomain": exhaustive,
        "projection": {
            "records": len(records),
            "currentTransformReplayMismatches": 0,
            "currentAdvanceHwpunit": current_total,
            "virtualRelationAdvanceHwpunit": virtual_total,
            "exactAdvanceHwpunit": exact_total,
            "currentToVirtualDeltaHwpunit": virtual_total - current_total,
            "virtualToExactDeltaHwpunit": exact_total - virtual_total,
            "recordDeltaCounts": {"currentToVirtualChanged": 0, "virtualToExactChanged": 0, "equal": len(records)},
            "transformKindCounts": dict(sorted(transform_kinds.items())),
            "axes": [
                {
                    "ratio": key[0],
                    "letterSpacing": key[1],
                    "additiveSpacing": key[2],
                    "negativeSpacingClamp": key[3],
                    **value,
                    "currentToVirtualDeltaHwpunit": value["virtualAdvanceHwpunit"] - value["currentAdvanceHwpunit"],
                    "virtualToExactDeltaHwpunit": value["exactAdvanceHwpunit"] - value["virtualAdvanceHwpunit"],
                }
                for key, value in sorted(axis_rows.items())
            ],
            "fixedGeometry": fixed_geometry,
        },
        "hypothesis": {
            "status": "no-change",
            "reason": "The virtual layout-name relation selects MBatang metadata but changes no fixture advance; the exact source is advance-compatible for every source-covered layout codepoint.",
            "targetDecisionPlane": None,
            "decisionPlaneCount": 0,
            "layoutBenefitObserved": False,
            "metricSourceExactLineageEstablished": False,
            "productMutationAuthorized": False,
            "nextStage": "stop-rank1-no-change",
        },
        "executionPolicy": {
            "fullCorpusRerun": False,
            "hyperVOracleRerun": False,
            "productMutation": False,
            "virtualRelationOnly": True,
        },
        "privacy": {
            "absolutePathIncluded": False,
            "fontBytesIncluded": False,
            "privateCorpusAccessed": False,
            "privateDocumentIdentityIncluded": False,
            "fullTraceTracked": False,
        },
    }
    reject_absolute_paths(public)
    public["canonicalSha256"] = sha256_bytes(canonical_json_bytes(public))
    return public


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--font-root", type=Path, required=True)
    parser.add_argument("--ttf", required=True)
    parser.add_argument("--fixture", required=True)
    parser.add_argument("--q0", required=True)
    parser.add_argument("--q1", required=True)
    parser.add_argument("--lineage", required=True)
    parser.add_argument("--metric-source", required=True)
    parser.add_argument("--metric-lookup-source", required=True)
    parser.add_argument("--text-measurement-source", required=True)
    parser.add_argument("--font-decision-source", required=True)
    parser.add_argument("--font-trace-bin", required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    paths = {
        "exactFont": regular_input(args.font_root, args.ttf, MAX_FONT_BYTES),
        "fixture": regular_input(ROOT, args.fixture, 16 * 1024 * 1024),
        "q0": regular_input(ROOT, args.q0, 4 * 1024 * 1024),
        "q1": regular_input(ROOT, args.q1, 16 * 1024 * 1024),
        "lineage": regular_input(ROOT, args.lineage, 64 * 1024 * 1024),
        "metricSource": regular_input(ROOT, args.metric_source, 64 * 1024 * 1024),
        "metricLookupSource": regular_input(ROOT, args.metric_lookup_source, 8 * 1024 * 1024),
        "textMeasurementSource": regular_input(ROOT, args.text_measurement_source, 8 * 1024 * 1024),
        "fontDecisionSource": regular_input(ROOT, args.font_decision_source, 8 * 1024 * 1024),
        "nativeBinary": regular_input(ROOT, args.font_trace_bin, 512 * 1024 * 1024),
    }
    require_equal(sha256_file(paths["exactFont"]), EXPECTED_FONT_SHA256, "exact font")
    require_equal(sha256_file(paths["fixture"]), EXPECTED_FIXTURE_SHA256, "fixture")
    q0 = read_json(paths["q0"])
    q1 = read_json(paths["q1"])
    lineage = read_json(paths["lineage"])
    _canonical_check(q0, "Q0 baseline")
    _canonical_check(q1, "Q1 baseline")
    require_equal(q0.get("kind"), "font-rank1-qualification-baseline", "Q0 kind")
    require_equal(q1.get("kind"), "font-rank1-runtime-boundary-baseline", "Q1 kind")
    require_equal(q1.get("firstDivergence", {}).get("targetDecisionPlane"), "layout-name", "Q1 plane")
    entries = [entry for entry in lineage.get("entries", []) if entry.get("entryId") == EXPECTED_ENTRY_ID]
    require_equal(len(entries), 1, "MBatang lineage entry")
    entry = entries[0]
    require_equal(entry.get("currentIndex"), 370, "MBatang lineage index")
    require_equal(entry.get("origin", {}).get("status"), "unknown", "MBatang origin status")
    require_equal(entry.get("semanticHashes", {}).get("metricDataSha256"), q0["currentMetricAnchor"]["metricDataSha256"], "MBatang metric hash")

    metric_lookup_source = paths["metricLookupSource"].read_text(encoding="utf-8")
    font_decision_source = paths["fontDecisionSource"].read_text(encoding="utf-8")
    if "bold_fallback: bold" not in metric_lookup_source:
        raise Rank1MetricError("name-first bold fallback contract drifted")
    if "fauxBoldDoesNotChangeLayoutAdvance" not in font_decision_source:
        raise Rank1MetricError("bold fallback layout-advance contract drifted")

    metric = parse_generated_metric(paths["metricSource"].read_text(encoding="utf-8"))
    require_equal(metric["index"], 370, "MBatang source index")
    require_equal(metric["emSize"], 1000, "MBatang em size")
    trace = _read_trace(paths["nativeBinary"], paths["fixture"])
    require_equal(
        sha256_bytes(canonical_json_bytes(trace)),
        q1["formats"][0]["trace"]["canonicalTraceSha256"],
        "Q1/current trace",
    )

    try:
        font = TTFont(paths["exactFont"], lazy=False, fontNumber=0)
    except Exception as error:
        raise Rank1MetricError("exact source is not a readable SFNT") from error
    try:
        exact_cmap = font.getBestCmap() or {}
        exact_metrics = font["hmtx"].metrics
        exact_public = {
            "sha256": EXPECTED_FONT_SHA256,
            "unitsPerEm": font["head"].unitsPerEm,
            "glyphCount": font["maxp"].numGlyphs,
            "cmapCodepoints": len(exact_cmap),
            "os2FsType": font["OS/2"].fsType,
            "embeddingDisposition": "restricted-license-embedding",
            "officialArtifactByteMatch": False,
        }
        require_equal(exact_public["unitsPerEm"], 1000, "exact unitsPerEm")
        projection = build_projection(
            q0=q0,
            q1=q1,
            lineage=lineage,
            trace=trace,
            metric=metric,
            exact_public=exact_public,
            exact_cmap=exact_cmap,
            exact_metrics=exact_metrics,
            paths=paths,
        )
    finally:
        font.close()
    write_json(output_path(ROOT, args.output), projection, mode=0o644)
    print(
        json.dumps(
            {
                "status": projection["hypothesis"]["status"],
                "records": projection["projection"]["records"],
                "currentToVirtualDeltaHwpunit": projection["projection"]["currentToVirtualDeltaHwpunit"],
                "virtualToExactDeltaHwpunit": projection["projection"]["virtualToExactDeltaHwpunit"],
                "canonicalSha256": projection["canonicalSha256"],
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
