#!/usr/bin/env python3
"""#6374 canonical 정답지 선택 — 형식·엔진 미확인 PDF 를 허용 집합에 섞지 않는다."""
from __future__ import annotations

import io
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pairing import (  # noqa: E402
    is_canonical_oracle,
    newest_engine_oracles,
    pick_canonical_oracles,
    select_args_pdf,
)

HANDBOOK = 'samples/2025 행정업무운영 편람(최종).hwpx'
HANDBOOK_HWP = 'samples/2025 행정업무운영 편람(최종).hwp'
HANDBOOK_PDFS = [
    'pdf/2025 행정업무운영 편람(최종)-2010-kopub.pdf',
    'pdf/2025 행정업무운영 편람(최종)-2010-no-ttf.pdf',
    'pdf/2025 행정업무운영 편람(최종)-2020-kopub.pdf',
    'pdf/2025 행정업무운영 편람(최종)-2024.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwpx-2020.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf',
    'pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2020.pdf',
]


class CanonicalOracleTests(unittest.TestCase):
    def test_unlabeled_and_font_conditions_are_not_canonical(self) -> None:
        self.assertFalse(
            is_canonical_oracle('pdf/doc-2024.pdf', 'hwpx'),
        )
        self.assertFalse(
            is_canonical_oracle('pdf/doc-hwpx-kopub-2020.pdf', 'hwpx'),
        )
        self.assertFalse(
            is_canonical_oracle('pdf/doc-hwpx-2020-no-ttf.pdf', 'hwpx'),
        )
        self.assertTrue(
            is_canonical_oracle('pdf/doc-hwpx-2024.pdf', 'hwpx'),
        )
        self.assertFalse(
            is_canonical_oracle('pdf/doc-hwp-2024.pdf', 'hwpx'),
        )

    def test_handbook_hwpx_keeps_only_format_engine_pdfs(self) -> None:
        chosen = pick_canonical_oracles(HANDBOOK, HANDBOOK_PDFS)
        self.assertEqual(
            chosen,
            [
                'pdf/2025 행정업무운영 편람(최종)-hwpx-2020.pdf',
                'pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf',
            ],
        )

    def test_handbook_hwp_does_not_take_hwpx_or_kopub(self) -> None:
        chosen = pick_canonical_oracles(HANDBOOK_HWP, HANDBOOK_PDFS)
        self.assertEqual(
            chosen,
            [
                'pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf',
                'pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf',
            ],
        )

    def test_args_picks_newest_engine_and_rejects_ambiguity(self) -> None:
        chosen = pick_canonical_oracles(HANDBOOK, HANDBOOK_PDFS)
        selected, reason = select_args_pdf(chosen)
        self.assertIsNone(reason)
        self.assertEqual(
            selected,
            'pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf',
        )

        none, missing = select_args_pdf([])
        self.assertIsNone(none)
        self.assertIn('canonical PDF가 없다', missing)

        newest = newest_engine_oracles(chosen)
        self.assertEqual(
            newest,
            ['pdf/2025 행정업무운영 편람(최종)-hwpx-2024.pdf'],
        )
        ambiguous, why = select_args_pdf(
            newest + ['pdf/2025 행정업무운영 편람(최종)-2024-hwpx.pdf'],
        )
        self.assertIsNone(ambiguous)
        self.assertIn('모호', why)

    def test_unlabeled_only_pool_is_empty(self) -> None:
        chosen = pick_canonical_oracles(
            'samples/basic/sungeo.hwp',
            ['pdf/basic/sungeo-2022.pdf'],
        )
        self.assertEqual(chosen, [])

    def test_issue1510_format_tags_stay_split(self) -> None:
        cands = [
            'pdf/issue1510_coanchored_float_tables-hwp-2024.pdf',
            'pdf/issue1510_coanchored_float_tables-hwpx-2024.pdf',
        ]
        self.assertEqual(
            pick_canonical_oracles(
                'samples/issue1510_coanchored_float_tables.hwp', cands,
            ),
            ['pdf/issue1510_coanchored_float_tables-hwp-2024.pdf'],
        )
        self.assertEqual(
            pick_canonical_oracles(
                'samples/issue1510_coanchored_float_tables.hwpx', cands,
            ),
            ['pdf/issue1510_coanchored_float_tables-hwpx-2024.pdf'],
        )


class FixtureContractTests(unittest.TestCase):
    def test_handbook_tsv_uses_canonical_384_not_mixed_font_counts(self) -> None:
        root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        path = os.path.join(root, 'tests', 'fixtures', 'oracle_page_count_baseline.tsv')
        with io.open(path, encoding='utf-8') as fh:
            rows = {
                cols[0]: (cols[1], cols[2])
                for line in fh
                if line.strip() and not line.startswith('#')
                for cols in [line.rstrip('\n').split('\t')]
            }
        self.assertEqual(rows[HANDBOOK_HWP], ('384', '384'))
        self.assertEqual(rows[HANDBOOK], ('384', '382'))
        self.assertNotIn('383', rows[HANDBOOK][0])
        self.assertNotIn('388', rows[HANDBOOK][0])
        self.assertNotIn('389', rows[HANDBOOK][0])


if __name__ == '__main__':
    unittest.main()
