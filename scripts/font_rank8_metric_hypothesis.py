#!/usr/bin/env python3
"""Qualify the rank-8 exact-metric hypothesis without mutating product rules."""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont

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


TARGET_FACE = "KoPubWorld바탕체 Light"
SUBSTITUTION_FACE = "KoPubWorld돋움체 Light"
FIXTURE_SHA256 = "f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084"
EXACT_TTF_SHA256 = "e3ee21a86b6a6728c567a95aaebd8883480f27ce4f230207b0d7266b5cb3fb18"
CDN_OTF_SHA256 = "895fdc6de0ff0fe24b1a63ae16601c174c810b24daa23ade78115b7e134c4c0a"
CDN_WOFF2_SHA256 = "679e263af731c5a23bbe666d67cc1cc5ebed2a16c3985ed7879fce5b0d447ed9"
PACKAGE_SHA256 = "301715bd8dcf0f6f7943c9fce7cdebd783d435756517ce2929601919a01befa7"
LICENSE_SHA256 = "411e7de3f06d32aa0f1c9ab35c5760a7cbe9543ee5867ec491f9b696ed8e0816"
REGISTRY_SHA256 = "fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16"
Q1_KIND = "font-rank8-current-trace-baseline"
MAX_FONT_BYTES = 64 * 1024 * 1024
MAX_JSON_BYTES = 64 * 1024 * 1024
FONT_SIZE_HWPUNIT = 1000
HWPUNIT_PER_PX = 75
NAME_IDS = {
    1: "family",
    2: "subfamily",
    4: "fullName",
    6: "postScriptName",
    16: "preferredFamily",
    17: "preferredSubfamily",
}
FIXED_CONTEXTS = (
    {"context": "table-cell", "parentParaIdx": 19, "contentWidthHwpunit": 28980},
    {"context": "text-box", "parentParaIdx": 20, "contentWidthHwpunit": 29434},
)


class Rank8MetricError(OracleStage2Error):
    """A fail-closed W8-Q2 contract violation."""


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise Rank8MetricError(f"{label} mismatch")


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
            raise Rank8MetricError(f"{label} exposes an absolute path")


def _normalize_pen_value(value: Any) -> Any:
    if isinstance(value, (tuple, list)):
        return [_normalize_pen_value(entry) for entry in value]
    if isinstance(value, float):
        if not math.isfinite(value):
            raise Rank8MetricError("non-finite outline coordinate")
        return float(f"{value:.9g}")
    if isinstance(value, (str, int, bool)) or value is None:
        return value
    return str(value)


def _names(font: TTFont) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    for name_id, label in NAME_IDS.items():
        values = set()
        for record in font["name"].names:
            if record.nameID != name_id:
                continue
            try:
                value = record.toUnicode().strip()
            except Exception:
                continue
            if value:
                values.add(value)
        result[label] = sorted(values, key=lambda value: value.encode("utf-8"))
    return result


def _outline_digest(font: TTFont, glyph_name: str) -> str:
    pen = RecordingPen()
    try:
        font.getGlyphSet()[glyph_name].draw(pen)
    except Exception as error:
        raise Rank8MetricError(f"glyph outline failed: {type(error).__name__}") from error
    return sha256_bytes(canonical_json_bytes(_normalize_pen_value(pen.value)))


def inspect_font(path: Path, *, label: str, fixture_codepoints: set[int]) -> dict[str, Any]:
    try:
        font = TTFont(path, lazy=False, recalcBBoxes=False, recalcTimestamp=False)
    except Exception as error:
        raise Rank8MetricError(f"{label} parse failed: {type(error).__name__}") from error
    try:
        for table in ("name", "head", "maxp", "cmap", "hmtx", "OS/2"):
            if table not in font:
                raise Rank8MetricError(f"{label} lacks {table}")
        cmap = font.getBestCmap() or {}
        metrics = font["hmtx"].metrics
        missing = fixture_codepoints - set(cmap)
        if missing:
            raise Rank8MetricError(f"{label} misses fixture codepoints")
        names = _names(font)
        fixture_metrics = {
            codepoint: metrics[cmap[codepoint]][0] for codepoint in fixture_codepoints
        }
        fixture_outlines = {
            codepoint: _outline_digest(font, cmap[codepoint]) for codepoint in fixture_codepoints
        }
        sfnt_version = font.sfntVersion
        if isinstance(sfnt_version, bytes):
            sfnt_version = sfnt_version.decode("latin-1")
        if "glyf" in font:
            technology = "truetype-glyf"
        elif "CFF " in font:
            technology = "opentype-cff"
        else:
            technology = "other"
        identity_names = {
            name
            for field in ("family", "fullName", "preferredFamily")
            for name in names[field]
        }
        public = {
            "label": label,
            "sha256": sha256_file(path),
            "bytes": path.stat().st_size,
            "sfntVersion": str(sfnt_version),
            "technology": technology,
            "flavor": font.flavor,
            "unitsPerEm": font["head"].unitsPerEm,
            "glyphCount": font["maxp"].numGlyphs,
            "cmapCodepointCount": len(cmap),
            "os2FsType": font["OS/2"].fsType,
            "nameTable": names,
            "requestedNameExact": TARGET_FACE in identity_names,
            "fixture": {
                "codepointCount": len(fixture_codepoints),
                "missingCodepoints": 0,
                "advanceSha256": sha256_bytes(
                    canonical_json_bytes(sorted(fixture_metrics.items()))
                ),
                "outlineSha256": sha256_bytes(
                    canonical_json_bytes(sorted(fixture_outlines.items()))
                ),
            },
        }
        return {
            "public": public,
            "cmap": cmap,
            "metrics": metrics,
            "fixtureMetrics": fixture_metrics,
            "fixtureOutlines": fixture_outlines,
        }
    finally:
        font.close()


def compare_fonts(left: dict[str, Any], right: dict[str, Any]) -> dict[str, Any]:
    left_cmap = left["cmap"]
    right_cmap = right["cmap"]
    common = set(left_cmap) & set(right_cmap)
    mismatch = [
        codepoint
        for codepoint in common
        if left["metrics"][left_cmap[codepoint]][0]
        != right["metrics"][right_cmap[codepoint]][0]
    ]
    fixture_common = set(left["fixtureMetrics"]) & set(right["fixtureMetrics"])
    fixture_metric_mismatch = [
        codepoint
        for codepoint in fixture_common
        if left["fixtureMetrics"][codepoint] != right["fixtureMetrics"][codepoint]
    ]
    outline_matches = sum(
        left["fixtureOutlines"][codepoint] == right["fixtureOutlines"][codepoint]
        for codepoint in fixture_common
    )
    return {
        "commonCodepoints": len(common),
        "leftOnlyCodepoints": len(set(left_cmap) - set(right_cmap)),
        "rightOnlyCodepoints": len(set(right_cmap) - set(left_cmap)),
        "advanceMismatchCount": len(mismatch),
        "commonAdvancePairSha256": sha256_bytes(
            canonical_json_bytes(
                [
                    (
                        codepoint,
                        left["metrics"][left_cmap[codepoint]][0],
                        right["metrics"][right_cmap[codepoint]][0],
                    )
                    for codepoint in sorted(common)
                ]
            )
        ),
        "fixtureCodepoints": len(fixture_common),
        "fixtureAdvanceMismatchCount": len(fixture_metric_mismatch),
        "fixtureOutlineDigestMatches": outline_matches,
        "fixtureOutlineDigestMismatches": len(fixture_common) - outline_matches,
        "byteIdentity": left["public"]["sha256"] == right["public"]["sha256"],
        "nameIdentity": left["public"]["nameTable"] == right["public"]["nameTable"],
        "technologyIdentity": left["public"]["technology"]
        == right["public"]["technology"],
    }


def apply_metric_transform(
    record: dict[str, Any], candidate_base_hwpunit: int, *, font_size_hwpunit: int = 1000
) -> int:
    """Replay the current transform order with a substituted base advance."""

    if candidate_base_hwpunit < 0 or font_size_hwpunit <= 0:
        raise Rank8MetricError("invalid candidate metric")
    metric = record["layoutMetric"]
    ratio = 1.0
    letter_spacing = 0.0
    additive = 0.0
    clamp = False
    unsupported = []
    for step in metric.get("transforms", []):
        kind = step.get("kind")
        if kind == "ratio":
            ratio = float(step["output"])
        elif kind == "letterSpacing":
            letter_spacing = float(step["input"])
        elif kind in {"extraCharacterSpacing", "extraWordSpacing", "extraDashAdvance"}:
            additive += float(step["input"])
        elif kind == "negativeSpacingClamp":
            clamp = True
        elif kind in {"boldFallback"}:
            continue
        else:
            unsupported.append(kind)
    if unsupported:
        raise Rank8MetricError(f"unsupported fixture transform: {unsupported[0]}")
    base_px = candidate_base_hwpunit / HWPUNIT_PER_PX
    font_size_px = font_size_hwpunit / HWPUNIT_PER_PX
    scaled = base_px * ratio
    final_px = scaled + letter_spacing * (scaled / font_size_px) + additive
    if clamp:
        final_px = max(final_px, scaled * 0.5)
    if not math.isfinite(final_px) or final_px < 0:
        raise Rank8MetricError("candidate final advance is invalid")
    return int(final_px * HWPUNIT_PER_PX)


def crossing_index(advances: list[int], frame_hwpunit: int) -> int | None:
    total = 0
    for index, advance in enumerate(advances):
        total += advance
        if total > frame_hwpunit:
            return index
    return None


def crossing_disposition(current: int | None, candidate: int | None) -> str:
    if current is None and candidate is None:
        return "no-crossing"
    if current is not None and candidate is None:
        return "crossing-removed"
    if current is None and candidate is not None:
        return "crossing-introduced"
    assert current is not None and candidate is not None
    if candidate > current:
        return "crossing-delayed"
    if candidate < current:
        return "crossing-earlier"
    return "crossing-unchanged"


def _read_child_json(binary: Path, arguments: list[str]) -> dict[str, Any]:
    stdout, _ = run_bounded(
        [str(binary), *arguments],
        timeout_seconds=60,
        maximum_output_bytes=MAX_JSON_BYTES,
    )
    try:
        value = json.loads(stdout)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Rank8MetricError("query output is invalid JSON") from error
    if not isinstance(value, dict):
        raise Rank8MetricError("query output must be an object")
    return value


def _validate_canonical(value: dict[str, Any], label: str) -> None:
    body = dict(value)
    claimed = body.pop("canonicalSha256", None)
    if claimed != sha256_bytes(canonical_json_bytes(body)):
        raise Rank8MetricError(f"{label} canonical SHA-256 drifted")


def _font_paths(args: argparse.Namespace) -> dict[str, Path]:
    return {
        "exactTtf": regular_input(args.font_root, args.ttf, MAX_FONT_BYTES),
        "cdnOtf": regular_input(args.font_root, args.otf, MAX_FONT_BYTES),
        "cdnWoff2": regular_input(args.font_root, args.woff2, MAX_FONT_BYTES),
        "packageJson": regular_input(args.font_root, args.package_json, 1024 * 1024),
        "license": regular_input(args.font_root, args.license, 1024 * 1024),
    }


def _style_map(manifest: dict[str, Any]) -> dict[int, dict[str, Any]]:
    styles = {
        entry["charPropertyId"]: entry for entry in manifest["semantic"]["matrix"]
    }
    if len(styles) != 18:
        raise Rank8MetricError("fixture style matrix drifted")
    return styles


def build_projection(
    *,
    manifest: dict[str, Any],
    q1: dict[str, Any],
    trace: dict[str, Any],
    layout: dict[str, Any],
    fonts: dict[str, dict[str, Any]],
    package: dict[str, Any],
    registry: dict[str, Any],
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
        raise Rank8MetricError("current trace is incomplete")
    if not all(record.get("document", {}).get("face") == TARGET_FACE for record in records):
        raise Rank8MetricError("current trace contains another face")
    layout_runs = layout.get("runs")
    if layout.get("tool") != "rhwp-q-text-layout" or not isinstance(layout_runs, list):
        raise Rank8MetricError("text layout identity mismatch")
    require_equal(len(layout_runs), trace["counts"]["runsSeen"], "layout run count")

    exact = fonts["exactTtf"]
    styles = _style_map(manifest)
    candidate_by_record: dict[str, int] = {}
    candidate_by_run: dict[int, int] = defaultdict(int)
    current_by_run: dict[int, int] = defaultdict(int)
    delta_signs: Counter[str] = Counter()
    transform_kinds: Counter[str] = Counter()
    axes: dict[tuple[int, int, bool], dict[str, int]] = defaultdict(
        lambda: {"records": 0, "currentAdvanceHwpunit": 0, "candidateAdvanceHwpunit": 0}
    )
    current_replay_mismatches = 0

    for record in records:
        source = record["source"]
        style = styles.get(source["charShapeId"])
        if style is None:
            raise Rank8MetricError("trace character style is outside the fixture matrix")
        codepoint = source["codePoint"]
        glyph = exact["cmap"].get(codepoint)
        if glyph is None:
            raise Rank8MetricError("exact TTF misses a traced character")
        replay = apply_metric_transform(record, record["layoutMetric"]["baseAdvanceHwpunit"])
        current = record["layoutMetric"]["finalAdvanceHwpunit"]
        current_replay_mismatches += replay != current
        raw_advance = exact["metrics"][glyph][0]
        candidate_base = raw_advance * FONT_SIZE_HWPUNIT // exact["public"]["unitsPerEm"]
        candidate = apply_metric_transform(record, candidate_base)
        candidate_by_record[record["recordId"]] = candidate
        run_index = source["runIndex"]
        candidate_by_run[run_index] += candidate
        current_by_run[run_index] += current
        delta = candidate - current
        delta_signs["narrower" if delta < 0 else "wider" if delta > 0 else "equal"] += 1
        axis = (style["ratio"], style["spacing"], style["kerning"])
        axes[axis]["records"] += 1
        axes[axis]["currentAdvanceHwpunit"] += current
        axes[axis]["candidateAdvanceHwpunit"] += candidate
        for step in record["layoutMetric"].get("transforms", []):
            transform_kinds[step["kind"]] += 1

    require_equal(current_replay_mismatches, 0, "current transform replay")
    for run_index, run in enumerate(layout_runs):
        trace_text = "".join(
            record["source"]["character"]
            for record in records
            if record["source"]["runIndex"] == run_index
        )
        require_equal(trace_text, run["text"], f"trace/layout run {run_index}")

    fixed_rows = []
    manifest_contexts = manifest["semantic"]["contexts"]
    q1_rows = {
        (row["context"], row["representativeIndex"]): row for row in q1["fixedGeometry"]
    }
    for definition in FIXED_CONTEXTS:
        context = definition["context"]
        frame = definition["contentWidthHwpunit"]
        context_manifest = [row for row in manifest_contexts if row["context"] == context]
        require_equal(len(context_manifest), 3, f"{context} manifest rows")
        for representative_index in range(3):
            context_row = context_manifest[representative_index]
            style = styles[context_row["charPropertyId"]]
            selected_records = [
                record
                for record in records
                if record["source"]["paragraphIndex"] == definition["parentParaIdx"]
                and record["source"]["nestedPath"] == [0, 0, representative_index]
            ]
            if not selected_records:
                raise Rank8MetricError("fixed trace records are missing")
            current_advances = [
                record["layoutMetric"]["finalAdvanceHwpunit"] for record in selected_records
            ]
            candidate_advances = [
                candidate_by_record[record["recordId"]] for record in selected_records
            ]
            current_crossing = crossing_index(current_advances, frame)
            candidate_crossing = crossing_index(candidate_advances, frame)
            disposition = crossing_disposition(current_crossing, candidate_crossing)

            selected_runs = [
                (index, run)
                for index, run in enumerate(layout_runs)
                if run.get("parentParaIdx") == definition["parentParaIdx"]
                and run.get("cellParaIdx") == representative_index
            ]
            lines: dict[float, list[tuple[int, dict[str, Any]]]] = defaultdict(list)
            for index, run in selected_runs:
                lines[run["y"]].append((index, run))
            candidate_line_widths = [
                sum(candidate_by_run[index] for index, _ in line) / HWPUNIT_PER_PX
                for _, line in sorted(lines.items())
            ]
            current_line_widths = [
                max(run["x"] + run["w"] for _, run in line)
                - min(run["x"] for _, run in line)
                for _, line in sorted(lines.items())
            ]
            q1_row = q1_rows[(context, representative_index)]
            require_equal(q1_row["contentWidthHwpunit"], frame, "Q1 frame width")
            require_equal(q1_row["lineCount"], len(lines), "Q1 line count")
            require_equal(
                q1_row["maximumLineWidthPx"],
                round(max(current_line_widths), 1),
                "Q1 current line width",
            )
            first_divergence = (
                None
                if current_crossing == candidate_crossing
                else min(
                    value for value in (current_crossing, candidate_crossing) if value is not None
                )
            )
            non_worsening = disposition not in {"crossing-introduced", "crossing-earlier"}
            fixed_rows.append(
                {
                    "context": context,
                    "representativeIndex": representative_index,
                    "lineSegLane": context_row["lineSegLane"],
                    "ratio": style["ratio"],
                    "spacing": style["spacing"],
                    "kerning": style["kerning"],
                    "contentWidthHwpunit": frame,
                    "currentObservedLineCount": len(lines),
                    "currentParagraphAdvanceHwpunit": sum(current_advances),
                    "candidateParagraphAdvanceHwpunit": sum(candidate_advances),
                    "advanceDeltaHwpunit": sum(candidate_advances) - sum(current_advances),
                    "currentFirstFrameCrossingCharacterIndex": current_crossing,
                    "candidateFirstFrameCrossingCharacterIndex": candidate_crossing,
                    "firstMetricBoundaryDivergenceCharacterIndex": first_divergence,
                    "boundaryDisposition": disposition,
                    "candidateNonWorsening": non_worsening,
                    "currentMaximumObservedLineWidthPx": round(max(current_line_widths), 1),
                    "candidateMaximumSamePartitionLineWidthPx": round(
                        max(candidate_line_widths), 1
                    ),
                    "candidateMinimumSamePartitionSlackPx": round(
                        frame / HWPUNIT_PER_PX - max(candidate_line_widths), 1
                    ),
                }
            )

    all_non_worsening = all(row["candidateNonWorsening"] for row in fixed_rows)
    pair_ttf_otf = compare_fonts(fonts["exactTtf"], fonts["cdnOtf"])
    pair_ttf_woff2 = compare_fonts(fonts["exactTtf"], fonts["cdnWoff2"])
    pair_otf_woff2 = compare_fonts(fonts["cdnOtf"], fonts["cdnWoff2"])
    require_equal(pair_ttf_otf["advanceMismatchCount"], 0, "TTF/OTF common advances")
    require_equal(pair_ttf_woff2["advanceMismatchCount"], 0, "TTF/WOFF2 common advances")
    require_equal(pair_otf_woff2["advanceMismatchCount"], 0, "OTF/WOFF2 advances")
    require_equal(pair_otf_woff2["fixtureOutlineDigestMismatches"], 0, "OTF/WOFF2 outlines")

    package_license = package.get("licenses")
    if (
        package.get("name") != "font-kopubworld"
        or package.get("version") != "1.0.3"
        or not isinstance(package_license, list)
        or package_license[0].get("type") != "KOPUS-Custom"
    ):
        raise Rank8MetricError("pinned package metadata drifted")

    active_rules = [
        rule
        for rule in registry.get("rules", [])
        if rule.get("status") == "active" and rule.get("sourceFace") == TARGET_FACE
    ]
    if registry.get("schemaVersion") != "2.0" or len(active_rules) != 2:
        raise Rank8MetricError("current registry target rules drifted")
    rules_by_projection = {
        rule["projections"][0]["id"]: rule for rule in active_rules if len(rule["projections"]) == 1
    }
    expected_urls = {
        "canvas2d-webfont": "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Light.woff2",
        "canvaskit-sfnt": "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Light.otf",
    }
    if set(rules_by_projection) != set(expected_urls):
        raise Rank8MetricError("current registry projection set drifted")
    canvas_rule = rules_by_projection["canvas2d-webfont"]
    canvaskit_rule = rules_by_projection["canvaskit-sfnt"]
    require_equal(canvas_rule["supply"]["sourceUrl"], expected_urls["canvas2d-webfont"], "Canvas2D source URL")
    require_equal(
        canvaskit_rule["supply"]["online"]["sources"][0]["url"],
        expected_urls["canvaskit-sfnt"],
        "CanvasKit source URL",
    )
    for rule in active_rules:
        require_equal(rule["decisionPlane"], "supply", "current target decision plane")
        require_equal(rule["metricEntryIds"], [], "current target metric entries")

    current_total = sum(record["layoutMetric"]["finalAdvanceHwpunit"] for record in records)
    candidate_total = sum(candidate_by_record.values())
    public = {
        "schemaVersion": 1,
        "kind": "font-rank8-metric-hypothesis",
        "issue": 4967,
        "stage": "W8-Q2",
        "target": {"documentFace": TARGET_FACE, "queueRank": 8},
        "inputs": {
            "fixture": {"sha256": FIXTURE_SHA256},
            "q1Baseline": {
                "canonicalSha256": q1["canonicalSha256"],
                "traceSha256": q1["trace"]["canonicalTraceSha256"],
            },
            "fontRuleRegistryVersion": "font-kopubworld@1.0.3",
            "fontRuleRegistryV2": {
                "sha256": REGISTRY_SHA256,
                "ruleIds": sorted(rule["ruleId"] for rule in active_rules),
            },
        },
        "sources": {
            "exactTtf": fonts["exactTtf"]["public"],
            "cdnOtf": fonts["cdnOtf"]["public"],
            "cdnWoff2": fonts["cdnWoff2"]["public"],
            "comparisons": {
                "exactTtfVsCdnOtf": pair_ttf_otf,
                "exactTtfVsCdnWoff2": pair_ttf_woff2,
                "cdnOtfVsCdnWoff2": pair_otf_woff2,
            },
            "distributionProvenance": {
                "package": "font-kopubworld",
                "version": "1.0.3",
                "packageSha256": sha256_file(paths["packageJson"]),
                "licenseType": package_license[0]["type"],
                "licenseSha256": sha256_file(paths["license"]),
                "packageRepository": package["repository"]["url"],
                "packageAuthority": "third-party-wrapper",
                "officialByteLineage": "not-proven-by-package-metadata",
                "officialLicenseReference": "https://www.heritagelab-chacl.org/9e8dfdd2-0faf-4224-a88e-6bd841c3ce06",
                "liveReferenceVerifiedAt": "2026-08-25",
                "redistributionDecision": "not-made-in-qualification",
            },
        },
        "projection": {
            "records": len(records),
            "fixtureCodepoints": len({record["source"]["codePoint"] for record in records}),
            "currentTransformReplayMismatches": current_replay_mismatches,
            "currentAdvanceHwpunit": current_total,
            "candidateAdvanceHwpunit": candidate_total,
            "advanceDeltaHwpunit": candidate_total - current_total,
            "recordDeltaCounts": dict(sorted(delta_signs.items())),
            "transformKindCounts": dict(sorted(transform_kinds.items())),
            "axes": [
                {
                    "ratio": key[0],
                    "spacing": key[1],
                    "kerning": key[2],
                    **value,
                    "advanceDeltaHwpunit": value["candidateAdvanceHwpunit"]
                    - value["currentAdvanceHwpunit"],
                }
                for key, value in sorted(axes.items())
            ],
            "fixedGeometry": fixed_rows,
        },
        "hypothesis": {
            "status": "qualified-for-q3" if all_non_worsening else "rejected",
            "targetDecisionPlane": "layout-metric" if all_non_worsening else None,
            "decisionPlaneCount": 1 if all_non_worsening else 0,
            "sharedCmapAdvanceCompatibility": True,
            "fixtureAdvanceCompatibility": True,
            "paintIdentityEstablished": False,
            "fontIdentityEstablished": False,
            "fixedGeometryNonWorsening": all_non_worsening,
            "productMutationAuthorized": False,
            "nextStage": "bounded-private-cohort" if all_non_worsening else "stop-no-change",
        },
        "executionPolicy": {
            "fullCorpusRerun": False,
            "hyperVOracleRerun": False,
            "registryMutation": False,
            "metricDbMutation": False,
            "fallbackMutation": False,
            "paintOrSupplyMutation": False,
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
    parser.add_argument("--otf", required=True)
    parser.add_argument("--woff2", required=True)
    parser.add_argument("--package-json", required=True)
    parser.add_argument("--license", required=True)
    parser.add_argument("--fixture", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--q1", required=True)
    parser.add_argument("--registry", required=True)
    parser.add_argument("--font-trace-bin", required=True)
    parser.add_argument("--text-layout-bin", required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    font_paths = _font_paths(args)
    expected_hashes = {
        "exactTtf": EXACT_TTF_SHA256,
        "cdnOtf": CDN_OTF_SHA256,
        "cdnWoff2": CDN_WOFF2_SHA256,
        "packageJson": PACKAGE_SHA256,
        "license": LICENSE_SHA256,
    }
    for label, expected in expected_hashes.items():
        require_equal(sha256_file(font_paths[label]), expected, label)

    fixture_path = regular_input(ROOT, args.fixture, 16 * 1024 * 1024)
    manifest_path = regular_input(ROOT, args.manifest, 4 * 1024 * 1024)
    q1_path = regular_input(ROOT, args.q1, 4 * 1024 * 1024)
    registry_path = regular_input(ROOT, args.registry, 16 * 1024 * 1024)
    font_trace_bin = regular_input(ROOT, args.font_trace_bin, 512 * 1024 * 1024)
    text_layout_bin = regular_input(ROOT, args.text_layout_bin, 512 * 1024 * 1024)
    require_equal(sha256_file(fixture_path), FIXTURE_SHA256, "rank-8 fixture")
    manifest = read_json(manifest_path)
    q1 = read_json(q1_path)
    registry = read_json(registry_path)
    require_equal(sha256_file(registry_path), REGISTRY_SHA256, "font registry v2")
    if (
        manifest.get("issue") != 4963
        or manifest.get("semantic", {}).get("documentFace") != TARGET_FACE
        or manifest.get("semantic", {}).get("substitutionFace") != SUBSTITUTION_FACE
        or manifest.get("semantic", {}).get("fontBytesEmbedded") is not False
    ):
        raise Rank8MetricError("fixture manifest identity mismatch")
    if q1.get("kind") != Q1_KIND or q1.get("issue") != 4967:
        raise Rank8MetricError("Q1 baseline identity mismatch")
    _validate_canonical(q1, "Q1 baseline")

    trace_envelope = _read_child_json(
        font_trace_bin,
        [str(fixture_path), "--page", "0", "--max-characters", "4096", "--json"],
    )
    if trace_envelope.get("tool") != "rhwp-q-font-trace":
        raise Rank8MetricError("font trace envelope identity mismatch")
    layout = _read_child_json(
        text_layout_bin, [str(fixture_path), "--page", "0", "--json"]
    )
    trace = trace_envelope["trace"]
    require_equal(
        sha256_bytes(canonical_json_bytes(trace)),
        q1["trace"]["canonicalTraceSha256"],
        "Q1/current trace",
    )
    fixture_codepoints = {record["source"]["codePoint"] for record in trace["records"]}
    fonts = {
        label: inspect_font(path, label=label, fixture_codepoints=fixture_codepoints)
        for label, path in font_paths.items()
        if label in {"exactTtf", "cdnOtf", "cdnWoff2"}
    }
    package = read_json(font_paths["packageJson"])
    projection = build_projection(
        manifest=manifest,
        q1=q1,
        trace=trace,
        layout=layout,
        fonts=fonts,
        package=package,
        registry=registry,
        paths=font_paths,
    )
    write_json(output_path(ROOT, args.output), projection, mode=0o644)
    print(
        json.dumps(
            {
                "status": projection["hypothesis"]["status"],
                "targetDecisionPlane": projection["hypothesis"]["targetDecisionPlane"],
                "records": projection["projection"]["records"],
                "canonicalSha256": projection["canonicalSha256"],
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
