#!/usr/bin/env python3
"""#6374 --args 는 형식·엔진 미확인 PDF 를 고르지 않고 fail-closed 한다."""
from __future__ import annotations

import importlib.util
import io
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

MODULE_PATH = Path(__file__).resolve().parent / 'oracle_pair_index.py'
SPEC = importlib.util.spec_from_file_location('oracle_pair_index', MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f'oracle_pair_index 모듈을 불러올 수 없습니다: {MODULE_PATH}')
PAIR = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = PAIR
SPEC.loader.exec_module(PAIR)

HANDBOOK = 'samples/2025 행정업무운영 편람(최종).hwpx'


class ArgsFailClosedTests(unittest.TestCase):
    def test_handbook_args_selects_hwpx_2024_not_2010_kopub(self) -> None:
        pdfs = [
            'pdf/2025 행정업무운영 편람(최종)-2010-kopub.pdf',
            'pdf/2025 행정업무운영 편람(최종)-2024.pdf',
            'pdf/2025 행정업무운영 편람(최종)-hwpx-2020.pdf',
            'pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf',
        ]
        with mock.patch.object(PAIR, 'git_pdfs', return_value=pdfs):
            with mock.patch.object(PAIR, 'samples', return_value=[HANDBOOK]):
                stdout = io.StringIO()
                stderr = io.StringIO()
                with mock.patch.object(sys, 'argv', ['oracle_pair_index.py', '--args', HANDBOOK]):
                    with mock.patch('sys.stdout', stdout), mock.patch('sys.stderr', stderr):
                        code = PAIR.main()
        self.assertEqual(code, 0, stderr.getvalue())
        out = stdout.getvalue()
        self.assertIn('--reference-pdf "pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf"', out)
        self.assertNotIn('2010-kopub', out)
        self.assertNotIn('편람(최종)-2024.pdf', out)

    def test_args_fails_when_only_unlabeled_or_font_condition_pdfs_exist(self) -> None:
        sample = 'samples/example.hwpx'
        pdfs = [
            'pdf/example-2024.pdf',
            'pdf/example-hwpx-kopub-2020.pdf',
        ]
        with mock.patch.object(PAIR, 'git_pdfs', return_value=pdfs):
            with mock.patch.object(PAIR, 'samples', return_value=[sample]):
                stdout = io.StringIO()
                stderr = io.StringIO()
                with mock.patch.object(sys, 'argv', ['oracle_pair_index.py', '--args', sample]):
                    with mock.patch('sys.stdout', stdout), mock.patch('sys.stderr', stderr):
                        code = PAIR.main()
        self.assertEqual(code, 1)
        self.assertEqual(stdout.getvalue().strip(), '')
        self.assertIn('canonical', stderr.getvalue())

    def test_args_fails_when_newest_engine_has_two_pdfs(self) -> None:
        sample = 'samples/example.hwpx'
        pdfs = [
            'pdf/example-hwpx-2024.pdf',
            'pdf/example-2024-hwpx.pdf',
        ]
        with mock.patch.object(PAIR, 'git_pdfs', return_value=pdfs):
            with mock.patch.object(PAIR, 'samples', return_value=[sample]):
                stdout = io.StringIO()
                stderr = io.StringIO()
                with mock.patch.object(sys, 'argv', ['oracle_pair_index.py', '--args', sample]):
                    with mock.patch('sys.stdout', stdout), mock.patch('sys.stderr', stderr):
                        code = PAIR.main()
        self.assertEqual(code, 1)
        self.assertEqual(stdout.getvalue().strip(), '')
        self.assertIn('모호', stderr.getvalue())


class LiveHandbookArgsTests(unittest.TestCase):
    def test_repo_handbook_hwpx_args_do_not_emit_2010_kopub(self) -> None:
        repo = Path(__file__).resolve().parents[2]
        sample = repo / 'samples' / '2025 행정업무운영 편람(최종).hwpx'
        if not sample.is_file():
            self.skipTest('편람 HWPX 샘플이 없다')
        completed = subprocess.run(
            [sys.executable, str(MODULE_PATH), '--args', HANDBOOK],
            cwd=repo,
            check=False,
            text=True,
            encoding='utf-8',
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn(
            '--reference-pdf "pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf"',
            completed.stdout,
        )
        self.assertNotIn('2010-kopub', completed.stdout)


if __name__ == '__main__':
    unittest.main()
