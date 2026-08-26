#!/usr/bin/env python3
"""Contract tests for the W8 rank-7 existing-evidence projector."""

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

from font_rank7_qualification import (  # noqa: E402
    ENGLISH_FACE,
    EXPECTED_CANVASKIT_URL,
    EXPECTED_EXACT_PROFILE_SHA256,
    EXPECTED_FIXTURE_SHA256,
    EXPECTED_FONT_SHA256,
    EXPECTED_WEBFONT_URL,
    POSTSCRIPT_NAME,
    QualificationError,
    TARGET_FACE,
    build_outputs,
    canonical_json_bytes,
    reject_absolute_paths,
    safe_write_json,
    scan_journal,
    sha256_bytes,
)


def row(*, characters: int, context: str, stored: bool = True) -> dict:
    return {
        "font": TARGET_FACE,
        "charCount": characters,
        "documentCount": 1,
        "coverageCategory": "face-miss",
        "context": context,
        "ratio": 90,
        "spacing": -5,
        "storedLineSeg": stored,
        "bold": False,
        "italic": False,
    }


class Rank7QualificationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.documents = [
            {"source": "/private/a.hwp", "format": "hwp", "blake3": "a" * 64},
            {"source": "/private/b.hwpx", "format": "hwpx", "blake3": "b" * 64},
        ]
        self.rows = [
            [row(characters=10, context="body")],
            [row(characters=5, context="table-cell")],
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
            "aggregateHash": {"value": "d" * 64},
            "decisionUsage": [item for rows in self.rows for item in rows],
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
                    "baseRank": 7,
                    "actionRank": 7,
                    "documentFace": TARGET_FACE,
                    "empiricalRiskBand": "B",
                    "w5Queue": True,
                    "riskCharacters": 15,
                    "categoryRiskCharacters": {
                        "face-miss": 15,
                        "char-miss": 0,
                        "heuristic": 0,
                    },
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
            "target": {"queueRank": 7, "documentFace": TARGET_FACE},
            "fixtureSha256": EXPECTED_FIXTURE_SHA256,
            "runs": [
                {
                    "physicalState": "exact-only",
                    "outputProfileSha256": EXPECTED_EXACT_PROFILE_SHA256,
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
        self.readiness = {
            "schemaVersion": 1,
            "kind": "font-oracle-readiness-ledger",
            "issue": 4963,
            "candidates": [
                {
                    "documentFace": TARGET_FACE,
                    "queueRank": 7,
                    "sourceReadiness": "ready-local-sfnt",
                    "officialSupply": {
                        "officialRecord": "https://example.invalid/font",
                        "uci": "G905-13287211",
                        "downloadSha256": "1" * 64,
                        "fontSha256": EXPECTED_FONT_SHA256,
                        "os2FsType": 8,
                        "licenseDecision": "local-analysis-only-no-font-redistribution",
                    },
                    "sfnt": {
                        "sha256": EXPECTED_FONT_SHA256,
                        "unitsPerEm": 1000,
                        "os2FsType": 8,
                    },
                }
            ],
            "privacy": {
                "absolutePathsPublished": False,
                "fontBytesTracked": False,
                "privateDocumentIdentityPublished": False,
            },
        }
        self.registry = {
            "schemaVersion": "2.0",
            "kind": "canonical-font-rule-lifecycle-registry",
            "rulesSha256": "2" * 64,
            "rules": [
                {
                    "ruleId": "rule.studio-supply.b4a81472cc52c505ee6d.canvas2d",
                    "status": "active",
                    "sourceFace": TARGET_FACE,
                    "decisionPlane": "supply",
                    "projections": [{"id": "canvas2d-webfont"}],
                    "supply": {"sourceUrl": EXPECTED_WEBFONT_URL},
                },
                {
                    "ruleId": "rule.studio-supply.b4a81472cc52c505ee6d.canvaskit",
                    "status": "active",
                    "sourceFace": TARGET_FACE,
                    "decisionPlane": "supply",
                    "projections": [{"id": "canvaskit-sfnt"}],
                    "supply": {"online": {"sources": [{"url": EXPECTED_CANVASKIT_URL}]}},
                },
            ],
        }
        self.projection = {
            "schemaVersion": "1.0",
            "kind": "font-rule-projection-pre-migration-baseline",
            "issue": 4966,
            "hashes": {"projectionBundleSha256": "3" * 64},
            "projections": {
                "rustLayoutName": {"rules": []},
                "rustLayoutMetric": {"rules": []},
                "webfontSupply": {
                    "rules": [
                        {
                            "sourceFace": TARGET_FACE,
                            "ruleId": "rule.studio-supply.b4a81472cc52c505ee6d.canvas2d",
                        }
                    ]
                },
                "canvasKitSfnt": {
                    "rules": [
                        {
                            "sourceFace": TARGET_FACE,
                            "ruleId": "rule.studio-supply.b4a81472cc52c505ee6d.canvaskit",
                        }
                    ]
                },
            },
        }
        self.font_identity = {
            "sha256": EXPECTED_FONT_SHA256,
            "sfntCount": 1,
            "faceIndex": 0,
            "familyNames": [ENGLISH_FACE, TARGET_FACE],
            "fullNames": [ENGLISH_FACE, TARGET_FACE],
            "postScriptNames": [POSTSCRIPT_NAME],
            "unitsPerEm": 1000,
            "glyphs": 31556,
            "horizontalMetrics": 31556,
            "cmapCodepoints": 25974,
            "os2FsType": 8,
            "embeddingDisposition": "editable-embedding",
        }
        self.paths = {}
        values = {
            "w3Manifest": self.manifest,
            "w3Coverage": self.coverage,
            "w4Ranking": self.ranking,
            "w5Rank7Ladder": self.ladder,
            "w5SourceReadiness": self.readiness,
            "fontRuleRegistryV2": self.registry,
            "fontRuleProjectionBaseline": self.projection,
        }
        for name, value in values.items():
            path = self.root / f"{name}.json"
            path.write_bytes(canonical_json_bytes(value))
            self.paths[name] = path
        self.paths["w3Journal"] = self.journal
        exact = self.root / "exact.ttf"
        exact.write_bytes(b"test-font-placeholder")
        self.paths["exactFont"] = exact

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
        selected, journal_sha256 = scan_journal(
            self.journal, self.documents, target_face=TARGET_FACE
        )
        with patch("font_rank8_qualification.ROOT", self.root):
            return build_outputs(
                manifest=self.manifest,
                coverage=self.coverage,
                ranking=self.ranking,
                ladder=self.ladder,
                readiness=self.readiness,
                registry=self.registry,
                projection=self.projection,
                font_identity=self.font_identity,
                selected=selected,
                journal_sha256=journal_sha256,
                paths=self.paths,
            )

    def test_projects_private_documents_and_public_aggregate_separately(self):
        private, public = self._build()
        self.assertEqual(private["cohort"]["documents"], 2)
        self.assertEqual(len(private["documents"]), 2)
        self.assertEqual(public["cohort"]["riskCharacters"], 15)
        self.assertEqual(public["cohort"]["styleDomain"]["boldCharacters"], 0)
        self.assertEqual(public["cohort"]["styleDomain"]["italicCharacters"], 0)
        self.assertEqual(public["cohort"]["styleDomain"]["axes"][0]["characters"], 15)
        self.assertEqual(public["currentRegistry"]["decisionPlanes"], ["supply"])
        self.assertTrue(public["gates"]["layoutProjectionAbsent"])
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
            scan_journal(self.journal, self.documents, target_face=TARGET_FACE)

    def test_w4_count_drift_fails_closed(self):
        self.ranking["ranking"][0]["riskCharacters"] += 1
        with self.assertRaisesRegex(QualificationError, "W4 risk characters"):
            self._build()

    def test_cross_plane_registry_rule_fails_closed(self):
        self.registry["rules"][0]["decisionPlane"] = "layout-metric"
        with self.assertRaisesRegex(QualificationError, "decision planes"):
            self._build()

    def test_readiness_font_hash_drift_fails_closed(self):
        self.readiness["candidates"][0]["officialSupply"]["fontSha256"] = "0" * 64
        with self.assertRaisesRegex(QualificationError, "official font SHA-256"):
            self._build()

    def test_style_flags_must_be_boolean(self):
        del self.rows[0][0]["bold"]
        self._write_journal()
        with self.assertRaisesRegex(QualificationError, "style flags must be boolean"):
            self._build()

    def test_supply_url_drift_fails_closed(self):
        self.registry["rules"][0]["supply"]["sourceUrl"] = "https://example.invalid/drift"
        with self.assertRaisesRegex(QualificationError, "supply URLs"):
            self._build()

    def test_output_symlink_is_rejected(self):
        real_output = self.root / "real-output.json"
        real_output.write_text("{}\n", encoding="utf-8")
        linked_output = self.root / "linked-output.json"
        linked_output.symlink_to(real_output)
        with self.assertRaisesRegex(QualificationError, "symlink"):
            safe_write_json(linked_output, {"value": True}, 0o600)


if __name__ == "__main__":
    unittest.main()
