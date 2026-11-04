from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from PIL import Image

from scripts import visual_sweep as sweep
from scripts.visual_sweep_resume import prepare_target_plan, render_silhouette_resumable


class SilhouetteResumeTests(unittest.TestCase):
    def test_multiple_target_plan_detects_missing_or_changed_target(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            for name in ('one.hwp', 'two.hwp', 'one.pdf', 'two.pdf', 'bin'):
                (root / name).write_bytes(name.encode())
            out = root / 'out'
            out.mkdir()
            targets = [sweep.Target('one', root / 'one.hwp', root / 'one.pdf'),
                       sweep.Target('two', root / 'two.hwp', root / 'two.pdf')]
            options = dict(rhwp_bin='bin', dpi=96, compat='2022', rasterizer='png',
                           wasm_pkg=None, linked_images=[])
            prepare_target_plan(sweep, root, out, targets, None, resume=False, **options)
            prepare_target_plan(sweep, root, out, targets, None, resume=True, **options)
            with self.assertRaisesRegex(SystemExit, '대상 목록'):
                prepare_target_plan(sweep, root, out, targets[:1], None, resume=True, **options)
            (root / 'two.pdf').write_bytes(b'changed')
            with self.assertRaisesRegex(SystemExit, '대상 목록'):
                prepare_target_plan(sweep, root, out, targets, None, resume=True, **options)

    def test_linked_images_record_content_and_reject_duplicate_names(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            first = root / 'a' / 'photo.gif'
            second = root / 'b' / 'PHOTO.gif'
            first.parent.mkdir()
            second.parent.mkdir()
            first.write_bytes(b'first')
            second.write_bytes(b'second')
            records = sweep.linked_image_records(root, [first])
            self.assertEqual(records[0]['name'], 'photo.gif')
            self.assertEqual(records[0]['sha256'], sweep.sha256_file(first))
            with self.assertRaisesRegex(SystemExit, '중복'):
                sweep.linked_image_records(root, [first, second])
            if os.name != 'nt':
                alias = root / 'alias.gif'
                alias.symlink_to(first)
                self.assertEqual(sweep.linked_image_records(root, [alias])[0]['name'], 'alias.gif')

    def test_excluded_regions_are_part_of_resume_identity(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            a, b = root / 'a.png', root / 'b.png'
            Image.new('RGB', (12, 12), 'black').save(a)
            Image.new('RGB', (12, 12), 'black').save(b)
            pairs = [(1, a, b)]
            regions = {1: [(0, 0, 3, 3)]}
            evidence = {'sha256': 'manifest-a'}
            sweep.write_silhouette_tsv(pairs, root / 'out', 'pair',
                                       excluded_regions=regions, exclusion_evidence=evidence)
            result = sweep.write_silhouette_tsv(pairs, root / 'out', 'pair', resume=True,
                                                excluded_regions=regions, exclusion_evidence=evidence)
            self.assertEqual(result['completed_pages'], [1])
            with self.assertRaisesRegex(SystemExit, '입력 또는 계산 환경'):
                sweep.write_silhouette_tsv(pairs, root / 'out', 'pair', resume=True,
                                           excluded_regions={1: [(0, 0, 4, 4)]},
                                           exclusion_evidence=evidence)

    def test_png_pair_interruption_resumes_and_rejects_changed_input(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            pairs = []
            for page in (1, 2):
                a, b = root / f'a{page}.png', root / f'b{page}.png'
                Image.new('RGB', (12, 12), 'black').save(a)
                Image.new('RGB', (12, 12), 'black').save(b)
                pairs.append((page, a, b))
            out = root / 'result'
            real = sweep.silhouette_pair_metrics
            def interrupted(pair):
                if pair[0] == 2:
                    raise RuntimeError('simulated interruption')
                return real(pair)
            with patch.object(sweep, 'silhouette_pair_metrics', side_effect=interrupted):
                with self.assertRaisesRegex(RuntimeError, 'simulated interruption'):
                    sweep.write_silhouette_tsv(pairs, out, 'pair')
            self.assertTrue((out / 'silhouette-metrics/p000001.json').is_file())
            self.assertFalse((out / 'silhouette.tsv').exists())
            self.assertEqual(json.loads((out / 'silhouette_manifest.json').read_text())['run_state'], 'incomplete')
            with patch.object(sweep, 'silhouette_pair_metrics', wraps=real) as calculate:
                result = sweep.write_silhouette_tsv(pairs, out, 'pair', resume=True)
            self.assertEqual(calculate.call_count, 1)
            self.assertEqual(result['completed_pages'], [1, 2])
            with self.assertRaisesRegex(SystemExit, '쪽 목록'):
                sweep.write_silhouette_tsv(pairs[:1], out, 'pair', resume=True)
            self.assertTrue((out / 'silhouette.tsv').is_file())
            Image.new('RGB', (12, 12), 'white').save(pairs[0][1])
            with self.assertRaisesRegex(SystemExit, '입력 또는 계산 환경'):
                sweep.write_silhouette_tsv(pairs, out, 'pair', resume=True)
            self.assertFalse((out / 'silhouette.tsv').exists())

    def test_document_shards_resume_only_after_provenance_validation(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            source, pdf = root / 'doc.hwp', root / 'doc.pdf'
            source.write_bytes(b'document')
            pdf.write_bytes(b'pdf')
            (root / 'bin').write_bytes(b'test binary')
            target = sweep.Target('doc', source, pdf)
            calls = []
            fail = True
            def fake_render(root, target, out, binary, dpi, threshold, pages, *, resume, **kwargs):
                nonlocal fail
                calls.append((tuple(pages), resume))
                part = out / target.key
                part.mkdir(parents=True, exist_ok=True)
                sweep.write_json_atomic(part / 'run_manifest.json', {'valid': True})
                if pages[0] == 2 and fail:
                    fail = False
                    raise RuntimeError('interrupted')
                pairs = []
                for page in pages:
                    a, b = part / f'rhwp_{page:03}.png', part / f'pdf_{page:03}.png'
                    Image.new('RGB', (8, 8), 'black').save(a)
                    Image.new('RGB', (8, 8), 'black').save(b)
                    pairs.append((page, a, b))
                result = sweep.write_silhouette_tsv(pairs, part, target.key,
                                                     document_counts={'rhwp': 3, 'pdf': 3})
                sweep.write_json_atomic(part / 'native-export.json', {
                    'pageCount': 3, 'pages': [{'page': p - 1} for p in pages],
                })
                result['provenance'] = {'fixture': True}
                sweep.write_json_atomic(part / 'silhouette_manifest.json', result)
                return result
            kwargs = {'svg_rasterizer': 'webfont', 'page_workers': 1}
            with patch.object(sweep, 'render_target', side_effect=fake_render):
                with self.assertRaisesRegex(RuntimeError, 'interrupted'):
                    render_silhouette_resumable(sweep, root, target, root / 'out', 'bin', 96, 32,
                                                None, resume=False, **kwargs)
                result = render_silhouette_resumable(sweep, root, target, root / 'out', 'bin', 96, 32,
                                                     None, resume=True, **kwargs)
            self.assertEqual(result['completed_pages'], [1, 2, 3])
            self.assertEqual(calls, [((1,), False), ((2, 3), False), ((1,), True), ((2, 3), False)])
            self.assertEqual(len((root / 'out/doc/silhouette.tsv').read_text().splitlines()), 4)
            Image.new('RGB', (8, 8), 'white').save(
                root / 'out/doc/silhouette-parts/p0001-0001/rhwp_001.png'
            )
            with patch.object(sweep, 'render_target', side_effect=fake_render):
                with self.assertRaisesRegex(SystemExit, '체크포인트 PNG'):
                    render_silhouette_resumable(sweep, root, target, root / 'out', 'bin', 96, 32,
                                                None, resume=True, **kwargs)
            self.assertFalse((root / 'out/doc/silhouette.tsv').exists())
            source.write_bytes(b'changed document')
            with self.assertRaisesRegex(SystemExit, '실행 조건'):
                render_silhouette_resumable(sweep, root, target, root / 'out', 'bin', 96, 32,
                                            None, resume=True, **kwargs)
            source.write_bytes(b'document')
            (root / 'bin').write_bytes(b'changed binary')
            with self.assertRaisesRegex(SystemExit, '실행 조건'):
                render_silhouette_resumable(sweep, root, target, root / 'out', 'bin', 96, 32,
                                            None, resume=True, **kwargs)


if __name__ == '__main__':
    unittest.main()
