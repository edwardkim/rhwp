#!/usr/bin/env python3
"""Project local-only W5-3 evidence into one path-free Oracle Profile."""

from __future__ import annotations

import argparse
import json
import re
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
READINESS = INVESTIGATION / "font_oracle_readiness.json"
ABSOLUTE_PATH = re.compile(r"^(?:/home/|/mnt/|[A-Za-z]:[\\/])")


def evidence(status: str, value: Any = None, reason: str | None = None) -> dict[str, Any]:
    if status == "observed":
        if value is None or reason is not None:
            raise OracleStage2Error("observed evidence requires a value and null reason")
    elif value is not None or not reason:
        raise OracleStage2Error("unobserved evidence requires null value and a reason")
    return {"status": status, "value": value, "reason": reason}


def read_external(root: Path, relative: str, maximum_bytes: int) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum_bytes)
    try:
        with path.open(encoding="utf-8") as stream:
            return path, json.load(stream)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid local-only JSON evidence: {relative}") from error


def reject_absolute_paths(value: Any, label: str = "evidence") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            reject_absolute_paths(child, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            reject_absolute_paths(child, f"{label}[{index}]")
    elif isinstance(value, str) and ABSOLUTE_PATH.match(value):
        raise OracleStage2Error(f"{label} exposes an absolute path")


def require_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise OracleStage2Error(f"Stage W5-3 evidence mismatch: {label}")


def _candidate(readiness: dict[str, Any], rank: int, face: str) -> dict[str, Any]:
    candidates = [
        entry
        for entry in readiness["candidates"]
        if entry["queueRank"] == rank and entry["documentFace"] == face
    ]
    if len(candidates) != 1 or candidates[0].get("sourceReadiness") != "ready-local-sfnt":
        raise OracleStage2Error("exact-installed canary lacks one ready SFNT candidate")
    return candidates[0]


def _sample(candidate: dict[str, Any], codepoint: int) -> dict[str, Any]:
    samples = [
        entry
        for entry in candidate["sfnt"]["sampleGlyphEvidence"]
        if entry["codepoint"] == codepoint and entry["status"] == "observed"
    ]
    if len(samples) != 1:
        raise OracleStage2Error("representative SFNT glyph evidence is unavailable")
    return samples[0]


def generate_profile(
    *,
    evidence_root: Path,
    evidence_relative: str,
    pdf_observation_relative: str,
    fixture_manifest_relative: str,
) -> dict[str, Any]:
    stage3 = read_json(STAGE3_CONTRACT)
    contract = read_json(PROFILE_CONTRACT)
    readiness = read_json(READINESS)
    canary = stage3["currentHostCanary"]["exactInstalledCanary"]

    run_path, run = read_external(evidence_root, evidence_relative, 1024 * 1024)
    observation_path, observation = read_external(
        evidence_root, pdf_observation_relative, 16 * 1024 * 1024
    )
    fixture_manifest_path, fixture_manifest = read_external(
        evidence_root, fixture_manifest_relative, 4 * 1024 * 1024
    )
    for value in (run, observation, fixture_manifest):
        reject_absolute_paths(value)

    require_equal(sha256_file(run_path), canary["runEvidenceSha256"], "run evidence hash")
    require_equal(
        sha256_file(observation_path),
        canary["pdfObservationSha256"],
        "PDF observation file hash",
    )
    require_equal(
        sha256_file(fixture_manifest_path),
        canary["fixtureManifestSha256"],
        "fixture manifest file hash",
    )
    require_equal(
        sha256_file(ROOT / "scripts/oracle_stage3_windows_canary.ps1"),
        canary["runnerSha256"],
        "Windows canary runner hash",
    )

    rank = canary["queueRank"]
    face = canary["documentFace"]
    require_equal(contract["inputPreconditions"]["queueFaces"][rank - 1], face, "queue identity")
    require_equal(run["kind"], "font-oracle-stage3-windows-canary-evidence", "run kind")
    require_equal(run["candidate"], {"queueRank": rank, "documentFace": face}, "candidate")
    require_equal(run["input"]["sha256"], canary["fixtureSha256"], "fixture hash")
    require_equal(run["input"]["sourceFormat"], "hwpx", "source format")
    require_equal(fixture_manifest["inputSha256"], canary["fixtureSha256"], "manifest input hash")
    require_equal(
        fixture_manifest["semanticSha256"],
        canary["fixtureSemanticSha256"],
        "fixture semantic hash",
    )
    require_equal(fixture_manifest["semantic"]["documentFace"], face, "fixture face")
    require_equal(fixture_manifest["semantic"]["queueRank"], rank, "fixture rank")
    require_equal(run["environment"]["hancomVersion"], stage3["currentHostCanary"]["hancomVersion"], "Hancom build")
    require_equal(run["environment"]["os"], stage3["currentHostCanary"]["os"], "OS")
    require_equal(run["environment"]["locale"], stage3["currentHostCanary"]["locale"], "locale")
    require_equal(run["environment"]["processReset"], True, "process reset")
    require_equal(run["environment"]["securityModuleRegistered"], True, "security module")
    require_equal(
        run["environment"]["ambientFontManifestSha256"],
        canary["ambientFontManifestSha256"],
        "ambient font manifest",
    )
    require_equal(run["featureDetection"]["opened"], True, "HWPX feature detection")
    require_equal(run["fontState"]["selection"]["exact"], True, "exact readback")
    require_equal(run["fontState"]["selection"]["readbackFace"], face, "readback face")
    require_equal(run["export"]["pdfSha256"], canary["pdfSha256"], "PDF hash")
    require_equal(observation["inputSha256"], canary["pdfSha256"], "observed PDF hash")
    require_equal(
        observation["canonicalSha256"],
        canary["pdfObservationCanonicalSha256"],
        "PDF observation canonical hash",
    )

    source = _candidate(readiness, rank, face)
    require_equal(source["sfnt"]["sha256"], canary["installedFontSha256"], "SFNT source hash")
    require_equal(run["fontState"]["installedFontSha256"], canary["installedFontSha256"], "installed font hash")
    glyph = _sample(source, 0xAC00)

    matching_fonts = [
        entry
        for entry in observation["fonts"]
        if "MalgunGothic" in entry["name"] and entry["embedded"] and entry["subset"]
    ]
    if len(matching_fonts) != 1:
        raise OracleStage2Error("PDF does not expose one embedded MalgunGothic subset")
    pdf_font = matching_fonts[0]
    matching_glyphs = [
        entry
        for entry in observation["glyphObservations"]
        if entry["unicode"] == "가" and entry["font"] == pdf_font["name"]
    ]
    if not matching_glyphs:
        raise OracleStage2Error("PDF lacks the representative U+AC00 glyph observation")
    pdf_glyph = matching_glyphs[0]

    direct_anchor = {
        "runEvidenceSha256": canary["runEvidenceSha256"],
        "pdfSha256": canary["pdfSha256"],
        "pdfObservationCanonicalSha256": observation["canonicalSha256"],
        "requestedFace": face,
        "readbackFace": run["fontState"]["selection"]["readbackFace"],
        "readbackFontType": run["fontState"]["selection"]["readbackFontType"],
        "installedFontSha256": canary["installedFontSha256"],
        "sourceNameTable": source["sfnt"]["nameTable"],
        "representativeCodepoint": glyph["codepoint"],
        "sourceGlyphName": glyph["glyphName"],
        "sourceGlyphOutlineSha256": glyph["outlineSha256"],
        "pdfSubsetFontName": pdf_font["name"],
        "pdfFontType": pdf_font["type"],
        "pdfUnicodeMap": pdf_font["unicodeMap"],
        "pdfGlyphOrCid": pdf_glyph["glyphName"],
        "pdfToolVersions": observation["toolVersions"],
    }
    return {
        "schemaVersion": stage3["profileSchemaVersion"],
        "kind": "font-oracle-profile",
        "issue": 4963,
        "candidate": {"queueRank": rank, "documentFace": face},
        "questionId": "exact-installed",
        "exactMissingState": "exact-installed",
        "input": {
            "sourceFormat": evidence("observed", "hwpx"),
            "sha256": evidence("observed", canary["fixtureSha256"]),
            "fixtureContractVersion": canary["fixtureContractVersion"],
            "fixtureGeneratorCommit": canary["fixtureGeneratorCommit"],
        },
        "environment": {
            "os": evidence("observed", stage3["currentHostCanary"]["os"]),
            "locale": evidence("observed", stage3["currentHostCanary"]["locale"]),
            "hancomVersion": evidence("observed", stage3["currentHostCanary"]["hancomVersion"]),
            "pdfProducer": evidence("observed", stage3["currentHostCanary"]["pdfProducer"]),
            "exportRoute": evidence("observed", run["export"]["route"]),
            "oracleAuthority": "acceptance-primary",
            "ambientFontManifestSha256": evidence(
                "observed", canary["ambientFontManifestSha256"]
            ),
            "processReset": evidence("observed", True),
            "rebooted": evidence(
                "unavailable",
                reason="The read-only W5-3 run did not record a preceding OS reboot.",
            ),
        },
        "execution": {
            "evidenceClass": "oracle-run",
            "measurementDate": run["execution"]["startedAt"][:10],
            "startedAt": evidence("observed", run["execution"]["startedAt"]),
            "finishedAt": evidence("observed", run["execution"]["finishedAt"]),
            "repeatIndex": run["execution"]["repeatIndex"],
        },
        "fontState": {
            "requestedFace": face,
            "relatedFaceSet": [face],
            "installedFontSha256": evidence("observed", canary["installedFontSha256"]),
            "readbackFace": evidence("observed", run["fontState"]["selection"]["readbackFace"]),
            "readbackFontType": evidence(
                "observed", run["fontState"]["selection"]["readbackFontType"]
            ),
        },
        "observations": {
            "subsetFontName": evidence("observed", pdf_font["name"]),
            "glyphOutlineDigest": evidence("observed", glyph["outlineSha256"]),
            "hmtxAdvance": evidence(
                "observed",
                {
                    "advance": glyph["hmtxAdvance"],
                    "unitsPerEm": source["sfnt"]["unitsPerEm"],
                    "faceIndex": source["sfnt"]["faceIndex"],
                    "sourceFontSha256": source["sfnt"]["sha256"],
                },
            ),
            "pdfObservedAdvance": evidence(
                "observed",
                {
                    "advance": pdf_glyph["pdfObservedAdvance"]["distance"],
                    "unit": "pdf-user-space",
                    "glyphOrCid": f'U+AC00/{pdf_glyph["glyphName"]}',
                },
            ),
            "firstTypesettingDivergence": evidence(
                "not-applicable",
                reason="W5-3 records one exact-installed state; no paired missing-font state was run.",
            ),
            "lineCount": evidence("observed", observation["visualLineCount"]),
            "pageCount": evidence("observed", observation["pageCount"]),
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


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--evidence", default="malgun_gothic_exact_installed.evidence.json")
    parser.add_argument(
        "--pdf-observation", default="malgun_gothic_exact_installed.pdf_observation.json"
    )
    parser.add_argument(
        "--fixture-manifest", default="malgun_gothic_exact_installed.manifest.json"
    )
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument(
        "--output", default="windows_hwp2020_malgun_gothic_exact_installed.json"
    )
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    profile = generate_profile(
        evidence_root=args.evidence_root,
        evidence_relative=args.evidence,
        pdf_observation_relative=args.pdf_observation,
        fixture_manifest_relative=args.fixture_manifest,
    )
    target = output_path(args.output_root, args.output)
    write_json(target, profile, mode=0o644)
    print(sha256_bytes(canonical_json_bytes(profile)))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
