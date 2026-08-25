#!/usr/bin/env python3
"""Contract tests for the W8 rank-8 existing-evidence projector."""

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

from font_rank8_qualification import (  # noqa: E402
    QualificationError,
    TARGET_FACE,
    build_outputs,
    canonical_json_bytes,
    reject_absolute_paths,
    scan_journal,
    sha256_bytes,
)


def row(
    *,
    characters: int,
    category: str | None,
    context: str,
    ratio: int,
    spacing: int,
    stored: bool,
) -> dict:
    return {
        "font": TARGET_FACE,
        "charCount": characters,
        "documentCount": 1,
        "coverageCategory": category,
        "context": context,
        "ratio": ratio,
        "spacing": spacing,
        "storedLineSeg": stored,
    }


class Rank8QualificationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.documents = [
            {
                "source": "/private/a.hwp",
                "format": "hwp",
                "inputFormat": "hwp",
                "blake3": "a" * 64,
            },
            {
                "source": "/private/b.hwpx",
                "format": "hwpx",
                "inputFormat": "hwpx",
                "blake3": "b" * 64,
            },
        ]
        self.rows = [
            [
                row(
                    characters=10,
                    category="face-miss",
                    context="body",
                    ratio=90,
                    spacing=-5,
                    stored=True,
                ),
                row(
                    characters=2,
                    category=None,
                    context="body",
                    ratio=100,
                    spacing=0,
                    stored=True,
                ),
            ],
            [
                row(
                    characters=5,
                    category="heuristic",
                    context="table-cell",
                    ratio=80,
                    spacing=-10,
                    stored=True,
                )
            ],
        ]
        self.journal = self.root / "journal.ndjson"
        self._write_journal()
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
            "aggregateHash": {"algorithm": "sha256", "value": "d" * 64},
            "decisionUsage": [row for rows in self.rows for row in rows],
            "checkpoint": {
                "identity": {
                    "sourceHead": "c" * 40,
                    "documentCount": 2,
                    "checkpointPolicySha256": "e" * 64,
                },
                "chain": {"algorithm": "sha256-chain-v1", "value": "f" * 64},
                "entries": 2,
            },
        }
        self.ranking = {
            "schemaVersion": 1,
            "kind": "font-typesetting-risk-public-ranking",
            "issue": 4962,
            "ranking": [
                {
                    "baseRank": 13,
                    "actionRank": 8,
                    "documentFace": TARGET_FACE,
                    "empiricalRiskBand": "B",
                    "w5Queue": True,
                    "riskCharacters": 15,
                    "categoryRiskCharacters": {
                        "face-miss": 10,
                        "char-miss": 0,
                        "heuristic": 5,
                    },
                    "compressedFixedContextRiskCharacters": 5,
                    "formatCharacters": {"hwp": 12, "hwpx": 5},
                    "freshCandidateRiskMass": 0,
                }
            ],
        }
        self.ladder = {
            "schemaVersion": 1,
            "kind": "font-oracle-stage5-ladder-evidence",
            "issue": 4963,
            "target": {"queueRank": 8, "documentFace": TARGET_FACE},
            "fixture": {"sha256": "1" * 64},
            "profiles": [{"questionId": str(index)} for index in range(4)],
            "runs": [
                {
                    "physicalState": "exact-only",
                    "typesettingProjectionSha256": "2" * 64,
                },
                {
                    "physicalState": "subst-only",
                    "typesettingProjectionSha256": "3" * 64,
                },
                {
                    "physicalState": "none-related",
                    "typesettingProjectionSha256": "3" * 64,
                },
            ],
            "privacy": {
                "absolutePathIncluded": False,
                "fontBytesIncluded": False,
                "hostNameIncluded": False,
                "privateCorpusAccessed": False,
                "privateDocumentIdentityIncluded": False,
            },
        }
        self.registry = {
            "schemaVersion": "2.0",
            "kind": "canonical-font-rule-lifecycle-registry",
            "rulesSha256": "4" * 64,
            "rules": [
                {
                    "ruleId": "rule.canvas2d",
                    "status": "active",
                    "sourceFace": TARGET_FACE,
                    "decisionPlane": "supply",
                    "projections": [{"id": "canvas2d-webfont"}],
                },
                {
                    "ruleId": "rule.canvaskit",
                    "status": "active",
                    "sourceFace": TARGET_FACE,
                    "decisionPlane": "supply",
                    "projections": [{"id": "canvaskit-sfnt"}],
                },
            ],
        }
        self.paths = {}
        for name, value in {
            "w3Manifest": self.manifest,
            "w3Coverage": self.coverage,
            "w4Ranking": self.ranking,
            "w5Rank8Ladder": self.ladder,
            "fontRuleRegistryV2": self.registry,
        }.items():
            path = self.root / f"{name}.json"
            path.write_bytes(canonical_json_bytes(value))
            self.paths[name] = path
        self.paths["w3Journal"] = self.journal

    def tearDown(self):
        self.temp.cleanup()

    def _write_journal(self, duplicate: bool = False):
        records = []
        for index, rows in enumerate(self.rows):
            records.append(
                {
                    "schemaVersion": 1,
                    "kind": "font-metric-coverage-checkpoint-record",
                    "index": index,
                    "format": self.documents[index]["format"],
                    "status": "complete",
                    "aggregate": {
                        "aggregateHash": {"value": str(index + 1) * 64},
                        "decisionUsage": rows,
                    },
                }
            )
        if duplicate:
            records.append(copy.deepcopy(records[0]))
        self.journal.write_text(
            "".join(json.dumps(value, ensure_ascii=False) + "\n" for value in records),
            encoding="utf-8",
        )

    def _build(self):
        selected, journal_sha256 = scan_journal(self.journal, self.documents)
        with patch("font_rank8_qualification.ROOT", self.root):
            return build_outputs(
                manifest=self.manifest,
                coverage=self.coverage,
                ranking=self.ranking,
                ladder=self.ladder,
                registry=self.registry,
                selected=selected,
                journal_sha256=journal_sha256,
                paths=self.paths,
            )

    def test_projects_private_documents_and_public_aggregate_separately(self):
        private, public = self._build()
        self.assertEqual(private["cohort"]["documents"], 2)
        self.assertEqual(len(private["documents"]), 2)
        self.assertEqual(private["documents"][0]["source"], "/private/a.hwp")
        self.assertEqual(public["cohort"]["documentsByFormat"], {"hwp": 1, "hwpx": 1})
        self.assertEqual(public["cohort"]["aggregateUsageRows"], 3)
        self.assertEqual(public["cohort"]["documentUsageRows"], 3)
        self.assertEqual(public["cohort"]["riskCharacters"], 15)
        self.assertEqual(public["cohort"]["freshRiskCharacters"], 0)
        self.assertFalse(public["executionPolicy"]["fullCorpusRerun"])
        self.assertFalse(public["executionPolicy"]["hyperVOracleRerun"])
        reject_absolute_paths(public)
        self.assertNotIn("/private", json.dumps(public, ensure_ascii=False))

    def test_public_hash_is_canonical_and_excludes_itself(self):
        _, public = self._build()
        claimed = public.pop("canonicalSha256")
        self.assertEqual(claimed, sha256_bytes(canonical_json_bytes(public)))

    def test_duplicate_journal_index_fails_closed(self):
        self._write_journal(duplicate=True)
        with self.assertRaisesRegex(QualificationError, "duplicated"):
            scan_journal(self.journal, self.documents)

    def test_w4_count_drift_fails_closed(self):
        self.ranking["ranking"][0]["riskCharacters"] += 1
        with self.assertRaisesRegex(QualificationError, "W4 risk characters"):
            self._build()

    def test_cross_plane_registry_rule_fails_closed(self):
        self.registry["rules"][0]["decisionPlane"] = "paint"
        with self.assertRaisesRegex(QualificationError, "decision planes"):
            self._build()


if __name__ == "__main__":
    unittest.main()
