#!/usr/bin/env python3
"""Project the existing W3 journal into the W8 rank-8 qualification baseline."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
from collections import Counter
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
TARGET_FACE = "KoPubWorld바탕체 Light"
RISK_CATEGORIES = {"face-miss", "char-miss", "heuristic"}
FIXED_CONTEXTS = {
    "table-cell",
    "text-box",
    "caption",
    "header",
    "footer",
    "master-page",
}
ABSOLUTE_PATH = re.compile(r"^(?:/|[A-Za-z]:[\\/]|\\\\)")
MAX_MANIFEST_BYTES = 16 * 1024 * 1024
MAX_COVERAGE_BYTES = 256 * 1024 * 1024
MAX_JOURNAL_BYTES = 1024 * 1024 * 1024
MAX_JOURNAL_LINE_BYTES = 16 * 1024 * 1024
MAX_DOCUMENTS = 100_000
MAX_ROWS_PER_DOCUMENT = 1_000_000


class QualificationError(RuntimeError):
    """A fail-closed W8 qualification input or projection error."""


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def pretty_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    ).encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def regular_file(path: Path, maximum_bytes: int) -> Path:
    if path.is_symlink():
        raise QualificationError(f"symlink input is forbidden: {path}")
    try:
        metadata = path.stat()
    except FileNotFoundError as error:
        raise QualificationError(f"input is missing: {path}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise QualificationError(f"input is not a regular file: {path}")
    if metadata.st_size <= 0 or metadata.st_size > maximum_bytes:
        raise QualificationError(
            f"input byte limit exceeded: {path} ({metadata.st_size} > {maximum_bytes})"
        )
    return path.resolve(strict=True)


def read_json(path: Path, maximum_bytes: int) -> Any:
    path = regular_file(path, maximum_bytes)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise QualificationError(f"invalid JSON input: {path}") from error


def checked_nonnegative(value: Any, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise QualificationError(f"{label} must be a non-negative integer")
    return value


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise QualificationError(f"{label} mismatch: {actual!r} != {expected!r}")


def reject_absolute_paths(value: Any, label: str = "public") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            reject_absolute_paths(child, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            reject_absolute_paths(child, f"{label}[{index}]")
    elif isinstance(value, str) and ABSOLUTE_PATH.match(value):
        raise QualificationError(f"{label} exposes an absolute path")


def relative_repo_path(path: Path) -> str:
    try:
        return path.resolve(strict=True).relative_to(ROOT).as_posix()
    except ValueError as error:
        raise QualificationError(f"tracked evidence is outside the repository: {path}") from error


def safe_write_json(path: Path, value: Any, mode: int) -> None:
    if path.exists() and path.is_symlink():
        raise QualificationError(f"refusing to overwrite a symlink: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.parent.is_symlink() or not path.parent.is_dir():
        raise QualificationError(f"output parent must be a real directory: {path.parent}")
    path.write_bytes(pretty_json_bytes(value))
    os.chmod(path, mode)


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
        raise QualificationError("W4 rank-8 target is missing or duplicated")
    entry = matches[0]
    require_equal(entry.get("actionRank"), 8, "W4 action rank")
    require_equal(entry.get("empiricalRiskBand"), "B", "W4 risk band")
    require_equal(entry.get("w5Queue"), True, "W4 W5 queue membership")
    return entry


def validate_ladder(ladder: dict[str, Any]) -> None:
    require_equal(ladder.get("schemaVersion"), 1, "W5 ladder schema")
    require_equal(
        ladder.get("kind"), "font-oracle-stage5-ladder-evidence", "W5 ladder kind"
    )
    require_equal(ladder.get("issue"), 4963, "W5 ladder issue")
    require_equal(
        ladder.get("target"),
        {"queueRank": 8, "documentFace": TARGET_FACE},
        "W5 ladder target",
    )
    require_equal(
        [run.get("physicalState") for run in ladder.get("runs", [])],
        ["exact-only", "subst-only", "none-related"],
        "W5 physical-state ladder",
    )
    if len(ladder.get("profiles", [])) != 4:
        raise QualificationError("W5 rank-8 profile inventory is incomplete")
    privacy = ladder.get("privacy", {})
    for field in (
        "absolutePathIncluded",
        "fontBytesIncluded",
        "hostNameIncluded",
        "privateCorpusAccessed",
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
        if rule.get("status") == "active" and rule.get("sourceFace") == TARGET_FACE
    ]
    if len(rules) != 2:
        raise QualificationError("rank-8 active registry rule inventory drifted")
    require_equal(
        {rule.get("decisionPlane") for rule in rules}, {"supply"}, "rank-8 decision planes"
    )
    projection_ids = {
        projection.get("id")
        for rule in rules
        for projection in rule.get("projections", [])
    }
    require_equal(
        projection_ids,
        {"canvas2d-webfont", "canvaskit-sfnt"},
        "rank-8 supply projections",
    )
    return sorted(rules, key=lambda rule: rule["ruleId"])


def validate_manifest_and_coverage(
    manifest: dict[str, Any], coverage: dict[str, Any]
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    require_equal(manifest.get("schemaVersion"), 1, "W3 manifest schema")
    require_equal(
        manifest.get("kind"),
        "font-metric-coverage-private-corpus-manifest",
        "W3 manifest kind",
    )
    require_equal(manifest.get("localOnly"), True, "W3 manifest privacy")
    documents = manifest.get("documents")
    if not isinstance(documents, list) or not 1 <= len(documents) <= MAX_DOCUMENTS:
        raise QualificationError("W3 manifest document inventory is invalid")
    require_equal(manifest.get("corpus", {}).get("documents"), len(documents), "W3 corpus size")

    require_equal(coverage.get("schemaVersion"), 1, "W3 aggregate schema")
    require_equal(
        coverage.get("kind"), "font-metric-coverage-aggregate", "W3 aggregate kind"
    )
    require_equal(coverage.get("status"), "complete", "W3 aggregate status")
    checkpoint = coverage.get("checkpoint", {})
    require_equal(checkpoint.get("entries"), len(documents), "W3 checkpoint entries")
    require_equal(
        checkpoint.get("identity", {}).get("sourceHead"),
        manifest.get("sourceHead"),
        "W3 source head",
    )
    require_equal(
        checkpoint.get("identity", {}).get("documentCount"),
        len(documents),
        "W3 checkpoint document count",
    )
    return documents, checkpoint


def validate_target_row(
    row: Any, label: str, target_face: str = TARGET_FACE
) -> dict[str, Any]:
    if not isinstance(row, dict) or row.get("font") != target_face:
        raise QualificationError(f"{label} is not the requested target usage row")
    checked_nonnegative(row.get("charCount"), f"{label}.charCount")
    checked_nonnegative(row.get("documentCount"), f"{label}.documentCount")
    if row.get("documentCount") != 1:
        raise QualificationError(f"{label}.documentCount must be one in a journal record")
    if not isinstance(row.get("ratio"), int) or not isinstance(row.get("spacing"), int):
        raise QualificationError(f"{label} has an invalid ratio or spacing")
    if not isinstance(row.get("storedLineSeg"), bool):
        raise QualificationError(f"{label}.storedLineSeg must be boolean")
    if row.get("coverageCategory") not in RISK_CATEGORIES | {None}:
        raise QualificationError(f"{label}.coverageCategory is unknown")
    if not isinstance(row.get("context"), str) or not row["context"]:
        raise QualificationError(f"{label}.context is invalid")
    return row


def summarize_rows(rows: list[dict[str, Any]]) -> dict[str, Any]:
    if not rows or len(rows) > MAX_ROWS_PER_DOCUMENT:
        raise QualificationError("target row inventory is empty or exceeds the bound")
    categories: Counter[str] = Counter()
    contexts: Counter[str] = Counter()
    total_characters = 0
    compressed_characters = 0
    risk_characters = 0
    compressed_risk_characters = 0
    compressed_fixed_risk_characters = 0
    stored_risk_characters = 0
    fresh_risk_characters = 0
    for row in rows:
        characters = row["charCount"]
        total_characters += characters
        contexts[row["context"]] += characters
        compressed = row["ratio"] < 100 or row["spacing"] < 0
        if compressed:
            compressed_characters += characters
        category = row.get("coverageCategory")
        if category in RISK_CATEGORIES:
            categories[category] += characters
            risk_characters += characters
            if compressed:
                compressed_risk_characters += characters
                if row["context"] in FIXED_CONTEXTS:
                    compressed_fixed_risk_characters += characters
            if row["storedLineSeg"]:
                stored_risk_characters += characters
            else:
                fresh_risk_characters += characters
    return {
        "usageRows": len(rows),
        "totalCharacters": total_characters,
        "riskCharacters": risk_characters,
        "categoryRiskCharacters": {
            category: categories.get(category, 0)
            for category in ("face-miss", "char-miss", "heuristic")
        },
        "compressedCharacters": compressed_characters,
        "compressedRiskCharacters": compressed_risk_characters,
        "compressedFixedContextRiskCharacters": compressed_fixed_risk_characters,
        "storedRiskCharacters": stored_risk_characters,
        "freshRiskCharacters": fresh_risk_characters,
        "contextCharacters": dict(sorted(contexts.items())),
    }


def scan_journal(
    journal_path: Path,
    documents: list[dict[str, Any]],
    target_face: str = TARGET_FACE,
) -> tuple[list[dict[str, Any]], str]:
    journal_path = regular_file(journal_path, MAX_JOURNAL_BYTES)
    digest = hashlib.sha256()
    seen_indexes: set[int] = set()
    selected: list[dict[str, Any]] = []
    with journal_path.open("rb") as stream:
        for line_number, raw_line in enumerate(stream, start=1):
            digest.update(raw_line)
            if len(raw_line) > MAX_JOURNAL_LINE_BYTES:
                raise QualificationError(f"journal line {line_number} exceeds the byte bound")
            try:
                record = json.loads(raw_line)
            except (UnicodeDecodeError, json.JSONDecodeError) as error:
                raise QualificationError(f"journal line {line_number} is invalid JSON") from error
            index = record.get("index")
            if not isinstance(index, int) or isinstance(index, bool) or not 0 <= index < len(documents):
                raise QualificationError(f"journal line {line_number} has an invalid index")
            if index in seen_indexes:
                raise QualificationError(f"journal index {index} is duplicated")
            seen_indexes.add(index)
            require_equal(record.get("schemaVersion"), 1, f"journal[{index}] schema")
            require_equal(
                record.get("kind"),
                "font-metric-coverage-checkpoint-record",
                f"journal[{index}] kind",
            )
            if record.get("status") != "complete":
                continue
            usage = record.get("aggregate", {}).get("decisionUsage", [])
            if not isinstance(usage, list) or len(usage) > MAX_ROWS_PER_DOCUMENT:
                raise QualificationError(f"journal[{index}] decisionUsage is invalid")
            rows = [
                validate_target_row(
                    row,
                    f"journal[{index}].decisionUsage[{row_index}]",
                    target_face,
                )
                for row_index, row in enumerate(usage)
                if isinstance(row, dict) and row.get("font") == target_face
            ]
            if not rows:
                continue
            document = documents[index]
            require_equal(record.get("format"), document.get("format"), f"journal[{index}] format")
            source = document.get("source")
            blake3 = document.get("blake3")
            if not isinstance(source, str) or not ABSOLUTE_PATH.match(source):
                raise QualificationError(f"manifest document {index} lacks a private absolute source")
            if not isinstance(blake3, str) or not re.fullmatch(r"[0-9a-f]{64}", blake3):
                raise QualificationError(f"manifest document {index} has an invalid BLAKE3")
            selected.append(
                {
                    "manifestIndex": index,
                    "format": record["format"],
                    "source": source,
                    "blake3": blake3,
                    "aggregateHash": record.get("aggregate", {})
                    .get("aggregateHash", {})
                    .get("value"),
                    "summary": summarize_rows(rows),
                    "targetRows": rows,
                }
            )
    require_equal(len(seen_indexes), len(documents), "journal record count")
    return selected, digest.hexdigest()


def combine_document_summaries(selected: list[dict[str, Any]]) -> dict[str, Any]:
    if not selected:
        raise QualificationError("target journal cohort is empty")
    formats: Counter[str] = Counter()
    format_characters: Counter[str] = Counter()
    contexts: Counter[str] = Counter()
    categories: Counter[str] = Counter()
    totals = Counter()
    for document in selected:
        summary = document["summary"]
        formats[document["format"]] += 1
        format_characters[document["format"]] += summary["totalCharacters"]
        contexts.update(summary["contextCharacters"])
        categories.update(summary["categoryRiskCharacters"])
        for field in (
            "usageRows",
            "totalCharacters",
            "riskCharacters",
            "compressedCharacters",
            "compressedRiskCharacters",
            "compressedFixedContextRiskCharacters",
            "storedRiskCharacters",
            "freshRiskCharacters",
        ):
            totals[field] += summary[field]
    return {
        "documents": len(selected),
        "documentsByFormat": dict(sorted(formats.items())),
        "documentUsageRows": totals["usageRows"],
        "totalCharacters": totals["totalCharacters"],
        "riskCharacters": totals["riskCharacters"],
        "categoryRiskCharacters": {
            category: categories.get(category, 0)
            for category in ("face-miss", "char-miss", "heuristic")
        },
        "formatCharacters": dict(sorted(format_characters.items())),
        "contextCharacters": dict(sorted(contexts.items())),
        "compressedCharacters": totals["compressedCharacters"],
        "compressedRiskCharacters": totals["compressedRiskCharacters"],
        "compressedFixedContextRiskCharacters": totals[
            "compressedFixedContextRiskCharacters"
        ],
        "storedRiskCharacters": totals["storedRiskCharacters"],
        "freshRiskCharacters": totals["freshRiskCharacters"],
    }


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
    require_equal(
        cohort["formatCharacters"], ranking_entry["formatCharacters"], "W4 format characters"
    )
    require_equal(cohort["freshRiskCharacters"], 0, "rank-8 fresh risk characters")
    require_equal(ranking_entry["freshCandidateRiskMass"], 0, "W4 fresh risk mass")


def build_outputs(
    *,
    manifest: dict[str, Any],
    coverage: dict[str, Any],
    ranking: dict[str, Any],
    ladder: dict[str, Any],
    registry: dict[str, Any],
    selected: list[dict[str, Any]],
    journal_sha256: str,
    paths: dict[str, Path],
) -> tuple[dict[str, Any], dict[str, Any]]:
    documents, checkpoint = validate_manifest_and_coverage(manifest, coverage)
    del documents
    ranking_entry = target_ranking_entry(ranking)
    validate_ladder(ladder)
    registry_rules = target_registry_rules(registry)
    cohort = combine_document_summaries(selected)
    aggregate_usage_rows = [
        row
        for row in coverage.get("decisionUsage", [])
        if isinstance(row, dict) and row.get("font") == TARGET_FACE
    ]
    if not aggregate_usage_rows:
        raise QualificationError("W3 aggregate rank-8 usage rows are missing")
    cohort["aggregateUsageRows"] = len(aggregate_usage_rows)
    validate_against_w4(cohort, ranking_entry)

    private_documents = sorted(selected, key=lambda entry: entry["manifestIndex"])
    private_body = {
        "schemaVersion": 1,
        "kind": "font-rank8-private-cohort",
        "issue": 4967,
        "target": {"documentFace": TARGET_FACE, "queueRank": 8},
        "inputs": {
            name: {
                "path": str(path.resolve(strict=True)),
                "sha256": journal_sha256 if name == "w3Journal" else sha256_file(path),
            }
            for name, path in paths.items()
        },
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
        "w5Rank8Ladder": paths["w5Rank8Ladder"],
        "fontRuleRegistryV2": paths["fontRuleRegistryV2"],
    }
    public_body = {
        "schemaVersion": 1,
        "kind": "font-rank8-qualification-baseline",
        "issue": 4967,
        "stage": "W8-Q0",
        "target": {
            "documentFace": TARGET_FACE,
            "queueRank": 8,
            "baseRank": ranking_entry["baseRank"],
            "empiricalRiskBand": ranking_entry["empiricalRiskBand"],
        },
        "inputs": {
            name: {
                "artifact": relative_repo_path(path),
                "sha256": sha256_file(path),
            }
            for name, path in tracked_inputs.items()
        },
        "w3LocalAttestation": {
            "sourceHead": checkpoint["identity"]["sourceHead"],
            "documentCount": checkpoint["identity"]["documentCount"],
            "checkpointPolicySha256": checkpoint["identity"]["checkpointPolicySha256"],
            "checkpointChainSha256": checkpoint["chain"]["value"],
            "aggregateSha256": coverage["aggregateHash"]["value"],
        },
        "cohort": cohort,
        "w5Oracle": {
            "disposition": "complete-acceptance-ladder",
            "physicalStates": ["exact-only", "subst-only", "none-related"],
            "profileCount": len(ladder["profiles"]),
            "fixtureSha256": ladder["fixture"]["sha256"],
            "exactTypesettingProjectionSha256": ladder["runs"][0][
                "typesettingProjectionSha256"
            ],
            "substitutionAndMissingProjectionEqual": (
                ladder["runs"][1]["typesettingProjectionSha256"]
                == ladder["runs"][2]["typesettingProjectionSha256"]
            ),
        },
        "currentRegistry": {
            "rawSha256": sha256_file(paths["fontRuleRegistryV2"]),
            "rulesSha256": registry["rulesSha256"],
            "targetRuleIds": [rule["ruleId"] for rule in registry_rules],
            "decisionPlanes": sorted({rule["decisionPlane"] for rule in registry_rules}),
            "projectionIds": sorted(
                projection["id"]
                for rule in registry_rules
                for projection in rule["projections"]
            ),
        },
        "executionPolicy": {
            "fullCorpusRerun": False,
            "hyperVOracleRerun": False,
            "nextStage": "public-fixture-and-current-trace",
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
            "registrySupplyOnly": True,
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
        "w5Rank8Ladder": regular_file(args.ladder, MAX_MANIFEST_BYTES),
        "fontRuleRegistryV2": regular_file(args.registry, MAX_MANIFEST_BYTES),
    }
    manifest = read_json(paths["w3Manifest"], MAX_MANIFEST_BYTES)
    coverage = read_json(paths["w3Coverage"], MAX_COVERAGE_BYTES)
    ranking = read_json(paths["w4Ranking"], MAX_MANIFEST_BYTES)
    ladder = read_json(paths["w5Rank8Ladder"], MAX_MANIFEST_BYTES)
    registry = read_json(paths["fontRuleRegistryV2"], MAX_MANIFEST_BYTES)
    documents, _ = validate_manifest_and_coverage(manifest, coverage)
    selected, journal_sha256 = scan_journal(paths["w3Journal"], documents)
    return build_outputs(
        manifest=manifest,
        coverage=coverage,
        ranking=ranking,
        ladder=ladder,
        registry=registry,
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
