"""관계형: Native 글꼴 공급 우선순위와 실제 웹폰트 대체 계약을 검증한다."""
# 주석 갱신: 2026-11-03 22:50:01 KST
# 합성 요청·응답을 사용하는 공급 경로 관계 검사이며 시각 일치율을 고정하지 않는다.
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import visual_sweep_native_webfonts as web


class NativeWebfontTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.hwp = self.root / 'input.hwp'
        self.hwp.write_bytes(b'document')
        self.registry = self.root / web.REGISTRY
        self.registry.parent.mkdir(parents=True)
        self.registry.write_text(json.dumps({'rules': [
            {'status': 'active', 'sourceFace': 'Missing', 'supply': {
                'kind': 'canvas2d-webfont', 'sourceUrl': 'fonts/example.woff', 'unicodeRange': None}},
            {'status': 'active', 'sourceFace': 'Embedded', 'supply': {
                'kind': 'canvas2d-webfont', 'sourceUrl': 'fonts/embedded.woff', 'unicodeRange': None}},
        ]}))
        web._PREPARED.clear()

    def resource(self, root, cache, url):
        name = 'WebExactPS' if url.endswith('example.woff') else 'WebGenericPS'
        return {'url': url, 'native_path': str(cache / name / 'font.ttf'), 'native_face': name,
                'source_sha256': 'source', 'native_sha256': 'native'}

    def test_local_and_embedded_win_missing_uses_web_and_unknown_is_reported(self):
        plan = {'embeddedFamilies': ['Embedded'], 'requests': [
            {'source': 'Local', 'primary': ['Local'], 'substitute': []},
            {'source': 'Embedded', 'primary': ['Embedded'], 'substitute': []},
            {'source': 'Missing', 'primary': ['Missing'], 'substitute': ['OldLocal']},
            {'source': 'Unknown', 'primary': ['Unknown'], 'substitute': []},
            {'source': 'UnknownWithLocal', 'primary': ['UnknownWithLocal'], 'substitute': ['LocalSub']},
            {'source': 'Mixed', 'primary': ['MissingMixed'], 'substitute': []},
            {'source': 'Mixed', 'primary': ['LocalMixed'], 'substitute': []},
        ]}
        old_env = {'id': 'existing', 'substitutions': {'Other': 'Old'},
                   'preserveSourceMetrics': True,
                   'documentSubstitutions': [{'source': 'Missing', 'target': 'OldLocal'},
                                             {'source': 'Other', 'target': 'Keep'}]}
        with patch.object(web, '_installed_names', return_value={'local', 'localsub', 'localmixed'}), \
                patch.object(web, '_resource', side_effect=self.resource) as resource, \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': old_env}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        self.assertEqual(result['evidence']['mappings'], {'Missing': 'WebExactPS', 'Unknown': 'WebGenericPS'})
        self.assertEqual(result['evidence']['unmapped_sources'], ['Unknown', 'UnknownWithLocal'])
        self.assertEqual(result['evidence']['local_document_substitutes'], ['UnknownWithLocal'])
        self.assertEqual(resource.call_count, 2)
        env = json.loads(result['font_environment'].read_text())
        self.assertEqual(env['substitutions'], {'Other': 'Old', 'Missing': 'WebExactPS', 'Unknown': 'WebGenericPS'})
        self.assertEqual(env['documentSubstitutions'], [{'source': 'Other', 'target': 'Keep'}])
        self.assertTrue(env['preserveSourceMetrics'])
        self.assertEqual(len(result['font_paths']), 2)

    def test_explicit_user_mapping_is_not_replaced_or_downloaded(self):
        plan = {'embeddedFamilies': [], 'requests': [{'source': 'Missing', 'primary': ['Missing'], 'substitute': []}]}
        path = self.root / 'explicit.json'
        path.write_text(json.dumps({'id': 'user', 'substitutions': {'Missing': 'UserFace'}}))
        with patch.object(web, '_installed_names', return_value=set()), \
                patch.object(web, '_resource') as resource, \
                patch.object(web, '_run', return_value=plan) as run:
            result = web.prepare(self.root, self.hwp, 'rhwp', [], path, '2024', 96)
        resource.assert_not_called()
        self.assertEqual(run.call_count, 1)
        self.assertEqual(result['font_environment'], path)
        self.assertEqual(result['evidence']['mappings'], {})

    def test_supplied_postscript_alias_wins_document_substitute(self):
        plan = {'embeddedFamilies': [], 'requests': [
            {'source': 'KoPub바탕체 Light', 'primary': ['KoPubBatangLight'], 'substitute': ['HCR Batang']},
        ]}
        old_env = {'id': 'existing', 'substitutions': {}, 'preserveSourceMetrics': True,
                   'documentSubstitutions': [{'source': 'KoPub바탕체 Light', 'target': 'HCR Batang'}]}
        with patch.object(web, '_installed_names', return_value={'hcrbatang'}), \
                patch.object(web, '_postscript_aliases', return_value={'kopubbatanglight': 'KoPubBatang-Light'}), \
                patch.object(web, '_resource') as resource, \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': old_env}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        resource.assert_not_called()
        env = json.loads(result['font_environment'].read_text())
        self.assertEqual(env['substitutions']['KoPub바탕체 Light'], 'KoPubBatang-Light')
        self.assertEqual(env['documentSubstitutions'], [])
        self.assertEqual(env['glyphFallbacks']['KoPubBatang-Light'], ['HCR Batang'])

    def test_full_name_presence_does_not_hide_native_substitution(self):
        plan = {'embeddedFamilies': [], 'requests': [
            {'source': 'KoPub바탕체 Light', 'primary': ['KoPub바탕체 Light', 'KoPubBatangLight'], 'substitute': ['HCR Batang']},
        ]}
        old_env = {'id': 'existing', 'substitutions': {}, 'preserveSourceMetrics': True,
                   'documentSubstitutions': [{'source': 'KoPub바탕체 Light', 'target': 'HCR Batang'}]}
        with patch.object(web, '_installed_names', return_value={'kopub바탕체light', 'hcrbatang'}), \
                patch.object(web, '_postscript_aliases', return_value={'kopubbatanglight': 'KoPubBatang-Light'}), \
                patch.object(web, '_resource') as resource, \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': old_env}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        resource.assert_not_called()
        env = json.loads(result['font_environment'].read_text())
        self.assertEqual(env['substitutions']['KoPub바탕체 Light'], 'KoPubBatang-Light')
        self.assertEqual(env['documentSubstitutions'], [])

    def test_existing_primary_is_not_replaced_when_no_native_substitution(self):
        plan = {'embeddedFamilies': [], 'requests': [
            {'source': 'Original', 'primary': ['Original', 'FaceLight'], 'substitute': []},
        ]}
        with patch.object(web, '_installed_names', return_value={'original'}), \
                patch.object(web, '_postscript_aliases', return_value={'facelight': 'Face-Light'}), \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': None}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        self.assertEqual(result['evidence']['mappings'], {})

    def test_document_fallback_is_prepared_only_for_its_declared_slot(self):
        plan = {'embeddedFamilies': [], 'requests': [
            {'source': 'Local', 'primary': ['Local'], 'substitute': ['Missing']},
            {'source': 'Local', 'primary': ['Local'], 'substitute': []},
        ], 'scopedRequests': [
            {'languageSlot': 5, 'fontId': 3, 'source': 'Local',
             'primary': ['Local'], 'substitute': ['Missing']},
            {'languageSlot': 2, 'fontId': 3, 'source': 'Local',
             'primary': ['Local'], 'substitute': []},
        ]}
        with patch.object(web, '_installed_names', return_value={'local'}), \
                patch.object(web, '_postscript_aliases', return_value={}), \
                patch.object(web, '_resource', side_effect=self.resource), \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': None}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        env = json.loads(result['font_environment'].read_text())
        self.assertEqual(env['documentGlyphFallbacks'], [
            {'languageSlot': 5, 'fontId': 3, 'source': 'Local', 'faces': ['WebExactPS']},
        ])
        self.assertEqual(env['substitutions'], {})

    def test_ambiguous_postscript_names_are_not_used_as_aliases(self):
        from fontTools.ttLib import TTFont
        fixture = Path(__file__).resolve().parents[2] / 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'
        for filename, name in (('first.ttf', 'Face-Light'), ('second.ttf', 'FaceLight')):
            font = TTFont(fixture)
            for record in font['name'].names:
                if record.nameID == 6:
                    record.string = name.encode(record.getEncoding())
            font.save(self.root / filename)
            font.close()
        aliases = web._postscript_aliases([self.root])
        self.assertNotIn('facelight', aliases)

    def test_no_font_plan_fails_instead_of_claiming_fallback(self):
        with patch.object(web, '_installed_names', return_value=set()), \
                patch.object(web, '_run', return_value={'requests': None}):
            with self.assertRaisesRegex(ValueError, '요청 목록'):
                web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)

    def test_untrusted_url_is_rejected(self):
        with self.assertRaisesRegex(ValueError, '공급 URL'):
            web._resource(self.root, self.root / 'cache', 'https://example.invalid/font.ttf')

    def test_rokg_uses_requested_cdn_without_legacy_supply(self):
        root = Path(__file__).resolve().parents[2]
        rules, _ = web._rules(root)
        url = 'https://cdn.jsdelivr.net/gh/Alex4386/gov-symbol@eee1e7d37a5ff02ab8bf6c6eb77272d71f1829a2/src/assets/fonts/ROKG_R.woff2'
        for family in ('ROKG', 'ROKG R', 'ROKGR', '대한민국정부상징체 R'):
            self.assertEqual(rules[web._normal(family)]['supply']['sourceUrl'], url)
        self.assertNotIn(web._normal('정부상징 부처명_16040911'), rules)
        self.assertNotIn(web._normal('Government_16040911'), rules)
        self.assertEqual(web.PUBLIC_SOURCE_SHA256['ROKG_R.woff2'],
                         '2c79a998f6b730afe2e311b6098055e9b6b29c1f2ba75f2871fd5dd8f4787426')


if __name__ == '__main__':
    unittest.main()
