#!/usr/bin/env python3
"""Stage W5-4 snapshot, state and ambient-font invariants."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
INVESTIGATION = ROOT / "mydocs/tech/investigations/issue-4963"
sys.path.insert(0, str(SCRIPTS))

from generate_oracle_typesetting_fixture import generate_fixture  # noqa: E402
from oracle_stage2_common import (  # noqa: E402
    canonical_json_bytes,
    read_contract,
    sha256_bytes,
    sha256_file,
)
from oracle_stage4_contract import (  # noqa: E402
    validate_attestation,
    validate_contract,
    validate_ladder,
    validate_preflight,
)
from oracle_stage4_reproduction_compare import compare  # noqa: E402


def read_json(path: Path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


class OracleStage4Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = read_json(INVESTIGATION / "oracle_stage4_contract.json")
        cls.preflight = read_json(
            INVESTIGATION / "oracle_stage4_current_host_preflight.json"
        )
        cls.ladder = read_json(
            INVESTIGATION / "oracle_stage4_public_fixtures.json"
        )["validLadder"]

    def test_contract_preflight_and_public_ladder_are_valid(self):
        self.assertEqual(validate_contract(self.contract), [])
        self.assertEqual(validate_preflight(self.preflight), [])
        self.assertEqual(
            validate_ladder(self.ladder, self.contract, allow_contract_fixture=True),
            [],
        )
        self.assertFalse(self.preflight["qualified"])
        self.assertFalse(self.preflight["mutationAllowed"])

    def test_three_target_fixtures_are_byte_exact_and_hash_frozen(self):
        stage2 = read_contract()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for target in self.contract["targets"]:
                rank = target["queueRank"]
                output = f"rank-{rank}.hwpx"
                manifest = f"rank-{rank}.json"
                generated = generate_fixture(
                    contract=stage2,
                    output_root=root,
                    output_relative=output,
                    manifest_relative=manifest,
                    document_face=target["documentFace"],
                    substitution_face=target["documentSubstitution"]["face"],
                )
                self.assertEqual(sha256_file(root / output), target["fixture"]["sha256"])
                self.assertEqual(
                    sha256_file(root / manifest), target["fixture"]["manifestSha256"]
                )
                self.assertEqual(
                    generated["semanticSha256"], target["fixture"]["semanticSha256"]
                )
                self.assertEqual(
                    generated["semantic"]["substitutionFace"],
                    target["documentSubstitution"]["face"],
                )

    def test_unqualified_host_cannot_enable_mutation(self):
        changed = copy.deepcopy(self.preflight)
        changed["mutationAllowed"] = True
        self.assertIn(
            "an unqualified environment cannot allow mutation",
            validate_preflight(changed),
        )

        changed = copy.deepcopy(self.preflight)
        changed["qualified"] = True
        changed["mutationAllowed"] = True
        errors = validate_preflight(changed)
        self.assertIn("qualified preflight requires observed vmInventory", errors)
        self.assertIn("qualified preflight requires observed checkpointIdentity", errors)
        self.assertIn("qualified preflight requires observed restoreVerification", errors)

    def test_attestation_requires_external_restore_control(self):
        changed = copy.deepcopy(self.ladder["attestation"])
        changed["externalControlPlane"] = False
        self.assertIn(
            "snapshot control plane must be external to the guest",
            validate_attestation(changed, self.contract, allow_contract_fixture=True),
        )

        changed = copy.deepcopy(self.ladder["attestation"])
        changed["restoreProbe"]["recoveredManifestSha256"] = "9" * 64
        self.assertIn(
            "attestation restore probe does not recover the baseline manifest",
            validate_attestation(changed, self.contract, allow_contract_fixture=True),
        )

    def test_input_and_unrelated_ambient_drift_fail_closed(self):
        changed = copy.deepcopy(self.ladder)
        changed["runs"][0]["inputSha256"] = "9" * 64
        self.assertIn(
            "runs[0] input hash drifted",
            validate_ladder(changed, self.contract, allow_contract_fixture=True),
        )

        changed = copy.deepcopy(self.ladder)
        changed["runs"][1]["unrelatedFontProjectionSha256"] = "8" * 64
        self.assertIn(
            "runs[1] unrelated ambient font state drifted",
            validate_ladder(changed, self.contract, allow_contract_fixture=True),
        )

    def test_managed_font_membership_and_restore_fail_closed(self):
        changed = copy.deepcopy(self.ladder)
        changed["runs"][1]["managedFonts"][0]["present"] = True
        errors = validate_ladder(changed, self.contract, allow_contract_fixture=True)
        self.assertTrue(any("managed state mismatch" in error for error in errors))

        changed = copy.deepcopy(self.ladder)
        changed["runs"][2]["restore"]["restoredAfterRun"] = False
        self.assertIn(
            "runs[2] snapshot restore verification failed",
            validate_ladder(changed, self.contract, allow_contract_fixture=True),
        )

    def test_successor_without_direct_anchor_is_not_run(self):
        changed = copy.deepcopy(self.ladder)
        changed["dispositions"][0]["status"] = "observed"
        self.assertIn(
            "official successor disposition is invalid",
            validate_ladder(changed, self.contract, allow_contract_fixture=True),
        )

    def test_public_contract_artifacts_are_path_and_byte_free(self):
        for path in (
            INVESTIGATION / "oracle_stage4_contract.json",
            INVESTIGATION / "oracle_stage4_current_host_preflight.json",
            INVESTIGATION / "oracle_stage4_public_fixtures.json",
        ):
            text = path.read_text(encoding="utf-8")
            self.assertNotIn("/home/", text)
            self.assertNotIn("/mnt/", text)
            self.assertNotRegex(text, r"[A-Za-z]:[\\/]")
        self.assertEqual(list(INVESTIGATION.glob("*.ttf")), [])
        self.assertEqual(list(INVESTIGATION.glob("*.hft")), [])

    def test_hyperv_reproduction_compare_reconciles_three_restored_states(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture_hash = "1" * 64
            baseline_hash = "2" * 64
            unrelated_hash = "3" * 64
            exact_hash = "4" * 64
            subst_hash = "5" * 64
            state_specs = {
                "exact-only": ("rank8-exact-only", "exact-only", [exact_hash], "Exact"),
                "subst-only": ("rank8-subst-only", "subst-only", [subst_hash], "Fallback"),
                "none-related": ("rank8-none-related", "none-related", [], "Fallback"),
            }
            config_states = {}
            for state, (state_directory, stem, managed, font_name) in state_specs.items():
                state_root = root / state_directory
                state_root.mkdir()
                pdf = state_root / f"{stem}.pdf"
                pdf.write_bytes(f"pdf-{state}".encode())
                pdf_hash = sha256_file(pdf)
                run = {
                    "status": "observed",
                    "queueRank": 8,
                    "documentFace": "KoPubWorld바탕체 Light",
                    "inputSha256": fixture_hash,
                    "export": {"pdfSha256": pdf_hash},
                    "featureDetection": {
                        "opened": True,
                        "pageCount": 1,
                        "textLength": 10,
                    },
                    "environment": {
                        "securityModuleRegistered": True,
                        "processReset": True,
                    },
                    "privacy": {"privateCorpusAccessed": False},
                }
                manifest = {
                    "managedInstalledByExactBytes": managed,
                    "manifestSha256": (
                        baseline_hash if state == "none-related" else "6" * 64
                    ),
                    "unrelatedProjectionSha256": unrelated_hash,
                    "hwpProcessCount": 0,
                }
                observation = {
                    "schemaVersion": 1,
                    "kind": "font-oracle-pdf-observation",
                    "inputSha256": pdf_hash,
                    "toolVersions": {"fixture": "1"},
                    "fonts": [{"name": font_name}],
                    "pageCount": 1,
                    "visualLineCount": 30,
                    "glyphObservationCount": 1,
                    "glyphObservations": [{"unicode": "가", "font": font_name}],
                }
                observation["canonicalSha256"] = sha256_bytes(
                    canonical_json_bytes(observation)
                )
                recovered = {
                    "manifestSha256": baseline_hash,
                    "unrelatedProjectionSha256": unrelated_hash,
                    "hwpProcessCount": 0,
                    "managedInstalledByExactBytes": [],
                }
                for path, value in (
                    (state_root / f"{stem}.interactive.json", run),
                    (state_root / f"{stem}.ambient-manifest.json", manifest),
                    (state_root / f"{stem}.pdf-observation.json", observation),
                    (state_root / "recovered.ambient-manifest.json", recovered),
                ):
                    path.write_text(json.dumps(value), encoding="utf-8")
                config_states[state] = {
                    "directory": state_directory,
                    "stem": stem,
                    "managedFontSha256": managed,
                }
            config = {
                "schemaVersion": 1,
                "kind": "font-oracle-hyperv-reproduction-config",
                "issue": 4963,
                "queueRank": 8,
                "documentFace": "KoPubWorld바탕체 Light",
                "fixtureSha256": fixture_hash,
                "baseline": {
                    "manifestSha256": baseline_hash,
                    "unrelatedProjectionSha256": unrelated_hash,
                },
                "states": config_states,
            }
            result = compare(root, config)
            self.assertFalse(result["comparisons"]["exactEqualsNone"])
            self.assertTrue(result["comparisons"]["substitutionEqualsNone"])
            self.assertTrue(result["privacy"]["absolutePathIncluded"] is False)

            recovered_path = root / "rank8-none-related/recovered.ambient-manifest.json"
            recovered = json.loads(recovered_path.read_text(encoding="utf-8"))
            recovered["manifestSha256"] = "9" * 64
            recovered_path.write_text(json.dumps(recovered), encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "baseline manifest was not recovered"):
                compare(root, config)


if __name__ == "__main__":
    unittest.main()
