#!/usr/bin/env python3
"""Project hash-matched #2430 evidence into W5 historical Oracle Profiles."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    INVESTIGATION,
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


STAGE3_CONTRACT = INVESTIGATION / "oracle_stage3_contract.json"
PROFILE_CONTRACT = INVESTIGATION / "oracle_profile_contract.json"


def evidence(status: str, value: Any = None, reason: str | None = None) -> dict[str, Any]:
    if status == "observed":
        if value is None or reason is not None:
            raise OracleStage2Error("observed evidence requires a value and null reason")
    elif value is not None or not reason:
        raise OracleStage2Error("unobserved evidence requires null value and a reason")
    return {"status": status, "value": value, "reason": reason}


def _verify_file(relative: str, expected_sha256: str) -> Path:
    path = regular_input(ROOT, relative, 16 * 1024 * 1024)
    if sha256_file(path) != expected_sha256:
        raise OracleStage2Error(f"historical evidence hash mismatch: {relative}")
    return path


def _read_preflight(path: Path) -> dict[str, dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    if len(rows) != 5:
        raise OracleStage2Error("preflight evidence must contain five rows")
    result = {row["requested_face"]: row for row in rows}
    if len(result) != len(rows):
        raise OracleStage2Error("preflight requested faces must be unique")
    return result


def _verify_ladder(path: Path, face: str) -> dict[str, Any]:
    with path.open("r", encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    codes = []
    for row in rows:
        if row.get("face") != face:
            raise OracleStage2Error("historical ladder face mismatch")
        try:
            code = int(row["code"])
            advance = float(row["adv_em"])
        except (KeyError, ValueError) as error:
            raise OracleStage2Error("historical ladder row is invalid") from error
        if not 32 <= code <= 126 or advance <= 0:
            raise OracleStage2Error("historical ladder value is outside its contract")
        codes.append(code)
    if len(rows) != 93 or len(set(codes)) != 93 or 34 in codes or 39 in codes:
        raise OracleStage2Error("historical ladder character inventory drift")
    return {
        "storedMeasuredCharacterCount": len(rows),
        "minimumCodepoint": min(codes),
        "maximumCodepoint": max(codes),
        "excludedAutocorrectCodepoints": [34, 39],
    }


def _profile(
    *,
    stage3: dict[str, Any],
    candidate: dict[str, Any],
    preflight_row: dict[str, str],
    ladder_summary: dict[str, Any],
) -> dict[str, Any]:
    historical = stage3["historicalImport"]
    face = candidate["documentFace"]
    if (
        preflight_row["readback_face"] != face
        or int(preflight_row["readback_fonttype"]) != candidate["readbackFontType"]
        or preflight_row["verdict"] != "OK"
    ):
        raise OracleStage2Error("historical preflight row does not prove exact readback")
    subset = (
        evidence("observed", candidate["subsetFontName"])
        if candidate["subsetFontName"] is not None
        else evidence(
            "unavailable",
            reason="The per-face probe PDF was not retained; only its Type3 classification was recorded.",
        )
    )
    direct_anchor = {
        "evidenceSha256": historical["evidenceSha256"],
        "preflightSha256": historical["preflightSha256"],
        "measurementScriptSha256": historical["measurementScriptSha256"],
        "ladderSha256": candidate["ladderSha256"],
        "requestedFace": face,
        "readbackFace": preflight_row["readback_face"],
        "readbackFontType": int(preflight_row["readback_fonttype"]),
        "verdict": preflight_row["verdict"],
        "pdfFontClassification": candidate["pdfFontClassification"],
        "repeatCount": historical["environment"]["repeatCount"],
        "ladderSummary": ladder_summary,
    }
    return {
        "schemaVersion": stage3["profileSchemaVersion"],
        "kind": "font-oracle-profile",
        "issue": 4963,
        "candidate": {
            "queueRank": candidate["queueRank"],
            "documentFace": face,
        },
        "questionId": "exact-installed",
        "exactMissingState": "exact-installed",
        "input": {
            "sourceFormat": evidence("observed", "in-memory-hwp"),
            "sha256": evidence(
                "unavailable",
                reason="The COM-generated source document was never persisted, so no input bytes exist to hash.",
            ),
            "fixtureContractVersion": historical["fixtureContractVersion"],
            "fixtureGeneratorCommit": historical["fixtureGeneratorCommit"],
        },
        "environment": {
            "os": evidence("observed", historical["environment"]["os"]),
            "locale": evidence(
                "unavailable", reason="The 2026-07-21 measurement did not record the Windows locale."
            ),
            "hancomVersion": evidence(
                "observed", historical["environment"]["hancomVersion"]
            ),
            "pdfProducer": evidence(
                "unavailable",
                reason="The per-face probe PDFs were not retained and their producer strings were not recorded.",
            ),
            "exportRoute": evidence(
                "observed", historical["environment"]["exportRoute"]
            ),
            "oracleAuthority": "secondary-historical",
            "ambientFontManifestSha256": evidence(
                "unavailable",
                reason="The 2026-07-21 measurement did not preserve an ambient font manifest.",
            ),
            "processReset": evidence(
                "observed", historical["environment"]["processReset"]
            ),
            "rebooted": evidence(
                "unavailable",
                reason="The 2026-07-21 measurement did not record reboot state.",
            ),
        },
        "execution": {
            "evidenceClass": "historical-import",
            "measurementDate": historical["measurementDate"],
            "startedAt": evidence(
                "unavailable", reason="Only the measurement date was preserved."
            ),
            "finishedAt": evidence(
                "unavailable", reason="Only the measurement date was preserved."
            ),
            "repeatIndex": 1,
        },
        "fontState": {
            "requestedFace": face,
            "relatedFaceSet": [face],
            "installedFontSha256": evidence(
                "unavailable",
                reason="The selected HFT bytes were not exposed or hashed by the historical run.",
            ),
            "readbackFace": evidence("observed", preflight_row["readback_face"]),
            "readbackFontType": evidence(
                "observed", int(preflight_row["readback_fonttype"])
            ),
        },
        "observations": {
            "subsetFontName": subset,
            "glyphOutlineDigest": evidence(
                "unavailable",
                reason="The historical probe PDFs were not retained for canonical outline extraction.",
            ),
            "hmtxAdvance": evidence(
                "not-applicable",
                reason="The historical exact selection was HFT FontType 2, not a verified source SFNT face.",
            ),
            "pdfObservedAdvance": evidence(
                "unavailable",
                reason="The stored ladder is an em-normalized ASCII aggregate, not a W5 glyph-level PDF user-space observation.",
            ),
            "firstTypesettingDivergence": evidence(
                "not-applicable",
                reason="The historical measurement has no paired missing-font ladder state.",
            ),
            "lineCount": evidence(
                "unavailable", reason="The historical evidence did not preserve line counts."
            ),
            "pageCount": evidence(
                "unavailable", reason="The historical evidence did not preserve page counts."
            ),
        },
        "relationEvidence": {
            "type": "identity-exact",
            "anchor": evidence("observed", direct_anchor),
        },
        "privacy": {
            "fontBytesEmbedded": False,
            "privateDocumentIdentityIncluded": False,
            "absoluteFontPathIncluded": False,
        },
    }


def generate_profiles(output_root: Path) -> dict[str, Any]:
    stage3 = read_json(STAGE3_CONTRACT)
    profile_contract = read_json(PROFILE_CONTRACT)
    if (
        stage3.get("kind") != "font-oracle-stage3-contract"
        or stage3.get("issue") != 4963
        or stage3.get("profileSchemaVersion") != 2
    ):
        raise OracleStage2Error("Stage W5-3 contract identity mismatch")
    historical = stage3["historicalImport"]
    _verify_file("tools/task2430/EVIDENCE.md", historical["evidenceSha256"])
    _verify_file(
        "tools/task2430/hy_ascii_ladder.py", historical["measurementScriptSha256"]
    )
    preflight = _read_preflight(
        _verify_file(
            "tools/task2430/measured/preflight_report.tsv",
            historical["preflightSha256"],
        )
    )
    expected_faces = profile_contract["inputPreconditions"]["queueFaces"]
    manifest_profiles = []
    for candidate in historical["candidates"]:
        face = candidate["documentFace"]
        if expected_faces[candidate["queueRank"] - 1] != face:
            raise OracleStage2Error("historical candidate drifted from the W4 queue")
        ladder_relative = f"tools/task2430/measured/ladder_{face}.tsv"
        ladder = _verify_file(ladder_relative, candidate["ladderSha256"])
        profile = _profile(
            stage3=stage3,
            candidate=candidate,
            preflight_row=preflight[face],
            ladder_summary=_verify_ladder(ladder, face),
        )
        slug = "hanyang_sinmyeongjo" if candidate["queueRank"] == 10 else "human_myeongjo"
        relative = f"historical_{slug}_exact_installed.json"
        target = output_path(output_root, relative)
        write_json(target, profile, mode=0o644)
        manifest_profiles.append(
            {
                "queueRank": candidate["queueRank"],
                "documentFace": face,
                "file": relative,
                "fileSha256": sha256_file(target),
                "canonicalSha256": sha256_bytes(canonical_json_bytes(profile)),
            }
        )
    manifest = {
        "schemaVersion": 1,
        "kind": "font-oracle-historical-import-manifest",
        "issue": 4963,
        "profileSchemaVersion": 2,
        "profileCount": len(manifest_profiles),
        "profiles": manifest_profiles,
        "scope": stage3["scope"],
    }
    manifest["canonicalSha256"] = sha256_bytes(canonical_json_bytes(manifest))
    write_json(output_path(output_root, "historical_import_manifest.json"), manifest, mode=0o644)
    return manifest


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    manifest = generate_profiles(parse_arguments().output_root)
    print(manifest["canonicalSha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
