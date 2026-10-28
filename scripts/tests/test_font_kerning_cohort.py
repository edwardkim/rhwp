#!/usr/bin/env python3
"""Contract tests for the #4968 W9-Q0 existing-evidence projector."""

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

from font_kerning_cohort import (  # noqa: E402
    KerningCohortError,
    build_outputs,
    canonical_json_bytes,
    reject_absolute_paths,
    scan_journal,
    sha256_bytes,
)


def row(
    *,
    font: str,
    characters: int,
    context: str,
    ratio: int,
    spacing: int,
    stored: bool,
    kerning: bool = True,
) -> dict:
    return {
        "font": font,
        "metricFace": "Metric",
        "kerning": kerning,
        "charCount": characters,
        "documentCount": 1,
        "paragraphCount": 1,
        "runCount": 1,
        "context": context,
        "ratio": ratio,
        "spacing": spacing,
        "storedLineSeg": stored,
    }


class KerningCohortTests(unittest.TestCase):
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
            {
                "source": "/private/c.hwp",
                "format": "hwp",
                "inputFormat": "hwp",
                "blake3": "c" * 64,
            },
        ]
        self.rows = [
            [
                row(
                    font="휴먼명조",
                    characters=10,
                    context="body",
                    ratio=100,
                    spacing=0,
                    stored=True,
                ),
                row(
                    font="KoPubWorld바탕체 Light",
                    characters=2,
                    context="table-cell",
                    ratio=90,
                    spacing=-5,
                    stored=True,
                ),
            ],
            [
                row(
                    font="휴먼명조",
                    characters=5,
                    context="text-box",
                    ratio=80,
                    spacing=-10,
                    stored=False,
                )
            ],
            [
                row(
                    font="휴먼명조",
                    characters=99,
                    context="body",
                    ratio=100,
                    spacing=0,
                    stored=True,
                    kerning=False,
                )
            ],
        ]
        self.journal = self.root / "journal.ndjson"
        self._write_journal()
        self.manifest = {
            "schemaVersion": 1,
            "kind": "font-metric-coverage-private-corpus-manifest",
            "localOnly": True,
            "sourceHead": "d" * 40,
            "corpus": {"documents": 3},
            "documents": self.documents,
        }
        aggregate_rows = []
        for document_format, rows in zip(("hwp", "hwpx"), self.rows[:2]):
            for value in rows:
                aggregate_rows.append({**copy.deepcopy(value), "format": document_format})
        self.coverage = {
            "schemaVersion": 1,
            "kind": "font-metric-coverage-aggregate",
            "status": "complete",
            "aggregateHash": {"algorithm": "sha256", "value": "e" * 64},
            "decisionUsage": aggregate_rows,
            "checkpoint": {
                "identity": {
                    "sourceHead": "d" * 40,
                    "documentCount": 3,
                    "checkpointPolicySha256": "f" * 64,
                },
                "chain": {"algorithm": "sha256-chain-v1", "value": "1" * 64},
                "entries": 3,
            },
            "documents": {
                "attempted": 3,
                "success": 3,
                "formats": {"hwp": {"success": 2}, "hwpx": {"success": 1}},
            },
            "counts": {"layoutCharacters": 116},
        }
        self.rank1 = {
            "issue": 4967,
            "target": {"documentFace": "문체부 바탕체"},
            "hypothesis": {"status": "no-change", "productMutationAuthorized": False},
        }
        self.rank7 = {
            "issue": 4967,
            "target": {"face": "KoPubWorld돋움체 Light"},
            "decision": {"status": "no-change", "productMutationAuthorized": False},
        }
        self.rank8 = {
            "issue": 4967,
            "target": {"face": "KoPubWorld바탕체 Light"},
            "decision": {"status": "no-change", "productMutationAuthorized": False},
        }
        self.paths = {}
        for name, value in {
            "w3Manifest": self.manifest,
            "w3Coverage": self.coverage,
            "w8Rank1Disposition": self.rank1,
            "w8Rank7Disposition": self.rank7,
            "w8Rank8Disposition": self.rank8,
            "w5FixtureManifest": {"fixture": True},
        }.items():
            path = self.root / f"{name}.json"
            path.write_bytes(canonical_json_bytes(value))
            self.paths[name] = path
        self.paths["w3Journal"] = self.journal
        self.paths["notoSansKrRegular"] = self.root / "NotoSansKR-Regular.ttf"
        self.paths["notoSansKrRegular"].write_bytes(b"fixture-font")

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
        selected, journal_sha256, complete, target_rows = scan_journal(
            self.journal, self.documents
        )
        tracked = {
            name: self.paths[name]
            for name in (
                "w5FixtureManifest",
                "notoSansKrRegular",
                "w8Rank1Disposition",
                "w8Rank7Disposition",
                "w8Rank8Disposition",
            )
        }
        local = {
            name: self.paths[name]
            for name in ("w3Manifest", "w3Coverage", "w3Journal")
        }
        with patch("font_kerning_cohort.ROOT", self.root):
            return build_outputs(
                manifest=self.manifest,
                coverage=self.coverage,
                selected=selected,
                journal_sha256=journal_sha256,
                complete_records=complete,
                target_row_count=target_rows,
                tracked_paths=tracked,
                local_paths=local,
                rank1=self.rank1,
                rank7=self.rank7,
                rank8=self.rank8,
            )

    def test_projects_private_identity_and_public_aggregate_separately(self):
        private, public = self._build()
        self.assertEqual(private["cohort"]["documents"], 2)
        self.assertEqual(len(private["documents"]), 2)
        self.assertEqual(private["documents"][0]["source"], "/private/a.hwp")
        self.assertEqual(public["cohort"]["documentsByFormat"], {"hwp": 1, "hwpx": 1})
        self.assertEqual(public["cohort"]["documentUsageRows"], 3)
        self.assertEqual(public["cohort"]["aggregateUsageRows"], 3)
        self.assertEqual(public["cohort"]["characters"], 17)
        self.assertEqual(public["cohort"]["byStoredLineSeg"][0]["characters"], 5)
        overlap = public["w8Freeze"]["overlap"]
        self.assertEqual(overlap[2]["documents"], 1)
        self.assertEqual(overlap[2]["characters"], 2)
        reject_absolute_paths(public)
        self.assertNotIn("/private", json.dumps(public, ensure_ascii=False))

    def test_public_hash_is_canonical_and_excludes_itself(self):
        _, public = self._build()
        claimed = public.pop("canonicalSha256")
        self.assertEqual(claimed, sha256_bytes(canonical_json_bytes(public)))

    def test_duplicate_journal_index_fails_closed(self):
        self._write_journal(duplicate=True)
        with self.assertRaisesRegex(KerningCohortError, "duplicated"):
            scan_journal(self.journal, self.documents)

    def test_final_aggregate_drift_fails_closed(self):
        self.coverage["decisionUsage"][0]["charCount"] += 1
        with self.assertRaisesRegex(KerningCohortError, "kerning aggregate row"):
            self._build()

    def test_w8_mutation_authority_fails_closed(self):
        self.rank8["decision"]["productMutationAuthorized"] = True
        with self.assertRaisesRegex(KerningCohortError, "W8 rank8 mutation"):
            self._build()


if __name__ == "__main__":
    unittest.main()
