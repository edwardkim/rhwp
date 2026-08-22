#!/usr/bin/env python3
"""Regression tests for Issue #4963 Stage W5-3 evidence projection."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
INVESTIGATION = ROOT / "mydocs/tech/investigations/issue-4963"
PROFILES = INVESTIGATION / "profiles"
sys.path.insert(0, str(SCRIPTS))

from oracle_stage2_common import sha256_file  # noqa: E402
from oracle_stage3_historical_import import generate_profiles  # noqa: E402


def read_json(path: Path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


class OracleStage3Tests(unittest.TestCase):
    def test_historical_import_is_byte_deterministic(self):
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            first_root = Path(first)
            second_root = Path(second)
            first_manifest = generate_profiles(first_root)
            second_manifest = generate_profiles(second_root)
            self.assertEqual(first_manifest, second_manifest)
            self.assertEqual(
                sorted(path.name for path in first_root.iterdir()),
                sorted(path.name for path in second_root.iterdir()),
            )
            for first_path in first_root.iterdir():
                self.assertEqual(first_path.read_bytes(), (second_root / first_path.name).read_bytes())

    def test_historical_import_does_not_invent_missing_provenance(self):
        for name in (
            "historical_hanyang_sinmyeongjo_exact_installed.json",
            "historical_human_myeongjo_exact_installed.json",
        ):
            profile = read_json(PROFILES / name)
            self.assertEqual(profile["execution"]["evidenceClass"], "historical-import")
            self.assertEqual(profile["environment"]["oracleAuthority"], "secondary-historical")
            self.assertEqual(profile["input"]["sha256"]["status"], "unavailable")
            self.assertEqual(
                profile["environment"]["ambientFontManifestSha256"]["status"],
                "unavailable",
            )
            self.assertEqual(profile["execution"]["startedAt"]["status"], "unavailable")
            self.assertEqual(profile["execution"]["finishedAt"]["status"], "unavailable")

    def test_current_canary_is_exact_installed_without_font_mutation(self):
        stage3 = read_json(INVESTIGATION / "oracle_stage3_contract.json")
        canary = stage3["currentHostCanary"]
        exact = canary["exactInstalledCanary"]
        profile = read_json(PROFILES / "windows_hwp2020_malgun_gothic_exact_installed.json")

        self.assertEqual(canary["oracleAuthority"], "acceptance-primary")
        self.assertEqual(canary["inputCapability"]["status"], "observed")
        self.assertFalse(exact["installationMutation"])
        self.assertTrue(exact["processReset"])
        self.assertEqual(profile["candidate"], {"queueRank": 9, "documentFace": "맑은 고딕"})
        self.assertEqual(profile["environment"]["oracleAuthority"], "acceptance-primary")
        self.assertEqual(profile["fontState"]["readbackFace"]["value"], "맑은 고딕")
        self.assertEqual(profile["fontState"]["readbackFontType"]["value"], 1)
        self.assertEqual(
            profile["fontState"]["installedFontSha256"]["value"],
            exact["installedFontSha256"],
        )
        self.assertEqual(
            profile["environment"]["ambientFontManifestSha256"]["value"],
            exact["ambientFontManifestSha256"],
        )
        self.assertEqual(profile["observations"]["subsetFontName"]["value"], "INPILL+MalgunGothic")
        self.assertEqual(profile["observations"]["hmtxAdvance"]["value"]["advance"], 2048)
        self.assertEqual(profile["observations"]["hmtxAdvance"]["value"]["unitsPerEm"], 2048)
        self.assertEqual(profile["observations"]["pdfObservedAdvance"]["value"]["advance"], 8.120862)
        self.assertEqual(profile["observations"]["lineCount"]["value"], 30)
        self.assertEqual(profile["observations"]["pageCount"]["value"], 1)
        self.assertEqual(
            profile["observations"]["firstTypesettingDivergence"]["status"],
            "not-applicable",
        )

    def test_selection_probe_negative_control_and_rank1_fail_closed(self):
        stage3 = read_json(INVESTIGATION / "oracle_stage3_contract.json")
        results = stage3["currentHostCanary"]["selectionProbe"]["results"]
        negative = [entry for entry in results if entry.get("negativeControl")]
        rank1 = [entry for entry in results if entry.get("queueRank") == 1]
        self.assertEqual(len(negative), 1)
        self.assertFalse(negative[0]["exact"])
        self.assertEqual(negative[0]["readbackFace"], "함초롬바탕")
        self.assertEqual(len(rank1), 1)
        self.assertFalse(rank1[0]["exact"])
        self.assertEqual(rank1[0]["readbackFace"], "함초롬바탕")

    def test_runner_and_public_profiles_are_path_free_hash_anchors(self):
        stage3 = read_json(INVESTIGATION / "oracle_stage3_contract.json")
        expected_runner = stage3["currentHostCanary"]["exactInstalledCanary"]["runnerSha256"]
        self.assertEqual(
            sha256_file(SCRIPTS / "oracle_stage3_windows_canary.ps1"),
            expected_runner,
        )
        for path in PROFILES.glob("*.json"):
            value = path.read_text(encoding="utf-8")
            self.assertNotIn("/home/", value)
            self.assertNotIn("/mnt/", value)
            self.assertNotRegex(value, r"[A-Za-z]:[\\/]")
        self.assertEqual(list(PROFILES.glob("*.pdf")), [])
        self.assertEqual(list(PROFILES.glob("*.ttf")), [])
        self.assertEqual(list(PROFILES.glob("*.hft")), [])


if __name__ == "__main__":
    unittest.main()
