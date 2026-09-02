#!/usr/bin/env python3
"""Contract tests for the W8 rank-1 existing-evidence projector."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank1_qualification import (  # noqa: E402
    CANONICAL_FACE,
    EXPECTED_FONT_SHA256,
    TARGET_FACE,
    QualificationError,
    build_outputs,
)
from font_rank8_qualification import canonical_json_bytes, scan_journal  # noqa: E402


def row(characters: int, category: str | None, context: str = "body") -> dict:
    return {
        "font": TARGET_FACE,
        "charCount": characters,
        "documentCount": 1,
        "coverageCategory": category,
        "context": context,
        "ratio": 90,
        "spacing": -5,
        "storedLineSeg": True,
    }


class Rank1QualificationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.documents = [
            {"source": "/private/a.hwp", "format": "hwp", "blake3": "a" * 64},
            {"source": "/private/b.hwpx", "format": "hwpx", "blake3": "b" * 64},
        ]
        self.selected = [
            {
                "manifestIndex": 0,
                "format": "hwp",
                "source": "/private/a.hwp",
                "blake3": "a" * 64,
                "aggregateHash": "1" * 64,
                "summary": {
                    "usageRows": 1,
                    "totalCharacters": 10,
                    "riskCharacters": 10,
                    "categoryRiskCharacters": {"face-miss": 10, "char-miss": 0, "heuristic": 0},
                    "compressedCharacters": 10,
                    "compressedRiskCharacters": 10,
                    "compressedFixedContextRiskCharacters": 0,
                    "storedRiskCharacters": 10,
                    "freshRiskCharacters": 0,
                    "contextCharacters": {"body": 10},
                },
                "targetRows": [row(10, "face-miss")],
            },
            {
                "manifestIndex": 1,
                "format": "hwpx",
                "source": "/private/b.hwpx",
                "blake3": "b" * 64,
                "aggregateHash": "2" * 64,
                "summary": {
                    "usageRows": 1,
                    "totalCharacters": 5,
                    "riskCharacters": 5,
                    "categoryRiskCharacters": {"face-miss": 0, "char-miss": 0, "heuristic": 5},
                    "compressedCharacters": 5,
                    "compressedRiskCharacters": 5,
                    "compressedFixedContextRiskCharacters": 5,
                    "storedRiskCharacters": 5,
                    "freshRiskCharacters": 0,
                    "contextCharacters": {"table-cell": 5},
                },
                "targetRows": [row(5, "heuristic", "table-cell")],
            },
        ]
        self.manifest = {
            "schemaVersion": 1,
            "kind": "font-metric-coverage-private-corpus-manifest",
            "localOnly": True,
            "sourceHead": "c" * 40,
            "corpus": {"documents": 2},
            "documents": self.documents,
        }
        self.coverage = {
            "schemaVersion": 1,
            "kind": "font-metric-coverage-aggregate",
            "status": "complete",
            "aggregateHash": {"value": "d" * 64},
            "decisionUsage": [row(10, "face-miss"), row(5, "heuristic", "table-cell")],
            "checkpoint": {
                "identity": {
                    "sourceHead": "c" * 40,
                    "documentCount": 2,
                    "checkpointPolicySha256": "e" * 64,
                },
                "chain": {"value": "f" * 64},
                "entries": 2,
            },
        }
        self.ranking = {
            "schemaVersion": 1,
            "kind": "font-typesetting-risk-public-ranking",
            "issue": 4962,
            "ranking": [
                {
                    "baseRank": 1,
                    "actionRank": 1,
                    "documentFace": TARGET_FACE,
                    "empiricalRiskBand": "A",
                    "w5Queue": True,
                    "riskCharacters": 15,
                    "categoryRiskCharacters": {"face-miss": 10, "char-miss": 0, "heuristic": 5},
                    "compressedFixedContextRiskCharacters": 5,
                    "formatCharacters": {"hwp": 10, "hwpx": 5},
                    "freshCandidateRiskMass": 0,
                }
            ],
        }
        self.ladder = {
            "schemaVersion": 1,
            "kind": "font-oracle-stage4-ladder-evidence",
            "issue": 4963,
            "target": {"queueRank": 1, "documentFace": TARGET_FACE},
            "runs": [
                {
                    "physicalState": "exact-only",
                    "outputProfileSha256": "1" * 64,
                    "managedFonts": [
                        {"face": TARGET_FACE, "present": True, "sha256": EXPECTED_FONT_SHA256}
                    ],
                },
                {"physicalState": "subst-only"},
                {"physicalState": "none-related"},
            ],
            "dispositions": [
                {
                    "question": "curated-official-successor-only",
                    "reason": "No direct publisher or byte lineage establishes an official successor.",
                    "status": "not-provided",
                }
            ],
            "privacy": {
                "absolutePathIncluded": False,
                "fontBytesIncluded": False,
                "hostNameIncluded": False,
                "privateDocumentIdentityIncluded": False,
            },
        }
        self.registry = {
            "schemaVersion": "2.0",
            "kind": "canonical-font-rule-lifecycle-registry",
            "rulesSha256": "2" * 64,
            "rules": [],
        }
        self.projection = {
            "metricAnchors": {
                "entries": [
                    {
                        "name": CANONICAL_FACE,
                        "currentIndex": 370,
                        "bold": False,
                        "italic": False,
                        "entryId": "font-metric.test",
                        "metricDataSha256": "3" * 64,
                        "widthProjectionSha256": "4" * 64,
                        "ruleIds": ["rule.test"],
                    }
                ]
            }
        }
        self.source_attestation = {
            "schemaVersion": 1,
            "kind": "font-rank1-source-provenance-attestation",
            "issue": 4967,
            "exactFile": {"sha256": EXPECTED_FONT_SHA256, "officialDownloadArtifactMatched": False},
            "officialReference": {"url": "https://example.invalid/official"},
            "portableSupplyDisposition": {
                "status": "blocked-unmatched-official-artifact-and-restricted-embedding"
            },
            "privacy": {"absolutePathIncluded": False, "fontBytesIncluded": False},
        }
        self.font_identity = {
            "sha256": EXPECTED_FONT_SHA256,
            "familyNames": [CANONICAL_FACE, TARGET_FACE],
            "unitsPerEm": 1000,
            "os2FsType": 2,
            "embeddingDisposition": "restricted-license-embedding",
        }
        self.paths = {}
        for name in (
            "w3Manifest",
            "w3Coverage",
            "w3Journal",
            "w4Ranking",
            "w5Rank1Ladder",
            "fontRuleRegistryV2",
            "fontRuleProjectionBaseline",
            "sourceProvenance",
            "exactFont",
        ):
            path = self.root / name
            path.write_text("{}\n", encoding="utf-8")
            self.paths[name] = path

    def tearDown(self):
        self.temp.cleanup()

    def _build(self):
        with (
            patch("font_rank1_qualification.relative_repo_path", side_effect=lambda path: path.name),
            patch("font_rank1_qualification.sha256_file", return_value="9" * 64),
        ):
            return build_outputs(
                manifest=self.manifest,
                coverage=self.coverage,
                ranking=self.ranking,
                ladder=self.ladder,
                registry=self.registry,
                projection=self.projection,
                source_attestation=self.source_attestation,
                font_identity=self.font_identity,
                selected=self.selected,
                journal_sha256="8" * 64,
                paths=self.paths,
            )

    def test_public_projection_reconciles_existing_evidence_without_identity(self):
        private, public = self._build()
        self.assertEqual(public["cohort"]["documents"], 2)
        self.assertEqual(public["cohort"]["riskCharacters"], 15)
        self.assertEqual(public["currentMetricAnchor"]["name"], CANONICAL_FACE)
        self.assertFalse(public["gates"]["portableSupplyQualified"])
        serialized = canonical_json_bytes(public).decode("utf-8")
        self.assertNotIn("/private/", serialized)
        self.assertNotIn(str(self.root), serialized)
        self.assertEqual(private["privacy"]["ownerModeRequired"], "0600")

    def test_w4_count_drift_fails_closed(self):
        self.ranking["ranking"][0]["riskCharacters"] += 1
        with self.assertRaisesRegex(QualificationError, "W4 risk characters mismatch"):
            self._build()

    def test_official_artifact_match_drift_fails_closed(self):
        self.source_attestation["exactFile"]["officialDownloadArtifactMatched"] = True
        with self.assertRaisesRegex(QualificationError, "official artifact match mismatch"):
            self._build()

    def test_shared_journal_scanner_accepts_explicit_rank1_target(self):
        journal = self.root / "rank1.ndjson"
        records = []
        for index, document in enumerate(self.documents):
            records.append(
                {
                    "schemaVersion": 1,
                    "kind": "font-metric-coverage-checkpoint-record",
                    "index": index,
                    "status": "complete",
                    "format": document["format"],
                    "aggregate": {"decisionUsage": [row(index + 1, "face-miss")]},
                }
            )
        journal.write_bytes(b"".join(canonical_json_bytes(record) for record in records))
        selected, _ = scan_journal(journal, self.documents, target_face=TARGET_FACE)
        self.assertEqual(len(selected), 2)
        self.assertEqual(sum(item["summary"]["totalCharacters"] for item in selected), 3)


if __name__ == "__main__":
    unittest.main()
