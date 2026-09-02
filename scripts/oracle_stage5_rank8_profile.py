#!/usr/bin/env python3
"""Project the W5-5C rank-8 controlled ladder into public profiles."""

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
from oracle_stage4_profile import (
    ATTESTATION,
    BASELINE_MANIFEST_SHA256,
    FIXTURE_GENERATOR_COMMIT,
    HCR_SUBSET,
    UNRELATED_PROJECTION_SHA256,
    evidence,
    reject_absolute_paths,
    require_equal,
)


READINESS_PATH = INVESTIGATION / "font_oracle_readiness.json"
LADDER_NAME = "oracle_stage5_rank8_acceptance_ladder.json"
FACE = "KoPubWorld바탕체 Light"
EXACT_ALIAS = "KoPubWorldBatang Light"
EXACT_SUBSET = "INPILL+KoPubWorldBatangLight"
SUBST_FACE = "KoPubWorld돋움체 Light"
SUBST_ALIAS = "KoPubWorldDotum Light"
FIXTURE_SHA256 = "f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084"
FIXTURE_MANIFEST_SHA256 = "1e18915164b677ed3de23ee8991a6d3f593fa479e840a8a39461482d7c8796b1"
FIXTURE_SEMANTIC_SHA256 = "4a72d8cc641e88e9aa0e4cdc7f10eb192b2811759546efc5ac974730944ec4de"
EXACT_FONT_SHA256 = "e3ee21a86b6a6728c567a95aaebd8883480f27ce4f230207b0d7266b5cb3fb18"
SUBST_FONT_SHA256 = "069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f"

STATES: dict[str, dict[str, Any]] = {
    "exact-only": {
        "directory": "rank8-exact-only",
        "stem": "exact-only",
        "runSha256": "09199628f3df57715979f6f4ca8bb40425ac286304ae7ffcadd950a4b852034d",
        "pdfSha256": "aa7d710607e06c0068b3f1e074e318ac93506bd715b5e09e20b19e0b99c961f1",
        "observationFileSha256": "03903e707049b53952ab355c153f9d3c7b4ec349556fd706d1e7bfecfd1815a8",
        "observationCanonicalSha256": "48b62a7067f0375bf4e12eb27c39e842797647ba59fd260eae70cc6965dd51ae",
        "manifestFileSha256": "97bc290b8515667880d19eefd860be6f63207c194046745a493f991e5c4f214f",
        "manifestSha256": "b6804297959be101efb8f446029ae0921ef920cc5d439a54e21f1fabf7cd4820",
        "managedFontSha256": EXACT_FONT_SHA256,
        "typesettingProjectionSha256": "38f83a7973aaa49b4464b4e8d7579c05dfd47f8209997ceb1a740fa0baf2b4c7",
    },
    "subst-only": {
        "directory": "rank8-subst-only",
        "stem": "subst-only",
        "runSha256": "ba96b8c04431518c07369f71b82963331893c2f26cbfc5b10e4ba2289cef7496",
        "pdfSha256": "32b6d4d81d6dee8ff8c333ea2fd56283a05922bb872ac4ac6da0340bf2af7cf5",
        "observationFileSha256": "17bd8aa6061b985cdb985ee3236483e81d71dc91e4f0ada671657b9300eeb56d",
        "observationCanonicalSha256": "8392104ff5e0e504755a91db624f1fe53f2ea8a3c11655df5a53b09184721054",
        "manifestFileSha256": "3fb6d0f07f3f29e7660719f309b183069424a27af0cdab24bb5ad6e11f6139e7",
        "manifestSha256": "d825dd3db24df0f344cf5737fd76beb711c6445a98146c8bbf93ca1c01c98a9a",
        "managedFontSha256": SUBST_FONT_SHA256,
        "typesettingProjectionSha256": "59801255a8663ef14c9796be76942bc95b2fa46f80bc6b0631bf5bd220c827be",
    },
    "none-related": {
        "directory": "rank8-none-related",
        "stem": "none-related",
        "runSha256": "8ad63a4285b59121829d453bd2e6faed2de1f96a27bd3c4b7eb7847f503584a7",
        "pdfSha256": "e17e58055ba31894e1676fe3c98fc2e8113816bb8920d283772760dcbed00511",
        "observationFileSha256": "1dd2dca59ce28a283c3d38914c1be7bdaeaa09c8657d10cbf270f74f2fc3e47d",
        "observationCanonicalSha256": "c86adec7b7a0b110a262fd8447f7cf7f98ec87751c254446f84aa2a8e0deeb86",
        "manifestFileSha256": "5f21dc917a725107b86ac2ac98bf70e7a1dd3ab1d622a7c7c6cd4596ac432bda",
        "manifestSha256": BASELINE_MANIFEST_SHA256,
        "managedFontSha256": None,
        "typesettingProjectionSha256": "59801255a8663ef14c9796be76942bc95b2fa46f80bc6b0631bf5bd220c827be",
    },
}

QUESTION_STATE = {
    "exact-installed": "exact-only",
    "document-subst-font-only": "subst-only",
    "exact-removed": "none-related",
    "all-related-fonts-missing": "none-related",
}


def _read_external_json(root: Path, relative: str, maximum_bytes: int) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum_bytes)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid local-only JSON evidence: {relative}") from error
    reject_absolute_paths(value)
    return path, value


def _candidate(readiness: dict[str, Any], rank: int) -> dict[str, Any]:
    matches = [entry for entry in readiness["candidates"] if entry["queueRank"] == rank]
    if len(matches) != 1 or matches[0]["sourceReadiness"] != "ready-local-sfnt":
        raise OracleStage2Error(f"rank {rank} ready SFNT candidate is unavailable")
    return matches[0]


def _selection(run: dict[str, Any], face: str) -> dict[str, Any]:
    matches = [entry for entry in run["fontSelections"] if entry["requestedFace"] == face]
    if len(matches) != 1:
        raise OracleStage2Error(f"selection is not unique: {face}")
    return matches[0]


def _typesetting_projection(observation: dict[str, Any]) -> str:
    projected = dict(observation)
    for key in ("canonicalSha256", "inputSha256", "toolVersions"):
        projected.pop(key, None)
    return sha256_bytes(canonical_json_bytes(projected))


def _state_evidence(evidence_root: Path, state: str) -> dict[str, Any]:
    spec = STATES[state]
    prefix = f'{spec["directory"]}/{spec["stem"]}'
    run_path, run = _read_external_json(evidence_root, f"{prefix}.interactive.json", 1024 * 1024)
    observation_path, observation = _read_external_json(
        evidence_root, f"{prefix}.pdf-observation.json", 32 * 1024 * 1024
    )
    manifest_path, manifest = _read_external_json(
        evidence_root, f"{prefix}.ambient-manifest.json", 1024 * 1024
    )
    pdf_path = regular_input(evidence_root, f"{prefix}.pdf", 64 * 1024 * 1024)
    for path, expected, label in (
        (run_path, spec["runSha256"], f"rank 8 {state} run"),
        (observation_path, spec["observationFileSha256"], f"rank 8 {state} observation"),
        (manifest_path, spec["manifestFileSha256"], f"rank 8 {state} manifest"),
        (pdf_path, spec["pdfSha256"], f"rank 8 {state} PDF"),
    ):
        require_equal(sha256_file(path), expected, label)

    canonical_input = dict(observation)
    claimed = canonical_input.pop("canonicalSha256")
    require_equal(
        sha256_bytes(canonical_json_bytes(canonical_input)),
        claimed,
        f"rank 8 {state} observation self hash",
    )
    require_equal(claimed, spec["observationCanonicalSha256"], "observation hash")
    require_equal(
        _typesetting_projection(observation),
        spec["typesettingProjectionSha256"],
        "typesetting projection",
    )
    require_equal(run["status"], "observed", "run status")
    require_equal(run["queueRank"], 8, "queue rank")
    require_equal(run["documentFace"], FACE, "document face")
    require_equal(run["inputSha256"], FIXTURE_SHA256, "fixture input")
    require_equal(run["featureDetection"]["opened"], True, "HWPX open")
    require_equal(run["environment"]["hancomVersion"], "11, 0, 0, 9136", "Hancom build")
    require_equal(run["environment"]["processReset"], True, "process reset")
    require_equal(run["export"]["pdfSha256"], spec["pdfSha256"], "run PDF hash")
    require_equal(run["privacy"]["privateCorpusAccessed"], False, "private corpus access")
    require_equal(observation["inputSha256"], spec["pdfSha256"], "observed PDF hash")
    require_equal(manifest["manifestSha256"], spec["manifestSha256"], "font manifest")
    require_equal(
        manifest["unrelatedProjectionSha256"],
        UNRELATED_PROJECTION_SHA256,
        "unrelated font projection",
    )
    managed = manifest["managedInstalledByExactBytes"]
    expected_managed = [] if spec["managedFontSha256"] is None else [spec["managedFontSha256"]]
    require_equal(managed, expected_managed, "managed font state")
    require_equal(manifest["privacy"]["privateCorpusAccessed"], False, "manifest privacy")
    return {"run": run, "observation": observation, "manifest": manifest, "spec": spec}


def _profile_name(question: str) -> str:
    return f'windows_hwp2020_kopubworld_batang_light_{question.replace("-", "_")}.json'


def _representative_glyph(observation: dict[str, Any]) -> dict[str, Any]:
    glyph = observation["glyphObservations"][18]
    require_equal(glyph["unicode"], "가", "representative glyph")
    return glyph


def build_profile(
    *,
    question: str,
    state_record: dict[str, Any],
    exact_source: dict[str, Any],
    subst_source: dict[str, Any],
) -> dict[str, Any]:
    state = QUESTION_STATE[question]
    run = state_record["run"]
    observation = state_record["observation"]
    spec = state_record["spec"]
    requested = _selection(run, FACE)
    glyph = _representative_glyph(observation)
    expected_subset = EXACT_SUBSET if state == "exact-only" else HCR_SUBSET
    require_equal(glyph["font"], expected_subset, "representative subset")
    require_equal([entry["name"] for entry in observation["fonts"]], [expected_subset], "PDF font set")

    if state == "exact-only":
        alias = _selection(run, EXACT_ALIAS)
        require_equal(alias["exact"], True, "exact alias selection")
        source_glyph = next(
            item for item in exact_source["sfnt"]["sampleGlyphEvidence"] if item["codepoint"] == 0xAC00
        )
        installed = evidence("observed", EXACT_FONT_SHA256)
        outline = evidence("observed", source_glyph["outlineSha256"])
        hmtx = evidence(
            "observed",
            {
                "advance": source_glyph["hmtxAdvance"],
                "unitsPerEm": exact_source["sfnt"]["unitsPerEm"],
                "faceIndex": exact_source["sfnt"]["faceIndex"],
                "sourceFontSha256": EXACT_FONT_SHA256,
            },
        )
        relation_type = "identity-alias"
        anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "requestedDocumentFace": FACE,
            "documentFaceReadback": requested["readbackFace"],
            "exactSfntAlias": EXACT_ALIAS,
            "sourceNameTable": exact_source["sfnt"]["nameTable"],
            "sourceFontSha256": EXACT_FONT_SHA256,
            "sourceGlyphOutlineSha256": source_glyph["outlineSha256"],
            "exportSubsetFontName": glyph["font"],
            "exportUsedExactBytes": True,
            "firstDivergenceComparedWith": "all-related-fonts-missing",
        }
    elif state == "subst-only":
        alias = _selection(run, SUBST_ALIAS)
        require_equal(alias["exact"], True, "substitution alias selection")
        installed = evidence("observed", SUBST_FONT_SHA256)
        outline = evidence(
            "unavailable",
            reason="The selected HCR fallback source bytes were outside the managed font set.",
        )
        hmtx = evidence(
            "unavailable",
            reason="The selected HCR fallback hmtx source was not frozen by this controlled run.",
        )
        relation_type = "document-substitution"
        anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "fixtureRelationAnchor": "fixture-declared-substFont",
            "declaredSubstitutionFace": SUBST_FACE,
            "installedSubstitutionFontSha256": SUBST_FONT_SHA256,
            "substitutionSourceNameTable": subst_source["sfnt"]["nameTable"],
            "exportSubsetFontName": glyph["font"],
            "exportUsedSubstitution": False,
            "firstDivergenceComparedWith": "exact-installed",
        }
    else:
        installed = evidence(
            "not-applicable",
            reason="The controlled none-related state contains neither managed related font.",
        )
        outline = evidence(
            "unavailable",
            reason="The selected HCR fallback source bytes were outside the managed font set.",
        )
        hmtx = evidence(
            "unavailable",
            reason="The selected HCR fallback hmtx source was not frozen by this controlled run.",
        )
        relation_type = "hancom-missing-font"
        anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "managedExactPresent": False,
            "managedDocumentSubstitutionPresent": False,
            "exportSubsetFontName": glyph["font"],
            "firstDivergenceComparedWith": "exact-installed",
        }

    profile = {
        "schemaVersion": 2,
        "kind": "font-oracle-profile",
        "issue": 4963,
        "candidate": {"queueRank": 8, "documentFace": FACE},
        "questionId": question,
        "exactMissingState": question,
        "input": {
            "sourceFormat": evidence("observed", "hwpx"),
            "sha256": evidence("observed", FIXTURE_SHA256),
            "fixtureContractVersion": "w5-oracle-typesetting-v1-subst-v1",
            "fixtureGeneratorCommit": FIXTURE_GENERATOR_COMMIT,
        },
        "environment": {
            "os": evidence("observed", "Microsoft Windows 11 Pro"),
            "locale": evidence("observed", "ko-KR"),
            "hancomVersion": evidence("observed", run["environment"]["hancomVersion"]),
            "pdfProducer": evidence("observed", "Hancom PDF 1.3.0.550"),
            "exportRoute": evidence("observed", "HAction.FileSaveAsPdf"),
            "oracleAuthority": "acceptance-primary",
            "ambientFontManifestSha256": evidence("observed", spec["manifestSha256"]),
            "processReset": evidence("observed", True),
            "rebooted": evidence("observed", True),
        },
        "execution": {
            "evidenceClass": "oracle-run",
            "measurementDate": run["startedAt"][:10],
            "startedAt": evidence("observed", run["startedAt"]),
            "finishedAt": evidence("observed", run["finishedAt"]),
            "repeatIndex": 1,
        },
        "fontState": {
            "requestedFace": FACE,
            "relatedFaceSet": [FACE, SUBST_FACE],
            "installedFontSha256": installed,
            "readbackFace": evidence("observed", requested["readbackFace"]),
            "readbackFontType": evidence("observed", requested["readbackFontType"]),
        },
        "observations": {
            "subsetFontName": evidence("observed", glyph["font"]),
            "glyphOutlineDigest": outline,
            "hmtxAdvance": hmtx,
            "pdfObservedAdvance": evidence(
                "observed",
                {
                    "advance": glyph["pdfObservedAdvance"]["distance"],
                    "unit": "pdf-user-space",
                    "glyphOrCid": f'U+AC00/{glyph["glyphName"]}',
                },
            ),
            "firstTypesettingDivergence": evidence(
                "observed",
                {"plane": "selection", "characterIndex": 18, "lineIndex": 0, "pageIndex": 0},
            ),
            "lineCount": evidence("observed", observation["visualLineCount"]),
            "pageCount": evidence("observed", observation["pageCount"]),
        },
        "relationEvidence": {"type": relation_type, "anchor": evidence("observed", anchor)},
        "privacy": {
            "fontBytesEmbedded": False,
            "privateDocumentIdentityIncluded": False,
            "absoluteFontPathIncluded": False,
        },
    }
    reject_absolute_paths(profile, "rank8Profile")
    return profile


def generate_rank8_artifacts(evidence_root: Path) -> dict[str, Any]:
    fixture_path = regular_input(evidence_root, "rank8-fixture/rank8.hwpx", 16 * 1024 * 1024)
    manifest_path, fixture_manifest = _read_external_json(
        evidence_root, "rank8-fixture/rank8.manifest.json", 4 * 1024 * 1024
    )
    require_equal(sha256_file(fixture_path), FIXTURE_SHA256, "rank 8 fixture")
    require_equal(sha256_file(manifest_path), FIXTURE_MANIFEST_SHA256, "rank 8 fixture manifest")
    require_equal(fixture_manifest["inputSha256"], FIXTURE_SHA256, "fixture input")
    require_equal(fixture_manifest["semanticSha256"], FIXTURE_SEMANTIC_SHA256, "fixture semantic")
    require_equal(fixture_manifest["semantic"]["queueRank"], 8, "fixture queue")
    require_equal(fixture_manifest["semantic"]["documentFace"], FACE, "fixture face")
    require_equal(fixture_manifest["semantic"]["substitutionFace"], SUBST_FACE, "fixture substitution")
    require_equal(fixture_manifest["semantic"]["fontBytesEmbedded"], False, "fixture font bytes")

    readiness = read_json(READINESS_PATH)
    exact_source = _candidate(readiness, 8)
    subst_source = _candidate(readiness, 7)
    require_equal(exact_source["sfnt"]["sha256"], EXACT_FONT_SHA256, "exact source")
    require_equal(subst_source["sfnt"]["sha256"], SUBST_FONT_SHA256, "substitution source")
    states = {state: _state_evidence(evidence_root, state) for state in STATES}
    require_equal(
        states["subst-only"]["spec"]["typesettingProjectionSha256"],
        states["none-related"]["spec"]["typesettingProjectionSha256"],
        "substitution and missing projection equivalence",
    )

    artifacts: dict[str, Any] = {}
    profile_hashes: dict[str, str] = {}
    for question, state in QUESTION_STATE.items():
        profile = build_profile(
            question=question,
            state_record=states[state],
            exact_source=exact_source,
            subst_source=subst_source,
        )
        relative = f"profiles/{_profile_name(question)}"
        artifacts[relative] = profile
        profile_hashes[question] = sha256_bytes(pretty_json_bytes(profile))

    runs = []
    for state, questions, profile_question, exact_present, subst_present in (
        ("exact-only", ["exact-installed"], "exact-installed", True, False),
        ("subst-only", ["document-subst-font-only"], "document-subst-font-only", False, True),
        (
            "none-related",
            ["exact-removed", "all-related-fonts-missing"],
            "exact-removed",
            False,
            False,
        ),
    ):
        spec = STATES[state]
        runs.append(
            {
                "executionId": f"rank-8-{state}-20260822",
                "physicalState": state,
                "questions": questions,
                "inputSha256": FIXTURE_SHA256,
                "unrelatedFontProjectionSha256": UNRELATED_PROJECTION_SHA256,
                "managedFonts": [
                    {"face": FACE, "sha256": EXACT_FONT_SHA256, "present": exact_present},
                    {"face": SUBST_FACE, "sha256": SUBST_FONT_SHA256, "present": subst_present},
                ],
                "processReset": True,
                "fontCacheAction": (
                    "font-cache-refresh-and-process-reset"
                    if state != "none-related"
                    else "process-reset"
                ),
                "ambientFontManifestSha256": spec["manifestSha256"],
                "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
                "outputProfileSha256": profile_hashes[profile_question],
                "restore": {
                    "restoredBeforeRun": True,
                    "restoredAfterRun": True,
                    "baselineManifestSha256": BASELINE_MANIFEST_SHA256,
                    "recoveredManifestSha256": BASELINE_MANIFEST_SHA256,
                },
            }
        )

    ladder = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage5-ladder-evidence",
        "issue": 4963,
        "evidenceClass": "acceptance-primary",
        "target": {"queueRank": 8, "documentFace": FACE},
        "fixture": {
            "sha256": FIXTURE_SHA256,
            "semanticSha256": FIXTURE_SEMANTIC_SHA256,
            "documentSubstitution": {
                "face": SUBST_FACE,
                "sha256": SUBST_FONT_SHA256,
                "relationAnchor": "fixture-declared-substFont",
                "identityAliasOrSuccessor": False,
            },
        },
        "attestation": ATTESTATION,
        "unrelatedFontProjectionSha256": UNRELATED_PROJECTION_SHA256,
        "runs": runs,
        "profiles": [
            {"questionId": question, "sha256": profile_hashes[question]}
            for question in QUESTION_STATE
        ],
        "dispositions": [
            {
                "question": "curated-official-successor-only",
                "status": "not-provided",
                "reason": "No direct publisher or byte lineage establishes an official successor.",
            }
        ],
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
            "privateCorpusAccessed": False,
        },
    }
    reject_absolute_paths(ladder, "rank8Ladder")
    artifacts[LADDER_NAME] = ladder
    return artifacts


def write_rank8_artifacts(output_root: Path, artifacts: dict[str, Any]) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for relative, value in sorted(artifacts.items()):
        payload = pretty_json_bytes(value)
        write_bytes(output_path(output_root, relative), payload, mode=0o644)
        hashes[relative] = sha256_bytes(payload)
    return hashes


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    hashes = write_rank8_artifacts(
        args.output_root,
        generate_rank8_artifacts(args.evidence_root),
    )
    print(json.dumps(hashes, ensure_ascii=False, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
