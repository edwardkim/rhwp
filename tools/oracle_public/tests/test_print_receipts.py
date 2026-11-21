"""Print 갱신의 다운로드 검증과 독립 감사 사이의 파일 계약."""

import hashlib
import json
import os
from pathlib import Path
from types import SimpleNamespace
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import refresh_oracle_pdfs as refresh
from oracle_public.audit_print_pairs import audit


class PrintReceipts(unittest.TestCase):
    def setUp(self):
        work = tempfile.TemporaryDirectory()
        self.addCleanup(work.cleanup)
        self.addCleanup(os.chdir, os.getcwd())
        os.chdir(work.name)
        self.source = Path('samples/doc.hwp')
        self.pdf = Path('pdf/doc-hwp-2020.pdf')
        self.source.parent.mkdir()
        self.source.write_bytes(b'input')
        self.pdf.parent.mkdir()
        self.pdf.write_bytes(b'previous')
        self.out = Path('receipts')
        self.args = SimpleNamespace(receipt_dir=self.out, timeout_seconds=5, status_interval_seconds=1,
                                    status_retry_count=1, npx='npx', package='test')
        self.result = {'engine': '2020', 'hancom_version': '11.0.0.9136', 'pdf_print_method': 0,
                       'pdf_output_mode': 'hancom2020_pdf_driver_one_up', 'preprocess': None,
                       'pdf_page_count': 1}
        self.status = {'status': 'succeeded', 'job_id': 'job', 'result': self.result}
        self.download = {'status': 'success', 'job_id': 'job', 'size': 6,
                         'sha256': hashlib.sha256(b'output').hexdigest()}

    def convert(self, change_source=False):
        def client(argv, label, logger):
            action = argv[argv.index('hwp2024-mcp-convert') + 1]
            if action == 'start':
                submitted = Path(argv[argv.index('--input') + 1])
                self.assertEqual(submitted.read_bytes(), b'input')
                if change_source:
                    self.source.write_bytes(b'changed')
                    self.assertEqual(submitted.read_bytes(), b'input')
                return {'job_id': 'job'}
            if action == 'status':
                return self.status
            directory = Path(argv[argv.index('--output-dir') + 1])
            (directory / self.pdf.name).write_bytes(b'output')
            return self.download
        with patch.object(refresh, 'run_mcp', side_effect=client):
            refresh.convert_one(self.args, str(self.source), '2020', self.pdf, Mock(), 'env', 0)

    def test_receipt_matches_installed_pair_and_detects_replacement(self):
        self.convert()
        receipt, = self.out.glob('*.json')
        self.assertEqual(receipt.stem, hashlib.sha256(receipt.read_bytes()).hexdigest())
        document = json.loads(receipt.read_bytes())
        manifest = {'pairs': [], 'unmatched': []}
        self.assertEqual(audit(Path.cwd(), manifest, document)['statusCounts'], {'verified': 1})
        self.source.write_bytes(b'new')
        self.assertEqual(audit(Path.cwd(), manifest, document)['statusCounts'], {'mismatch': 1})

    def test_invalid_evidence_preserves_previous_pdf(self):
        for field, value in [('sha256', 'wrong'), ('job_id', 'other'), ('size', 4)]:
            with self.subTest(field=field), patch.dict(self.download, {field: value}):
                with self.assertRaises(RuntimeError):
                    self.convert()
        with patch.dict(self.status, job_id='other'):
            with self.assertRaises(RuntimeError):
                self.convert()
        with patch.dict(self.status, status='failed', error='input preflight rejected the document: test'):
            with self.assertRaises(refresh.DocumentPreflightRejected):
                self.convert()
        with patch.dict(self.result, pdf_output_mode='export'):
            with self.assertRaises(RuntimeError):
                self.convert()
        with self.assertRaises(RuntimeError):
            self.convert(change_source=True)
        self.assertEqual(self.pdf.read_bytes(), b'previous')
        self.assertEqual(list(self.out.glob('*.json')), [])

    def test_existing_download_without_receipt_option(self):
        self.args.receipt_dir = None
        self.result.clear()
        self.download.clear()
        self.convert()
        self.assertEqual(self.pdf.read_bytes(), b'output')
        self.assertFalse(self.out.exists())
