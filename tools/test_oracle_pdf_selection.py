#!/usr/bin/env python3
"""`oracle_pdf_selection`의 형식·엔진 fail-closed 계약."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from oracle_pdf_selection import (
    canonical_candidates,
    canonical_filename,
    choose_canonical,
    engine_for_product,
)


class OraclePdfSelectionTests(unittest.TestCase):
    def setUp(self):
        self.candidates = [
            'pdf/guide-2010-kopub.pdf',
            'pdf/guide-hwp-2020.pdf',
            'pdf/guide-hwp-2024.pdf',
            'pdf/guide-hwpx-2020.pdf',
            'pdf/guide-hwpx-2024.pdf',
        ]

    def test_candidates_never_cross_source_format(self):
        self.assertEqual(
            canonical_candidates('samples/guide.hwpx', self.candidates),
            ['pdf/guide-hwpx-2020.pdf', 'pdf/guide-hwpx-2024.pdf'],
        )

    def test_multiple_engines_require_explicit_choice(self):
        with self.assertRaisesRegex(ValueError, '--engine'):
            choose_canonical('samples/guide.hwpx', self.candidates)
        self.assertEqual(
            choose_canonical('samples/guide.hwpx', self.candidates, engine='2024'),
            'pdf/guide-hwpx-2024.pdf',
        )

    def test_legacy_only_pdf_is_not_automatic_reference(self):
        with self.assertRaisesRegex(ValueError, 'canonical'):
            choose_canonical('samples/guide.hwp', ['pdf/guide-2022.pdf'])

    def test_engine_comes_from_saved_product_not_extension(self):
        self.assertEqual(engine_for_product('hancom-office-2024'), '2024')
        self.assertEqual(engine_for_product('hancom-office-2022'), '2020')
        self.assertEqual(engine_for_product(None), '2020')
        self.assertEqual(
            canonical_filename('samples/guide.hwpx', '2024'),
            'guide-hwpx-2024.pdf',
        )


if __name__ == '__main__':
    unittest.main()
