"""독립 Print 기록을 원본·PDF의 경로와 실제 바이트에 결합하는 계약."""

import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from audit_print_pairs import audit, index_receipts, print_details


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.sample = "samples/a/doc.hwp"
        self.pdf = "pdf/a/doc-2024.pdf"
        other = "samples/b/doc.hwp"
        for name in (self.sample, self.pdf, other):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
        self.manifest = {"pairs": [{"sample": name, "pdf": self.pdf} for name in (self.sample, other)],
                         "unmatched": [{"sample": "samples/unmatched.hwp"}]}
        self.record = {"input": self.sample, "pdf": self.pdf,
                       "input_sha256": hashlib.sha256(self.sample.encode()).hexdigest(),
                       "pdf_sha256": hashlib.sha256(self.pdf.encode()).hexdigest(),
                       "origin": {"kind": "user-provided manual Print", "engine": "2024",
                                  "hancom_version": "13.0.0.3901", "print_attestation": "Print"}}

    def result(self):
        return audit(self.root, self.manifest, {"cases": [self.record]})

    def test_exact_pair_and_other_same_name(self):
        result = self.result()
        self.assertEqual(result["statusCounts"], {"verified": 1, "candidate": 1})
        self.assertEqual(result["resolverUnmatched"], self.manifest["unmatched"])

    def test_changed_source_or_pdf(self):
        for name in (self.sample, self.pdf):
            with self.subTest(name=name):
                path = self.root / name
                path.write_bytes(b"changed")
                self.assertEqual(self.result()["pairs"][0]["status"], "mismatch")
                path.write_bytes(name.encode())

    def test_hashes_without_print_and_missing_file(self):
        self.record["origin"] = {"kind": "SaveAs"}
        row = self.result()["pairs"][0]
        self.assertEqual((row["status"], row["pdfHash"]["state"]), ("candidate", "match"))
        (self.root / self.pdf).unlink()
        self.assertEqual(self.result()["pairs"][0]["status"], "unavailable")

    def test_explicit_pair_outside_name_matching(self):
        self.manifest["pairs"] = []
        row = self.result()["pairs"][0]
        self.assertEqual(row["status"], "verified")
        self.assertFalse(row["resolverCandidate"])

    def test_refuse_duplicate_or_escaping_record(self):
        with self.assertRaises(ValueError):
            index_receipts({"cases": [self.record, self.record]})
        self.record["pdf"] = "../outside.pdf"
        with self.assertRaises(ValueError):
            self.result()

    def test_mcp_print_requires_driver_and_matching_engine(self):
        origin = {"kind": "MCP Print", "engine": "2024", "job_id": "job",
                  "print_result": {"engine": "2024", "hancom_version": "13.0.0.3901",
                                   "preprocess": None, "pdf_print_method": 0, "pdf_output_mode": "frame_print_to_pdf_ex_one_up"}}
        self.assertIsNotNone(print_details(origin))
        origin["print_result"]["pdf_output_mode"] = "export"
        self.assertIsNone(print_details(origin))
