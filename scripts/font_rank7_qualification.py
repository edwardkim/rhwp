#!/usr/bin/env python3
"""Project existing W3/W4/W5/W7 evidence into the W8 rank-7 Q0 baseline."""

from __future__ import annotations

import argparse
import json
from collections import Counter
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


TARGET_FACE = "KoPubWorld돋움체 Light"
ENGLISH_FACE = "KoPubWorldDotum Light"
POSTSCRIPT_NAME = "KoPubWorldDotumLight"
QUEUE_RANK = 7
EXPECTED_FONT_SHA256 = "069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f"
EXPECTED_FIXTURE_SHA256 = "1cc8062c6fd0da39cfddc4182115226717516d4250e693b43596293374236f9e"
EXPECTED_EXACT_PROFILE_SHA256 = (
    "e1952e4136b6ad756c9b51f424e56db032389f4848960f7e75342a322794beca"
)
EXPECTED_WEBFONT_URL = (
    "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/"
    "KoPubWorld-Dotum-Light.woff2"
)
EXPECTED_CANVASKIT_URL = (
    "https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/"
    "KoPubWorld-Dotum-Light.otf"
)
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
        raise QualificationError("W4 rank-7 target is missing or duplicated")
    entry = matches[0]
    require_equal(entry.get("actionRank"), QUEUE_RANK, "W4 action rank")
    require_equal(entry.get("baseRank"), QUEUE_RANK, "W4 base rank")
    require_equal(entry.get("empiricalRiskBand"), "B", "W4 risk band")
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
    require_equal(ladder.get("fixtureSha256"), EXPECTED_FIXTURE_SHA256, "W5 fixture")
    runs = ladder.get("runs", [])
    require_equal(
        [run.get("physicalState") for run in runs],
        ["exact-only", "subst-only", "none-related"],
        "W5 physical-state ladder",
    )
    require_equal(
        runs[0].get("outputProfileSha256"),
        EXPECTED_EXACT_PROFILE_SHA256,
        "W5 exact profile SHA-256",
    )
    exact_matches = [
        font
        for font in runs[0].get("managedFonts", [])
        if font.get("face") == TARGET_FACE and font.get("present") is True
    ]
    if len(exact_matches) != 1:
        raise QualificationError("W5 exact source is missing or duplicated")
    require_equal(
        exact_matches[0].get("sha256"), EXPECTED_FONT_SHA256, "W5 exact font SHA-256"
    )
    require_equal(
        ladder.get("dispositions"),
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


def source_readiness(readiness: dict[str, Any]) -> dict[str, Any]:
    require_equal(readiness.get("schemaVersion"), 1, "readiness schema")
    require_equal(
        readiness.get("kind"), "font-oracle-readiness-ledger", "readiness kind"
    )
    require_equal(readiness.get("issue"), 4963, "readiness issue")
    matches = [
        candidate
        for candidate in readiness.get("candidates", [])
        if candidate.get("documentFace") == TARGET_FACE
    ]
    if len(matches) != 1:
        raise QualificationError("rank-7 source readiness is missing or duplicated")
    candidate = matches[0]
    require_equal(candidate.get("queueRank"), QUEUE_RANK, "readiness queue rank")
    require_equal(candidate.get("sourceReadiness"), "ready-local-sfnt", "source readiness")
    official = candidate.get("officialSupply", {})
    require_equal(official.get("fontSha256"), EXPECTED_FONT_SHA256, "official font SHA-256")
    require_equal(official.get("os2FsType"), 8, "official OS/2.fsType")
    require_equal(
        official.get("licenseDecision"),
        "local-analysis-only-no-font-redistribution",
        "official license decision",
    )
    sfnt = candidate.get("sfnt", {})
    require_equal(sfnt.get("sha256"), EXPECTED_FONT_SHA256, "readiness SFNT SHA-256")
    require_equal(sfnt.get("unitsPerEm"), 1000, "readiness unitsPerEm")
    require_equal(sfnt.get("os2FsType"), 8, "readiness SFNT OS/2.fsType")
    privacy = readiness.get("privacy", {})
    require_equal(privacy.get("absolutePathsPublished"), False, "readiness privacy path")
    require_equal(privacy.get("fontBytesTracked"), False, "readiness privacy bytes")
    require_equal(
        privacy.get("privateDocumentIdentityPublished"), False, "readiness privacy identity"
    )
    return candidate


def target_registry_rules(registry: dict[str, Any]) -> list[dict[str, Any]]:
    require_equal(registry.get("schemaVersion"), "2.0", "registry schema")
    require_equal(
        registry.get("kind"), "canonical-font-rule-lifecycle-registry", "registry kind"
    )
    rules = [
        rule
        for rule in registry.get("rules", [])
        if rule.get("status") == "active" and rule.get("sourceFace") == TARGET_FACE
    ]
    if len(rules) != 2:
        raise QualificationError("rank-7 active registry rule inventory drifted")
    require_equal(
        {rule.get("decisionPlane") for rule in rules}, {"supply"}, "rank-7 decision planes"
    )
    projections = {
        projection.get("id")
        for rule in rules
        for projection in rule.get("projections", [])
    }
    require_equal(
        projections,
        {"canvas2d-webfont", "canvaskit-sfnt"},
        "rank-7 supply projections",
    )
    urls = {
        rule.get("supply", {}).get("sourceUrl")
        or rule.get("supply", {}).get("online", {}).get("sources", [{}])[0].get("url")
        for rule in rules
    }
    require_equal(
        urls, {EXPECTED_WEBFONT_URL, EXPECTED_CANVASKIT_URL}, "rank-7 supply URLs"
    )
    return sorted(rules, key=lambda rule: rule["ruleId"])


def validate_projection_baseline(projection: dict[str, Any]) -> dict[str, Any]:
    require_equal(projection.get("schemaVersion"), "1.0", "projection schema")
    require_equal(
        projection.get("kind"),
        "font-rule-projection-pre-migration-baseline",
        "projection kind",
    )
    require_equal(projection.get("issue"), 4966, "projection issue")
    projections = projection.get("projections", {})

    def matches(name: str) -> list[dict[str, Any]]:
        return [
            rule
            for rule in projections.get(name, {}).get("rules", [])
            if rule.get("sourceFace") == TARGET_FACE
        ]

    require_equal(matches("rustLayoutName"), [], "rank-7 layout-name projection")
    require_equal(matches("rustLayoutMetric"), [], "rank-7 layout-metric projection")
    webfont = matches("webfontSupply")
    canvaskit = matches("canvasKitSfnt")
    if len(webfont) != 1 or len(canvaskit) != 1:
        raise QualificationError("rank-7 supply projection inventory drifted")
    require_equal(webfont[0].get("ruleId"), "rule.studio-supply.b4a81472cc52c505ee6d.canvas2d", "webfont rule")
    require_equal(canvaskit[0].get("ruleId"), "rule.studio-supply.b4a81472cc52c505ee6d.canvaskit", "CanvasKit rule")
    return {
        "rustLayoutNameRules": 0,
        "rustLayoutMetricRules": 0,
        "webfontSupplyRules": 1,
        "canvasKitSfntRules": 1,
        "projectionBundleSha256": projection.get("hashes", {}).get(
            "projectionBundleSha256"
        ),
    }


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
    except Exception as error:
        raise QualificationError("exact source is not a readable single SFNT") from error
    try:
        family_names = _font_names(font, 1)
        full_names = _font_names(font, 4)
        postscript_names = _font_names(font, 6)
        if TARGET_FACE not in family_names or ENGLISH_FACE not in family_names:
            raise QualificationError("exact source lacks the localized/English family pair")
        if POSTSCRIPT_NAME not in postscript_names:
            raise QualificationError("exact source lacks the expected PostScript name")
        if "hmtx" not in font or "cmap" not in font:
            raise QualificationError("exact source lacks cmap or hmtx")
        return {
            "sha256": EXPECTED_FONT_SHA256,
            "sfntCount": 1,
            "faceIndex": 0,
            "familyNames": family_names,
            "fullNames": full_names,
            "postScriptNames": postscript_names,
            "unitsPerEm": font["head"].unitsPerEm,
            "glyphs": font["maxp"].numGlyphs,
            "horizontalMetrics": font["hhea"].numberOfHMetrics,
            "cmapCodepoints": len(font.getBestCmap() or {}),
            "os2FsType": font["OS/2"].fsType,
            "embeddingDisposition": "editable-embedding",
        }
    finally:
        font.close()


def validate_against_w4(cohort: dict[str, Any], ranking: dict[str, Any]) -> None:
    require_equal(cohort["riskCharacters"], ranking["riskCharacters"], "W4 risk characters")
    require_equal(
        cohort["categoryRiskCharacters"],
        ranking["categoryRiskCharacters"],
        "W4 category risk characters",
    )
    require_equal(
        cohort["compressedFixedContextRiskCharacters"],
        ranking["compressedFixedContextRiskCharacters"],
        "W4 compressed fixed-context characters",
    )
    require_equal(cohort["formatCharacters"], ranking["formatCharacters"], "W4 formats")
    require_equal(cohort["freshRiskCharacters"], 0, "rank-7 fresh risk characters")
    require_equal(ranking["freshCandidateRiskMass"], 0, "W4 fresh risk mass")


def summarize_style_domain(selected: list[dict[str, Any]]) -> dict[str, Any]:
    axes: Counter[tuple[int, int, bool, bool]] = Counter()
    rows: Counter[tuple[int, int, bool, bool]] = Counter()
    for document in selected:
        for row in document["targetRows"]:
            if not isinstance(row.get("bold"), bool) or not isinstance(
                row.get("italic"), bool
            ):
                raise QualificationError("rank-7 style flags must be boolean")
            key = (row["ratio"], row["spacing"], row["bold"], row["italic"])
            axes[key] += row["charCount"]
            rows[key] += 1
    ordered = sorted(axes, key=lambda key: (-axes[key], key))
    return {
        "axes": [
            {
                "ratio": key[0],
                "spacing": key[1],
                "bold": key[2],
                "italic": key[3],
                "characters": axes[key],
                "rows": rows[key],
            }
            for key in ordered
        ],
        "boldCharacters": sum(
            characters for (_, _, bold, _), characters in axes.items() if bold
        ),
        "italicCharacters": sum(
            characters for (_, _, _, italic), characters in axes.items() if italic
        ),
    }


def build_outputs(
    *,
    manifest: dict[str, Any],
    coverage: dict[str, Any],
    ranking: dict[str, Any],
    ladder: dict[str, Any],
    readiness: dict[str, Any],
    registry: dict[str, Any],
    projection: dict[str, Any],
    font_identity: dict[str, Any],
    selected: list[dict[str, Any]],
    journal_sha256: str,
    paths: dict[str, Path],
) -> tuple[dict[str, Any], dict[str, Any]]:
    _, checkpoint = validate_manifest_and_coverage(manifest, coverage)
    ranking_entry = target_ranking_entry(ranking)
    validate_ladder(ladder)
    readiness_entry = source_readiness(readiness)
    registry_rules = target_registry_rules(registry)
    projection_inventory = validate_projection_baseline(projection)
    require_equal(font_identity.get("sha256"), EXPECTED_FONT_SHA256, "font identity")
    require_equal(font_identity.get("unitsPerEm"), 1000, "exact unitsPerEm")
    require_equal(font_identity.get("os2FsType"), 8, "exact OS/2.fsType")

    cohort = combine_document_summaries(selected)
    aggregate_usage = [
        row
        for row in coverage.get("decisionUsage", [])
        if isinstance(row, dict) and row.get("font") == TARGET_FACE
    ]
    if not aggregate_usage:
        raise QualificationError("W3 aggregate rank-7 usage rows are missing")
    cohort["aggregateUsageRows"] = len(aggregate_usage)
    cohort["styleDomain"] = summarize_style_domain(selected)
    validate_against_w4(cohort, ranking_entry)

    private_documents = sorted(selected, key=lambda entry: entry["manifestIndex"])
    private_body = {
        "schemaVersion": 1,
        "kind": "font-rank7-private-cohort",
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
        "w5Rank7Ladder": paths["w5Rank7Ladder"],
        "w5SourceReadiness": paths["w5SourceReadiness"],
        "fontRuleRegistryV2": paths["fontRuleRegistryV2"],
        "fontRuleProjectionBaseline": paths["fontRuleProjectionBaseline"],
    }
    official = readiness_entry["officialSupply"]
    public_body = {
        "schemaVersion": 1,
        "kind": "font-rank7-qualification-baseline",
        "issue": 4967,
        "stage": "W8-R7-Q0",
        "target": {
            "documentFace": TARGET_FACE,
            "englishFace": ENGLISH_FACE,
            "postScriptName": POSTSCRIPT_NAME,
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
            "fixtureSha256": ladder["fixtureSha256"],
            "exactProfileSha256": ladder["runs"][0]["outputProfileSha256"],
            "officialSuccessor": "not-provided",
            "substitutionIsObservedFallback": False,
        },
        "exactSource": font_identity,
        "officialSource": {
            "record": official["officialRecord"],
            "uci": official["uci"],
            "downloadSha256": official["downloadSha256"],
            "fontSha256": official["fontSha256"],
            "licenseDecision": official["licenseDecision"],
        },
        "currentRegistry": {
            "rawSha256": sha256_file(paths["fontRuleRegistryV2"]),
            "rulesSha256": registry["rulesSha256"],
            "targetRuleIds": [rule["ruleId"] for rule in registry_rules],
            "decisionPlanes": ["supply"],
            "projectionIds": ["canvas2d-webfont", "canvaskit-sfnt"],
            "supplyUrls": [EXPECTED_WEBFONT_URL, EXPECTED_CANVASKIT_URL],
        },
        "projectionInventory": projection_inventory,
        "executionPolicy": {
            "fullCorpusRerun": False,
            "hyperVOracleRerun": False,
            "productMutation": False,
            "nextStage": "public-fixture-and-current-runtime-boundary",
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
            "registrySupplyOnly": True,
            "layoutProjectionAbsent": True,
            "freshLaneIsNonTargetGuard": cohort["freshRiskCharacters"] == 0,
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
        "w5Rank7Ladder": regular_file(args.ladder, MAX_MANIFEST_BYTES),
        "w5SourceReadiness": regular_file(args.readiness, MAX_MANIFEST_BYTES),
        "fontRuleRegistryV2": regular_file(args.registry, MAX_MANIFEST_BYTES),
        "fontRuleProjectionBaseline": regular_file(args.projection, MAX_COVERAGE_BYTES),
        "exactFont": regular_file(args.exact_font, MAX_FONT_BYTES),
    }
    manifest = read_json(paths["w3Manifest"], MAX_MANIFEST_BYTES)
    coverage = read_json(paths["w3Coverage"], MAX_COVERAGE_BYTES)
    ranking = read_json(paths["w4Ranking"], MAX_MANIFEST_BYTES)
    ladder = read_json(paths["w5Rank7Ladder"], MAX_MANIFEST_BYTES)
    readiness = read_json(paths["w5SourceReadiness"], MAX_MANIFEST_BYTES)
    registry = read_json(paths["fontRuleRegistryV2"], MAX_MANIFEST_BYTES)
    projection = read_json(paths["fontRuleProjectionBaseline"], MAX_COVERAGE_BYTES)
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
        readiness=readiness,
        registry=registry,
        projection=projection,
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
    parser.add_argument("--readiness", type=Path, required=True)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--projection", type=Path, required=True)
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
                "productMutation": False,
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
