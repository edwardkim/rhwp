#!/usr/bin/env python3
"""`oracle_pdf_selection`의 형식·엔진 fail-closed 계약."""

import pathlib
import contextlib
import hashlib
import io
import json
import os
import tempfile
from unittest import mock
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from oracle_pdf_selection import (
    canonical_candidates,
    canonical_filename,
    canonical_pdf_path,
    choose_canonical,
    engine_for_product,
)


from oracle_page_count import regenerate


class OraclePdfSelectionTests(unittest.TestCase):
    def setUp(self):
        self.guide_hwp_2020 = canonical_pdf_path('samples/guide.hwp', '2020')
        self.guide_hwp_2024 = canonical_pdf_path('samples/guide.hwp', '2024')
        self.guide_hwpx_2020 = canonical_pdf_path('samples/guide.hwpx', '2020')
        self.guide_hwpx_2024 = canonical_pdf_path('samples/guide.hwpx', '2024')
        self.candidates = [
            'pdf/guide-2010-kopub.pdf',
            self.guide_hwp_2020,
            self.guide_hwp_2024,
            self.guide_hwpx_2020,
            self.guide_hwpx_2024,
        ]

    def test_candidates_never_cross_source_format(self):
        self.assertEqual(
            canonical_candidates('samples/guide.hwpx', self.candidates),
            [self.guide_hwpx_2020, self.guide_hwpx_2024],
        )

    def test_multiple_engines_require_explicit_choice(self):
        with self.assertRaisesRegex(ValueError, '--engine'):
            choose_canonical('samples/guide.hwpx', self.candidates)
        self.assertEqual(
            choose_canonical('samples/guide.hwpx', self.candidates, engine='2024'),
            self.guide_hwpx_2024,
        )

    def test_legacy_only_pdf_is_not_automatic_reference(self):
        with self.assertRaisesRegex(ValueError, 'canonical'):
            choose_canonical('samples/guide.hwp', ['pdf/guide-2010-kopub.pdf'])

    def test_same_name_sources_never_share_a_canonical_pdf(self):
        root = 'samples/guide.hwp'
        nested = 'samples/basic/guide.hwp'
        root_pdf = canonical_pdf_path(root, '2020')
        nested_pdf = canonical_pdf_path(nested, '2020')
        self.assertNotEqual(root_pdf, nested_pdf)
        self.assertEqual(canonical_candidates(root, [root_pdf, nested_pdf]), [root_pdf])
        self.assertEqual(canonical_candidates(nested, [root_pdf, nested_pdf]), [nested_pdf])

    def test_source_version_suffix_is_part_of_the_canonical_identity(self):
        older = canonical_pdf_path('samples/hwp3-sample16-hwp5-2018.hwp', '2020')
        newer = canonical_pdf_path('samples/hwp3-sample16-hwp5-2022.hwp', '2020')
        self.assertEqual(older, 'pdf/hwp3-sample16-hwp5-2018-hwp-2020.pdf')
        self.assertEqual(newer, 'pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf')
        self.assertNotEqual(older, newer)

    def test_engine_comes_from_saved_product_not_extension(self):
        self.assertEqual(engine_for_product('hancom-office-2024'), '2024')
        self.assertEqual(engine_for_product('hancom-office-2022'), '2020')
        self.assertEqual(engine_for_product(None), '2020')
        self.assertEqual(canonical_filename('samples/guide.hwpx', '2024'),
                         'guide-hwpx-2024.pdf')


class OracleVerifiedReferenceTests(unittest.TestCase):
    """검증한 과거 경로의 수집 보존과 바뀐 바이트의 거절을 검사한다."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.sample = 'samples/guide.hwp'
        self.pdf = 'pdf/guide-2020.pdf'
        self.pdf_bytes = b'%PDF-legacy-test-reference'
        for relative, data in [(self.sample, b'original-source'), (self.pdf, self.pdf_bytes)]:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.entry = dict(sample=self.sample, source_sha256=self.digest(self.sample),
                          pdf=self.pdf, pdf_sha256=self.digest(self.pdf), pages=3,
                          provenance='desktop_hancom_print', evidence='verified-test-fixture')
        self.registry = self.root / 'tests/fixtures/oracle_page_count_verified_references.json'
        self.registry.parent.mkdir(parents=True, exist_ok=True)
        self.write_registry([self.entry])

    def digest(self, relative):
        return hashlib.sha256((self.root / relative).read_bytes()).hexdigest()

    def write_registry(self, entries):
        self.registry.write_text(json.dumps(dict(schema_version=1, references=entries)),
                                 encoding='utf-8')

    def run_regeneration(self, extracted_pdf=None):
        scratch = self.root / 'scratch'
        scratch.mkdir(exist_ok=True)
        baseline = self.root / 'generated.tsv'
        def oracle(_pdf, temporary):
            pathlib.Path(temporary).write_bytes(self.pdf_bytes if extracted_pdf is None else extracted_pdf)
            return 3, ['표본'] * 3
        before = pathlib.Path.cwd()
        try:
            os.chdir(self.root)
            with (mock.patch.object(sys, 'argv', ['regenerate.py', '--rhwp', 'mock-rhwp']),
                  mock.patch.object(regenerate, 'FIXTURE', str(baseline)),
                  mock.patch.object(regenerate, 'sample_paths', return_value=[self.sample]),
                  mock.patch.object(regenerate, 'git_pdf_paths', return_value=[self.pdf]),
                  mock.patch.object(regenerate, 'rhwp_info', return_value=(3, False, None)),
                  mock.patch.object(regenerate, 'rhwp_text', return_value=['표본'] * 3),
                  mock.patch.object(regenerate, 'oracle_pages_and_text', side_effect=oracle),
                  mock.patch.object(regenerate.tempfile, 'mkdtemp', return_value=str(scratch)),
                  contextlib.redirect_stdout(io.StringIO())):
                regenerate.main()
        finally:
            os.chdir(before)
        return baseline.read_text(encoding='utf-8')

    def test_registered_legacy_pair_does_not_weaken_canonical_selection(self):
        with self.assertRaises(ValueError):
            choose_canonical(self.sample, [self.pdf], engine='2020')
        refs = regenerate.load_verified_references(self.root)
        self.assertEqual(refs[self.sample]['pdf'], self.pdf)
        self.assertNotIn('samples/another/guide.hwp', refs)

    def test_regenerator_retains_exact_registered_legacy_pair(self):
        output = self.run_regeneration()
        self.assertIn('samples/guide.hwp\t3\t3', output)
        self.assertIn(self.pdf, output)

    def test_changed_source_or_reference_bytes_are_rejected(self):
        for relative in [self.sample, self.pdf]:
            with self.subTest(path=relative):
                path = self.root / relative
                old = path.read_bytes()
                path.write_bytes(old + b'changed')
                try:
                    with self.assertRaisesRegex(ValueError, '바이트'):
                        regenerate.load_verified_references(self.root)
                finally:
                    path.write_bytes(old)

    def test_git_reference_bytes_cannot_change_behind_the_same_page_count(self):
        with self.assertRaisesRegex(ValueError, 'Git'):
            self.run_regeneration(extracted_pdf=b'%PDF-other-program-same-page-count')

    def test_duplicate_input_and_unverified_provenance_are_rejected(self):
        self.write_registry([self.entry, self.entry])
        with self.assertRaisesRegex(ValueError, '중복'):
            regenerate.load_verified_references(self.root)
        self.write_registry([dict(self.entry, provenance='web_drafter')])
        with self.assertRaisesRegex(ValueError, '출처'):
            regenerate.load_verified_references(self.root)

    def test_invalid_independent_page_count_is_rejected(self):
        for count in [0, -1, True, '3']:
            with self.subTest(count=count):
                self.write_registry([dict(self.entry, pages=count)])
                with self.assertRaisesRegex(ValueError, '쪽수'):
                    regenerate.load_verified_references(self.root)

    def test_registered_independent_page_count_must_match_extracted_pdf(self):
        self.write_registry([dict(self.entry, pages=4)])
        with self.assertRaisesRegex(ValueError, '독립 PDF'):
            self.run_regeneration()

    def test_restored_five_sources_keep_their_verified_references(self):
        root = pathlib.Path(__file__).resolve().parents[1]
        refs = regenerate.load_verified_references(root)
        expected = {
            'samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp': 103,
            'samples/한글문서파일형식_5.0_revision1.3.hwp': 71,
            'samples/pr4093/outline_navigation_panel_demo.hwpx': 3,
            'samples/issue6776/78494-virtual-convergence-industry-decree.hwpx': 74,
            'samples/pr4093/outline_navigation_table_cell_number.hwpx': 1,
        }
        for sample, pages in expected.items():
            with self.subTest(sample=sample):
                self.assertEqual(refs[sample]['pages'], pages)
        self.assertEqual(refs['samples/한글문서파일형식_5.0_revision1.3.hwp']['provenance'],
                         'official_hancom_specification')


if __name__ == '__main__':
    unittest.main()
