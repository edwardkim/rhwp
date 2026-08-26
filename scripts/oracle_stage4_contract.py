#!/usr/bin/env python3
"""Fail-closed contract validator for Issue #4963 Stage W5-4 ladders."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

from oracle_stage2_common import INVESTIGATION, ROOT, read_json, sha256_file


CONTRACT_PATH = INVESTIGATION / "oracle_stage4_contract.json"
PREFLIGHT_PATH = INVESTIGATION / "oracle_stage4_current_host_preflight.json"
FIXTURES_PATH = INVESTIGATION / "oracle_stage4_public_fixtures.json"
PROFILE_CONTRACT_PATH = INVESTIGATION / "oracle_profile_contract.json"
READINESS_PATH = INVESTIGATION / "font_oracle_readiness.json"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
ABSOLUTE_PATH = re.compile(r"^(?:/home/|/mnt/|[A-Za-z]:[\\/])")


def exact_keys(value: Any, keys: set[str], label: str, errors: list[str]) -> bool:
    if not isinstance(value, dict):
        errors.append(f"{label} must be an object")
        return False
    if set(value) != keys:
        errors.append(f"{label} schema drift")
        return False
    return True


def digest(value: Any) -> bool:
    return isinstance(value, str) and SHA256.fullmatch(value) is not None


def relative_path(value: Any) -> bool:
    if not isinstance(value, str) or not value or "\\" in value:
        return False
    path = Path(value)
    return not path.is_absolute() and ".." not in path.parts


def walk_strings(value: Any, errors: list[str], label: str = "value") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            walk_strings(child, errors, f"{label}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_strings(child, errors, f"{label}[{index}]")
    elif isinstance(value, str) and ABSOLUTE_PATH.match(value):
        errors.append(f"{label} exposes an absolute path")


def validate_evidence(value: Any, label: str, errors: list[str]) -> None:
    if not exact_keys(value, {"status", "value", "reason"}, label, errors):
        return
    if value["status"] not in {"observed", "unavailable", "blocked"}:
        errors.append(f"{label}.status is invalid")
    if value["status"] == "observed":
        if value["value"] is None or value["reason"] is not None:
            errors.append(f"{label} observed evidence is invalid")
    elif value["value"] is not None or not isinstance(value["reason"], str) or not value["reason"]:
        errors.append(f"{label} unobserved evidence is invalid")


def _readiness_by_face() -> dict[str, dict[str, Any]]:
    return {
        entry["documentFace"]: entry for entry in read_json(READINESS_PATH)["candidates"]
    }


def validate_contract(contract: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not exact_keys(
        contract,
        {
            "schemaVersion",
            "kind",
            "issue",
            "stage",
            "predecessor",
            "fixturePolicy",
            "environmentPolicy",
            "statePolicy",
            "targets",
            "mutationPolicy",
        },
        "contract",
        errors,
    ):
        return errors
    if (
        contract["schemaVersion"] != 1
        or contract["kind"] != "font-oracle-stage4-contract"
        or contract["issue"] != 4963
        or contract["stage"] != "W5-4"
    ):
        errors.append("Stage W5-4 contract identity mismatch")

    predecessor = contract["predecessor"]
    if exact_keys(
        predecessor,
        {"stage3Contract", "stage3ContractSha256", "readiness", "readinessSha256"},
        "predecessor",
        errors,
    ):
        for path_key, hash_key in (
            ("stage3Contract", "stage3ContractSha256"),
            ("readiness", "readinessSha256"),
        ):
            if not relative_path(predecessor[path_key]):
                errors.append(f"predecessor.{path_key} must be repository-relative")
                continue
            path = ROOT / predecessor[path_key]
            if not path.is_file() or sha256_file(path) != predecessor[hash_key]:
                errors.append(f"predecessor.{path_key} hash mismatch")

    fixture = contract["fixturePolicy"]
    if exact_keys(
        fixture,
        {
            "generator",
            "generatorSha256",
            "contractVersion",
            "sameInputBytesAcrossStates",
            "documentSubstitutionMustBeExplicit",
            "fontBytesEmbedded",
        },
        "fixturePolicy",
        errors,
    ):
        generator = ROOT / fixture["generator"] if relative_path(fixture["generator"]) else None
        if generator is None or not generator.is_file() or sha256_file(generator) != fixture["generatorSha256"]:
            errors.append("fixture generator hash mismatch")
        if fixture["contractVersion"] != "w5-oracle-typesetting-v1-subst-v1":
            errors.append("fixture contract version mismatch")
        if (
            fixture["sameInputBytesAcrossStates"] is not True
            or fixture["documentSubstitutionMustBeExplicit"] is not True
            or fixture["fontBytesEmbedded"] is not False
        ):
            errors.append("fixture protection boundary drift")

    environment = contract["environmentPolicy"]
    if (
        environment.get("currentHostMutationAllowed") is not False
        or environment.get("disposableAttestationRequired") is not True
        or environment.get("externalControlPlaneRequired") is not True
        or environment.get("restoreBeforeEachUniqueState") is not True
        or environment.get("restoreAfterEachUniqueState") is not True
        or environment.get("baselineAndRecoveredManifestMustMatch") is not True
        or environment.get("unrelatedFontProjectionMustRemainIdentical") is not True
        or environment.get("newHancomProcessPerRun") is not True
        or environment.get("featureDetectionRequired") is not True
        or environment.get("versionBranching") is not False
    ):
        errors.append("environment safety boundary drift")
    providers = environment.get("acceptedProviders")
    if not isinstance(providers, list) or len(providers) != 3 or len(set(providers)) != 3:
        errors.append("accepted snapshot provider inventory is invalid")

    state = contract["statePolicy"]
    expected_questions = [
        "exact-installed",
        "exact-removed",
        "document-subst-font-only",
        "curated-official-successor-only",
        "all-related-fonts-missing",
    ]
    expected_mapping = {
        "exact-installed": "exact-only",
        "exact-removed": "none-related",
        "document-subst-font-only": "subst-only",
        "curated-official-successor-only": "not-provided-no-direct-anchor",
        "all-related-fonts-missing": "none-related",
    }
    if state.get("orderedQuestions") != expected_questions:
        errors.append("ordered state questions drifted")
    if state.get("uniquePhysicalStates") != ["exact-only", "subst-only", "none-related"]:
        errors.append("unique physical state inventory drifted")
    if state.get("stateMapping") != expected_mapping:
        errors.append("state mapping drifted")
    if state.get("equivalentExecutionReuse") != [["exact-removed", "all-related-fonts-missing"]]:
        errors.append("equivalent execution reuse drifted")
    if state.get("successorRunWithoutDirectAnchor") is not False:
        errors.append("successor run requires a direct anchor")

    queue = read_json(PROFILE_CONTRACT_PATH)["inputPreconditions"]["queueFaces"]
    readiness = _readiness_by_face()
    targets = contract["targets"]
    if not isinstance(targets, list) or [entry.get("queueRank") for entry in targets] != [1, 13, 7]:
        errors.append("Stage W5-4 target order drifted")
        targets = []
    seen_faces: set[str] = set()
    for index, target in enumerate(targets):
        label = f"targets[{index}]"
        if not exact_keys(
            target,
            {
                "queueRank",
                "documentFace",
                "exactFont",
                "documentSubstitution",
                "officialSuccessor",
                "fixture",
                "immutableExactPolicy",
            },
            label,
            errors,
        ):
            continue
        rank = target["queueRank"]
        face = target["documentFace"]
        if face in seen_faces or not 1 <= rank <= len(queue) or queue[rank - 1] != face:
            errors.append(f"{label} queue identity mismatch")
        seen_faces.add(face)
        exact = target["exactFont"]
        source = readiness.get(face, {}).get("sfnt", {})
        if (
            not relative_path(exact.get("relativePath"))
            or exact.get("sha256") != source.get("sha256")
        ):
            errors.append(f"{label} exact source mismatch")
        substitution = target["documentSubstitution"]
        subst_source = readiness.get(substitution.get("face"), {}).get("sfnt", {})
        if (
            substitution.get("relationAnchor") != "fixture-declared-substFont"
            or not relative_path(substitution.get("relativePath"))
            or substitution.get("sha256") != subst_source.get("sha256")
        ):
            errors.append(f"{label} substitution source mismatch")
        successor = target["officialSuccessor"]
        if successor.get("status") != "not-provided" or not successor.get("reason"):
            errors.append(f"{label} unsupported successor claim")
        fixture_entry = target["fixture"]
        if (
            not digest(fixture_entry.get("sha256"))
            or not digest(fixture_entry.get("manifestSha256"))
            or not digest(fixture_entry.get("semanticSha256"))
            or not isinstance(fixture_entry.get("bytes"), int)
            or fixture_entry["bytes"] <= 0
        ):
            errors.append(f"{label} fixture evidence is invalid")
        if target["immutableExactPolicy"] != "stop-if-exact-readback-survives-none-related":
            errors.append(f"{label} immutable exact policy drifted")

    mutation = contract["mutationPolicy"]
    if (
        mutation.get("allowedScope") != "attested-disposable-guest-only"
        or mutation.get("allowedFontSet") != "target-exact-and-fixture-substitution-only"
        or any(value is not False for key, value in mutation.items() if key not in {"allowedScope", "allowedFontSet"})
    ):
        errors.append("mutation scope drifted")
    walk_strings(contract, errors, "contract")
    return errors


def validate_preflight(preflight: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not exact_keys(
        preflight,
        {
            "schemaVersion",
            "kind",
            "issue",
            "stage",
            "observedAt",
            "environment",
            "disposableEvidence",
            "qualified",
            "mutationAllowed",
            "decision",
            "scope",
        },
        "preflight",
        errors,
    ):
        return errors
    if (
        preflight["schemaVersion"] != 1
        or preflight["kind"] != "font-oracle-stage4-environment-preflight"
        or preflight["issue"] != 4963
        or preflight["stage"] != "W5-4"
    ):
        errors.append("preflight identity mismatch")
    for field in ("hyperVCommand", "vmInventory", "checkpointIdentity", "restoreVerification"):
        validate_evidence(preflight["disposableEvidence"].get(field), f"preflight.{field}", errors)
    if preflight["qualified"] is False:
        if preflight["mutationAllowed"] is not False:
            errors.append("an unqualified environment cannot allow mutation")
    elif preflight["qualified"] is True:
        if preflight["mutationAllowed"] is not True:
            errors.append("a qualified environment must explicitly allow mutation")
        for field in ("vmInventory", "checkpointIdentity", "restoreVerification"):
            if preflight["disposableEvidence"][field].get("status") != "observed":
                errors.append(f"qualified preflight requires observed {field}")
    else:
        errors.append("preflight qualified must be boolean")
    if any(value is not False for value in preflight["scope"].values()):
        errors.append("preflight scope reports an unauthorized mutation")
    walk_strings(preflight, errors, "preflight")
    return errors


def validate_attestation(
    attestation: dict[str, Any], contract: dict[str, Any], *, allow_contract_fixture: bool
) -> list[str]:
    errors: list[str] = []
    if not exact_keys(
        attestation,
        {
            "schemaVersion",
            "kind",
            "issue",
            "evidenceClass",
            "provider",
            "vmIdentitySha256",
            "baselineSnapshotIdentitySha256",
            "baselineFontManifestSha256",
            "externalControlPlane",
            "restoreProbe",
            "privacy",
        },
        "attestation",
        errors,
    ):
        return errors
    if (
        attestation["schemaVersion"] != 1
        or attestation["kind"] != "font-oracle-disposable-environment-attestation"
        or attestation["issue"] != 4963
    ):
        errors.append("attestation identity mismatch")
    evidence_class = attestation["evidenceClass"]
    if evidence_class == "synthetic-contract-fixture":
        if not allow_contract_fixture or attestation["provider"] != "contract-fixture":
            errors.append("synthetic attestation is not allowed here")
    elif evidence_class == "acceptance-primary":
        if attestation["provider"] not in contract["environmentPolicy"]["acceptedProviders"]:
            errors.append("attestation provider is not approved")
    else:
        errors.append("attestation evidence class is invalid")
    for field in (
        "vmIdentitySha256",
        "baselineSnapshotIdentitySha256",
        "baselineFontManifestSha256",
    ):
        if not digest(attestation[field]):
            errors.append(f"attestation {field} is invalid")
    if attestation["externalControlPlane"] is not True:
        errors.append("snapshot control plane must be external to the guest")
    restore = attestation["restoreProbe"]
    if (
        restore.get("performed") is not True
        or restore.get("beforeManifestSha256") != attestation["baselineFontManifestSha256"]
        or restore.get("recoveredManifestSha256") != attestation["baselineFontManifestSha256"]
    ):
        errors.append("attestation restore probe does not recover the baseline manifest")
    if any(value is not False for value in attestation["privacy"].values()):
        errors.append("attestation privacy boundary drifted")
    walk_strings(attestation, errors, "attestation")
    return errors


def validate_ladder(
    ladder: dict[str, Any], contract: dict[str, Any], *, allow_contract_fixture: bool
) -> list[str]:
    errors: list[str] = []
    if not exact_keys(
        ladder,
        {
            "schemaVersion",
            "kind",
            "issue",
            "evidenceClass",
            "target",
            "fixtureSha256",
            "attestation",
            "unrelatedFontProjectionSha256",
            "runs",
            "dispositions",
            "privacy",
        },
        "ladder",
        errors,
    ):
        return errors
    if (
        ladder["schemaVersion"] != 1
        or ladder["kind"] != "font-oracle-stage4-ladder-evidence"
        or ladder["issue"] != 4963
    ):
        errors.append("ladder identity mismatch")
    errors.extend(
        validate_attestation(
            ladder["attestation"], contract, allow_contract_fixture=allow_contract_fixture
        )
    )
    if ladder["evidenceClass"] != ladder["attestation"].get("evidenceClass"):
        errors.append("ladder and attestation evidence classes differ")

    target_matches = [
        entry
        for entry in contract["targets"]
        if entry["queueRank"] == ladder["target"].get("queueRank")
        and entry["documentFace"] == ladder["target"].get("documentFace")
    ]
    if len(target_matches) != 1:
        errors.append("ladder target is not in the Stage W5-4 contract")
        return errors
    target = target_matches[0]
    if ladder["fixtureSha256"] != target["fixture"]["sha256"]:
        errors.append("ladder input differs from the frozen target fixture")
    if not digest(ladder["unrelatedFontProjectionSha256"]):
        errors.append("ladder unrelated font projection digest is invalid")

    expected_questions = {
        "exact-only": ["exact-installed"],
        "subst-only": ["document-subst-font-only"],
        "none-related": ["exact-removed", "all-related-fonts-missing"],
    }
    expected_presence = {
        "exact-only": (True, False),
        "subst-only": (False, True),
        "none-related": (False, False),
    }
    runs = ladder["runs"]
    if not isinstance(runs, list) or len(runs) != 3:
        errors.append("ladder must contain exactly three unique physical runs")
        runs = []
    states = [run.get("physicalState") for run in runs]
    if len(set(states)) != 3 or set(states) != set(expected_questions):
        errors.append("ladder physical state coverage is invalid")
    execution_ids = [run.get("executionId") for run in runs]
    if any(not isinstance(value, str) or not value for value in execution_ids) or len(set(execution_ids)) != len(execution_ids):
        errors.append("ladder execution ids must be unique non-empty strings")

    exact = target["exactFont"]
    substitution = target["documentSubstitution"]
    baseline = ladder["attestation"].get("baselineFontManifestSha256")
    for index, run in enumerate(runs):
        label = f"runs[{index}]"
        state = run.get("physicalState")
        if run.get("questions") != expected_questions.get(state):
            errors.append(f"{label} question mapping is invalid")
        if run.get("inputSha256") != target["fixture"]["sha256"]:
            errors.append(f"{label} input hash drifted")
        if run.get("unrelatedFontProjectionSha256") != ladder["unrelatedFontProjectionSha256"]:
            errors.append(f"{label} unrelated ambient font state drifted")
        managed = run.get("managedFonts")
        if not isinstance(managed, list) or len(managed) != 2:
            errors.append(f"{label} managed font inventory must have two entries")
            continue
        by_face = {entry.get("face"): entry for entry in managed}
        if set(by_face) != {target["documentFace"], substitution["face"]}:
            errors.append(f"{label} managed font set drifted")
            continue
        expected_exact, expected_subst = expected_presence.get(state, (None, None))
        for face, source, present in (
            (target["documentFace"], exact, expected_exact),
            (substitution["face"], substitution, expected_subst),
        ):
            entry = by_face[face]
            if entry.get("sha256") != source["sha256"] or entry.get("present") is not present:
                errors.append(f"{label} managed state mismatch for {face}")
        if run.get("processReset") is not True:
            errors.append(f"{label} did not reset the Hancom process")
        if run.get("fontCacheAction") not in {"guest-reboot", "font-cache-refresh-and-process-reset"}:
            errors.append(f"{label} lacks an approved font cache action")
        if not digest(run.get("outputProfileSha256")):
            errors.append(f"{label} output profile digest is invalid")
        restore = run.get("restore", {})
        if (
            restore.get("restoredBeforeRun") is not True
            or restore.get("restoredAfterRun") is not True
            or restore.get("baselineManifestSha256") != baseline
            or restore.get("recoveredManifestSha256") != baseline
        ):
            errors.append(f"{label} snapshot restore verification failed")

    dispositions = ladder["dispositions"]
    if (
        not isinstance(dispositions, list)
        or len(dispositions) != 1
        or dispositions[0].get("question") != "curated-official-successor-only"
        or dispositions[0].get("status") != "not-provided"
        or not dispositions[0].get("reason")
    ):
        errors.append("official successor disposition is invalid")
    if any(value is not False for value in ladder["privacy"].values()):
        errors.append("ladder privacy boundary drifted")
    walk_strings(ladder, errors, "ladder")
    return errors


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("check",))
    parser.add_argument("--attestation", type=Path)
    parser.add_argument("--ladder", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_arguments()
    contract = read_json(CONTRACT_PATH)
    errors = validate_contract(contract)
    errors.extend(validate_preflight(read_json(PREFLIGHT_PATH)))
    fixtures = read_json(FIXTURES_PATH)
    errors.extend(
        validate_ladder(fixtures["validLadder"], contract, allow_contract_fixture=True)
    )
    if args.attestation:
        errors.extend(
            validate_attestation(
                read_json(args.attestation), contract, allow_contract_fixture=False
            )
        )
    if args.ladder:
        errors.extend(
            validate_ladder(read_json(args.ladder), contract, allow_contract_fixture=False)
        )
    if errors:
        print(json.dumps({"ok": False, "errors": errors}, ensure_ascii=False, indent=2))
        return 1
    print(
        json.dumps(
            {
                "ok": True,
                "issue": 4963,
                "targets": len(contract["targets"]),
                "uniquePhysicalStatesPerTarget": len(
                    contract["statePolicy"]["uniquePhysicalStates"]
                ),
                "currentHostQualified": read_json(PREFLIGHT_PATH)["qualified"],
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
