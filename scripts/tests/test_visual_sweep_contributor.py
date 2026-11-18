"""관계형·고정형: 기여자 공통 검사의 범위·실행 신원과 정본 하한을 보호한다.

주석 갱신: 2026-11-16T00:24:09+09:00 (Asia/Seoul).
측정: 2026-11-16, source 4e0b20d2922953ee7ab9a1716ac819b921fe484b.
96 DPI, 보정 전 2px raw. 공개 환경의 라틴·좁은 셀·그림 자르기·그룹 도형은
각 1쪽, Native/fresh WASM 전쪽 최저 각각 100.00%다.
근거: mydocs/pr/assets/issue_9101/. 실물 절대 px 좌표를 고정하지 않는다.
"""
import copy
import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import visual_sweep_contributor as gate


class ContributorGateTests(unittest.TestCase):
    def setUp(self):
        self.documents = gate.load_catalog(gate.ROOT)
        self.doc = self.documents[0]
        self.manifest = {
            'run_state': 'complete', 'radius_px': 2, 'silhouette_method': gate.handbook.METHOD,
            'provenance': {
                'hwp': {'sha256': self.doc['input']['sha256']},
                'pdf': {'sha256': self.doc['pdf']['sha256']},
                'raster_backend': 'native-skia', 'layout_generation': self.doc['compat'],
                'comparison_profile': 'print',
                'pdf_rasterizer': {'policy': gate.handbook.POLICY},
            },
            'pr_review_gate': {'document_page_counts': {'rhwp': 1, 'pdf': 1}},
            'metrics': [{'page': 1, gate.METRIC: '100.00'}],
        }

    def test_public_scope_matches_committed_inputs_without_os_only_styles(self):
        self.assertEqual(len(self.documents), 4)
        self.assertNotIn('4314830495cfde87', {d['id'] for d in self.documents})
        self.assertNotIn('2b43ec3782a748e1', {d['id'] for d in self.documents})
        for doc in self.documents:
            self.assertEqual(set(doc['baseline']), {'native', 'wasm'})
            self.assertEqual(doc['pages'], 1)

    def test_all_pages_pass_only_without_displayed_loss(self):
        self.assertEqual(gate.compare(self.doc, 'native', self.manifest)['status'], 'passed')
        self.manifest['metrics'][0][gate.METRIC] = '99.99'
        result = gate.compare(self.doc, 'native', self.manifest)
        self.assertEqual(result['status'], 're_review_required')
        self.assertEqual(result['regressions'][0]['page'], 1)

    def test_invalid_metric_or_missing_duplicate_extra_page_is_rejected(self):
        cases = [[], [{'page': 2, gate.METRIC: 100}],
                 [{'page': 1, gate.METRIC: 100}] * 2]
        cases += [[{'page': 1, gate.METRIC: v}] for v in ('NaN', 'Infinity', '-1', '101')]
        for rows in cases:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                manifest = copy.deepcopy(self.manifest)
                manifest['metrics'] = rows
                gate.compare(self.doc, 'native', manifest)

    def test_wrong_input_backend_layout_profile_or_page_count_cannot_pass(self):
        for path, value in [
                (['run_state'], 'incomplete'), (['radius_px'], 3),
                (['excluded_regions'], {'1': [[0, 0, 100, 100]]}),
                (['provenance', 'hwp', 'sha256'], '0' * 64),
                (['provenance', 'pdf', 'sha256'], '0' * 64),
                (['provenance', 'raster_backend'], 'wasm-canvas2d'),
                (['provenance', 'layout_generation'], '2024'),
                (['provenance', 'comparison_profile'], 'screen'),
                (['pr_review_gate', 'document_page_counts'], {'rhwp': 2, 'pdf': 1})]:
            manifest = copy.deepcopy(self.manifest)
            target = manifest
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] = value
            with self.subTest(path=path), self.assertRaises(ValueError):
                gate.compare(self.doc, 'native', manifest)

    def test_fresh_wasm_must_be_compared_as_wasm(self):
        with self.assertRaises(ValueError):
            gate.compare(self.doc, 'wasm', self.manifest)
        self.manifest['provenance']['raster_backend'] = 'wasm-canvas2d'
        self.assertEqual(gate.compare(self.doc, 'wasm', self.manifest)['status'], 'passed')

    def test_changed_input_bytes_and_reduced_catalog_fail_before_render(self):
        original = json.loads((gate.ROOT / gate.CATALOG).read_text())
        with patch.object(gate.handbook, 'sha', side_effect=lambda p:
                          '0' * 64 if Path(p).suffix == '.hwp' else
                          hashlib.sha256(Path(p).read_bytes()).hexdigest()):
            with self.assertRaisesRegex(ValueError, '해시'):
                gate.load_catalog(gate.ROOT)
        original['documents'].pop()
        with patch.object(gate.json, 'loads', return_value=original):
            with self.assertRaisesRegex(ValueError, '4개'):
                gate.load_catalog(gate.ROOT)

    def test_first_run_and_resume_use_official_plan_and_resume_cli(self):
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(gate, 'prepare_target_plan') as prepare:
            out = Path(tmp) / 'native'
            command = gate.render_command(gate.ROOT, out, self.doc, 'native', 'rhwp', None, 2, [])
            self.assertIn('--resume', command)
            self.assertIn('--silhouette-only', command)
            prepare.assert_called_once()
            self.assertFalse(prepare.call_args.kwargs['resume'])
            (out / 'silhouette-target-plan.json').write_text('{}')
            prepare.reset_mock()
            gate.render_command(gate.ROOT, out, self.doc, 'native', 'rhwp', None, 2, [])
            prepare.assert_not_called()

    def test_unidentified_existing_output_is_not_deleted(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'existing.txt'
            path.write_text('보존')
            with self.assertRaisesRegex(ValueError, '재사용하지'):
                gate.render_command(gate.ROOT, Path(tmp), self.doc, 'native', 'rhwp', None, 2, [])
            self.assertEqual(path.read_text(), '보존')

    def test_execution_failure_cannot_reuse_an_old_passing_report(self):
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(gate.sweep, 'rhwp_binary_identifier', return_value={}), \
                patch.object(gate.sweep, 'wasm_package_provenance', return_value={}), \
                patch.object(gate, 'render_command', side_effect=RuntimeError('실행 실패')):
            out = Path(tmp)
            (out / 'contributor-gate.json').write_text('{"status":"passed"}')
            report = gate.run(gate.ROOT, out, 'rhwp', Path('pkg'), 1, [])
            self.assertEqual(len(report['results']), 8)
            self.assertEqual(report['status'], 're_review_required')
            self.assertTrue(all(r['status'] == 're_review_required' for r in report['results']))
            self.assertEqual(json.loads((out / 'contributor-gate.json').read_text())['status'],
                             're_review_required')


if __name__ == '__main__':
    unittest.main()
