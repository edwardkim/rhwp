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

from font_kerning_boundary import (  # noqa: E402
    MAX_FONT_BYTES,
    analyze_font_bytes,
    generate_boundary,
    synthetic_font,
)
from generate_kerning_pair_fixture import generate_fixture  # noqa: E402
from kerning_q2_hancom_adjudication import (  # noqa: E402
    PAIR_SAMPLE,
    project_hwpml,
    project_pdf_observation,
)
from oracle_stage2_common import OracleStage2Error  # noqa: E402
from oracle_stage2_common import canonical_json_bytes, sha256_bytes  # noqa: E402


def synthetic_hwpml(manifest: dict, *, drop_first_on: bool = False) -> bytes:
    root = ET.Element("HWPML", {"Version": "2.91", "SubVersion": "10.0.0.0"})
    head = ET.SubElement(root, "HEAD")
    for language in ("Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User"):
        group = ET.SubElement(head, "FONTFACE", {"Lang": language})
        ET.SubElement(group, "FONT", {"Id": "1", "Name": "Noto Sans KR"})
    shapes = ET.SubElement(head, "CHARSHAPES")
    for row in manifest["semantic"]["matrix"]:
        kerning = bool(row["kerning"])
        if drop_first_on and row["charPropertyId"] == 8:
            kerning = False
        shape = ET.SubElement(
            shapes,
            "CHARSHAPE",
            {
                "Id": str(row["charPropertyId"]),
                "UseKerning": str(kerning).lower(),
            },
        )
        common = {
            language: str(row["ratio"])
            for language in ("Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User")
        }
        ET.SubElement(shape, "RATIO", common)
        ET.SubElement(
            shape,
            "CHARSPACING",
            {language: str(row["spacing"]) for language in common},
        )
        ET.SubElement(shape, "FONTID", {language: "1" for language in common})
    body = ET.SubElement(root, "BODY")
    matrix = {row["charPropertyId"]: row for row in manifest["semantic"]["matrix"]}
    for context in manifest["semantic"]["contexts"]:
        row = matrix[context["charPropertyId"]]
        marker = f"K{1 if row['kerning'] else 0}"
        if context["context"] == "body":
            lane = "stored" if context["lineSegLane"] == "stored-line-lane" else "fresh"
            text = f"BODY R{row['ratio']} S{row['spacing']} {marker} L{lane} | {PAIR_SAMPLE}"
        elif context["context"] == "table-cell":
            text = f"TABLE {marker} | {PAIR_SAMPLE}"
        else:
            text = f"TEXTBOX {marker} | {PAIR_SAMPLE}"
        paragraph = ET.SubElement(body, "P", {"InstId": str(context["paragraphId"])})
        text_node = ET.SubElement(
            paragraph, "TEXT", {"CharShape": str(context["charPropertyId"])}
        )
        ET.SubElement(text_node, "CHAR").text = text
    return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def synthetic_pdf_observation(*, first_on_delta: float = 0.0) -> dict:
    glyphs = []
    y = 100.0
    for ratio in (100, 90, 80):
        for spacing_index, spacing in enumerate((0, -5, -10)):
            lane = "stored" if spacing_index % 2 == 0 else "fresh"
            for kerning in (False, True):
                text = (
                    f"BODY R{ratio} S{spacing} K{1 if kerning else 0} "
                    f"L{lane} | {PAIR_SAMPLE}"
                )
                step = ratio / 100 + spacing / 100
                sample_start = text.index(PAIR_SAMPLE)
                for index, character in enumerate(text):
                    x = 10.0 + index * step
                    if (
                        first_on_delta
                        and ratio == 100
                        and spacing == 0
                        and kerning
                        and index == sample_start + 1
                    ):
                        x += first_on_delta
                    glyphs.append(
                        {
                            "page": 1,
                            "unicode": character,
                            "position": {"x": x, "y": y},
                        }
                    )
                y += 10.0
    for label in ("TABLE", "TEXTBOX"):
        for kerning in (False, True):
            text = f"{label} K{1 if kerning else 0} | {PAIR_SAMPLE}"
            for index, character in enumerate(text):
                glyphs.append(
                    {
                        "page": 1,
                        "unicode": character,
                        "position": {"x": 10.0 + index, "y": y},
                    }
                )
            y += 10.0
    return {"glyphObservations": glyphs}


class KerningQ2Test(unittest.TestCase):
    def test_fixture_is_byte_exact_and_contains_bounded_axes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = generate_fixture(
                output_root=root,
                output_relative="first.hwpx",
                manifest_relative="first.json",
            )
            second = generate_fixture(
                output_root=root,
                output_relative="second.hwpx",
                manifest_relative="second.json",
            )
            self.assertEqual((root / "first.hwpx").read_bytes(), (root / "second.hwpx").read_bytes())
            self.assertEqual(first, second)
            self.assertEqual(len(first["semantic"]["matrix"]), 18)
            self.assertEqual(len(first["semantic"]["contexts"]), 22)
            self.assertEqual(first["lineSegLaneCounts"], {
                "stored-line-lane": 12,
                "fresh-candidate-lane": 10,
            })
            self.assertEqual(
                {item["context"] for item in first["semantic"]["contexts"]},
                {"body", "table-cell", "text-box"},
            )
            with zipfile.ZipFile(root / "first.hwpx") as archive:
                self.assertEqual(archive.infolist()[0].filename, "mimetype")
                self.assertEqual(archive.infolist()[0].compress_type, zipfile.ZIP_STORED)
                self.assertFalse(any("font" in name.lower() for name in archive.namelist()))
                header = archive.read("Contents/header.xml").decode("utf-8")
                section = archive.read("Contents/section0.xml").decode("utf-8")
            self.assertEqual(header.count('face="Noto Sans KR"'), 7)
            self.assertEqual(header.count('useKerning="1"'), 9)
            self.assertEqual(header.count('useKerning="0"'), 16)
            for text in ("AV", "To", "WA", "HH", "가나다"):
                self.assertIn(text, section)

    def test_synthetic_capability_precedence_and_pair_values(self) -> None:
        gpos = analyze_font_bytes(synthetic_font(gpos=True, legacy=False))
        legacy = analyze_font_bytes(synthetic_font(gpos=False, legacy=True))
        both = analyze_font_bytes(synthetic_font(gpos=True, legacy=True))
        none = analyze_font_bytes(synthetic_font(gpos=False, legacy=False))
        self.assertEqual(gpos["capability"], "gpos-kern")
        self.assertEqual(legacy["capability"], "legacy-kern")
        self.assertEqual(both["capability"], "gpos-kern")
        self.assertEqual(none["capability"], "unsupported")
        self.assertEqual(gpos["pairs"][0]["totalXAdvance"], -80)
        self.assertEqual(legacy["pairs"][0]["totalXAdvance"], -70)
        self.assertEqual(both["pairs"][0]["totalXAdvance"], -80)
        self.assertEqual(none["pairs"][0]["disposition"], "fail-closed")
        self.assertEqual(none["pairs"][0]["fallbackReason"], "pair-table-unsupported")

    def test_malformed_and_oversized_fonts_fail_closed(self) -> None:
        malformed = analyze_font_bytes(b"not-an-sfnt")
        oversized = analyze_font_bytes(bytes(MAX_FONT_BYTES + 1))
        self.assertEqual(malformed["status"], "fail-closed")
        self.assertEqual(malformed["fallbackReason"], "malformed-sfnt")
        self.assertEqual(oversized["status"], "fail-closed")
        self.assertEqual(oversized["fallbackReason"], "font-byte-limit-exceeded")

    def test_boundary_is_deterministic_and_path_free(self) -> None:
        first = generate_boundary()
        second = generate_boundary()
        self.assertEqual(first, second)
        payload = json.dumps(first, ensure_ascii=False)
        self.assertNotIn(str(ROOT), payload)
        public_pairs = {item["text"]: item["totalXAdvance"] for item in first["publicFont"]["pairs"]}
        self.assertEqual(public_pairs, {"AV": -18, "To": -76, "WA": 0, "HH": 0})

    def test_hwpml_readback_projection_requires_all_fixture_flags(self) -> None:
        manifest = json.loads(
            (ROOT / "mydocs/tech/investigations/issue-4968/fixtures/kerning_pair_fixture.manifest.json").read_text()
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "readback.hml"
            path.write_bytes(synthetic_hwpml(manifest))
            projection = project_hwpml(path, manifest)
            self.assertTrue(projection["allFixtureFlagsPreserved"])
            self.assertEqual(projection["fixtureContextCount"], 22)
            self.assertEqual(projection["fixtureFlagCounts"], {"off": 11, "on": 11})
            path.write_bytes(synthetic_hwpml(manifest, drop_first_on=True))
            with self.assertRaisesRegex(OracleStage2Error, "kerning flag changed"):
                project_hwpml(path, manifest)

    def test_pdf_projection_distinguishes_zero_and_nonzero_on_off_delta(self) -> None:
        equal = project_pdf_observation(synthetic_pdf_observation())
        self.assertEqual(equal["controlledGroupCount"], 9)
        self.assertTrue(equal["allControlledOnOffPositionsEqual"])
        self.assertEqual(equal["maximumAbsoluteOnOffDelta"], 0.0)
        changed = project_pdf_observation(synthetic_pdf_observation(first_on_delta=0.125))
        self.assertFalse(changed["allControlledOnOffPositionsEqual"])
        self.assertEqual(changed["maximumAbsoluteOnOffDelta"], 0.125)

    def test_public_hancom_adjudication_is_canonical_and_path_free(self) -> None:
        path = (
            ROOT
            / "mydocs/tech/investigations/issue-4968/kerning_q2_hancom_adjudication.json"
        )
        value = json.loads(path.read_text(encoding="utf-8"))
        claimed = value.pop("canonicalSha256")
        self.assertEqual(sha256_bytes(canonical_json_bytes(value)), claimed)
        self.assertEqual(
            value["adjudication"]["classification"],
            "flag-preserved-no-pdf-layout-differential",
        )
        self.assertTrue(value["hwpmlReadback"]["allFixtureFlagsPreserved"])
        self.assertTrue(value["pdfLayout"]["allControlledOnOffPositionsEqual"])
        payload = json.dumps(value, ensure_ascii=False)
        self.assertNotIn(str(ROOT), payload)
        self.assertNotIn("ExpectedVmId", payload)

    def test_hyperv_controller_requires_exact_restore_and_has_no_font_removal(self) -> None:
        controller = (ROOT / "scripts/kerning_q2_hyperv_adjudication.ps1").read_text(
            encoding="utf-8"
        )
        for marker in (
            "SupportsShouldProcess",
            "CheckpointRestoreApproved",
            "ExpectedVmId",
            "ExpectedCheckpointId",
            "AutomaticCheckpointsEnabled",
            "Restore-Baseline",
            "Get-InteractiveIdentity",
            "host-stage",
            "finally",
            "stagingRootPresent",
        ):
            self.assertIn(marker, controller)
        self.assertNotIn("RemoveFontResource", controller)
        self.assertNotRegex(controller, r"(?i)Remove-Item.+Windows\\Fonts")

        interactive = (ROOT / "scripts/oracle_stage4_windows_interactive.ps1").read_text(
            encoding="utf-8"
        )
        task = (ROOT / "scripts/oracle_stage4_windows_task.ps1").read_text(encoding="utf-8")
        self.assertIn("[ValidateSet(4963, 4968)][int]$Issue", interactive)
        self.assertIn("$hwp.GetTextFile('HWPML2X', '')", interactive)
        self.assertIn("hwpmlReadback = $hwpmlReadback", interactive)
        self.assertIn("$spec.issue -notin @(4963, 4968)", task)
        self.assertIn("$arguments.HwpmlOutput", task)
        self.assertIn("hwpmlOutput = $hwpmlPath", controller)
        self.assertIn("kerning-q2.readback.hml", controller)


if __name__ == "__main__":
    unittest.main()
