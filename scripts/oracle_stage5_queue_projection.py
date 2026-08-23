#!/usr/bin/env python3
"""Build the W5-5 17-face action matrix without redundant Oracle runs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    INVESTIGATION,
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    pretty_json_bytes,
    read_json,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_bytes,
)
from oracle_stage4_profile import reject_absolute_paths, require_equal
from oracle_stage5_rank16_disposition import (
    DISPOSITION_NAME as RANK16_DISPOSITION_NAME,
    build_rank16_read_only_disposition,
    validate_rank16_read_only_disposition,
)
from oracle_stage5_rank8_profile import LADDER_NAME as RANK8_LADDER_NAME


READINESS_PATH = INVESTIGATION / "font_oracle_readiness.json"
PROFILE_CONTRACT_PATH = INVESTIGATION / "oracle_profile_contract.json"
STAGE3_CONTRACT_PATH = INVESTIGATION / "oracle_stage3_contract.json"
STAGE4_PROJECTION_PATH = INVESTIGATION / "oracle_stage4_acceptance_projection.json"
RANK13_DISPOSITION_NAME = "oracle_stage4_rank13_blocked_disposition.json"
QUEUE_PROJECTION_NAME = "oracle_stage5_queue_projection.json"
QUESTIONS = [
    "exact-installed",
    "exact-removed",
    "document-subst-font-only",
    "curated-official-successor-only",
    "all-related-fonts-missing",
]
SOURCE_UNAVAILABLE_RANKS = {2, 3, 4, 5, 6, 11, 12, 14, 15, 17}

RANK13_RAW = {
    "directory": "rank13-none-related",
    "stem": "none-related",
    "runFileSha256": "e2f01644a3d0c3666455af9f6c1031033ae130b2ca46b20f6d69a6393321cacd",
    "pdfSha256": "601c805d01c067ca0ac144336eddfc0c9b9dea20fd958d91290d35960761cb89",
    "observationFileSha256": "e02024f16d0d61d48269511024c8c8f7c6efc6c3bb3e07986992fa039671e24e",
    "observationCanonicalSha256": "be30463bd3f96e979bded3fc7279d63c96f727991dd3521ca61f3c0133cab76f",
    "fixtureSha256": "a6dbe726d8718513f59a78fc562bded4721e9cdfa03673b87cc907d45dc3f124",
    "fixtureManifestSha256": "1e10ab6c8b67ca231007c3578c8074e4694a68bd7b7e2cb3d3d09eb3a989b515",
    "ambientFontManifestSha256": "796ba7d2a9759c63d71098c5d3182af2d1a653cc096c332d8e987347a45700fb",
}


def _read_external_json(root: Path, relative: str, maximum_bytes: int) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum_bytes)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid local-only JSON evidence: {relative}") from error
    reject_absolute_paths(value)
    return path, value


def build_rank13_blocked_disposition(evidence_root: Path) -> dict[str, Any]:
    prefix = f'{RANK13_RAW["directory"]}/{RANK13_RAW["stem"]}'
    run_path, run = _read_external_json(
        evidence_root, f"{prefix}.interactive.json", 1024 * 1024
    )
    observation_path, observation = _read_external_json(
        evidence_root, f"{prefix}.pdf-observation.json", 32 * 1024 * 1024
    )
    pdf_path = regular_input(evidence_root, f"{prefix}.pdf", 64 * 1024 * 1024)
    fixture_path = regular_input(
        evidence_root,
        f'{RANK13_RAW["directory"]}/rank13.hwpx',
        16 * 1024 * 1024,
    )
    fixture_manifest_path = regular_input(
        evidence_root,
        f'{RANK13_RAW["directory"]}/rank13.manifest.json',
        4 * 1024 * 1024,
    )

    for path, expected, label in (
        (run_path, RANK13_RAW["runFileSha256"], "rank 13 run"),
        (observation_path, RANK13_RAW["observationFileSha256"], "rank 13 observation"),
        (pdf_path, RANK13_RAW["pdfSha256"], "rank 13 PDF"),
        (fixture_path, RANK13_RAW["fixtureSha256"], "rank 13 fixture"),
        (
            fixture_manifest_path,
            RANK13_RAW["fixtureManifestSha256"],
            "rank 13 fixture manifest",
        ),
    ):
        require_equal(sha256_file(path), expected, label)

    canonical_input = dict(observation)
    claimed = canonical_input.pop("canonicalSha256")
    require_equal(
        sha256_bytes(canonical_json_bytes(canonical_input)),
        claimed,
        "rank 13 observation self hash",
    )
    require_equal(claimed, RANK13_RAW["observationCanonicalSha256"], "rank 13 observation")
    require_equal(run["queueRank"], 13, "rank 13 queue identity")
    require_equal(run["documentFace"], "휴먼명조", "rank 13 document face")
    require_equal(run["inputSha256"], RANK13_RAW["fixtureSha256"], "rank 13 input")
    require_equal(run["environment"]["fontResourceCounts"], [], "rank 13 managed font set")
    require_equal(run["environment"]["processReset"], True, "rank 13 process reset")
    require_equal(run["featureDetection"]["opened"], True, "rank 13 HWPX open")
    require_equal(run["privacy"]["privateCorpusAccessed"], False, "private corpus access")
    selection = run["fontSelections"][0]
    require_equal(selection["requestedFace"], "휴먼명조", "rank 13 selection request")
    require_equal(selection["readbackFace"], "휴먼명조", "rank 13 exact readback")
    require_equal(selection["exact"], True, "rank 13 exact state")
    require_equal(run["export"]["pdfSha256"], RANK13_RAW["pdfSha256"], "rank 13 export")
    require_equal(observation["inputSha256"], RANK13_RAW["pdfSha256"], "rank 13 PDF observation")

    result = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage4-blocked-disposition",
        "issue": 4963,
        "candidate": {"queueRank": 13, "documentFace": "휴먼명조"},
        "physicalState": "none-related",
        "status": "blocked-immutable-or-unmanaged-font",
        "evidenceClass": "acceptance-blocking-observation",
        "inputSha256": RANK13_RAW["fixtureSha256"],
        "environment": {
            "hancomVersion": run["environment"]["hancomVersion"],
            "ambientFontManifestSha256": RANK13_RAW["ambientFontManifestSha256"],
            "processReset": True,
        },
        "fontState": {
            "managedRelatedFontCount": 0,
            "requestedFace": selection["requestedFace"],
            "readbackFace": selection["readbackFace"],
            "readbackFontType": selection["readbackFontType"],
            "exactReadbackSurvived": True,
        },
        "output": {
            "pdfSha256": RANK13_RAW["pdfSha256"],
            "pdfObservationCanonicalSha256": RANK13_RAW[
                "observationCanonicalSha256"
            ],
            "pageCount": observation["pageCount"],
            "visualLineCount": observation["visualLineCount"],
            "textSpanCount": observation["textSpanCount"],
            "glyphObservationCount": observation["glyphObservationCount"],
        },
        "reason": (
            "The exact face remained selectable with zero managed related fonts; destructive "
            "mutation of a bundled HFT or an unmanaged ambient source is outside the contract."
        ),
        "recoveryCondition": (
            "Resume only in a disposable image where every exact provider is inventoried and "
            "can be removed and restored without mutating a Hancom bundle."
        ),
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateCorpusAccessed": False,
        },
    }
    reject_absolute_paths(result, "rank13Disposition")
    return result


def _profile_inventory() -> tuple[dict[int, list[dict[str, Any]]], str]:
    inventory: list[dict[str, Any]] = []
    for path in sorted((INVESTIGATION / "profiles").glob("*.json")):
        value = read_json(path)
        if value.get("kind") != "font-oracle-profile":
            continue
        relative = path.relative_to(ROOT).as_posix()
        inventory.append(
            {
                "queueRank": value["candidate"]["queueRank"],
                "documentFace": value["candidate"]["documentFace"],
                "questionId": value["questionId"],
                "oracleAuthority": value["environment"]["oracleAuthority"],
                "artifact": relative,
                "sha256": sha256_file(path),
            }
        )
    inventory.sort(key=lambda entry: (entry["queueRank"], entry["questionId"], entry["artifact"]))
    by_rank: dict[int, list[dict[str, Any]]] = {}
    for entry in inventory:
        by_rank.setdefault(entry["queueRank"], []).append(entry)
    return by_rank, sha256_bytes(canonical_json_bytes(inventory))


def _question(status: str, reason: str, evidence: list[str] | None = None) -> dict[str, Any]:
    return {"status": status, "reason": reason, "evidence": evidence or []}


def _profile_evidence(profiles: list[dict[str, Any]], question: str) -> list[str]:
    return [entry["artifact"] for entry in profiles if entry["questionId"] == question]


def _candidate_projection(
    entry: dict[str, Any],
    profiles: list[dict[str, Any]],
    stage3: dict[str, Any],
    rank13_artifact: str,
    rank16_artifact: str,
    rank8_ladder: dict[str, Any],
) -> dict[str, Any]:
    rank = entry["queueRank"]
    face = entry["documentFace"]
    successor = _question(
        "not-provided-no-direct-anchor",
        "No direct publisher or byte-lineage anchor establishes an official successor.",
    )

    if rank in {1, 7}:
        questions = {
            question: _question(
                "observed-primary",
                "A controlled disposable-environment profile is tracked and hash-addressed.",
                _profile_evidence(profiles, question),
            )
            for question in (
                "exact-installed",
                "exact-removed",
                "document-subst-font-only",
                "all-related-fonts-missing",
            )
        }
        questions["curated-official-successor-only"] = successor
        disposition = "complete-acceptance-ladder"
        next_action = "reuse-tracked-profiles"
    elif rank in SOURCE_UNAVAILABLE_RANKS:
        questions = {
            "exact-installed": _question(
                "blocked-source-unavailable",
                "No exact source bytes are available to establish the exact-installed anchor.",
            ),
            "exact-removed": _question(
                "blocked-no-exact-anchor",
                "A removed-state comparison is not meaningful without an exact-installed anchor.",
            ),
            "document-subst-font-only": _question(
                "blocked-no-direct-relation-anchor",
                "No document substitution relation has been directly established for this face.",
            ),
            "curated-official-successor-only": successor,
            "all-related-fonts-missing": _question(
                "blocked-no-related-font-inventory",
                "The related font set cannot be proven complete while the exact source is unavailable.",
            ),
        }
        disposition = "terminal-source-unavailable"
        next_action = "source-discovery-only"
    elif rank == 8:
        expected_questions = {
            "exact-installed",
            "exact-removed",
            "document-subst-font-only",
            "all-related-fonts-missing",
        }
        actual_questions = {profile["questionId"] for profile in profiles}
        if actual_questions != expected_questions:
            raise OracleStage2Error("rank 8 acceptance profile set is incomplete")
        ladder_hashes = {
            profile["questionId"]: profile["sha256"]
            for profile in rank8_ladder["profiles"]
        }
        for profile in profiles:
            require_equal(
                profile["sha256"],
                ladder_hashes.get(profile["questionId"]),
                f'rank 8 {profile["questionId"]} ladder profile',
            )
        questions = {
            question: _question(
                "observed-primary",
                "A controlled disposable-environment profile is tracked and hash-addressed.",
                _profile_evidence(profiles, question),
            )
            for question in expected_questions
        }
        questions.update({
            "curated-official-successor-only": successor,
        })
        disposition = "complete-acceptance-ladder"
        next_action = "reuse-tracked-profiles"
    elif rank == 9:
        questions = {
            "exact-installed": _question(
                "observed-primary",
                "The existing HWP 2020 exact-installed profile is reusable.",
                _profile_evidence(profiles, "exact-installed"),
            ),
            "exact-removed": _question(
                "blocked-protected-ambient-exact",
                "The exact provider is an ambient Windows system font and is not removed by this contract.",
            ),
            "document-subst-font-only": _question(
                "blocked-protected-ambient-exact",
                "The protected exact provider prevents isolation of a substitution-only state.",
            ),
            "curated-official-successor-only": successor,
            "all-related-fonts-missing": _question(
                "blocked-protected-ambient-exact",
                "The protected exact provider prevents an all-related-fonts-missing state.",
            ),
        }
        disposition = "terminal-protected-partial"
        next_action = "reuse-exact-profile-and-preserve-system-font"
    elif rank == 10:
        questions = {
            "exact-installed": _question(
                "observed-historical",
                "Hash-matched Hancom 2022 HFT evidence is reusable as secondary historical evidence.",
                _profile_evidence(profiles, "exact-installed"),
            ),
            "exact-removed": _question(
                "blocked-immutable-hft",
                "A bundled HFT provider is not removed by this contract.",
            ),
            "document-subst-font-only": _question(
                "blocked-immutable-hft",
                "The bundled exact provider prevents isolation of a substitution-only state.",
            ),
            "curated-official-successor-only": successor,
            "all-related-fonts-missing": _question(
                "blocked-immutable-hft",
                "The bundled exact provider prevents an all-related-fonts-missing state.",
            ),
        }
        disposition = "terminal-protected-partial"
        next_action = "reuse-historical-profile-and-preserve-hft"
    elif rank == 13:
        blocked = _question(
            "blocked-immutable-or-unmanaged-font",
            "Exact readback survived with zero managed related fonts; the provider is outside the mutable set.",
            [rank13_artifact],
        )
        questions = {
            "exact-installed": _question(
                "observed-historical",
                "Hash-matched Hancom 2022 exact-installed evidence remains reusable.",
                _profile_evidence(profiles, "exact-installed"),
            ),
            "exact-removed": blocked,
            "document-subst-font-only": blocked,
            "curated-official-successor-only": successor,
            "all-related-fonts-missing": blocked,
        }
        disposition = "terminal-protected-partial"
        next_action = "preserve-provider-and-reuse-blocked-disposition"
    elif rank == 16:
        results = stage3["currentHostCanary"]["selectionProbe"]["results"]
        matches = [result for result in results if result.get("queueRank") == 16]
        if len(matches) != 1 or matches[0].get("exact") is not True:
            raise OracleStage2Error("rank 16 exact selection evidence is unavailable")
        protected = _question(
            "blocked-protected-ambient-alias",
            "The ambient English alias remains outside the managed set; this read-only lane does not remove it or create missing states.",
            [rank16_artifact],
        )
        questions = {
            "exact-installed": _question(
                "blocked-document-face-name-resolution",
                "The restored document face falls back to HCRBatang and the PDF does not use the exact SFNT bytes.",
                [rank16_artifact],
            ),
            "exact-removed": protected,
            "document-subst-font-only": protected,
            "curated-official-successor-only": successor,
            "all-related-fonts-missing": protected,
        }
        disposition = "terminal-read-only-capability-mismatch"
        next_action = "preserve-read-only-disposition"
    else:
        raise OracleStage2Error(f"unclassified W5 queue rank: {rank}")

    return {
        "queueRank": rank,
        "documentFace": face,
        "sourceReadiness": entry["sourceReadiness"],
        "disposition": disposition,
        "availableProfiles": profiles,
        "questions": {question: questions[question] for question in QUESTIONS},
        "nextAction": next_action,
    }


def build_queue_projection(
    rank13_disposition_path: Path,
    rank16_disposition_path: Path,
    rank8_ladder_path: Path,
) -> dict[str, Any]:
    readiness = read_json(READINESS_PATH)
    profile_contract = read_json(PROFILE_CONTRACT_PATH)
    stage3 = read_json(STAGE3_CONTRACT_PATH)
    stage4 = read_json(STAGE4_PROJECTION_PATH)
    rank13_disposition = read_json(rank13_disposition_path)
    rank16_disposition = read_json(rank16_disposition_path)
    rank8_ladder = read_json(rank8_ladder_path)
    reject_absolute_paths(rank13_disposition)
    reject_absolute_paths(rank16_disposition)
    reject_absolute_paths(rank8_ladder)
    require_equal(
        rank13_disposition.get("status"),
        "blocked-immutable-or-unmanaged-font",
        "rank 13 blocked disposition",
    )
    require_equal(
        rank16_disposition.get("status"),
        "blocked-document-face-name-resolution",
        "rank 16 read-only disposition",
    )
    rank16_errors = validate_rank16_read_only_disposition(rank16_disposition)
    if rank16_errors:
        raise OracleStage2Error("; ".join(rank16_errors))
    require_equal(rank8_ladder.get("kind"), "font-oracle-stage5-ladder-evidence", "rank 8 ladder")
    require_equal(
        rank8_ladder.get("target"),
        {"queueRank": 8, "documentFace": "KoPubWorld바탕체 Light"},
        "rank 8 ladder target",
    )
    require_equal(
        [run.get("physicalState") for run in rank8_ladder.get("runs", [])],
        ["exact-only", "subst-only", "none-related"],
        "rank 8 physical states",
    )
    require_equal(
        rank8_ladder.get("privacy", {}).get("privateCorpusAccessed"),
        False,
        "rank 8 ladder privacy",
    )
    by_rank, inventory_sha256 = _profile_inventory()
    rank13_artifact = (INVESTIGATION / RANK13_DISPOSITION_NAME).relative_to(ROOT).as_posix()
    rank16_artifact = (INVESTIGATION / RANK16_DISPOSITION_NAME).relative_to(ROOT).as_posix()
    candidates = [
        _candidate_projection(
            entry,
            by_rank.get(entry["queueRank"], []),
            stage3,
            rank13_artifact,
            rank16_artifact,
            rank8_ladder,
        )
        for entry in readiness["candidates"]
    ]
    require_equal([entry["queueRank"] for entry in candidates], list(range(1, 18)), "queue order")
    require_equal(
        [entry["documentFace"] for entry in candidates],
        profile_contract["inputPreconditions"]["queueFaces"],
        "queue face identity",
    )

    counts: dict[str, int] = {}
    for candidate in candidates:
        counts[candidate["disposition"]] = counts.get(candidate["disposition"], 0) + 1
    result = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage5-queue-projection",
        "issue": 4963,
        "stage": "W5-5C",
        "inputs": {
            "readinessSha256": sha256_file(READINESS_PATH),
            "stage3ContractSha256": sha256_file(STAGE3_CONTRACT_PATH),
            "stage4AcceptanceProjectionSha256": sha256_file(STAGE4_PROJECTION_PATH),
            "rank13BlockedDispositionSha256": sha256_file(rank13_disposition_path),
            "rank16ReadOnlyDispositionSha256": sha256_file(rank16_disposition_path),
            "rank8AcceptanceLadderSha256": sha256_file(rank8_ladder_path),
            "profileInventorySha256": inventory_sha256,
        },
        "policy": {
            "questionIds": QUESTIONS,
            "remeasurementTrigger": [
                "input-or-font-bytes-changed",
                "oracle-environment-identity-changed",
                "profile-schema-or-canonicalization-changed",
                "blocked-provider-became-safely-inventoried-and-restorable",
            ],
            "privateCorpusRemeasurementRequired": False,
            "productBehaviorChanged": False,
        },
        "candidateCount": len(candidates),
        "counts": dict(sorted(counts.items())),
        "actionableRanks": [],
        "recommendedExecutionOrder": [],
        "candidates": candidates,
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateCorpusAccessed": False,
        },
    }
    reject_absolute_paths(result, "queueProjection")
    result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
    return result


def validate_queue_projection(value: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if value.get("schemaVersion") != 1 or value.get("kind") != "font-oracle-stage5-queue-projection":
        errors.append("queue projection identity mismatch")
    if value.get("stage") != "W5-5C":
        errors.append("queue projection stage drifted")
    candidates = value.get("candidates")
    if not isinstance(candidates, list) or len(candidates) != 17:
        errors.append("queue projection must contain 17 candidates")
        candidates = []
    if [entry.get("queueRank") for entry in candidates] != list(range(1, 18)):
        errors.append("queue projection ranks are not exact and ordered")
    if len(candidates) == 17:
        rank8 = candidates[7]
        if (
            rank8.get("disposition") != "complete-acceptance-ladder"
            or len(rank8.get("availableProfiles", [])) != 4
        ):
            errors.append("queue projection rank 8 acceptance ladder drifted")
        rank16 = candidates[15]
        if (
            rank16.get("disposition") != "terminal-read-only-capability-mismatch"
            or rank16.get("questions", {}).get("exact-installed", {}).get("status")
            != "blocked-document-face-name-resolution"
        ):
            errors.append("queue projection rank 16 disposition drifted")
    if value.get("actionableRanks") != []:
        errors.append("queue projection actionable rank boundary drifted")
    if value.get("recommendedExecutionOrder") != []:
        errors.append("queue projection safe execution order drifted")
    if value.get("policy", {}).get("privateCorpusRemeasurementRequired") is not False:
        errors.append("queue projection reintroduced private corpus measurement")
    if value.get("policy", {}).get("productBehaviorChanged") is not False:
        errors.append("queue projection changed product behavior")
    claimed = value.get("canonicalSha256")
    projection = dict(value)
    projection.pop("canonicalSha256", None)
    if claimed != sha256_bytes(canonical_json_bytes(projection)):
        errors.append("queue projection canonical hash mismatch")
    try:
        reject_absolute_paths(value)
    except OracleStage2Error as error:
        errors.append(str(error))
    return errors


def write_outputs(
    output_root: Path,
    rank13_disposition: dict[str, Any],
    rank16_disposition: dict[str, Any],
    queue_projection: dict[str, Any],
) -> dict[str, str]:
    hashes = {}
    for name, value in (
        (RANK13_DISPOSITION_NAME, rank13_disposition),
        (RANK16_DISPOSITION_NAME, rank16_disposition),
        (QUEUE_PROJECTION_NAME, queue_projection),
    ):
        payload = pretty_json_bytes(value)
        write_bytes(output_path(output_root, name), payload, mode=0o644)
        hashes[name] = sha256_bytes(payload)
    return hashes


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage4-evidence-root", type=Path, required=True)
    parser.add_argument("--stage5-evidence-root", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    rank13 = build_rank13_blocked_disposition(args.stage4_evidence_root)
    rank16 = build_rank16_read_only_disposition(args.stage5_evidence_root)
    with_rank13 = output_path(args.output_root, RANK13_DISPOSITION_NAME)
    with_rank16 = output_path(args.output_root, RANK16_DISPOSITION_NAME)
    write_bytes(with_rank13, pretty_json_bytes(rank13), mode=0o644)
    write_bytes(with_rank16, pretty_json_bytes(rank16), mode=0o644)
    rank8_ladder = INVESTIGATION / RANK8_LADDER_NAME
    projection = build_queue_projection(with_rank13, with_rank16, rank8_ladder)
    errors = validate_queue_projection(projection)
    if errors:
        raise OracleStage2Error("; ".join(errors))
    hashes = write_outputs(args.output_root, rank13, rank16, projection)
    print(json.dumps(hashes, ensure_ascii=False, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
