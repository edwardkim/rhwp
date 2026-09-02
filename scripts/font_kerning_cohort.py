#!/usr/bin/env python3
"""Project the existing W3 checkpoint into the #4968 W9-Q0 kerning cohort.

The 10k source corpus is never opened.  This projector streams the already
sealed checkpoint journal once, keeps document identity in an owner-only local
ledger, and emits only de-identified aggregate evidence for the repository.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
ABSOLUTE_PATH = re.compile(r"^(?:/|[A-Za-z]:[\\/]|\\\\)")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
MAX_MANIFEST_BYTES = 16 * 1024 * 1024
MAX_COVERAGE_BYTES = 256 * 1024 * 1024
MAX_EVIDENCE_BYTES = 64 * 1024 * 1024
MAX_JOURNAL_BYTES = 1024 * 1024 * 1024
MAX_JOURNAL_LINE_BYTES = 16 * 1024 * 1024
MAX_DOCUMENTS = 100_000
MAX_ROWS_PER_DOCUMENT = 1_000_000
COUNT_FIELDS = ("documentCount", "paragraphCount", "runCount", "charCount")
W8_FACES = (
    (1, "문체부 바탕체"),
    (7, "KoPubWorld돋움체 Light"),
    (8, "KoPubWorld바탕체 Light"),
)


class KerningCohortError(RuntimeError):
    """A fail-closed W9-Q0 input or projection error."""


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


def checked_nonnegative(value: Any, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise KerningCohortError(f"{label} must be a non-negative integer")
    return value


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise KerningCohortError(f"{label} mismatch: {actual!r} != {expected!r}")


def regular_file(path: Path, maximum_bytes: int) -> Path:
    if path.is_symlink():
        raise KerningCohortError(f"symlink input is forbidden: {path}")
    try:
        metadata = path.stat()
    except FileNotFoundError as error:
        raise KerningCohortError(f"input is missing: {path}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise KerningCohortError(f"input is not a regular file: {path}")
    if metadata.st_size <= 0 or metadata.st_size > maximum_bytes:
        raise KerningCohortError(
            f"input byte limit exceeded: {path} ({metadata.st_size} > {maximum_bytes})"
        )
    return path.resolve(strict=True)


def read_json(path: Path, maximum_bytes: int) -> Any:
    path = regular_file(path, maximum_bytes)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise KerningCohortError(f"invalid JSON input: {path}") from error


def relative_repo_path(path: Path) -> str:
    try:
        return path.resolve(strict=True).relative_to(ROOT).as_posix()
    except ValueError as error:
        raise KerningCohortError(f"tracked evidence is outside the repository: {path}") from error


def reject_absolute_paths(value: Any, label: str = "public") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            reject_absolute_paths(child, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            reject_absolute_paths(child, f"{label}[{index}]")
    elif isinstance(value, str) and ABSOLUTE_PATH.match(value):
        raise KerningCohortError(f"{label} exposes an absolute path")


def safe_write_json(path: Path, value: Any, mode: int) -> None:
    if path.exists() and path.is_symlink():
        raise KerningCohortError(f"refusing to overwrite a symlink: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.parent.is_symlink() or not path.parent.is_dir():
        raise KerningCohortError(f"output parent must be a real directory: {path.parent}")
    path.write_bytes(pretty_json_bytes(value))
    os.chmod(path, mode)


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
        raise KerningCohortError("W3 manifest document inventory is invalid")
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


def row_dimensions(row: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in row.items() if key not in COUNT_FIELDS}


def dimension_key(row: dict[str, Any]) -> str:
    return canonical_json_bytes(row_dimensions(row)).decode("utf-8").rstrip("\n")


def validate_kerning_row(row: Any, label: str, *, journal: bool) -> dict[str, Any]:
    if not isinstance(row, dict) or row.get("kerning") is not True:
        raise KerningCohortError(f"{label} is not a kerning usage row")
    for field in COUNT_FIELDS:
        checked_nonnegative(row.get(field), f"{label}.{field}")
    if journal and row.get("documentCount") != 1:
        raise KerningCohortError(f"{label}.documentCount must be one")
    for field in ("font", "context"):
        if not isinstance(row.get(field), str) or not row[field]:
            raise KerningCohortError(f"{label}.{field} is invalid")
    for field in ("ratio", "spacing"):
        if not isinstance(row.get(field), int) or isinstance(row.get(field), bool):
            raise KerningCohortError(f"{label}.{field} is invalid")
    if not isinstance(row.get("storedLineSeg"), bool):
        raise KerningCohortError(f"{label}.storedLineSeg is invalid")
    return row


def coverage_kerning_rows(coverage: dict[str, Any]) -> list[dict[str, Any]]:
    usage = coverage.get("decisionUsage")
    if not isinstance(usage, list):
        raise KerningCohortError("W3 aggregate decisionUsage is invalid")
    rows = [
        validate_kerning_row(row, f"coverage.decisionUsage[{index}]", journal=False)
        for index, row in enumerate(usage)
        if isinstance(row, dict) and row.get("kerning") is True
    ]
    if not rows:
        raise KerningCohortError("W3 aggregate kerning cohort is empty")
    return rows


def _counter_projection(
    rows: Iterable[tuple[int, str, dict[str, Any]]], fields: tuple[str, ...]
) -> list[dict[str, Any]]:
    counts: Counter[tuple[Any, ...]] = Counter()
    documents: defaultdict[tuple[Any, ...], set[int]] = defaultdict(set)
    for index, document_format, row in rows:
        values = tuple(document_format if field == "format" else row.get(field) for field in fields)
        counts[values] += row["charCount"]
        documents[values].add(index)
    result = []
    for values in sorted(counts, key=lambda item: canonical_json_bytes(item)):
        entry = {field: value for field, value in zip(fields, values)}
        entry["documents"] = len(documents[values])
        entry["characters"] = counts[values]
        result.append(entry)
    return result


def scan_journal(
    journal_path: Path, documents: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], str, int, int]:
    journal_path = regular_file(journal_path, MAX_JOURNAL_BYTES)
    digest = hashlib.sha256()
    seen_indexes: set[int] = set()
    selected: list[dict[str, Any]] = []
    complete_records = 0
    target_rows = 0
    with journal_path.open("rb") as stream:
        for line_number, raw_line in enumerate(stream, start=1):
            digest.update(raw_line)
            if len(raw_line) > MAX_JOURNAL_LINE_BYTES:
                raise KerningCohortError(
                    f"journal line {line_number} exceeds the byte bound"
                )
            try:
                record = json.loads(raw_line)
            except (UnicodeDecodeError, json.JSONDecodeError) as error:
                raise KerningCohortError(
                    f"journal line {line_number} is invalid JSON"
                ) from error
            index = record.get("index")
            if (
                not isinstance(index, int)
                or isinstance(index, bool)
                or not 0 <= index < len(documents)
            ):
                raise KerningCohortError(f"journal line {line_number} has an invalid index")
            if index in seen_indexes:
                raise KerningCohortError(f"journal index {index} is duplicated")
            seen_indexes.add(index)
            require_equal(record.get("schemaVersion"), 1, f"journal[{index}] schema")
            require_equal(
                record.get("kind"),
                "font-metric-coverage-checkpoint-record",
                f"journal[{index}] kind",
            )
            if record.get("status") != "complete":
                continue
            complete_records += 1
            usage = record.get("aggregate", {}).get("decisionUsage", [])
            if not isinstance(usage, list) or len(usage) > MAX_ROWS_PER_DOCUMENT:
                raise KerningCohortError(f"journal[{index}] decisionUsage is invalid")
            rows = [
                validate_kerning_row(
                    row,
                    f"journal[{index}].decisionUsage[{row_index}]",
                    journal=True,
                )
                for row_index, row in enumerate(usage)
                if isinstance(row, dict) and row.get("kerning") is True
            ]
            if not rows:
                continue
            target_rows += len(rows)
            document = documents[index]
            require_equal(record.get("format"), document.get("format"), f"journal[{index}] format")
            source = document.get("source")
            blake3 = document.get("blake3")
            if not isinstance(source, str) or not ABSOLUTE_PATH.match(source):
                raise KerningCohortError(
                    f"manifest document {index} lacks a private absolute source"
                )
            if not isinstance(blake3, str) or not HEX_64.fullmatch(blake3):
                raise KerningCohortError(f"manifest document {index} has an invalid BLAKE3")
            selected.append(
                {
                    "manifestIndex": index,
                    "format": record["format"],
                    "source": source,
                    "blake3": blake3,
                    "aggregateHash": record.get("aggregate", {})
                    .get("aggregateHash", {})
                    .get("value"),
                    "targetRows": rows,
                }
            )
    require_equal(len(seen_indexes), len(documents), "journal record count")
    return selected, digest.hexdigest(), complete_records, target_rows


def reconcile_aggregate(
    selected: list[dict[str, Any]], aggregate_rows: list[dict[str, Any]]
) -> None:
    journal: dict[str, dict[str, Any]] = {}
    document_sets: defaultdict[str, set[int]] = defaultdict(set)
    for document in selected:
        index = document["manifestIndex"]
        for row in document["targetRows"]:
            journal_row = {**row, "format": document["format"]}
            key = dimension_key(journal_row)
            current = journal.setdefault(
                key,
                {
                    **row_dimensions(journal_row),
                    "paragraphCount": 0,
                    "runCount": 0,
                    "charCount": 0,
                },
            )
            for field in ("paragraphCount", "runCount", "charCount"):
                current[field] += row[field]
            document_sets[key].add(index)
    for key, value in journal.items():
        value["documentCount"] = len(document_sets[key])

    aggregate = {dimension_key(row): row for row in aggregate_rows}
    if len(aggregate) != len(aggregate_rows):
        raise KerningCohortError("W3 aggregate kerning dimensions are duplicated")
    if set(journal) != set(aggregate):
        journal_only = canonical_json_bytes(sorted(set(journal) - set(aggregate)))
        aggregate_only = canonical_json_bytes(sorted(set(aggregate) - set(journal)))
        raise KerningCohortError(
            "journal/final kerning dimensions mismatch: "
            f"journal={len(journal)} aggregate={len(aggregate)} "
            f"journalOnly={len(set(journal) - set(aggregate))}:"
            f"{sha256_bytes(journal_only)[:16]} "
            f"aggregateOnly={len(set(aggregate) - set(journal))}:"
            f"{sha256_bytes(aggregate_only)[:16]}"
        )
    for key in journal:
        require_equal(
            journal[key],
            aggregate[key],
            f"kerning aggregate row {sha256_bytes(key.encode('utf-8'))[:16]}",
        )


def validate_w8_dispositions(
    rank1: dict[str, Any], rank7: dict[str, Any], rank8: dict[str, Any]
) -> None:
    require_equal(rank1.get("issue"), 4967, "W8 rank1 issue")
    require_equal(
        rank1.get("target", {}).get("documentFace"), W8_FACES[0][1], "W8 rank1 face"
    )
    require_equal(rank1.get("hypothesis", {}).get("status"), "no-change", "W8 rank1 status")
    require_equal(
        rank1.get("hypothesis", {}).get("productMutationAuthorized"),
        False,
        "W8 rank1 mutation",
    )
    for rank, expected_face, value in (
        (7, W8_FACES[1][1], rank7),
        (8, W8_FACES[2][1], rank8),
    ):
        require_equal(value.get("issue"), 4967, f"W8 rank{rank} issue")
        require_equal(value.get("target", {}).get("face"), expected_face, f"W8 rank{rank} face")
        require_equal(value.get("decision", {}).get("status"), "no-change", f"W8 rank{rank} status")
        require_equal(
            value.get("decision", {}).get("productMutationAuthorized"),
            False,
            f"W8 rank{rank} mutation",
        )


def build_outputs(
    *,
    manifest: dict[str, Any],
    coverage: dict[str, Any],
    selected: list[dict[str, Any]],
    journal_sha256: str,
    complete_records: int,
    target_row_count: int,
    tracked_paths: dict[str, Path],
    local_paths: dict[str, Path],
    rank1: dict[str, Any],
    rank7: dict[str, Any],
    rank8: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, Any]]:
    documents, checkpoint = validate_manifest_and_coverage(manifest, coverage)
    del documents
    aggregate_rows = coverage_kerning_rows(coverage)
    reconcile_aggregate(selected, aggregate_rows)
    validate_w8_dispositions(rank1, rank7, rank8)

    flattened = [
        (document["manifestIndex"], document["format"], row)
        for document in selected
        for row in document["targetRows"]
    ]
    document_formats = Counter(document["format"] for document in selected)
    characters = sum(row["charCount"] for _, _, row in flattened)
    overlaps = []
    for rank, face in W8_FACES:
        matching = [(index, fmt, row) for index, fmt, row in flattened if row["font"] == face]
        overlaps.append(
            {
                "queueRank": rank,
                "documentFace": face,
                "documents": len({index for index, _, _ in matching}),
                "characters": sum(row["charCount"] for _, _, row in matching),
                "w8Disposition": "no-change",
                "productMutationAuthorized": False,
            }
        )

    private_documents = sorted(selected, key=lambda value: value["manifestIndex"])
    private = {
        "schemaVersion": 1,
        "kind": "font-kerning-private-cohort",
        "issue": 4968,
        "stage": "W9-Q0",
        "inputs": {
            name: {
                "path": str(path.resolve(strict=True)),
                "sha256": journal_sha256 if name == "w3Journal" else sha256_file(path),
            }
            for name, path in local_paths.items()
        },
        "cohort": {
            "documents": len(selected),
            "documentsByFormat": dict(sorted(document_formats.items())),
            "documentUsageRows": target_row_count,
            "aggregateUsageRows": len(aggregate_rows),
            "characters": characters,
        },
        "documents": private_documents,
        "privacy": {
            "localOnly": True,
            "ownerModeRequired": "0600",
            "publicProjectionContainsDocumentIdentity": False,
        },
    }
    private["cohortSha256"] = sha256_bytes(canonical_json_bytes(private_documents))

    document_counts = coverage.get("documents", {})
    counts = coverage.get("counts", {})
    public = {
        "schemaVersion": 1,
        "kind": "font-kerning-cohort-baseline",
        "issue": 4968,
        "stage": "W9-Q0",
        "inputs": {
            name: {"artifact": relative_repo_path(path), "sha256": sha256_file(path)}
            for name, path in tracked_paths.items()
        },
        "w3LocalAttestation": {
            "sourceHead": checkpoint["identity"]["sourceHead"],
            "documentCount": checkpoint["identity"]["documentCount"],
            "successfulDocuments": complete_records,
            "successfulDocumentsByFormat": {
                key: value["success"] for key, value in sorted(document_counts["formats"].items())
            },
            "layoutCharacters": counts["layoutCharacters"],
            "checkpointPolicySha256": checkpoint["identity"]["checkpointPolicySha256"],
            "checkpointChainSha256": checkpoint["chain"]["value"],
            "aggregateSha256": coverage["aggregateHash"]["value"],
        },
        "cohort": {
            "documents": len(selected),
            "documentsByFormat": dict(sorted(document_formats.items())),
            "documentUsageRows": target_row_count,
            "aggregateUsageRows": len(aggregate_rows),
            "characters": characters,
            "byFormat": _counter_projection(flattened, ("format",)),
            "byStoredLineSeg": _counter_projection(flattened, ("storedLineSeg",)),
            "byContext": _counter_projection(flattened, ("context",)),
            "byDocumentAndMetricFace": _counter_projection(
                flattened, ("font", "metricFace")
            ),
            "byRatioAndSpacing": _counter_projection(flattened, ("ratio", "spacing")),
        },
        "w8Freeze": {
            "status": "satisfied",
            "overlap": overlaps,
            "metricFallbackProductMutation": False,
        },
        "executionPolicy": {
            "fullCorpusRerun": False,
            "corpusSourceFilesOpened": 0,
            "hyperVOracleRerun": False,
            "productSourceMutation": False,
            "nextStage": "kerning-off-baseline-and-capability-contract",
        },
        "privacy": {
            "absolutePathIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateDocumentHashIncluded": False,
            "privateDocumentNameIncluded": False,
        },
        "gates": {
            "checkpointIdentityMatched": True,
            "journalAggregateReconciled": True,
            "privateCohortSeparated": True,
            "w8OverlapFrozen": True,
            "recalculationOnly": True,
        },
    }
    reject_absolute_paths(public)
    public["canonicalSha256"] = sha256_bytes(canonical_json_bytes(public))
    return private, public


def project(args: argparse.Namespace) -> tuple[dict[str, Any], dict[str, Any]]:
    local_paths = {
        "w3Manifest": regular_file(args.manifest, MAX_MANIFEST_BYTES),
        "w3Coverage": regular_file(args.coverage, MAX_COVERAGE_BYTES),
        "w3Journal": regular_file(args.journal, MAX_JOURNAL_BYTES),
    }
    tracked_paths = {
        "w5FixtureManifest": regular_file(args.w5_fixture, MAX_EVIDENCE_BYTES),
        "notoSansKrRegular": regular_file(args.noto_font, MAX_EVIDENCE_BYTES),
        "w8Rank1Disposition": regular_file(args.w8_rank1, MAX_EVIDENCE_BYTES),
        "w8Rank7Disposition": regular_file(args.w8_rank7, MAX_EVIDENCE_BYTES),
        "w8Rank8Disposition": regular_file(args.w8_rank8, MAX_EVIDENCE_BYTES),
    }
    manifest = read_json(local_paths["w3Manifest"], MAX_MANIFEST_BYTES)
    coverage = read_json(local_paths["w3Coverage"], MAX_COVERAGE_BYTES)
    rank1 = read_json(tracked_paths["w8Rank1Disposition"], MAX_EVIDENCE_BYTES)
    rank7 = read_json(tracked_paths["w8Rank7Disposition"], MAX_EVIDENCE_BYTES)
    rank8 = read_json(tracked_paths["w8Rank8Disposition"], MAX_EVIDENCE_BYTES)
    documents, _ = validate_manifest_and_coverage(manifest, coverage)
    selected, journal_sha256, complete_records, target_row_count = scan_journal(
        local_paths["w3Journal"], documents
    )
    return build_outputs(
        manifest=manifest,
        coverage=coverage,
        selected=selected,
        journal_sha256=journal_sha256,
        complete_records=complete_records,
        target_row_count=target_row_count,
        tracked_paths=tracked_paths,
        local_paths=local_paths,
        rank1=rank1,
        rank7=rank7,
        rank8=rank8,
    )


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--coverage", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--w5-fixture", type=Path, required=True)
    parser.add_argument("--noto-font", type=Path, required=True)
    parser.add_argument("--w8-rank1", type=Path, required=True)
    parser.add_argument("--w8-rank7", type=Path, required=True)
    parser.add_argument("--w8-rank8", type=Path, required=True)
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
                "characters": public["cohort"]["characters"],
                "publicCanonicalSha256": public["canonicalSha256"],
                "fullCorpusRerun": False,
                "corpusSourceFilesOpened": 0,
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KerningCohortError as error:
        raise SystemExit(str(error)) from error
