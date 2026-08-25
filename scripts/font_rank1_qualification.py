#!/usr/bin/env python3
"""Project existing W3/W4/W5 evidence into the W8 rank-1 Q0 baseline."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from fontTools.ttLib import TTFont

from font_rank8_qualification import (
    MAX_COVERAGE_BYTES,
    MAX_JOURNAL_BYTES,
    MAX_MANIFEST_BYTES,
    QualificationError,
    canonical_json_bytes,
    combine_document_summaries,
    read_json,
    regular_file,
    reject_absolute_paths,
    relative_repo_path,
    require_equal,
    safe_write_json,
    scan_journal,
    sha256_bytes,
    sha256_file,
    validate_manifest_and_coverage,
)


ROOT = Path(__file__).resolve().parents[1]
TARGET_FACE = "문체부 바탕체"
CANONICAL_FACE = "MBatang"
QUEUE_RANK = 1
EXPECTED_FONT_SHA256 = "d10509215d923fef07c1f2dffe8ebf55cbca706476559a861dff6f7cf969ff44"
MAX_FONT_BYTES = 64 * 1024 * 1024


def target_ranking_entry(ranking: dict[str, Any]) -> dict[str, Any]:
    if (
        ranking.get("schemaVersion") != 1
        or ranking.get("kind") != "font-typesetting-risk-public-ranking"
        or ranking.get("issue") != 4962
        or not isinstance(ranking.get("ranking"), list)
    ):
        raise QualificationError("W4 public ranking identity mismatch")
    matches = [
        entry
        for entry in ranking["ranking"]
        if entry.get("documentFace") == TARGET_FACE
    ]
    if len(matches) != 1:
        raise QualificationError("W4 rank-1 target is missing or duplicated")
    entry = matches[0]
    require_equal(entry.get("actionRank"), QUEUE_RANK, "W4 action rank")
    require_equal(entry.get("baseRank"), 1, "W4 base rank")
    require_equal(entry.get("empiricalRiskBand"), "A", "W4 risk band")
    require_equal(entry.get("w5Queue"), True, "W4 W5 queue membership")
    return entry


def validate_ladder(ladder: dict[str, Any]) -> None:
    require_equal(ladder.get("schemaVersion"), 1, "W5 ladder schema")
    require_equal(
        ladder.get("kind"), "font-oracle-stage4-ladder-evidence", "W5 ladder kind"
    )
    require_equal(ladder.get("issue"), 4963, "W5 ladder issue")
    require_equal(
        ladder.get("target"),
        {"queueRank": QUEUE_RANK, "documentFace": TARGET_FACE},
        "W5 ladder target",
    )
    require_equal(
        [run.get("physicalState") for run in ladder.get("runs", [])],
        ["exact-only", "subst-only", "none-related"],
        "W5 physical-state ladder",
    )
    exact_fonts = ladder["runs"][0].get("managedFonts", [])
    exact_matches = [
        font
        for font in exact_fonts
        if font.get("face") == TARGET_FACE and font.get("present") is True
    ]
    if len(exact_matches) != 1:
        raise QualificationError("W5 exact source is missing or duplicated")
    require_equal(
        exact_matches[0].get("sha256"), EXPECTED_FONT_SHA256, "W5 exact font SHA-256"
    )
    dispositions = ladder.get("dispositions", [])
    require_equal(
        dispositions,
        [
            {
                "question": "curated-official-successor-only",
                "reason": "No direct publisher or byte lineage establishes an official successor.",
                "status": "not-provided",
            }
        ],
        "W5 official successor disposition",
    )
    privacy = ladder.get("privacy", {})
    for field in (
        "absolutePathIncluded",
        "fontBytesIncluded",
        "hostNameIncluded",
        "privateDocumentIdentityIncluded",
    ):
        require_equal(privacy.get(field), False, f"W5 privacy {field}")


def target_registry_rules(registry: dict[str, Any]) -> list[dict[str, Any]]:
    require_equal(registry.get("schemaVersion"), "2.0", "registry schema")
    require_equal(
        registry.get("kind"), "canonical-font-rule-lifecycle-registry", "registry kind"
    )
    rules = [
        rule
        for rule in registry.get("rules", [])
        if rule.get("status") == "active"
        and rule.get("sourceFace") in {TARGET_FACE, CANONICAL_FACE}
    ]
    require_equal(rules, [], "rank-1 explicit v2 rules")
    return rules


def metric_anchor(projection: dict[str, Any]) -> dict[str, Any]:
    matches = [
        entry
        for entry in projection.get("metricAnchors", {}).get("entries", [])
        if entry.get("name") == CANONICAL_FACE
    ]
    if len(matches) != 1:
        raise QualificationError("MBatang metric anchor is missing or duplicated")
    anchor = matches[0]
    require_equal(anchor.get("currentIndex"), 370, "MBatang metric index")
    require_equal(anchor.get("bold"), False, "MBatang metric bold")
    require_equal(anchor.get("italic"), False, "MBatang metric italic")
    return anchor


def validate_source_attestation(attestation: dict[str, Any]) -> None:
    require_equal(attestation.get("schemaVersion"), 1, "source attestation schema")
    require_equal(
        attestation.get("kind"),
        "font-rank1-source-provenance-attestation",
        "source attestation kind",
    )
    require_equal(attestation.get("issue"), 4967, "source attestation issue")
    require_equal(
        attestation.get("exactFile", {}).get("sha256"),
        EXPECTED_FONT_SHA256,
        "source attestation exact SHA-256",
    )
    require_equal(
        attestation.get("exactFile", {}).get("officialDownloadArtifactMatched"),
        False,
        "official artifact match",
    )
    require_equal(
        attestation.get("portableSupplyDisposition", {}).get("status"),
        "blocked-unmatched-official-artifact-and-restricted-embedding",
        "portable supply disposition",
    )
    privacy = attestation.get("privacy", {})
    require_equal(privacy.get("absolutePathIncluded"), False, "source privacy path")
    require_equal(privacy.get("fontBytesIncluded"), False, "source privacy bytes")


def _font_names(font: TTFont, name_id: int) -> list[str]:
    values: list[str] = []
    for record in font["name"].names:
        if record.nameID != name_id:
            continue
        try:
            value = record.toUnicode()
        except UnicodeDecodeError:
            continue
        if value and value not in values:
            values.append(value)
    return sorted(values)


def inspect_exact_font(path: Path) -> dict[str, Any]:
    path = regular_file(path, MAX_FONT_BYTES)
    require_equal(sha256_file(path), EXPECTED_FONT_SHA256, "exact font SHA-256")
    try:
        font = TTFont(path, lazy=False, fontNumber=0)
    except Exception as error:  # fontTools raises several table-specific errors.
        raise QualificationError("exact source is not a readable single SFNT") from error
    try:
        family_names = _font_names(font, 1)
        full_names = _font_names(font, 4)
        postscript_names = _font_names(font, 6)
        if TARGET_FACE not in family_names or CANONICAL_FACE not in family_names:
            raise QualificationError("exact source lacks the localized/English family pair")
        if "hmtx" not in font or "cmap" not in font:
            raise QualificationError("exact source lacks cmap or hmtx")
        return {
            "sha256": EXPECTED_FONT_SHA256,
            "sfntCount": 1,
            "familyNames": family_names,
            "fullNames": full_names,
            "postScriptNames": postscript_names,
            "unitsPerEm": font["head"].unitsPerEm,
            "glyphs": font["maxp"].numGlyphs,
            "horizontalMetrics": font["hhea"].numberOfHMetrics,
            "cmapCodepoints": len(font.getBestCmap() or {}),
            "os2FsType": font["OS/2"].fsType,
            "embeddingDisposition": "restricted-license-embedding"
            if font["OS/2"].fsType & 0x0002
            else "not-restricted-by-fstype-bit-1",
        }
    finally:
        font.close()


def validate_against_w4(cohort: dict[str, Any], ranking_entry: dict[str, Any]) -> None:
    require_equal(cohort["riskCharacters"], ranking_entry["riskCharacters"], "W4 risk characters")
    require_equal(
        cohort["categoryRiskCharacters"],
        ranking_entry["categoryRiskCharacters"],
        "W4 category risk characters",
    )
    require_equal(
        cohort["compressedFixedContextRiskCharacters"],
        ranking_entry["compressedFixedContextRiskCharacters"],
        "W4 compressed fixed-context characters",
    )
    require_equal(cohort["formatCharacters"], ranking_entry["formatCharacters"], "W4 format characters")
    require_equal(cohort["freshRiskCharacters"], 0, "rank-1 fresh risk characters")
    require_equal(ranking_entry["freshCandidateRiskMass"], 0, "W4 fresh risk mass")


def build_outputs(
    *,
    manifest: dict[str, Any],
    coverage: dict[str, Any],
    ranking: dict[str, Any],
    ladder: dict[str, Any],
    registry: dict[str, Any],
    projection: dict[str, Any],
    source_attestation: dict[str, Any],
    font_identity: dict[str, Any],
    selected: list[dict[str, Any]],
    journal_sha256: str,
    paths: dict[str, Path],
) -> tuple[dict[str, Any], dict[str, Any]]:
    documents, checkpoint = validate_manifest_and_coverage(manifest, coverage)
    del documents
    ranking_entry = target_ranking_entry(ranking)
    validate_ladder(ladder)
    registry_rules = target_registry_rules(registry)
    anchor = metric_anchor(projection)
    validate_source_attestation(source_attestation)
    require_equal(font_identity.get("sha256"), EXPECTED_FONT_SHA256, "font identity SHA-256")
    require_equal(font_identity.get("unitsPerEm"), 1000, "exact unitsPerEm")
    require_equal(font_identity.get("os2FsType"), 2, "exact OS/2.fsType")

    cohort = combine_document_summaries(selected)
    aggregate_usage_rows = [
        row
        for row in coverage.get("decisionUsage", [])
        if isinstance(row, dict) and row.get("font") == TARGET_FACE
    ]
    if not aggregate_usage_rows:
        raise QualificationError("W3 aggregate rank-1 usage rows are missing")
    cohort["aggregateUsageRows"] = len(aggregate_usage_rows)
    validate_against_w4(cohort, ranking_entry)

    private_documents = sorted(selected, key=lambda entry: entry["manifestIndex"])
    private_body = {
        "schemaVersion": 1,
        "kind": "font-rank1-private-cohort",
        "issue": 4967,
        "target": {"documentFace": TARGET_FACE, "queueRank": QUEUE_RANK},
        "inputs": {
            name: {
                "path": str(path.resolve(strict=True)),
                "sha256": journal_sha256 if name == "w3Journal" else sha256_file(path),
            }
            for name, path in paths.items()
        },
        "exactFontIdentity": font_identity,
        "cohort": cohort,
        "documents": private_documents,
        "privacy": {
            "localOnly": True,
            "ownerModeRequired": "0600",
            "publicProjectionContainsDocumentIdentity": False,
        },
    }
    private_body["cohortSha256"] = sha256_bytes(canonical_json_bytes(private_documents))

    tracked_inputs = {
        "w4Ranking": paths["w4Ranking"],
        "w5Rank1Ladder": paths["w5Rank1Ladder"],
        "fontRuleRegistryV2": paths["fontRuleRegistryV2"],
        "fontRuleProjectionBaseline": paths["fontRuleProjectionBaseline"],
        "sourceProvenance": paths["sourceProvenance"],
    }
    public_body = {
        "schemaVersion": 1,
        "kind": "font-rank1-qualification-baseline",
        "issue": 4967,
        "stage": "W8-R1-Q0",
        "target": {
            "documentFace": TARGET_FACE,
            "canonicalFaceCandidate": CANONICAL_FACE,
            "queueRank": QUEUE_RANK,
            "baseRank": ranking_entry["baseRank"],
            "empiricalRiskBand": ranking_entry["empiricalRiskBand"],
        },
        "inputs": {
            name: {"artifact": relative_repo_path(path), "sha256": sha256_file(path)}
            for name, path in tracked_inputs.items()
        },
        "w3LocalAttestation": {
            "sourceHead": checkpoint["identity"]["sourceHead"],
            "documentCount": checkpoint["identity"]["documentCount"],
            "checkpointPolicySha256": checkpoint["identity"]["checkpointPolicySha256"],
            "checkpointChainSha256": checkpoint["chain"]["value"],
            "aggregateSha256": coverage["aggregateHash"]["value"],
            "journalSha256": journal_sha256,
        },
        "cohort": cohort,
        "w5Oracle": {
            "disposition": "complete-acceptance-ladder",
            "physicalStates": ["exact-only", "subst-only", "none-related"],
            "exactProfileSha256": ladder["runs"][0]["outputProfileSha256"],
            "officialSuccessor": "not-provided",
        },
        "exactSource": font_identity,
        "sourceProvenance": source_attestation,
        "currentMetricAnchor": anchor,
        "currentRegistry": {
            "rawSha256": sha256_file(paths["fontRuleRegistryV2"]),
            "rulesSha256": registry["rulesSha256"],
            "targetRuleIds": [rule["ruleId"] for rule in registry_rules],
            "decisionPlanes": sorted({rule["decisionPlane"] for rule in registry_rules}),
        },
        "executionPolicy": {
            "fullCorpusRerun": False,
            "hyperVOracleRerun": False,
            "productMutation": False,
            "nextStage": "runtime-name-and-metric-boundary",
        },
        "privacy": {
            "absolutePathIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateDocumentHashIncluded": False,
            "privateDocumentNameIncluded": False,
        },
        "gates": {
            "w4CountsReconciled": True,
            "w5LadderHashBound": True,
            "localizedAndEnglishNamesShareExactSfnt": True,
            "existingMetricAnchorFound": True,
            "portableSupplyQualified": False,
            "privateCohortSeparated": True,
        },
    }
    reject_absolute_paths(public_body)
    public_body["canonicalSha256"] = sha256_bytes(canonical_json_bytes(public_body))
    return private_body, public_body


def project(args: argparse.Namespace) -> tuple[dict[str, Any], dict[str, Any]]:
    paths = {
        "w3Manifest": regular_file(args.manifest, MAX_MANIFEST_BYTES),
        "w3Coverage": regular_file(args.coverage, MAX_COVERAGE_BYTES),
        "w3Journal": regular_file(args.journal, MAX_JOURNAL_BYTES),
        "w4Ranking": regular_file(args.ranking, MAX_MANIFEST_BYTES),
        "w5Rank1Ladder": regular_file(args.ladder, MAX_MANIFEST_BYTES),
        "fontRuleRegistryV2": regular_file(args.registry, MAX_MANIFEST_BYTES),
        "fontRuleProjectionBaseline": regular_file(args.projection, MAX_COVERAGE_BYTES),
        "sourceProvenance": regular_file(args.source_provenance, MAX_MANIFEST_BYTES),
        "exactFont": regular_file(args.exact_font, MAX_FONT_BYTES),
    }
    manifest = read_json(paths["w3Manifest"], MAX_MANIFEST_BYTES)
    coverage = read_json(paths["w3Coverage"], MAX_COVERAGE_BYTES)
    ranking = read_json(paths["w4Ranking"], MAX_MANIFEST_BYTES)
    ladder = read_json(paths["w5Rank1Ladder"], MAX_MANIFEST_BYTES)
    registry = read_json(paths["fontRuleRegistryV2"], MAX_MANIFEST_BYTES)
    projection = read_json(paths["fontRuleProjectionBaseline"], MAX_COVERAGE_BYTES)
    source_attestation = read_json(paths["sourceProvenance"], MAX_MANIFEST_BYTES)
    documents, _ = validate_manifest_and_coverage(manifest, coverage)
    selected, journal_sha256 = scan_journal(
        paths["w3Journal"], documents, target_face=TARGET_FACE
    )
    font_identity = inspect_exact_font(paths["exactFont"])
    return build_outputs(
        manifest=manifest,
        coverage=coverage,
        ranking=ranking,
        ladder=ladder,
        registry=registry,
        projection=projection,
        source_attestation=source_attestation,
        font_identity=font_identity,
        selected=selected,
        journal_sha256=journal_sha256,
        paths=paths,
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--coverage", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--ranking", type=Path, required=True)
    parser.add_argument("--ladder", type=Path, required=True)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--projection", type=Path, required=True)
    parser.add_argument("--source-provenance", type=Path, required=True)
    parser.add_argument("--exact-font", type=Path, required=True)
    parser.add_argument("--private-output", type=Path, required=True)
    parser.add_argument("--public-output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    private, public = project(args)
    safe_write_json(args.private_output, private, 0o600)
    safe_write_json(args.public_output, public, 0o644)
    print(
        json.dumps(
            {
                "documents": public["cohort"]["documents"],
                "riskCharacters": public["cohort"]["riskCharacters"],
                "publicCanonicalSha256": public["canonicalSha256"],
                "hyperVOracleRerun": False,
                "portableSupplyQualified": False,
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except QualificationError as error:
        raise SystemExit(str(error)) from error
