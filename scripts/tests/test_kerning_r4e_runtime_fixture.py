#!/usr/bin/env python3

from __future__ import annotations

import json
import sys
import tempfile
import unittest
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from generate_kerning_pair_fixture import generate_fixture  # noqa: E402
from generate_kerning_runtime_fixture import (  # noqa: E402
    CHAR_SHAPE_IDS,
    FONT,
    FONT_SHA256,
    PAIR_TEXT,
    PROJECTION_CONTRACT,
    generate_runtime_fixture,
)
from oracle_stage2_common import canonical_json_bytes, sha256_bytes  # noqa: E402


FIXTURE_ROOT = ROOT / "mydocs/tech/investigations/issue-4968/fixtures"


class KerningR4ERuntimeFixtureTest(unittest.TestCase):
    def test_runtime_fixture_is_deterministic_and_matches_tracked_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = generate_runtime_fixture(
                output_root=root,
                output_relative="first.hwpx",
                manifest_relative="first.json",
            )
            second = generate_runtime_fixture(
                output_root=root,
                output_relative="second.hwpx",
                manifest_relative="second.json",
            )
            self.assertEqual(
                (root / "first.hwpx").read_bytes(),
                (root / "second.hwpx").read_bytes(),
            )
            self.assertEqual(first, second)
            self.assertEqual(
                (root / "first.hwpx").read_bytes(),
                (FIXTURE_ROOT / "kerning_runtime_fixture.hwpx").read_bytes(),
            )
            self.assertEqual(
                first,
                json.loads(
                    (FIXTURE_ROOT / "kerning_runtime_fixture.manifest.json").read_text(
                        encoding="utf-8"
                    )
                ),
            )

    def test_q2_historic_fixture_remains_byte_exact_after_helper_reuse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = generate_fixture(
                output_root=root,
                output_relative="q2.hwpx",
                manifest_relative="q2.json",
            )
            self.assertEqual(
                (root / "q2.hwpx").read_bytes(),
                (FIXTURE_ROOT / "kerning_pair_fixture.hwpx").read_bytes(),
            )
            self.assertEqual(
                manifest,
                json.loads(
                    (FIXTURE_ROOT / "kerning_pair_fixture.manifest.json").read_text(
                        encoding="utf-8"
                    )
                ),
            )

    def test_runtime_matrix_slots_and_projection_contract_are_closed(self) -> None:
        manifest = json.loads(
            (FIXTURE_ROOT / "kerning_runtime_fixture.manifest.json").read_text(
                encoding="utf-8"
            )
        )
        semantic = manifest["semantic"]
        self.assertEqual(manifest["stage"], "W9-Q3-5R4E-0")
        self.assertEqual(len(semantic["matrix"]), 18)
        self.assertEqual(len(semantic["contexts"]), 22)
        self.assertEqual(
            manifest["lineSegLaneCounts"],
            {"stored-line-lane": 12, "fresh-candidate-lane": 10},
        )
        self.assertEqual(
            {(row["ratio"], row["spacing"]) for row in semantic["matrix"]},
            {(ratio, spacing) for ratio in (100, 90, 80) for spacing in (0, -5, -10)},
        )
        registration = semantic["exactSourceRegistration"]
        self.assertEqual(registration["languageIndex"], 1)
        self.assertEqual(registration["language"], "latin")
        self.assertEqual(registration["faceIndex"], 0)
        self.assertEqual(
            registration["slots"],
            [
                {"charShapeId": char_shape_id, "languageIndex": 1}
                for char_shape_id in CHAR_SHAPE_IDS
            ],
        )
        self.assertEqual(semantic["fontSource"]["path"], FONT)
        self.assertEqual(semantic["fontSource"]["sha256"], FONT_SHA256)
        self.assertTrue(semantic["fontSource"]["tracked"])
        self.assertFalse(semantic["fontSource"]["embedded"])
        self.assertEqual(semantic["canonicalProjectionContract"], PROJECTION_CONTRACT)
        self.assertEqual(
            manifest["projectionContractSha256"],
            sha256_bytes(canonical_json_bytes(PROJECTION_CONTRACT)),
        )
        self.assertEqual(
            PROJECTION_CONTRACT["normalization"]["paragraphRef"]["canonical"],
            "para:MAX",
        )
        self.assertIn("text", PROJECTION_CONTRACT["forbiddenFields"])
        self.assertNotIn("text", PROJECTION_CONTRACT["allowedFields"])

    def test_runtime_visible_text_uses_only_smoke_face_glyphs_and_embeds_no_font(self) -> None:
        fixture = FIXTURE_ROOT / "kerning_runtime_fixture.hwpx"
        font_bytes = (ROOT / FONT).read_bytes()
        with zipfile.ZipFile(fixture) as archive:
            self.assertEqual(archive.infolist()[0].filename, "mimetype")
            self.assertEqual(archive.infolist()[0].compress_type, zipfile.ZIP_STORED)
            self.assertFalse(
                any(name.lower().endswith((".ttf", ".otf")) for name in archive.namelist())
            )
            self.assertNotIn(font_bytes, b"".join(archive.read(name) for name in archive.namelist()))
            header = archive.read("Contents/header.xml").decode("utf-8")
            section = ET.fromstring(archive.read("Contents/section0.xml"))
            preview = archive.read("Preview/PrvText.txt").decode("utf-8")
        self.assertEqual(header.count('face="RHWP Kerning Boundary"'), 7)
        self.assertNotIn("Noto Sans KR", header)
        visible = [
            node.text or ""
            for node in section.iter()
            if node.tag.endswith("}t") and node.text
        ]
        self.assertEqual(len(visible), 22)
        self.assertTrue(all(text == PAIR_TEXT for text in visible))
        self.assertEqual(preview, PAIR_TEXT)
        self.assertLessEqual(set("".join(visible)), set(" AVToWH"))


if __name__ == "__main__":
    unittest.main()
