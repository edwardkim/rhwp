"""관계형: 편람 기준 TSV 하한·backend 신원과 웹폰트 공급 계약을 검사한다.

주석 갱신: 2026-11-08T22:50:22+09:00 (Asia/Seoul).
실물 절대 px 좌표를 고정하는 검사가 아니다.
"""
import copy
import csv
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import visual_sweep_handbook as handbook
from test_visual_sweep import SWEEP

ROOT = Path(__file__).resolve().parents[2]


class HandbookGateTests(unittest.TestCase):
    def setUp(self):
        # 제품의 선택 helper와 독립적으로 정본의 HWPX Native 행만 후보로 준비한다.
        with (ROOT / handbook.BASELINE).open(encoding='utf-8', newline='') as stream:
            self.rows = [row for row in csv.DictReader(stream, delimiter='\t')
                         if row['input_sha256'] == handbook.DOC_SHA
                         and row['reference_pdf_sha256'] == handbook.PDF_SHA
                         and row['backend'] == 'native']
        self.manifest = {
            'run_state': 'complete', 'radius_px': 2, 'silhouette_method': handbook.METHOD,
            'provenance': {
                'hwp': {'sha256': handbook.DOC_SHA}, 'pdf': {'sha256': handbook.PDF_SHA},
                'raster_backend': 'native-skia', 'layout_generation': '2024',
                'comparison_profile': 'print', 'pdf_rasterizer': {'policy': handbook.POLICY},
            },
            'pr_review_gate': {'document_page_counts': {'rhwp': 383, 'pdf': 383}},
            'metrics': copy.deepcopy(self.rows),
        }

    def test_all_recorded_pages_pass_at_recorded_precision(self):
        result = handbook.compare(ROOT, self.manifest)
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['pages'], 383)
        self.assertEqual(result['regressions'], [])

    def test_consolidated_baseline_selects_only_handbook_hwpx_and_requested_backend(self):
        for backend, label in [('native-skia', 'native'), ('wasm-canvas2d', 'wasm')]:
            rows = handbook.baseline_rows(ROOT, backend)
            self.assertEqual([int(row['page']) for row in rows], list(range(1, 384)))
            self.assertEqual({row['input_path'] for row in rows}, {str(handbook.DOC)})
            self.assertEqual({row['backend'] for row in rows}, {label})
            self.assertEqual(rows[129][handbook.METRIC], '99.96')
            self.assertTrue(all(row[handbook.METRIC] == '100.00'
                                for row in rows if row['page'] != '130'))

    def test_consolidated_baseline_rejects_changed_receipt_and_duplicate_dataset(self):
        original = json.loads((ROOT / handbook.BASELINE_MANIFEST).read_text(encoding='utf-8'))
        changed = copy.deepcopy(original)
        changed['table_sha256'] = '0' * 64
        duplicate = copy.deepcopy(original)
        dataset = next(item for item in duplicate['datasets']
                       if item['input']['sha256'] == handbook.DOC_SHA and item['backend'] == 'native')
        duplicate['datasets'].append(copy.deepcopy(dataset))
        for receipt in [changed, duplicate]:
            with patch.object(handbook.json, 'loads', return_value=receipt):
                self.assertEqual(handbook.compare(ROOT, self.manifest)['status'], 're_review_required')

    def test_explicit_wasm_comparison_keeps_the_same_floor_and_backend_identity(self):
        self.manifest['provenance']['raster_backend'] = 'wasm-canvas2d'
        self.assertEqual(handbook.compare(ROOT, self.manifest)['status'], 're_review_required')
        result = handbook.compare(ROOT, self.manifest, backend='wasm-canvas2d')
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['backend'], 'wasm-canvas2d')
        self.manifest['metrics'][197][handbook.METRIC] = '99.84'
        self.assertEqual(handbook.compare(ROOT, self.manifest, backend='wasm-canvas2d')['regressions'][0]['page'], 198)
        self.assertEqual(handbook.compare(ROOT, self.manifest, backend='svg')['status'], 're_review_required')

    def test_any_displayed_decrease_fails_without_offset_by_other_page(self):
        self.manifest['metrics'][129][handbook.METRIC] = '99.95'
        self.assertEqual(handbook.compare(ROOT, self.manifest)['regressions'][0]['page'], 130)
        self.assertEqual(handbook.compare(ROOT, self.manifest)['status'], 're_review_required')

    def test_recorded_two_decimal_precision_is_applied_to_candidates(self):
        self.manifest['metrics'][0][handbook.METRIC] = '99.99999'
        self.assertEqual(handbook.compare(ROOT, self.manifest)['status'], 'passed')
        self.manifest['metrics'][0][handbook.METRIC] = '99.994'
        self.assertEqual(handbook.compare(ROOT, self.manifest)['status'], 're_review_required')

    def test_partial_duplicate_mismatched_and_excluded_inputs_fail_closed(self):
        variants = []
        for mutate in (
            lambda x: x['metrics'].pop(),
            lambda x: x['metrics'].append(x['metrics'][0]),
            lambda x: x['metrics'][0].update({handbook.METRIC: 'NaN'}),
            lambda x: x['provenance']['hwp'].update(sha256='bad'),
            lambda x: x['provenance'].update(raster_backend='wasm-canvas2d'),
            lambda x: x['provenance']['pdf_rasterizer'].update(policy='other'),
            lambda x: x.update(excluded_regions={'regions': [1]}),
            lambda x: x.update(run_state='incomplete'),
        ):
            item = copy.deepcopy(self.manifest)
            mutate(item)
            variants.append(item)
        for item in variants:
            with self.subTest(item=item):
                self.assertEqual(handbook.compare(ROOT, item)['status'], 're_review_required')
        self.assertEqual(handbook.compare(ROOT, self.manifest, dpi=192)['status'], 're_review_required')

    def test_existing_fonts_do_not_trigger_network(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(handbook.urllib.request, 'urlopen') as fetch:
            self.assertEqual(handbook.supply_webfonts(ROOT, temp, lambda *_: True), [])
            fetch.assert_not_called()

    def test_unreadable_fontconfig_hit_is_not_an_installed_face(self):
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temp:
            blocked = Path(temp) / 'cached-font.ttf'
            blocked.write_bytes(b'font')
            original_open = Path.open

            def readable(path, *args, **kwargs):
                if path == blocked:
                    raise PermissionError('검증 환경에서 읽을 수 없는 글꼴')
                return original_open(path, *args, **kwargs)

            hit = SimpleNamespace(stdout=f'KoPubBatang|Light|{blocked}')
            with patch.object(handbook.subprocess, 'run', return_value=hit), \
                    patch.object(handbook, '_file_font_inventory', return_value=set()), \
                    patch.object(Path, 'open', readable):
                self.assertFalse(handbook.installed_face('KoPubBatang', 'Light'))
            with patch.object(handbook.subprocess, 'run', return_value=hit):
                self.assertTrue(handbook.installed_face('KoPubBatang', 'Light'))

    def test_inaccessible_font_directory_does_not_hide_other_fonts(self):
        blocked = Path('/unavailable-test-font-directory')
        fixture = ROOT / 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'
        original_is_file, original_is_dir, original_iterdir = Path.is_file, Path.is_dir, Path.iterdir
        handbook._file_font_inventory.cache_clear()
        try:
            with patch.object(Path, 'is_file', lambda path: False if path == blocked else original_is_file(path)), \
                    patch.object(Path, 'is_dir', lambda path: True if path == blocked else original_is_dir(path)), \
                    patch.object(Path, 'iterdir', lambda path: (_ for _ in ()).throw(PermissionError())
                                 if path == blocked else original_iterdir(path)):
                inventory = handbook._file_font_inventory((str(blocked), str(fixture)))
            self.assertTrue(inventory)
        finally:
            handbook._file_font_inventory.cache_clear()

    def test_wrong_webfont_hash_fails_closed(self):
        from io import BytesIO
        with tempfile.TemporaryDirectory() as temp, patch.object(
            handbook.urllib.request, 'urlopen', return_value=BytesIO(b'wrong')
        ):
            with self.assertRaisesRegex(ValueError, '해시'):
                handbook.supply_webfonts(ROOT, temp, lambda *_: False)

    def test_kopub_original_ttf_is_supplied_without_modification(self):
        from io import BytesIO
        original = (ROOT / 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf').read_bytes()
        expected = hashlib.sha256(original).hexdigest()
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(handbook, 'WEBFONTS', {('KoPubDotum', 'Light'): ('KoPubDotum-Light.ttf', expected)}), \
                patch.object(handbook.urllib.request, 'urlopen', return_value=BytesIO(original)) as fetch:
            records = handbook.supply_webfonts(ROOT, temp, lambda *_: False)
            fetch.assert_called_once_with(
                'https://cdn.jsdelivr.net/gh/fonts-archive/KoPubDotum@295e0fc83931fda749f129b6690c0fa93f798007/KoPubDotum-Light.ttf',
                timeout=60,
            )
            self.assertEqual(Path(records[0]['native_path']).read_bytes(), original)
            self.assertEqual(records[0]['native_sha256'], expected)

    def test_missing_face_is_converted_from_verified_webfont(self):
        from fontTools.ttLib import TTFont
        with tempfile.TemporaryDirectory() as temp:
            cache = Path(temp)
            original = ROOT / 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'
            font = TTFont(original, recalcTimestamp=False)
            font.flavor = 'woff'
            source = cache / 'fixture.woff'
            font.save(source)
            font.close()
            expected = hashlib.sha256(source.read_bytes()).hexdigest()
            (cache / expected).mkdir()
            source.replace(cache / expected / source.name)
            with patch.object(handbook, 'WEBFONTS', {('Missing', 'Regular'): ('fixture.woff', expected)}), \
                    patch.object(handbook.urllib.request, 'urlopen') as fetch, \
                    patch('fontTools.ttLib.tables._h_e_a_d.timestampNow', return_value=3_900_000_000):
                records = handbook.supply_webfonts(ROOT, cache, lambda *_: False)
            first_bytes = Path(records[0]['native_path']).read_bytes()
            with patch.object(handbook, 'WEBFONTS', {('Missing', 'Regular'): ('fixture.woff', expected)}), \
                    patch.object(handbook.urllib.request, 'urlopen') as second_fetch, \
                    patch('fontTools.ttLib.tables._h_e_a_d.timestampNow', return_value=3_900_000_060):
                repeated = handbook.supply_webfonts(ROOT, cache, lambda *_: False)
            fetch.assert_not_called()
            second_fetch.assert_not_called()
            self.assertEqual(records[0]['native_sha256'], repeated[0]['native_sha256'])
            self.assertEqual(first_bytes, Path(repeated[0]['native_path']).read_bytes())
            self.assertEqual(len(records), 1)
            self.assertEqual(records[0]['source_sha256'], expected)
            converted = TTFont(records[0]['native_path'])
            comparison = TTFont(original)
            try:
                self.assertEqual(converted.getBestCmap(), comparison.getBestCmap())
                self.assertEqual(converted['hmtx'].metrics, comparison['hmtx'].metrics)
            finally:
                converted.close()
                comparison.close()

    def test_unrelated_government_font_is_not_in_handbook_supply(self):
        self.assertFalse(any('Government_16040911' in name for name, _ in handbook.WEBFONTS.values()))

    def test_png_pair_only_runs_handbook_when_requested(self):
        for enabled in (False, True):
            with self.subTest(enabled=enabled), tempfile.TemporaryDirectory() as temp, \
                    patch.object(sys, 'argv', ['visual_sweep.py', '--silhouette-only', '--png-pair', 'a', 'b', '--out', temp] + (['--handbook-gate'] if enabled else [])), \
                    patch.object(SWEEP, 'silhouette_png_pairs', return_value=[(1, Path('a'), Path('b'))]), \
                    patch.object(SWEEP, 'write_silhouette_tsv', return_value={'pr_review_gate': {'status': 'not_evaluated'}}), \
                    patch.object(SWEEP, 'enforce_required_handbook') as enforce:
                SWEEP.main()
                self.assertEqual(enforce.call_count, int(enabled))

    def test_handbook_can_run_alone(self):
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(sys, 'argv', ['visual_sweep.py', '--handbook-gate', '--out', temp]), \
                patch.object(SWEEP, 'enforce_required_handbook') as enforce:
            SWEEP.main()
            enforce.assert_called_once()

    def test_handbook_only_cli_preserves_requested_font_environment(self):
        environment = ROOT / 'mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json'
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(sys, 'argv', ['visual_sweep.py', '--handbook-gate', '--out', temp,
                                         '--font-environment', str(environment)]), \
                patch.object(SWEEP, 'enforce_required_handbook') as enforce:
            SWEEP.main()
            self.assertEqual(enforce.call_args.kwargs.get('font_environment'), environment)

    def test_required_render_uses_requested_font_environment_with_all_pages(self):
        from scripts import visual_sweep_resume
        environment = ROOT / 'mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json'
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(SWEEP, 'ensure_tools'), \
                patch.object(SWEEP, 'ensure_default_rhwp_binary_is_current'), \
                patch.object(handbook, 'supply_webfonts', return_value=[]), \
                patch.object(visual_sweep_resume, 'render_silhouette_resumable',
                             return_value=self.manifest) as render:
            result = SWEEP.required_handbook_gate(ROOT, Path(temp), 'rhwp', 1, [],
                                                 font_environment=environment)
            self.assertEqual(render.call_args.kwargs['font_environment'], environment)
            self.assertIsNone(render.call_args.args[7])
            self.assertEqual(result['status'], 'passed')

    def test_required_render_is_native_all_pages_and_records_failed_gate(self):
        from scripts import visual_sweep_resume
        self.manifest['metrics'][0][handbook.METRIC] = '99.00'
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(SWEEP, 'ensure_tools'), \
                patch.object(SWEEP, 'ensure_default_rhwp_binary_is_current'), \
                patch.object(handbook, 'supply_webfonts',
                             return_value=[{'native_path': str(Path(temp) / 'face' / 'font.ttf')}]), \
                patch.object(visual_sweep_resume, 'render_silhouette_resumable',
                             return_value=self.manifest) as render:
            out = Path(temp)
            result = SWEEP.required_handbook_gate(ROOT, out, 'rhwp', 8, [])
            self.assertEqual(result['status'], 're_review_required')
            self.assertIsNone(render.call_args.args[7])  # 전쪽 선택
            self.assertIsNone(render.call_args.kwargs['wasm_pkg'])
            self.assertEqual(render.call_args.kwargs['rasterizer'], 'png')
            self.assertEqual(render.call_args.kwargs['compat'], '2024')
            self.assertIn(Path(temp) / 'face', render.call_args.kwargs['font_paths'])
            summary = json.loads((out / 'summary.json').read_text())
            self.assertEqual(summary[-1]['pr_review_gate']['status'], 're_review_required')

    def test_unavailable_webfont_is_reported_as_failed_gate(self):
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(SWEEP, 'ensure_tools'), \
                patch.object(SWEEP, 'ensure_default_rhwp_binary_is_current'), \
                patch.object(handbook, 'supply_webfonts', side_effect=OSError('network down')):
            out = Path(temp)
            result = SWEEP.required_handbook_gate(ROOT, out, 'rhwp', 8, [])
            self.assertEqual(result['reason'], 'handbook_execution_failed')
            self.assertEqual(json.loads((out / 'summary.json').read_text())[-1]
                             ['pr_review_gate']['status'], 're_review_required')

    def test_re_review_is_a_process_failure(self):
        with patch.object(SWEEP, 'required_handbook_gate', return_value={'status': 're_review_required'}):
            with self.assertRaisesRegex(SystemExit, '필수 편람'):
                SWEEP.enforce_required_handbook(ROOT, ROOT / 'output', 'rhwp', 8, [])
        with patch.object(SWEEP, 'required_handbook_gate', return_value={'status': 'passed'}):
            SWEEP.enforce_required_handbook(ROOT, ROOT / 'output', 'rhwp', 8, [])


if __name__ == '__main__':
    unittest.main()
