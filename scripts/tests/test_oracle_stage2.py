#!/usr/bin/env python3
"""Determinism and fail-closed controls for Issue #4963 Stage W5-2."""

from __future__ import annotations

import copy
import json
import os
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_oracle_inventory import inventory_relative_font  # noqa: E402
from generate_oracle_typesetting_fixture import generate_fixture  # noqa: E402
from oracle_stage2_common import (  # noqa: E402
    OracleStage2Error,
    canonical_json_bytes,
    output_path,
    read_contract,
    regular_input,
    sha256_bytes,
)
from pdf_oracle_observe import analyze_pdf  # noqa: E402


def minimal_pdf() -> bytes:
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        (
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
        ),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    stream = b"BT /F1 12 Tf 72 720 Td (ABC xyz 012) Tj ET\n"
    objects.append(
        b"<< /Length " + str(len(stream)).encode("ascii") + b" >>\nstream\n"
        + stream
        + b"endstream"
    )
    payload = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for index, body in enumerate(objects, start=1):
        offsets.append(len(payload))
        payload.extend(f"{index} 0 obj\n".encode("ascii"))
        payload.extend(body)
        payload.extend(b"\nendobj\n")
    xref = len(payload)
    payload.extend(f"xref\n0 {len(objects) + 1}\n".encode("ascii"))
    payload.extend(b"0000000000 65535 f \n")
    for offset in offsets[1:]:
        payload.extend(f"{offset:010d} 00000 n \n".encode("ascii"))
    payload.extend(
        (
            f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\n"
            f"startxref\n{xref}\n%%EOF\n"
        ).encode("ascii")
    )
    return bytes(payload)


class OracleStage2Test(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = read_contract()

    def test_fixture_is_byte_exact_and_semantically_complete(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = generate_fixture(
                contract=self.contract,
                output_root=root,
                output_relative="first.hwpx",
                manifest_relative="first.json",
                document_face="문체부 바탕체",
            )
            second = generate_fixture(
                contract=self.contract,
                output_root=root,
                output_relative="second.hwpx",
                manifest_relative="second.json",
                document_face="문체부 바탕체",
            )
            self.assertEqual(
                (root / "first.hwpx").read_bytes(),
                (root / "second.hwpx").read_bytes(),
            )
            self.assertEqual(first, second)
            tracked_root = (
                ROOT
                / "mydocs"
                / "tech"
                / "investigations"
                / "issue-4963"
                / "fixtures"
            )
            self.assertEqual(
                (root / "first.hwpx").read_bytes(),
                (tracked_root / "oracle_typesetting_fixture.hwpx").read_bytes(),
            )
            self.assertEqual(
                first,
                json.loads(
                    (tracked_root / "oracle_typesetting_fixture.manifest.json").read_text(
                        encoding="utf-8"
                    )
                ),
            )
            self.assertEqual(len(first["semantic"]["matrix"]), 18)
            self.assertEqual(
                {record["context"] for record in first["semantic"]["contexts"]},
                {"body", "table-cell", "text-box", "header", "footer"},
            )
            self.assertGreater(first["lineSegLaneCounts"]["stored-line-lane"], 0)
            self.assertGreater(first["lineSegLaneCounts"]["fresh-candidate-lane"], 0)
            with zipfile.ZipFile(root / "first.hwpx") as archive:
                self.assertEqual(archive.infolist()[0].filename, "mimetype")
                self.assertEqual(archive.infolist()[0].compress_type, zipfile.ZIP_STORED)
                self.assertFalse(any("font" in name.lower() for name in archive.namelist()))

    def test_sfnt_inventory_and_corrupt_font_controls(self) -> None:
        inventory = inventory_relative_font(
            contract=self.contract,
            font_root=ROOT / "tests" / "fixtures" / "fonts",
            relative_font="RHWPExactFaceSmoke.ttc",
            document_face="RHWP Exact Face Zero",
            face_index=0,
        )
        self.assertTrue(inventory["exactNameMatch"])
        self.assertEqual(inventory["collectionFaceCount"], 2)
        self.assertNotIn(str(ROOT), json.dumps(inventory))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "broken.ttf").write_bytes(b"not-an-sfnt")
            with self.assertRaises(OracleStage2Error):
                inventory_relative_font(
                    contract=self.contract,
                    font_root=root,
                    relative_font="broken.ttf",
                    document_face="broken",
                )

    def test_pdf_observation_is_byte_exact_and_advance_is_not_hmtx(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "input.pdf").write_bytes(minimal_pdf())
            first = analyze_pdf(
                contract=self.contract, pdf_root=root, relative_pdf="input.pdf"
            )
            second = analyze_pdf(
                contract=self.contract, pdf_root=root, relative_pdf="input.pdf"
            )
            self.assertEqual(first, second)
            self.assertEqual(first["pageCount"], 1)
            self.assertGreater(first["glyphObservationCount"], 0)
            self.assertFalse(first["advanceSemantics"]["sfntHmtxIncluded"])
            self.assertIn("pdfObservedAdvance", first["glyphObservations"][0])
            self.assertNotIn("hmtxAdvance", first["glyphObservations"][0])

    def test_corrupt_and_oversize_pdf_controls(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "broken.pdf").write_bytes(b"%PDF-1.4\nbroken")
            with self.assertRaises(OracleStage2Error):
                analyze_pdf(
                    contract=self.contract, pdf_root=root, relative_pdf="broken.pdf"
                )
            (root / "large.pdf").write_bytes(minimal_pdf())
            limited = copy.deepcopy(self.contract)
            limited["pdfAnalysis"]["maximumBytes"] = 1
            with self.assertRaises(OracleStage2Error):
                analyze_pdf(contract=limited, pdf_root=root, relative_pdf="large.pdf")

    def test_path_escape_and_symlink_controls(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "input.bin").write_bytes(b"x")
            with self.assertRaises(OracleStage2Error):
                regular_input(root, "../input.bin", 10)
            os.symlink(root / "input.bin", root / "link.bin")
            with self.assertRaises(OracleStage2Error):
                regular_input(root, "link.bin", 10)
            output_root = root / "output"
            output_root.mkdir()
            os.symlink(root, output_root / "escape")
            with self.assertRaises(OracleStage2Error):
                output_path(output_root, "escape/file.json")

    def test_public_readiness_ledger_is_complete_and_path_free(self) -> None:
        path = (
            ROOT
            / "mydocs"
            / "tech"
            / "investigations"
            / "issue-4963"
            / "font_oracle_readiness.json"
        )
        ledger = json.loads(path.read_text(encoding="utf-8"))
        claimed_hash = ledger.pop("canonicalSha256")
        self.assertEqual(claimed_hash, sha256_bytes(canonical_json_bytes(ledger)))
        self.assertEqual(ledger["candidateCount"], 17)
        self.assertEqual(
            ledger["counts"],
            {
                "readyExistingHftEvidence": 1,
                "readyLocalSfnt": 6,
                "sourceUnavailable": 10,
            },
        )
        serialized = json.dumps(ledger, ensure_ascii=False)
        self.assertNotIn("/" + "home/", serialized)
        self.assertNotIn("\\" + "Users\\", serialized)


if __name__ == "__main__":
    unittest.main()
