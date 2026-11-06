"""관계형: Native 글꼴 공급 우선순위와 실제 웹폰트 대체 계약을 검증한다."""
# 주석 갱신: 2026-11-06T21:53:53+09:00 (Asia/Seoul)
# 합성 요청·응답을 사용하는 공급 경로 관계 검사이며 시각 일치율을 고정하지 않는다.
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import visual_sweep_native_webfonts as web


class NativeWebfontTests(unittest.TestCase):
    def test_verified_woff_conversion_keeps_program_identity_across_clock_changes(self):
        from fontTools.ttLib import TTFont

        source = self.root / 'assets/fonts/stable.woff2'
        source.parent.mkdir(parents=True)
        fixture = (Path(__file__).resolve().parents[2]
                   / 'tests/fixtures/native_font_family_names/family-contract.ttf')
        with TTFont(fixture, recalcTimestamp=False) as font:
            font.flavor = 'woff2'
            font.save(source)
        cache = self.root / 'cache'
        with patch('fontTools.ttLib.tables._h_e_a_d.timestampNow', return_value=3_900_000_000):
            first = web._resource(self.root, cache, 'fonts/stable.woff2')
        first_bytes = Path(first['native_path']).read_bytes()
        with patch('fontTools.ttLib.tables._h_e_a_d.timestampNow', return_value=3_900_000_060):
            second = web._resource(self.root, cache, 'fonts/stable.woff2')
        self.assertEqual(first['source_sha256'], second['source_sha256'])
        self.assertEqual(first['native_sha256'], second['native_sha256'])
        self.assertEqual(first_bytes, Path(second['native_path']).read_bytes())
        with TTFont(source) as original, TTFont(second['native_path']) as converted:
            self.assertEqual(original['head'].modified, converted['head'].modified)
            self.assertEqual(original.getBestCmap(), converted.getBestCmap())
            self.assertEqual(original['hmtx'].metrics, converted['hmtx'].metrics)

    def test_fontconfig_names_require_readable_source_program(self):
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temp:
            available, blocked = Path(temp) / 'available.ttf', Path(temp) / 'blocked.ttf'
            available.write_bytes(b'font')
            blocked.write_bytes(b'font')
            output = (f'{available}\tAvailable\tAvailable Regular\tAvailablePS\n'
                      f'{blocked}\tBlocked\tBlocked Regular\tBlockedPS\n')
            original_open = Path.open

            def readable(path, *args, **kwargs):
                if path == blocked:
                    raise PermissionError('검증 환경에서 읽을 수 없는 글꼴')
                return original_open(path, *args, **kwargs)

            with patch.object(web.subprocess, 'run', return_value=SimpleNamespace(stdout=output)), \
                    patch.object(web.handbook, '_file_font_inventory', return_value=set()), \
                    patch.object(Path, 'open', readable):
                names = web._installed_names(Path(temp), [])
            self.assertFalse(any('blocked' in name for name in names))
            self.assertTrue({'available', 'availableregular', 'availableps'} <= names)

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

    def test_kopub_public_fallback_is_scoped_and_installed_substitute_wins(self):
        self.registry.write_text(json.dumps({'rules': [{
            'status': 'active', 'sourceFace': '한컴바탕', 'supply': {
                'kind': 'canvas2d-webfont', 'sourceUrl': web.HANCOM_BATANG_WEB,
                'unicodeRange': None}}]}))
        plan = {'embeddedFamilies': [], 'requests': [
            {'source': 'KoPub바탕체 Light', 'primary': ['KoPubBatangLight'],
             'substitute': ['한컴바탕']}], 'scopedRequests': [
            {'languageSlot': 5, 'fontId': 13, 'source': 'KoPub바탕체 Light',
             'primary': ['KoPubBatangLight'], 'substitute': ['한컴바탕']},
            {'languageSlot': 2, 'fontId': 13, 'source': 'KoPub바탕체 Light',
             'primary': ['KoPubBatangLight'], 'substitute': ['한컴바탕']}]}
        def resource(root, cache, url):
            face = 'Noto Serif' if url == web.PUBLIC_NOTO_SERIF else 'HCRBatang'
            return {'url': url, 'native_path': str(cache / face / 'font.ttf'),
                    'native_face': face, 'source_sha256': 'source', 'native_sha256': 'native'}
        with patch.object(web, '_installed_names', return_value={'kopubbatanglight'}), \
                patch.object(web, '_postscript_aliases', return_value={}), \
                patch.object(web, '_resource', side_effect=resource), \
                patch.object(web, '_run', side_effect=[plan, {'fontEnvironment': None}]):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        env = json.loads(result['font_environment'].read_text())
        self.assertEqual([entry['faces'] for entry in env['documentGlyphFallbacks']],
                         [['Noto Serif', 'HCRBatang'], ['HCRBatang']])
        self.assertEqual(result['font_paths'][0].name, 'Noto Serif')
        web._PREPARED.clear()
        with patch.object(web, '_installed_names', return_value={'kopubbatanglight', '한컴바탕'}), \
                patch.object(web, '_postscript_aliases', return_value={}), \
                patch.object(web, '_resource') as resource_mock, \
                patch.object(web, '_run', return_value=plan):
            result = web.prepare(self.root, self.hwp, 'rhwp', [], None, '2024', 96)
        resource_mock.assert_not_called()
        self.assertIsNone(result['font_environment'])

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
