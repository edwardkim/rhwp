#!/usr/bin/env python3
"""Project local-only W5-4 evidence into path-free acceptance artifacts."""

from __future__ import annotations

import argparse
import json
import re
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


CONTRACT_PATH = INVESTIGATION / "oracle_stage4_contract.json"
PROFILE_CONTRACT_PATH = INVESTIGATION / "oracle_profile_contract.json"
READINESS_PATH = INVESTIGATION / "font_oracle_readiness.json"
ABSOLUTE_PATH = re.compile(r"^(?:/home/|/mnt/|[A-Za-z]:[\\/])")
BASELINE_MANIFEST_SHA256 = "3bcd379d1f7fc217aad47a0b44b952d993c86ebbfabf46009386e4b3de768b40"
UNRELATED_PROJECTION_SHA256 = "437a36e513cce9d2909d904f3d07d2341051cc017e21be9ec6d35bbb9d87bc78"
FIXTURE_GENERATOR_COMMIT = "04c1d874d67b84afced3061b80d4e22d4d827f72"
HCR_SUBSET = "INPILL+HCRBatang-Bold"

ATTESTATION = {
    "schemaVersion": 1,
    "kind": "font-oracle-disposable-environment-attestation",
    "issue": 4963,
    "evidenceClass": "acceptance-primary",
    "provider": "hyper-v-checkpoint",
    "vmIdentitySha256": "466349124f6411dc1460697f8c2959256c28d682daf5f48376d05c83a1f5346d",
    "baselineSnapshotIdentitySha256": "7961e64697b76e8985d918abcf52a8fa0eca1d7cb5d5d46bea0af7f926b4dbe8",
    "baselineFontManifestSha256": BASELINE_MANIFEST_SHA256,
    "externalControlPlane": True,
    "restoreProbe": {
        "performed": True,
        "beforeManifestSha256": BASELINE_MANIFEST_SHA256,
        "recoveredManifestSha256": BASELINE_MANIFEST_SHA256,
    },
    "privacy": {
        "absolutePathIncluded": False,
        "hostNameIncluded": False,
        "fontBytesIncluded": False,
    },
}

TARGETS: dict[int, dict[str, Any]] = {
    1: {
        "slug": "mbatang",
        "face": "문체부 바탕체",
        "exactAlias": "MBatang",
        "exactSubset": "INPILL+MBatang",
        "fixtureDirectory": "rank1-exact-only",
        "fixture": "rank1.hwpx",
        "fixtureManifest": "rank1.manifest.json",
        "states": {
            "exact-only": {
                "directory": "rank1-exact-only",
                "stem": "exact-only-updated-v2",
                "runSha256": "3b505e88c5aa574cbe0f81d0cf34a6c31b985103e09624b421034801b6fff30c",
                "observationFileSha256": "e1620ffa014498431773187b91a008aaddc0daa7f9e117e527ea2d489bc37b87",
                "pdfSha256": "68fff42b7d4ff823ae95d25424e60eb2a70c14b3bd67bf7814dfea28247d7451",
                "observationCanonicalSha256": "5391d3359e56c0133cbe40ec46010ef4d96d73178a2a8fe96abd48548c8316c8",
                "manifestSha256": "21ab57e274238b7bcd823f3feca01c8ba9b9eb5ea88f5345ca7c7b1f46533bc3",
                "typesettingProjectionSha256": "eb44a80f60df7eacb0f7f1e884972749931b74bc037d9e37e81aed9cf13a0042",
                "repeatIndex": 2,
            },
            "subst-only": {
                "directory": "rank1-subst-only",
                "stem": "subst-only-updated",
                "runSha256": "04768725f119d87cbd605f21754c00845bd205ca1d7b5c9d819f4810e30afac9",
                "observationFileSha256": "a9356e99a242a0cdd436b28b91058360eb734e8b56747f0e3e16dc009656901e",
                "pdfSha256": "fef6ac438db1ddb4e03287c94b0954ddc88b12867a7369511bd7700a9939e64b",
                "observationCanonicalSha256": "e481a663860c8dd36287d09f15db1283f1224bafc4a1cf4723d3c61af114a16c",
                "manifestSha256": "8e2ba2554d97e07b652fb60ef3594b57f27f28a283a0740a5a037b925b1adc78",
                "typesettingProjectionSha256": "c5b00c879412d9f370ed2291b7f24b1cb5dfdd69efd892da20e6a280eecc5f36",
                "repeatIndex": 1,
            },
            "none-related": {
                "directory": "rank1-none-related",
                "stem": "none-related-updated",
                "runSha256": "70f4aec4fca48c1fcce7541e0a7dc30a5924899ff9e291cbbcf2d0c8ba78a573",
                "observationFileSha256": "50d39ce77fd5417a11b3124f8ecce7b2693e2e509b87c8d6b47f199e5d769553",
                "pdfSha256": "e46713378530c836fb129387dfac13757605506a05f5b39a00578ba69f2558a1",
                "observationCanonicalSha256": "ddf887c1f0cc112f3b33f33fad6414fa6ea85286f5f89bfde48468e6920ce02a",
                "manifestSha256": BASELINE_MANIFEST_SHA256,
                "typesettingProjectionSha256": "c5b00c879412d9f370ed2291b7f24b1cb5dfdd69efd892da20e6a280eecc5f36",
                "repeatIndex": 1,
            },
        },
    },
    7: {
        "slug": "kopubworld_dotum_light",
        "face": "KoPubWorld돋움체 Light",
        "exactAlias": "KoPubWorldDotum Light",
        "exactSubset": "INPILL+KoPubWorldDotumLight",
        "fixtureDirectory": "rank7-none-related",
        "fixture": "rank7.hwpx",
        "fixtureManifest": "rank7.manifest.json",
        "states": {
            "exact-only": {
                "directory": "rank7-exact-only",
                "stem": "exact-only-ready",
                "runSha256": "7e848e3a070aedf7abbe59d7a435bb871e1245ac08d547dc34507bdc8748ba7a",
                "observationFileSha256": "dc44727f38e30dde2353e8cb0c7743c20eca7865ad3fcacd7a8f14bcf86a2167",
                "pdfSha256": "51855408638c9bdf48c15eca3426cac3bcdf976f59068f26dacb2da26681b3ec",
                "observationCanonicalSha256": "8c2277e04b72e5a856b6b71ea217410be871a087049888c196adebd1c153edf0",
                "manifestSha256": "14b9a937e513e693bf9654ad7082117e3f73b16fb2a08fba410a8090337edf14",
                "typesettingProjectionSha256": "726ffb04c0b27231cfc1f4aeb3d8b17036bb3563529dcf883bfbc01f02181e4b",
                "repeatIndex": 1,
            },
            "subst-only": {
                "directory": "rank7-subst-only",
                "stem": "subst-only-ready",
                "runSha256": "b896a67b25af771acbf2b628121e800b3b0701e973b429ca04e8ba246a2fe59f",
                "observationFileSha256": "c00aa44fc54423c1bd9b49ce901e4766f206555967697ee1de1d59e01bfa7e1f",
                "pdfSha256": "966fe6438fc03e24f0ef5760402bbd67cb48cfcaffb7fc84024955bc54b16f31",
                "observationCanonicalSha256": "67efdc0b5877cc05d919a5a29accf3a20c58f0e7a863ea30461e525b313f9a02",
                "manifestSha256": "d3098221fa0b03f34d8902c1aa7faf25db5df05b02ba3bfba70a61736902df0d",
                "typesettingProjectionSha256": "c5b00c879412d9f370ed2291b7f24b1cb5dfdd69efd892da20e6a280eecc5f36",
                "repeatIndex": 1,
            },
            "none-related": {
                "directory": "rank7-none-related",
                "stem": "none-related-ready",
                "runSha256": "0ba23ed276e31ac762352c0663d1b48f261e8dddc60ca06e053e1ed0934d2a0b",
                "observationFileSha256": "031768b1ce3f46b3782e8fb08777b0816b373d9b3f4d2577ef8a62807b372d65",
                "pdfSha256": "92979b760d72d6594e50b92342165a3a8375168c3596b1c1fef4229a6fa5bc1e",
                "observationCanonicalSha256": "99f72b7b62c5903571342e2a118aa4ec95c9b802a627a18ef809bfa957ba03cb",
                "manifestSha256": BASELINE_MANIFEST_SHA256,
                "typesettingProjectionSha256": "c5b00c879412d9f370ed2291b7f24b1cb5dfdd69efd892da20e6a280eecc5f36",
                "repeatIndex": 1,
            },
        },
    },
}

QUESTION_STATE = {
    "exact-installed": "exact-only",
    "document-subst-font-only": "subst-only",
    "exact-removed": "none-related",
    "all-related-fonts-missing": "none-related",
}


def evidence(status: str, value: Any = None, reason: str | None = None) -> dict[str, Any]:
    if status == "observed":
        if value is None or reason is not None:
            raise OracleStage2Error("observed evidence requires a value and null reason")
    elif value is not None or not reason:
        raise OracleStage2Error("unobserved evidence requires null value and a reason")
    return {"status": status, "value": value, "reason": reason}


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
        raise OracleStage2Error(f"Stage W5-4 evidence mismatch: {label}")


def read_external_json(root: Path, relative: str, maximum_bytes: int) -> tuple[Path, Any]:
    path = regular_input(root, relative, maximum_bytes)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise OracleStage2Error(f"invalid local-only JSON evidence: {relative}") from error
    reject_absolute_paths(value)
    return path, value


def verify_file(path: Path, expected_sha256: str, label: str) -> None:
    require_equal(sha256_file(path), expected_sha256, label)


def _candidate(readiness: dict[str, Any], rank: int, face: str) -> dict[str, Any]:
    matches = [
        entry
        for entry in readiness["candidates"]
        if entry["queueRank"] == rank and entry["documentFace"] == face
    ]
    if len(matches) != 1 or matches[0].get("sourceReadiness") != "ready-local-sfnt":
        raise OracleStage2Error("target lacks one ready local SFNT source")
    return matches[0]


def _sample(candidate: dict[str, Any], codepoint: int = 0xAC00) -> dict[str, Any]:
    matches = [
        entry
        for entry in candidate["sfnt"]["sampleGlyphEvidence"]
        if entry["codepoint"] == codepoint and entry["status"] == "observed"
    ]
    if len(matches) != 1:
        raise OracleStage2Error("representative SFNT glyph evidence is unavailable")
    return matches[0]


def _target_contract(contract: dict[str, Any], rank: int) -> dict[str, Any]:
    matches = [entry for entry in contract["targets"] if entry["queueRank"] == rank]
    if len(matches) != 1:
        raise OracleStage2Error("Stage W5-4 target is unavailable")
    return matches[0]


def _state_evidence(evidence_root: Path, target: dict[str, Any], state: str) -> dict[str, Any]:
    spec = target["states"][state]
    prefix = f'{spec["directory"]}/{spec["stem"]}'
    run_path, run = read_external_json(evidence_root, f"{prefix}.interactive.json", 1024 * 1024)
    observation_path, observation = read_external_json(
        evidence_root, f"{prefix}.pdf-observation.json", 32 * 1024 * 1024
    )
    pdf_path = regular_input(evidence_root, f"{prefix}.pdf", 64 * 1024 * 1024)
    verify_file(run_path, spec["runSha256"], f"rank {run['queueRank']} {state} run")
    verify_file(
        observation_path,
        spec["observationFileSha256"],
        f"rank {run['queueRank']} {state} observation file",
    )
    verify_file(pdf_path, spec["pdfSha256"], f"rank {run['queueRank']} {state} PDF")

    canonical_claim = observation.get("canonicalSha256")
    canonical_input = dict(observation)
    canonical_input.pop("canonicalSha256", None)
    require_equal(
        sha256_bytes(canonical_json_bytes(canonical_input)),
        canonical_claim,
        f"rank {run['queueRank']} {state} observation self hash",
    )
    require_equal(canonical_claim, spec["observationCanonicalSha256"], "observation hash")
    require_equal(run["export"]["pdfSha256"], spec["pdfSha256"], "run PDF hash")
    require_equal(observation["inputSha256"], spec["pdfSha256"], "observation PDF hash")
    require_equal(run["status"], "observed", "run status")
    require_equal(run["featureDetection"]["opened"], True, "HWPX feature detection")
    require_equal(run["environment"]["processReset"], True, "Hancom process reset")
    require_equal(run["environment"]["securityModuleRegistered"], True, "security module")
    require_equal(run["privacy"]["privateCorpusAccessed"], False, "private corpus access")
    require_equal(run["privacy"]["absolutePathIncluded"], False, "absolute path privacy")
    require_equal(run["privacy"]["fontBytesIncluded"], False, "font byte privacy")
    return {"run": run, "observation": observation, "spec": spec}


def _representative_glyph(observation: dict[str, Any]) -> dict[str, Any]:
    if len(observation["glyphObservations"]) <= 18:
        raise OracleStage2Error("PDF observation lacks glyph index 18")
    glyph = observation["glyphObservations"][18]
    require_equal(glyph["unicode"], "가", "representative glyph U+AC00")
    require_equal(glyph["page"], 1, "representative glyph page")
    return glyph


def _profile_name(target: dict[str, Any], question: str) -> str:
    return f'windows_hwp2020_{target["slug"]}_{question.replace("-", "_")}.json'


def build_profile(
    *,
    target: dict[str, Any],
    target_contract: dict[str, Any],
    source: dict[str, Any],
    question: str,
    state_record: dict[str, Any],
) -> dict[str, Any]:
    state = QUESTION_STATE[question]
    run = state_record["run"]
    observation = state_record["observation"]
    spec = state_record["spec"]
    face = target["face"]
    subst = target_contract["documentSubstitution"]
    glyph = _representative_glyph(observation)
    requested = run["fontSelections"][0]
    require_equal(requested["requestedFace"], face, "document face probe")
    require_equal(run["queueRank"], target_contract["queueRank"], "queue rank")
    require_equal(run["documentFace"], face, "document face")
    require_equal(run["inputSha256"], target_contract["fixture"]["sha256"], "fixture input")
    require_equal(run["environment"]["hancomVersion"], "11, 0, 0, 9136", "Hancom build")
    require_equal(observation["pageCount"], run["featureDetection"]["pageCount"], "page count")

    embedded_fonts = {entry["name"]: entry for entry in observation["fonts"]}
    if glyph["font"] not in embedded_fonts or not embedded_fonts[glyph["font"]]["embedded"]:
        raise OracleStage2Error("representative PDF font is not embedded")
    expected_subset = target["exactSubset"] if state == "exact-only" else HCR_SUBSET
    require_equal(glyph["font"], expected_subset, "representative subset selection")

    if state == "exact-only":
        aliases = [
            entry
            for entry in run["fontSelections"]
            if entry["requestedFace"] == target["exactAlias"] and entry["exact"] is True
        ]
        if len(aliases) != 1:
            raise OracleStage2Error("exact-only state lacks one exact SFNT alias readback")
        source_glyph = _sample(source)
        installed_sha = evidence("observed", target_contract["exactFont"]["sha256"])
        outline = evidence("observed", source_glyph["outlineSha256"])
        hmtx = evidence(
            "observed",
            {
                "advance": source_glyph["hmtxAdvance"],
                "unitsPerEm": source["sfnt"]["unitsPerEm"],
                "faceIndex": source["sfnt"]["faceIndex"],
                "sourceFontSha256": source["sfnt"]["sha256"],
            },
        )
        relation_type = "identity-alias"
        relation_anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "requestedDocumentFace": face,
            "documentFaceReadback": requested["readbackFace"],
            "exactSfntAlias": target["exactAlias"],
            "sourceNameTable": source["sfnt"]["nameTable"],
            "sourceFontSha256": source["sfnt"]["sha256"],
            "sourceGlyphOutlineSha256": source_glyph["outlineSha256"],
            "exportSubsetFontName": glyph["font"],
            "exportUsedExactBytes": True,
            "firstDivergenceComparedWith": "all-related-fonts-missing",
        }
    elif state == "subst-only":
        installed_sha = evidence("observed", subst["sha256"])
        outline = evidence(
            "unavailable",
            reason="The selected HCR fallback source bytes were outside the managed two-font set.",
        )
        hmtx = evidence(
            "unavailable",
            reason="The selected HCR fallback hmtx source was not frozen by this controlled run.",
        )
        relation_type = "document-substitution"
        relation_anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "fixtureRelationAnchor": subst["relationAnchor"],
            "declaredSubstitutionFace": subst["face"],
            "installedSubstitutionFontSha256": subst["sha256"],
            "exportSubsetFontName": glyph["font"],
            "exportUsedSubstitution": False,
            "firstDivergenceComparedWith": "exact-installed",
        }
    else:
        installed_sha = evidence(
            "not-applicable",
            reason="The controlled none-related state contains neither managed related font.",
        )
        outline = evidence(
            "unavailable",
            reason="The selected HCR fallback source bytes were outside the managed two-font set.",
        )
        hmtx = evidence(
            "unavailable",
            reason="The selected HCR fallback hmtx source was not frozen by this controlled run.",
        )
        relation_type = "hancom-missing-font"
        relation_anchor = {
            "runEvidenceSha256": spec["runSha256"],
            "pdfSha256": spec["pdfSha256"],
            "pdfObservationCanonicalSha256": spec["observationCanonicalSha256"],
            "typesettingProjectionSha256": spec["typesettingProjectionSha256"],
            "managedExactPresent": False,
            "managedDocumentSubstitutionPresent": False,
            "exportSubsetFontName": glyph["font"],
            "firstDivergenceComparedWith": "exact-installed",
        }

    divergence = evidence(
        "observed",
        {"plane": "selection", "characterIndex": 18, "lineIndex": 0, "pageIndex": 0},
    )

    profile = {
        "schemaVersion": 2,
        "kind": "font-oracle-profile",
        "issue": 4963,
        "candidate": {"queueRank": target_contract["queueRank"], "documentFace": face},
        "questionId": question,
        "exactMissingState": question,
        "input": {
            "sourceFormat": evidence("observed", "hwpx"),
            "sha256": evidence("observed", target_contract["fixture"]["sha256"]),
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
            "repeatIndex": spec["repeatIndex"],
        },
        "fontState": {
            "requestedFace": face,
            "relatedFaceSet": [face, subst["face"]],
            "installedFontSha256": installed_sha,
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
            "firstTypesettingDivergence": divergence,
            "lineCount": evidence("observed", observation["visualLineCount"]),
            "pageCount": evidence("observed", observation["pageCount"]),
        },
        "relationEvidence": {"type": relation_type, "anchor": evidence("observed", relation_anchor)},
        "privacy": {
            "fontBytesEmbedded": False,
            "privateDocumentIdentityIncluded": False,
            "absoluteFontPathIncluded": False,
        },
    }
    reject_absolute_paths(profile, "profile")
    return profile


def _ladder(
    *,
    target: dict[str, Any],
    target_contract: dict[str, Any],
    profile_hashes: dict[str, str],
) -> dict[str, Any]:
    subst = target_contract["documentSubstitution"]
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
        spec = target["states"][state]
        runs.append(
            {
                "executionId": f'rank-{target_contract["queueRank"]}-{state}-20260822',
                "physicalState": state,
                "questions": questions,
                "inputSha256": target_contract["fixture"]["sha256"],
                "unrelatedFontProjectionSha256": UNRELATED_PROJECTION_SHA256,
                "managedFonts": [
                    {
                        "face": target_contract["documentFace"],
                        "sha256": target_contract["exactFont"]["sha256"],
                        "present": exact_present,
                    },
                    {"face": subst["face"], "sha256": subst["sha256"], "present": subst_present},
                ],
                "processReset": True,
                "fontCacheAction": "guest-reboot",
                "outputProfileSha256": profile_hashes[profile_question],
                "restore": {
                    "restoredBeforeRun": True,
                    "restoredAfterRun": True,
                    "baselineManifestSha256": BASELINE_MANIFEST_SHA256,
                    "recoveredManifestSha256": BASELINE_MANIFEST_SHA256,
                },
            }
        )
        require_equal(spec["manifestSha256"] == BASELINE_MANIFEST_SHA256, state == "none-related", "managed manifest state")
    ladder = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage4-ladder-evidence",
        "issue": 4963,
        "evidenceClass": "acceptance-primary",
        "target": {
            "queueRank": target_contract["queueRank"],
            "documentFace": target_contract["documentFace"],
        },
        "fixtureSha256": target_contract["fixture"]["sha256"],
        "attestation": ATTESTATION,
        "unrelatedFontProjectionSha256": UNRELATED_PROJECTION_SHA256,
        "runs": runs,
        "dispositions": [
            {
                "question": "curated-official-successor-only",
                "status": "not-provided",
                "reason": target_contract["officialSuccessor"]["reason"],
            }
        ],
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
        },
    }
    reject_absolute_paths(ladder, "ladder")
    return ladder


def generate_artifacts(evidence_root: Path) -> dict[str, Any]:
    contract = read_json(CONTRACT_PATH)
    profile_contract = read_json(PROFILE_CONTRACT_PATH)
    readiness = read_json(READINESS_PATH)
    artifacts: dict[str, Any] = {
        "oracle_stage4_acceptance_attestation.json": ATTESTATION,
    }
    projection_targets = []

    for rank, target in TARGETS.items():
        target_contract = _target_contract(contract, rank)
        require_equal(target_contract["documentFace"], target["face"], "target face")
        require_equal(
            profile_contract["inputPreconditions"]["queueFaces"][rank - 1],
            target["face"],
            "profile queue face",
        )
        fixture_prefix = f'{target["fixtureDirectory"]}/'
        fixture = regular_input(
            evidence_root, fixture_prefix + target["fixture"], 16 * 1024 * 1024
        )
        manifest_path, manifest = read_external_json(
            evidence_root, fixture_prefix + target["fixtureManifest"], 4 * 1024 * 1024
        )
        verify_file(fixture, target_contract["fixture"]["sha256"], f"rank {rank} fixture")
        verify_file(
            manifest_path,
            target_contract["fixture"]["manifestSha256"],
            f"rank {rank} fixture manifest",
        )
        require_equal(manifest["inputSha256"], target_contract["fixture"]["sha256"], "fixture manifest input")
        require_equal(manifest["semanticSha256"], target_contract["fixture"]["semanticSha256"], "fixture semantic")
        require_equal(manifest["semantic"]["documentFace"], target["face"], "fixture document face")
        require_equal(
            manifest["semantic"]["substitutionFace"],
            target_contract["documentSubstitution"]["face"],
            "fixture substitution face",
        )

        states = {
            state: _state_evidence(evidence_root, target, state)
            for state in ("exact-only", "subst-only", "none-related")
        }
        require_equal(
            states["subst-only"]["spec"]["typesettingProjectionSha256"],
            states["none-related"]["spec"]["typesettingProjectionSha256"],
            "substitution and missing projection equivalence",
        )
        source = _candidate(readiness, rank, target["face"])
        profiles: dict[str, Any] = {}
        profile_hashes: dict[str, str] = {}
        for question in QUESTION_STATE:
            profile = build_profile(
                target=target,
                target_contract=target_contract,
                source=source,
                question=question,
                state_record=states[QUESTION_STATE[question]],
            )
            name = _profile_name(target, question)
            artifacts[f"profiles/{name}"] = profile
            profiles[question] = profile
            profile_hashes[question] = sha256_bytes(pretty_json_bytes(profile))

        ladder_name = f'oracle_stage4_rank{rank}_acceptance_ladder.json'
        ladder = _ladder(
            target=target,
            target_contract=target_contract,
            profile_hashes=profile_hashes,
        )
        artifacts[ladder_name] = ladder
        projection_targets.append(
            {
                "queueRank": rank,
                "documentFace": target["face"],
                "fixtureSha256": target_contract["fixture"]["sha256"],
                "profiles": [
                    {
                        "questionId": question,
                        "sha256": profile_hashes[question],
                        "typesettingProjectionSha256": target["states"][QUESTION_STATE[question]][
                            "typesettingProjectionSha256"
                        ],
                    }
                    for question in QUESTION_STATE
                ],
            }
        )

    projection = {
        "schemaVersion": 1,
        "kind": "font-oracle-stage4-acceptance-projection",
        "issue": 4963,
        "evidenceClass": "acceptance-primary",
        "environment": {
            "vmIdentitySha256": ATTESTATION["vmIdentitySha256"],
            "baselineSnapshotIdentitySha256": ATTESTATION["baselineSnapshotIdentitySha256"],
            "baselineFontManifestSha256": BASELINE_MANIFEST_SHA256,
            "unrelatedFontProjectionSha256": UNRELATED_PROJECTION_SHA256,
        },
        "targets": projection_targets,
        "privacy": {
            "absolutePathIncluded": False,
            "hostNameIncluded": False,
            "fontBytesIncluded": False,
            "privateDocumentIdentityIncluded": False,
        },
    }
    reject_absolute_paths(projection, "projection")
    artifacts["oracle_stage4_acceptance_projection.json"] = projection
    return artifacts


def write_artifacts(output_root: Path, artifacts: dict[str, Any]) -> dict[str, str]:
    hashes = {}
    for relative, value in sorted(artifacts.items()):
        payload = pretty_json_bytes(value)
        target = output_path(output_root, relative)
        write_bytes(target, payload, mode=0o644)
        hashes[relative] = sha256_bytes(payload)
    return hashes


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    hashes = write_artifacts(args.output_root, generate_artifacts(args.evidence_root))
    print(json.dumps(hashes, ensure_ascii=False, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OracleStage2Error as error:
        raise SystemExit(str(error)) from error
