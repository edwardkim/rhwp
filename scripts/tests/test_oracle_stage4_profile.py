#!/usr/bin/env python3
"""Public-artifact and fail-closed tests for the W5-4 profile projection."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
INVESTIGATION = ROOT / "mydocs/tech/investigations/issue-4963"
sys.path.insert(0, str(SCRIPTS))

from oracle_stage2_common import (  # noqa: E402
    OracleStage2Error,
    pretty_json_bytes,
    sha256_bytes,
    sha256_file,
)
from oracle_stage4_contract import (  # noqa: E402
    validate_attestation,
    validate_ladder,
)
from oracle_stage4_profile import (  # noqa: E402
    reject_absolute_paths,
    verify_file,
    write_artifacts,
)


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


class OracleStage4ProfileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = read_json(INVESTIGATION / "oracle_stage4_contract.json")
        cls.attestation = read_json(
            INVESTIGATION / "oracle_stage4_acceptance_attestation.json"
        )
        cls.projection = read_json(
            INVESTIGATION / "oracle_stage4_acceptance_projection.json"
        )

    def test_acceptance_attestation_and_ladders_validate(self):
        self.assertEqual(
            validate_attestation(
                self.attestation,
                self.contract,
                allow_contract_fixture=False,
            ),
            [],
        )
        for rank in (1, 7):
            ladder = read_json(
                INVESTIGATION / f"oracle_stage4_rank{rank}_acceptance_ladder.json"
            )
            self.assertEqual(
                validate_ladder(ladder, self.contract, allow_contract_fixture=False),
                [],
            )

    def test_eight_profiles_validate_and_projection_hashes_are_file_exact(self):
        profile_entries = [
            (target, profile)
            for target in self.projection["targets"]
            for profile in target["profiles"]
        ]
        self.assertEqual(len(profile_entries), 8)
        self.assertEqual(
            {profile["questionId"] for _, profile in profile_entries},
            {
                "exact-installed",
                "exact-removed",
                "document-subst-font-only",
                "all-related-fonts-missing",
            },
        )
        for target, entry in profile_entries:
            slug = "mbatang" if target["queueRank"] == 1 else "kopubworld_dotum_light"
            name = entry["questionId"].replace("-", "_")
            path = INVESTIGATION / "profiles" / f"windows_hwp2020_{slug}_{name}.json"
            self.assertEqual(sha256_file(path), entry["sha256"])
            completed = subprocess.run(
                [
                    "node",
                    str(SCRIPTS / "oracle_profile_contract.mjs"),
                    "check",
                    "--profile",
                    str(path),
                ],
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)

    def test_ladder_profile_links_are_file_exact(self):
        mapping = {
            "exact-only": "exact_installed",
            "subst-only": "document_subst_font_only",
            "none-related": "exact_removed",
        }
        for rank, slug in ((1, "mbatang"), (7, "kopubworld_dotum_light")):
            ladder = read_json(
                INVESTIGATION / f"oracle_stage4_rank{rank}_acceptance_ladder.json"
            )
            for run in ladder["runs"]:
                profile = (
                    INVESTIGATION
                    / "profiles"
                    / f'windows_hwp2020_{slug}_{mapping[run["physicalState"]]}.json'
                )
                self.assertEqual(run["outputProfileSha256"], sha256_file(profile))

    def test_mbatang_identity_and_kopub_substitution_roles_stay_distinct(self):
        exact = read_json(
            INVESTIGATION / "profiles/windows_hwp2020_mbatang_exact_installed.json"
        )
        substitution = read_json(
            INVESTIGATION
            / "profiles/windows_hwp2020_mbatang_document_subst_font_only.json"
        )
        exact_sha = exact["fontState"]["installedFontSha256"]["value"]
        substitution_sha = substitution["fontState"]["installedFontSha256"]["value"]

        self.assertNotEqual(exact_sha, substitution_sha)
        self.assertEqual(exact["relationEvidence"]["type"], "identity-alias")
        self.assertEqual(
            exact["observations"]["subsetFontName"]["value"], "INPILL+MBatang"
        )
        self.assertEqual(
            substitution["relationEvidence"]["type"], "document-substitution"
        )
        anchor = substitution["relationEvidence"]["anchor"]["value"]
        self.assertEqual(anchor["declaredSubstitutionFace"], "KoPubWorld바탕체 Light")
        self.assertFalse(anchor["exportUsedSubstitution"])
        self.assertEqual(
            substitution["observations"]["subsetFontName"]["value"],
            "INPILL+HCRBatang-Bold",
        )
        self.assertEqual(
            substitution["observations"]["firstTypesettingDivergence"]["value"]["plane"],
            "selection",
        )

    def test_public_projection_is_path_and_font_byte_free(self):
        paths = [
            INVESTIGATION / "oracle_stage4_acceptance_attestation.json",
            INVESTIGATION / "oracle_stage4_acceptance_projection.json",
            INVESTIGATION / "oracle_stage4_rank1_acceptance_ladder.json",
            INVESTIGATION / "oracle_stage4_rank7_acceptance_ladder.json",
            *sorted((INVESTIGATION / "profiles").glob("windows_hwp2020_mbatang_*.json")),
            *sorted(
                (INVESTIGATION / "profiles").glob(
                    "windows_hwp2020_kopubworld_dotum_light_*.json"
                )
            ),
        ]
        self.assertEqual(len(paths), 12)
        for path in paths:
            text = path.read_text(encoding="utf-8")
            self.assertNotIn("/home/", text)
            self.assertNotIn("/mnt/", text)
            self.assertNotRegex(text, r"[A-Za-z]:[\\/]")
            reject_absolute_paths(read_json(path), str(path.name))
        self.assertEqual(list(INVESTIGATION.glob("*.ttf")), [])
        self.assertEqual(list(INVESTIGATION.glob("*.hft")), [])

    def test_hash_mismatch_and_absolute_paths_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "evidence.json"
            path.write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(OracleStage2Error, "evidence mismatch"):
                verify_file(path, "0" * 64, "negative evidence")
        with self.assertRaisesRegex(OracleStage2Error, "absolute path"):
            reject_absolute_paths({"path": "D:\\private\\font.ttf"})

    def test_public_artifact_writer_is_deterministic(self):
        artifacts = {"nested/result.json": {"한글": True, "value": [2, 1]}}
        expected = sha256_bytes(pretty_json_bytes(artifacts["nested/result.json"]))
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            first_hashes = write_artifacts(Path(first), artifacts)
            second_hashes = write_artifacts(Path(second), artifacts)
            self.assertEqual(first_hashes, second_hashes)
            self.assertEqual(first_hashes["nested/result.json"], expected)
            self.assertEqual(
                (Path(first) / "nested/result.json").read_bytes(),
                (Path(second) / "nested/result.json").read_bytes(),
            )


if __name__ == "__main__":
    unittest.main()
