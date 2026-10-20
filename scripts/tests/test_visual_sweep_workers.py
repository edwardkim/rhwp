"""병렬화가 쪽 순서, 손상 글꼴 판정, 점수와 입력 해시를 바꾸지 않는지 검사한다."""

import base64
import json
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

from scripts import visual_sweep as sweep
from scripts.visual_sweep_workers import ordered_page_map
from scripts.tests.test_visual_sweep_font_cmap import font_bytes


class PageWorkerTests(unittest.TestCase):
    def test_parallel_font_report_keeps_failed_fonts_and_page_order(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            sources = []
            for index, data in enumerate([font_bytes('AB'), b'broken font', font_bytes('A')]):
                source = folder / f'{index}.svg'
                source.write_text('<svg><style>@font-face{font-family:Fixture;src:url(data:font/ttf;base64,'
                                  + base64.b64encode(data).decode('ascii') + ')}</style></svg>')
                sources.append(source)
            reports = []
            for workers in (1, 4):
                report = folder / f'fonts-{workers}.json'
                with self.assertRaisesRegex(SystemExit, '임베딩 글꼴 검사 실패'):
                    sweep.check_sweep_embedded_fonts(sources, report, workers=workers)
                reports.append(json.loads(report.read_text()))
            self.assertEqual(reports[0], reports[1])
            self.assertEqual([p['svg'] for p in reports[1]['pages']], list(map(str, sources)))
            self.assertEqual(reports[1]['pages'][1]['fonts'][0]['status'], 'failed')

    def test_parallel_tsv_keeps_exact_metrics_input_hashes_and_missing_page_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            pairs = []
            for page, offset in [(1, 0), (2, 1), (3, 10), (4, 0)]:
                paths = []
                for name, x in [('rhwp', 12 + offset), ('pdf', 12)]:
                    image = Image.new('RGB', (64, 64), 'white')
                    ImageDraw.Draw(image).rectangle((x, 12, x + 10, 30), fill='black')
                    path = folder / f'{name}-{page}.png'
                    image.save(path)
                    paths.append(path)
                pairs.append((page, *paths))
            results = [sweep.write_silhouette_tsv(pairs, folder / str(workers), 'same',
                       document_counts={'rhwp': 4, 'pdf': 5}, workers=workers) for workers in (1, 4)]
            self.assertEqual(Path(results[0]['tsv']).read_bytes(), Path(results[1]['tsv']).read_bytes())
            for field in ['inputs', 'metrics', 'pr_review_gate', 'completed_pages', 'missing_pages']:
                self.assertEqual(results[0][field], results[1][field])
            self.assertEqual(results[1]['pr_review_gate']['status'], 're_review_required')

    def test_worker_count_rejects_zero_negative_and_noninteger(self):
        for workers in (0, -1, 1.5, True):
            with self.subTest(workers=workers), self.assertRaises(ValueError):
                list(ordered_page_map(abs, [-1], workers))


if __name__ == '__main__':
    unittest.main()
