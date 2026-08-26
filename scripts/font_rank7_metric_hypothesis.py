#!/usr/bin/env python3
"""Qualify the rank-7 exact-metric hypothesis without mutating product rules."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import font_rank8_metric_hypothesis as shared
from oracle_stage2_common import (
    ROOT,
    canonical_json_bytes,
    output_path,
    read_json,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


TARGET_FACE = "KoPubWorld돋움체 Light"
SUBSTITUTION_FACE = "KoPubWorld바탕체 Light"
FIXTURE_SHA256 = "1cc8062c6fd0da39cfddc4182115226717516d4250e693b43596293374236f9e"
EXACT_TTF_SHA256 = "069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f"
CDN_OTF_SHA256 = "529b2f02b96276d9209124a72181fcd7bfc656a567718670d0c3934f6c11adea"
CDN_WOFF2_SHA256 = "bc8cea5d8e4c4d82f631e40af09a7e7b92565b2124861c8387bea60dd5890417"
PACKAGE_SHA256 = "301715bd8dcf0f6f7943c9fce7cdebd783d435756517ce2929601919a01befa7"
LICENSE_SHA256 = "411e7de3f06d32aa0f1c9ab35c5760a7cbe9543ee5867ec491f9b696ed8e0816"
REGISTRY_SHA256 = "fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16"
Q0_KIND = "font-rank7-qualification-baseline"
Q1_KIND = "font-rank7-runtime-boundary-baseline"
EXPECTED_URLS = {
    "canvas2d-webfont": "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Light.woff2",
    "canvaskit-sfnt": "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Light.otf",
}


class Rank7MetricError(shared.Rank8MetricError):
    """A fail-closed W8-R7-Q2 contract violation."""


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise Rank7MetricError(f"{label} mismatch")


def validate_canonical(value: dict[str, Any], kind: str, stage: str) -> None:
    body = dict(value)
    claimed = body.pop("canonicalSha256", None)
    if (
        value.get("schemaVersion") != 1
        or value.get("kind") != kind
        or value.get("issue") != 4967
        or value.get("stage") != stage
        or value.get("target", {}).get("documentFace") != TARGET_FACE
        or value.get("target", {}).get("queueRank") != 7
        or claimed != sha256_bytes(canonical_json_bytes(body))
    ):
        raise Rank7MetricError(f"{stage} identity or canonical SHA-256 drifted")


def style_domain_audit(q0: dict[str, Any]) -> dict[str, Any]:
    domain = q0.get("cohort", {}).get("styleDomain", {})
    axes = domain.get("axes")
    if not isinstance(axes, list) or not axes:
        raise Rank7MetricError("Q0 style domain is missing")
    supported_characters = 0
    bold_characters = 0
    italic_characters = 0
    for row in axes:
        ratio = row.get("ratio")
        spacing = row.get("spacing")
        characters = row.get("characters")
        bold = row.get("bold")
        italic = row.get("italic")
        if (
            not isinstance(ratio, int)
            or ratio <= 0
            or not isinstance(spacing, int)
            or not isinstance(characters, int)
            or characters < 0
            or not isinstance(bold, bool)
            or not isinstance(italic, bool)
        ):
            raise Rank7MetricError("Q0 style axis is invalid")
        supported_characters += characters
        bold_characters += characters if bold else 0
        italic_characters += characters if italic else 0
    require_equal(
        supported_characters,
        q0["cohort"]["totalCharacters"],
        "Q0 style-domain character total",
    )
    require_equal(bold_characters, domain.get("boldCharacters"), "Q0 bold total")
    require_equal(italic_characters, domain.get("italicCharacters"), "Q0 italic total")
    return {
        "axes": len(axes),
        "characters": supported_characters,
        "ratioValues": sorted({row["ratio"] for row in axes}),
        "spacingValues": sorted({row["spacing"] for row in axes}),
        "boldCharacters": bold_characters,
        "italicCharacters": italic_characters,
        "transformDomainSupportedCharacters": supported_characters,
        "weightedAdvanceDeltaAvailableFromAggregate": False,
        "reason": "aggregate-style-axes-do-not-contain-codepoint-distribution",
    }


def bold_fallback_audit(
    q0: dict[str, Any], metric_lookup_source: Path, font_decision_source: Path
) -> dict[str, Any]:
    metric_text = metric_lookup_source.read_text(encoding="utf-8")
    decision_text = font_decision_source.read_text(encoding="utf-8")
    if "bold_fallback: bold" not in metric_text:
        raise Rank7MetricError("name-first bold fallback contract drifted")
    if "fauxBoldDoesNotChangeLayoutAdvance" not in decision_text:
        raise Rank7MetricError("bold fallback layout-advance contract drifted")
    bold_characters = q0["cohort"]["styleDomain"]["boldCharacters"]
    if bold_characters != 4468:
        raise Rank7MetricError("Q0 bold exposure drifted")
    return {
        "q0BoldCharacters": bold_characters,
        "q0ItalicCharacters": q0["cohort"]["styleDomain"]["italicCharacters"],
        "publicFixtureBoldRecords": 0,
        "candidateAssumption": "regular-metric-advance-with-synthetic-bold",
        "layoutAdvanceChangedByBoldRequest": False,
        "dynamicCohortConfirmationRequired": True,
        "sourceContracts": {
            "metricLookupSha256": sha256_file(metric_lookup_source),
            "fontDecisionSha256": sha256_file(font_decision_source),
            "boldFallbackSelectionPresent": True,
            "fauxBoldAdvanceInvariantPresent": True,
        },
    }


def q1_adapter(q1: dict[str, Any]) -> dict[str, Any]:
    formats = {entry.get("format"): entry for entry in q1.get("formats", [])}
    if set(formats) != {"hwpx", "hwp5"}:
        raise Rank7MetricError("Q1 format set drifted")
    comparison = q1.get("formatComparison", {})
    for key in (
        "layoutMetricProjectionEqual",
        "layoutRunProjectionEqual",
        "fixedGeometryEqual",
    ):
        if comparison.get(key) is not True:
            raise Rank7MetricError(f"Q1 {key} drifted")
    return {
        "canonicalSha256": q1["canonicalSha256"],
        "trace": {
            "canonicalTraceSha256": formats["hwpx"]["boundary"][
                "canonicalTraceSha256"
            ]
        },
        "fixedGeometry": formats["hwpx"]["fixedGeometry"],
        "formatTraceSha256": {
            name: entry["boundary"]["canonicalTraceSha256"]
            for name, entry in sorted(formats.items())
        },
        "layoutMetricProjectionSha256": comparison["layoutMetricProjectionSha256"],
    }


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
    parser.add_argument("--q0", required=True)
    parser.add_argument("--q1", required=True)
    parser.add_argument("--registry", required=True)
    parser.add_argument("--metric-lookup-source", required=True)
    parser.add_argument("--font-decision-source", required=True)
    parser.add_argument("--font-trace-bin", required=True)
    parser.add_argument("--text-layout-bin", required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    font_paths = shared._font_paths(args)
    expected_hashes = {
        "exactTtf": EXACT_TTF_SHA256,
        "cdnOtf": CDN_OTF_SHA256,
        "cdnWoff2": CDN_WOFF2_SHA256,
        "packageJson": PACKAGE_SHA256,
        "license": LICENSE_SHA256,
    }
    for label, expected in expected_hashes.items():
        shared.require_equal(sha256_file(font_paths[label]), expected, label)

    fixture_path = regular_input(ROOT, args.fixture, 16 * 1024 * 1024)
    manifest_path = regular_input(ROOT, args.manifest, 4 * 1024 * 1024)
    q0_path = regular_input(ROOT, args.q0, 4 * 1024 * 1024)
    q1_path = regular_input(ROOT, args.q1, 8 * 1024 * 1024)
    registry_path = regular_input(ROOT, args.registry, 16 * 1024 * 1024)
    metric_lookup_source = regular_input(ROOT, args.metric_lookup_source, 8 * 1024 * 1024)
    font_decision_source = regular_input(ROOT, args.font_decision_source, 8 * 1024 * 1024)
    font_trace_bin = regular_input(ROOT, args.font_trace_bin, 512 * 1024 * 1024)
    text_layout_bin = regular_input(ROOT, args.text_layout_bin, 512 * 1024 * 1024)
    shared.require_equal(sha256_file(fixture_path), FIXTURE_SHA256, "rank-7 fixture")
    shared.require_equal(sha256_file(registry_path), REGISTRY_SHA256, "font registry v2")

    manifest = read_json(manifest_path)
    q0 = read_json(q0_path)
    q1 = read_json(q1_path)
    registry = read_json(registry_path)
    validate_canonical(q0, Q0_KIND, "W8-R7-Q0")
    validate_canonical(q1, Q1_KIND, "W8-R7-Q1")
    if (
        manifest.get("issue") != 4963
        or manifest.get("semantic", {}).get("documentFace") != TARGET_FACE
        or manifest.get("semantic", {}).get("substitutionFace") != SUBSTITUTION_FACE
        or manifest.get("semantic", {}).get("fontBytesEmbedded") is not False
    ):
        raise Rank7MetricError("fixture manifest identity mismatch")
    adapted_q1 = q1_adapter(q1)

    trace_envelope = shared._read_child_json(
        font_trace_bin,
        [str(fixture_path), "--page", "0", "--max-characters", "4096", "--json"],
    )
    if trace_envelope.get("tool") != "rhwp-q-font-trace":
        raise Rank7MetricError("font trace envelope identity mismatch")
    trace = trace_envelope["trace"]
    shared.require_equal(
        sha256_bytes(canonical_json_bytes(trace)),
        adapted_q1["trace"]["canonicalTraceSha256"],
        "Q1/current HWPX trace",
    )
    layout = shared._read_child_json(
        text_layout_bin, [str(fixture_path), "--page", "0", "--json"]
    )
    fixture_codepoints = {record["source"]["codePoint"] for record in trace["records"]}
    fonts = {
        label: shared.inspect_font(
            path,
            label=label,
            fixture_codepoints=fixture_codepoints,
            target_face=TARGET_FACE,
        )
        for label, path in font_paths.items()
        if label in {"exactTtf", "cdnOtf", "cdnWoff2"}
    }
    package = read_json(font_paths["packageJson"])
    projection = shared.build_projection(
        manifest=manifest,
        q1=adapted_q1,
        trace=trace,
        layout=layout,
        fonts=fonts,
        package=package,
        registry=registry,
        paths=font_paths,
        target_face=TARGET_FACE,
        target_rank=7,
        kind="font-rank7-metric-hypothesis",
        stage="W8-R7-Q2",
        fixture_sha256=FIXTURE_SHA256,
        expected_urls=EXPECTED_URLS,
    )
    projection["inputs"]["q0Baseline"] = {
        "canonicalSha256": q0["canonicalSha256"]
    }
    projection["inputs"]["q1Baseline"]["formatTraceSha256"] = adapted_q1[
        "formatTraceSha256"
    ]
    projection["inputs"]["q1Baseline"][
        "layoutMetricProjectionSha256"
    ] = adapted_q1["layoutMetricProjectionSha256"]
    projection["sources"]["distributionProvenance"][
        "liveReferenceVerifiedAt"
    ] = "2026-08-26"
    projection["projection"]["q0StyleDomain"] = style_domain_audit(q0)
    projection["projection"]["styleFallbackAudit"] = bold_fallback_audit(
        q0, metric_lookup_source, font_decision_source
    )
    projection["hypothesis"]["boldCohortDynamicConfirmationRequired"] = True
    projection["hypothesis"]["hwpHwpxMetricProjectionEqualAtQ1"] = True
    projection.pop("canonicalSha256")
    shared.reject_absolute_paths(projection)
    projection["canonicalSha256"] = sha256_bytes(canonical_json_bytes(projection))
    write_json(output_path(ROOT, args.output), projection, mode=0o644)
    print(
        json.dumps(
            {
                "status": projection["hypothesis"]["status"],
                "targetDecisionPlane": projection["hypothesis"]["targetDecisionPlane"],
                "records": projection["projection"]["records"],
                "canonicalSha256": projection["canonicalSha256"],
                "privateCorpusAccessed": False,
            },
            ensure_ascii=False,
            separators=(",", ":"),
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except shared.OracleStage2Error as error:
        raise SystemExit(str(error)) from error
