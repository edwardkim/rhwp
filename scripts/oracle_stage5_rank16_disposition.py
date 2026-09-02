#!/usr/bin/env python3
"""Project the W5-5B rank-16 read-only run into a path-free disposition."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from oracle_stage2_common import (
    INVESTIGATION,
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


READINESS_PATH = INVESTIGATION / "font_oracle_readiness.json"
STAGE3_CONTRACT_PATH = INVESTIGATION / "oracle_stage3_contract.json"
DISPOSITION_NAME = "oracle_stage5_rank16_read_only_disposition.json"

RANK16_RAW = {
    "directory": "rank16-exact-installed",
    "stem": "rank16-exact-installed",
    "runFileSha256": "86469901faa7a5e34450b68070078be16a6a9555d551f26f16789f2b0111033b",
    "pdfSha256": "f9d8c38426cbf06d082039ccba5381eb4f95afc67a13ec4e84443b1373ea612b",
    "observationFileSha256": "7723964eeb9457b31d12974284642acdd19bce67a3c825f1cfef9e2e92325b4e",
    "observationCanonicalSha256": "8e68023c089fcddda06c05b9243ad8dcdba2f58bcae4491b01688751876d3a57",
    "fixtureSha256": "af49f4bced881f9f8aa23bc575fa30267b4ada83b5161c5a445d681dbf93e2be",
    "fixtureManifestSha256": "913551d49a4d65c2bb2b8036e15f1854a75a697297b602992b8751b12855027c",
    "fixtureSemanticSha256": "68e240c7d37ccfa7612d5b21e55d066a8f198d773b756220993a2d9f5088c093",
    "sourceFontSha256": "cdf746176aba807cbf9b168c2d749be6270f44f4b80568b9e18b05d1ba53a200",
    "baselineFontManifestSha256": "3bcd379d1f7fc217aad47a0b44b952d993c86ebbfabf46009386e4b3de768b40",
    "unrelatedProjectionSha256": "437a36e513cce9d2909d904f3d07d2341051cc017e21be9ec6d35bbb9d87bc78",
}


def _read_external_json(root: Path, relative: str, maximum_bytes: int) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum_bytes)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid local-only JSON evidence: {relative}") from error
    reject_absolute_paths(value)
    return path, value


def _selection(run: dict[str, Any], requested_face: str) -> dict[str, Any]:
    matches = [
        selection
        for selection in run["fontSelections"]
        if selection.get("requestedFace") == requested_face
    ]
    if len(matches) != 1:
        raise OracleStage2Error(f"rank 16 selection is not unique: {requested_face}")
    return matches[0]


def build_rank16_read_only_disposition(evidence_root: Path) -> dict[str, Any]:
    prefix = f'{RANK16_RAW["directory"]}/{RANK16_RAW["stem"]}'
    run_path, run = _read_external_json(evidence_root, f"{prefix}.interactive.json", 1024 * 1024)
    observation_path, observation = _read_external_json(
        evidence_root, f"{prefix}.pdf-observation.json", 32 * 1024 * 1024
    )
    pdf_path = regular_input(evidence_root, f"{prefix}.pdf", 64 * 1024 * 1024)
    fixture_path = regular_input(
        evidence_root,
        f'{RANK16_RAW["directory"]}/rank16.hwpx',
        16 * 1024 * 1024,
    )
    fixture_manifest_path, fixture_manifest = _read_external_json(
        evidence_root,
        f'{RANK16_RAW["directory"]}/rank16.manifest.json',
        4 * 1024 * 1024,
    )

    for path, expected, label in (
        (run_path, RANK16_RAW["runFileSha256"], "rank 16 run"),
        (observation_path, RANK16_RAW["observationFileSha256"], "rank 16 observation"),
        (pdf_path, RANK16_RAW["pdfSha256"], "rank 16 PDF"),
        (fixture_path, RANK16_RAW["fixtureSha256"], "rank 16 fixture"),
        (
            fixture_manifest_path,
            RANK16_RAW["fixtureManifestSha256"],
            "rank 16 fixture manifest",
        ),
    ):
        require_equal(sha256_file(path), expected, label)

    canonical_input = dict(observation)
    claimed = canonical_input.pop("canonicalSha256")
    require_equal(
        sha256_bytes(canonical_json_bytes(canonical_input)),
        claimed,
        "rank 16 observation self hash",
    )
    require_equal(claimed, RANK16_RAW["observationCanonicalSha256"], "rank 16 observation")
    require_equal(
        fixture_manifest["inputSha256"], RANK16_RAW["fixtureSha256"], "rank 16 fixture input"
    )
    require_equal(
        fixture_manifest["semanticSha256"],
        RANK16_RAW["fixtureSemanticSha256"],
        "rank 16 fixture semantic",
    )
    require_equal(fixture_manifest["semantic"]["queueRank"], 16, "rank 16 fixture queue")
    require_equal(
        fixture_manifest["semantic"]["documentFace"],
        "한컴 윤고딕 230",
        "rank 16 fixture face",
    )
    require_equal(fixture_manifest["semantic"]["fontBytesEmbedded"], False, "font bytes")

    readiness = read_json(READINESS_PATH)
    candidates = [entry for entry in readiness["candidates"] if entry["queueRank"] == 16]
    if len(candidates) != 1:
        raise OracleStage2Error("rank 16 readiness candidate is not unique")
    source = candidates[0]
    require_equal(source["documentFace"], "한컴 윤고딕 230", "rank 16 readiness face")
    require_equal(source["sourceReadiness"], "ready-local-sfnt", "rank 16 source readiness")
    require_equal(source["sfnt"]["sha256"], RANK16_RAW["sourceFontSha256"], "rank 16 SFNT")
    require_equal(
        source["sfnt"]["nameTable"]["postScriptName"],
        ["HaanYGodic23"],
        "rank 16 PostScript name",
    )

    require_equal(run["status"], "observed", "rank 16 run status")
    require_equal(run["queueRank"], 16, "rank 16 run queue")
    require_equal(run["documentFace"], "한컴 윤고딕 230", "rank 16 run face")
    require_equal(run["inputSha256"], RANK16_RAW["fixtureSha256"], "rank 16 run input")
    require_equal(run["documentFaceSelectable"], False, "rank 16 document face selection")
    require_equal(run["featureDetection"]["opened"], True, "rank 16 HWPX open")
    require_equal(run["environment"]["hancomVersion"], "11, 0, 0, 9136", "Hancom build")
    require_equal(run["environment"]["processReset"], True, "rank 16 process reset")
    require_equal(run["environment"]["fontResourceCounts"], [], "rank 16 managed font resources")
    require_equal(run["export"]["pdfSha256"], RANK16_RAW["pdfSha256"], "rank 16 export")
    require_equal(run["privacy"]["privateCorpusAccessed"], False, "private corpus access")

    document_selection = _selection(run, "한컴 윤고딕 230")
    alias_selection = _selection(run, "Haan YGodic 230")
    require_equal(document_selection["readbackFace"], "함초롬바탕", "document face readback")
    require_equal(document_selection["readbackFontType"], 5, "document face readback type")
    require_equal(document_selection["exact"], False, "document face exact selection")
    require_equal(alias_selection["readbackFace"], "Haan YGodic 230", "English alias readback")
    require_equal(alias_selection["readbackFontType"], 1, "English alias readback type")
    require_equal(alias_selection["exact"], True, "English alias exact selection")

    require_equal(observation["inputSha256"], RANK16_RAW["pdfSha256"], "rank 16 PDF observation")
    require_equal(observation["pageCount"], run["featureDetection"]["pageCount"], "page count")
    require_equal(
        [font["name"] for font in observation["fonts"]],
        ["INPILL+HCRBatang-Bold"],
        "rank 16 PDF font set",
    )
    glyph = observation["glyphObservations"][18]
    require_equal(glyph["unicode"], "가", "rank 16 representative glyph")
    require_equal(glyph["font"], "INPILL+HCRBatang-Bold", "rank 16 representative font")

    stage3 = read_json(STAGE3_CONTRACT_PATH)
    old_results = stage3["currentHostCanary"]["selectionProbe"]["results"]
    old_rank16 = [entry for entry in old_results if entry.get("queueRank") == 16]
    if len(old_rank16) != 1:
        raise OracleStage2Error("prior rank 16 selection evidence is not unique")
    require_equal(old_rank16[0]["exact"], True, "prior rank 16 exact selection")

    result = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage5-read-only-disposition",
        "issue": 4963,
        "candidate": {"queueRank": 16, "documentFace": "한컴 윤고딕 230"},
        "physicalState": "restored-baseline-no-managed-font-resource",
        "status": "blocked-document-face-name-resolution",
        "evidenceClass": "acceptance-blocking-observation",
        "inputSha256": RANK16_RAW["fixtureSha256"],
        "environment": {
            "hancomVersion": run["environment"]["hancomVersion"],
            "baselineFontManifestSha256": RANK16_RAW["baselineFontManifestSha256"],
            "unrelatedFontProjectionSha256": RANK16_RAW["unrelatedProjectionSha256"],
            "processReset": True,
            "managedFontResourceCount": 0,
        },
        "featureDetection": {
            "documentFace": {
                "requestedFace": document_selection["requestedFace"],
                "readbackFace": document_selection["readbackFace"],
                "readbackFontType": document_selection["readbackFontType"],
                "exact": False,
            },
            "sfntEnglishAlias": {
                "requestedFace": alias_selection["requestedFace"],
                "readbackFace": alias_selection["readbackFace"],
                "readbackFontType": alias_selection["readbackFontType"],
                "exact": True,
            },
            "priorSingleSelectionExact": True,
            "priorSelectionEvidenceSha256": sha256_file(STAGE3_CONTRACT_PATH),
            "precedence": "restored-document-open-and-pdf-export-over-single-selection-probe",
        },
        "source": {
            "sfntSha256": RANK16_RAW["sourceFontSha256"],
            "postScriptName": "HaanYGodic23",
            "sourceReadiness": source["sourceReadiness"],
            "fontBytesEmbedded": False,
        },
        "output": {
            "pdfSha256": RANK16_RAW["pdfSha256"],
            "pdfObservationCanonicalSha256": RANK16_RAW["observationCanonicalSha256"],
            "subsetFontNames": ["INPILL+HCRBatang-Bold"],
            "exactSourceSubsetObserved": False,
            "pageCount": observation["pageCount"],
            "visualLineCount": observation["visualLineCount"],
            "textSpanCount": observation["textSpanCount"],
            "glyphObservationCount": observation["glyphObservationCount"],
        },
        "reason": (
            "The restored baseline accepts the English SFNT alias, but the document Korean face "
            "falls back to HCRBatang and the PDF does not use the exact source bytes."
        ),
        "recoveryCondition": (
            "Resume exact-profile publication only after a separately approved controlled state "
            "makes the document face exact and the exported subset proves use of the same SFNT bytes."
        ),
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateCorpusAccessed": False,
        },
    }
    reject_absolute_paths(result, "rank16Disposition")
    return result


def validate_rank16_read_only_disposition(value: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if (
        value.get("schemaVersion") != 1
        or value.get("kind") != "font-oracle-stage5-read-only-disposition"
        or value.get("issue") != 4963
    ):
        errors.append("rank 16 disposition identity mismatch")
    if value.get("candidate") != {
        "queueRank": 16,
        "documentFace": "한컴 윤고딕 230",
    }:
        errors.append("rank 16 disposition candidate mismatch")
    if value.get("status") != "blocked-document-face-name-resolution":
        errors.append("rank 16 disposition status mismatch")
    feature = value.get("featureDetection", {})
    if feature.get("documentFace", {}).get("exact") is not False:
        errors.append("rank 16 document face must remain non-exact")
    if feature.get("sfntEnglishAlias", {}).get("exact") is not True:
        errors.append("rank 16 English alias exact anchor is missing")
    output = value.get("output", {})
    if output.get("exactSourceSubsetObserved") is not False:
        errors.append("rank 16 disposition invented an exact PDF subset")
    if output.get("subsetFontNames") != ["INPILL+HCRBatang-Bold"]:
        errors.append("rank 16 fallback subset evidence drifted")
    privacy = value.get("privacy", {})
    for key in (
        "absolutePathIncluded",
        "hostNameIncluded",
        "fontBytesIncluded",
        "privateDocumentIdentityIncluded",
        "privateCorpusAccessed",
    ):
        if privacy.get(key) is not False:
            errors.append(f"rank 16 privacy boundary drifted: {key}")
    try:
        reject_absolute_paths(value, "rank16Disposition")
    except OracleStage2Error as error:
        errors.append(str(error))
    return errors


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    disposition = build_rank16_read_only_disposition(args.evidence_root)
    errors = validate_rank16_read_only_disposition(disposition)
    if errors:
        raise OracleStage2Error("; ".join(errors))
    payload = pretty_json_bytes(disposition)
    write_bytes(output_path(args.output_root, DISPOSITION_NAME), payload, mode=0o644)
    print(sha256_bytes(payload))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
