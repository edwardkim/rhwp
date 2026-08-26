#!/usr/bin/env python3
"""Build the public, path-free W5 candidate readiness ledger for Issue #4963."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path
from typing import Any

from font_oracle_inventory import inventory_relative_font
from oracle_stage2_common import (
    INVESTIGATION,
    ROOT,
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    read_json,
    regular_input,
    sha256_bytes,
    sha256_file,
    write_json,
)


PROFILE_CONTRACT = INVESTIGATION / "oracle_profile_contract.json"


def _verify_existing_hft(source: dict[str, Any]) -> dict[str, Any]:
    evidence = regular_input(ROOT, source["evidenceArtifact"], 16 * 1024 * 1024)
    if sha256_file(evidence) != source["evidenceSha256"]:
        raise OracleStage2Error("HFT evidence hash mismatch")
    with evidence.open("r", encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    matches = [row for row in rows if row.get("requested_face") == source["documentFace"]]
    if len(matches) != 1:
        raise OracleStage2Error("HFT evidence must have exactly one requested-face row")
    row = matches[0]
    if (
        row.get("readback_face") != source["documentFace"]
        or row.get("readback_fonttype") != "2"
        or row.get("verdict") != "OK"
    ):
        raise OracleStage2Error("HFT exact-readback evidence is not acceptable")
    return {
        "evidenceClass": "historical-import",
        "evidenceSha256": source["evidenceSha256"],
        "requestedFace": source["documentFace"],
        "readbackFace": row["readback_face"],
        "readbackFontType": int(row["readback_fonttype"]),
        "verdict": row["verdict"],
    }


def _ladder_readiness(readiness: str) -> dict[str, str]:
    if readiness == "ready-local-sfnt":
        exact = "ready-for-read-only-canary"
    elif readiness == "ready-existing-hft-evidence":
        exact = "historical-import-ready"
    else:
        exact = "blocked-source-unavailable"
    return {
        "exact-installed": exact,
        "exact-removed": "mutable-environment-and-approval-required",
        "document-subst-font-only": "document-substitution-discovery-required",
        "curated-official-successor-only": "direct-authority-anchor-required",
        "all-related-fonts-missing": "mutable-environment-and-approval-required",
    }


def build_readiness(
    *, contract: dict[str, Any], font_root: Path
) -> dict[str, Any]:
    profile = read_json(PROFILE_CONTRACT)
    sources = contract["fontInventory"]["sources"]
    expected_faces = profile["inputPreconditions"]["queueFaces"]
    actual_faces = [source["documentFace"] for source in sources]
    actual_ranks = [source["queueRank"] for source in sources]
    if actual_faces != expected_faces or actual_ranks != list(range(1, 18)):
        raise OracleStage2Error("Stage W5-2 inventory drifted from the W4 queue")

    official = {
        entry["documentFace"]: entry for entry in contract["officialSupply"]
    }
    candidates = []
    counts = {
        "readyLocalSfnt": 0,
        "readyExistingHftEvidence": 0,
        "sourceUnavailable": 0,
    }
    for source in sources:
        readiness = source["readiness"]
        candidate: dict[str, Any] = {
            "queueRank": source["queueRank"],
            "documentFace": source["documentFace"],
            "sourceReadiness": readiness,
            "ladderReadiness": _ladder_readiness(readiness),
        }
        if readiness == "ready-local-sfnt":
            font_path = regular_input(
                font_root,
                source["relativePath"],
                contract["fontInventory"]["maximumBytes"],
            )
            if sha256_file(font_path) != source["sha256"]:
                raise OracleStage2Error("local SFNT hash mismatch")
            inventory = inventory_relative_font(
                contract=contract,
                font_root=font_root,
                relative_font=source["relativePath"],
                document_face=source["documentFace"],
            )
            if inventory["sha256"] != source["sha256"]:
                raise OracleStage2Error("local SFNT changed during inventory")
            if not inventory["exactNameMatch"]:
                raise OracleStage2Error("local SFNT lacks the exact document face name")
            candidate["sourceKind"] = source["sourceKind"]
            candidate["sfnt"] = {
                key: inventory[key]
                for key in contract["privacy"]["publicFontFields"]
            }
            if source["documentFace"] in official:
                supply = official[source["documentFace"]]
                if (
                    supply["fontSha256"] != inventory["sha256"]
                    or supply["os2FsType"] != inventory["os2FsType"]
                ):
                    raise OracleStage2Error("official supply and SFNT inventory disagree")
                candidate["officialSupply"] = supply
            counts["readyLocalSfnt"] += 1
        elif readiness == "ready-existing-hft-evidence":
            candidate["hftEvidence"] = _verify_existing_hft(source)
            counts["readyExistingHftEvidence"] += 1
        elif readiness == "source-unavailable":
            candidate["reason"] = (
                "No hash-verified bytes or direct identity anchor are available; "
                "identity and fallback relations remain unknown."
            )
            counts["sourceUnavailable"] += 1
        else:
            raise OracleStage2Error("unknown font source readiness")
        candidates.append(candidate)

    if counts != {
        "readyLocalSfnt": 6,
        "readyExistingHftEvidence": 1,
        "sourceUnavailable": 10,
    }:
        raise OracleStage2Error("candidate readiness totals do not match W5-2")
    result = {
        "schemaVersion": 1,
        "kind": "font-oracle-readiness-ledger",
        "issue": 4963,
        "candidateCount": len(candidates),
        "counts": counts,
        "privacy": {
            "fontBytesTracked": False,
            "absolutePathsPublished": False,
            "privateDocumentIdentityPublished": False,
        },
        "scope": contract["scope"],
        "candidates": candidates,
    }
    result["canonicalSha256"] = sha256_bytes(canonical_json_bytes(result))
    return result


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--font-root", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    result = build_readiness(contract=read_contract(), font_root=args.font_root)
    write_json(output_path(args.output_root, args.output), result, mode=0o644)
    print(result["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
