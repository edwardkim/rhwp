#!/usr/bin/env python3
"""`oracle_pdf_selection`의 형식·엔진 fail-closed 계약."""

import pathlib
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


if __name__ == '__main__':
    unittest.main()
