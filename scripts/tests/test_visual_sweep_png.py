"""직접 PNG 공급의 cmap 보존·부분 출력·이어받기 계약."""
import json
import shutil
import struct
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path
from types import SimpleNamespace

from PIL import Image
from scripts import visual_sweep_png as png
from scripts import visual_sweep as sweep


class TerminalCmapTests(unittest.TestCase):
    @staticmethod
    def cmap(delta=0, start=65535):
        table = struct.pack('>14H', 4, 32, 0, 4, 4, 1, 0,
                            65, 65535, 0, 65, start, 65472, delta)
        table += struct.pack('>2H', 0, 0)
        return struct.pack('>HHHHI', 0, 1, 3, 1, 12) + table

    def test_invalid_terminal_only_is_repaired(self):
        original = self.cmap()
        repaired = png.repair_terminal_cmap(original, 3)
        self.assertEqual(repaired[:38], original[:38])
        self.assertEqual(repaired[38:40], b'\0\1')
        self.assertEqual(repaired[40:], original[40:])
        self.assertIsNone(png.repair_terminal_cmap(repaired, 3))

    def test_valid_or_real_character_segment_is_preserved(self):
        self.assertIsNone(png.repair_terminal_cmap(self.cmap(delta=1), 3))
        self.assertIsNone(png.repair_terminal_cmap(self.cmap(start=65534), 3))

    def test_truncated_tables_are_not_rewritten(self):
        for end in range(len(self.cmap())):
            self.assertIsNone(png.repair_terminal_cmap(self.cmap()[:end], 3))


class PngExportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        for folder in ['rhwp_png', 'render_tree']:
            (self.base / folder).mkdir()
        self.commands = []
        self.break_manifest = lambda value: value
        self.vs = SimpleNamespace(run=self.run_command,
                                  resolve_input_path=lambda root, path: Path(path),
                                  load_json_object=lambda path, label: json.loads(path.read_text()),
                                  write_json_atomic=lambda path, value: path.write_text(json.dumps(value)))

    def run_command(self, args, **kwargs):
        self.commands.append(args)
        if args[1] == 'capabilities':
            result = {'commands': [{'name': 'export-png', 'available': True, 'json': True,
                                   'flags': ['--metadata-only', '--render-tree-output']}]}
        elif '--metadata-only' in args:
            result = {'metadataOnly': True, 'pageCount': 3}
        else:
            output = Path(args[args.index('-o') + 1])
            tree_dir = Path(args[args.index('--render-tree-output') + 1])
            tree_dir.mkdir()
            pages = [int(args[i + 1]) for i, item in enumerate(args) if item == '-p'] or list(range(3))
            items = []
            for page in pages:
                image = output / f'p{page}.png'
                Image.new('RGB', (3, 4), 'white').save(image)
                tree = tree_dir / f'render_tree_{page + 1:03d}.json'
                tree.write_text('{"type":"Page","bbox":[0,0,3,4]}')
                items.append({'page': page, 'path': str(image), 'bytes': image.stat().st_size,
                              'renderTreePath': str(tree)})
            result = {'pageCount': 3, 'backend': 'native-skia', 'comparisonProfile': 'print', 'layoutGeneration': '2022',
                      'renderedCount': len(items), 'pages': items}
            result = self.break_manifest(result)
        return SimpleNamespace(stdout=json.dumps(result))

    def export(self, selected=None, workers=2):
        png.export_native_png(self.vs, self.base, self.base / 'source.hwp', 'rhwp',
                              self.base, selected, [], ['--compat', '2022'], 96, workers)

    def test_workers_preserve_all_pages_and_same_document_trees(self):
        self.export()
        self.assertEqual(len(list((self.base / 'rhwp_png').glob('*.png'))), 3)
        self.assertEqual(len(list((self.base / 'render_tree').glob('*.json'))), 3)
        self.assertFalse(any(command[1] == 'export-render-tree' for command in self.commands))
        self.assertTrue(png.png_export_has_pages(self.base, False, None))

    def test_selected_export_does_not_render_unrequested_pages(self):
        self.export([2])
        self.assertEqual([p.name for p in (self.base / 'rhwp_png').glob('*.png')], ['rhwp_002.png'])
        self.assertTrue(png.png_export_has_pages(self.base, False, [2]))
        self.assertFalse(png.png_export_has_pages(self.base, False, None))

    def test_partial_success_manifest_is_rejected(self):
        self.break_manifest = lambda value: {**value, 'renderedCount': 0}
        with self.assertRaisesRegex(SystemExit, '누락'):
            self.export([2])
        self.assertFalse((self.base / 'native-export.json').exists())

    def test_duplicate_manifest_page_is_rejected(self):
        def duplicate(value):
            value['pages'].append(value['pages'][0])
            return value
        self.break_manifest = duplicate
        with self.assertRaisesRegex(SystemExit, '누락'):
            self.export([2])

    def test_wrong_generation_is_rejected(self):
        self.break_manifest = lambda value: {**value, 'layoutGeneration': '2024'}
        with self.assertRaisesRegex(SystemExit, 'profile'):
            self.export([2])

    def test_tree_outside_worker_is_rejected(self):
        def outside(value):
            value['pages'][0]['renderTreePath'] = str(self.base / 'source.hwp')
            return value
        self.break_manifest = outside
        with self.assertRaisesRegex(SystemExit, 'render tree'):
            self.export([2])

    def test_resume_rejects_wrong_backend_missing_tree_and_corrupt_metadata(self):
        self.export([2])
        metadata = self.base / 'native-export.json'
        value = json.loads(metadata.read_text())
        metadata.write_text(json.dumps({**value, 'rasterBackend': 'svg-webfont'}))
        self.assertFalse(png.png_export_has_pages(self.base, False, [2]))
        metadata.write_text(json.dumps(value))
        (self.base / 'render_tree/render_tree_002.json').unlink()
        self.assertFalse(png.png_export_has_pages(self.base, False, [2]))
        metadata.write_text('{')
        self.assertFalse(png.png_export_has_pages(self.base, False, [2]))


class RasterDispatchTests(unittest.TestCase):
    def test_legacy_svg_dispatch_yields_captures(self):
        records = [(1, Path('page.svg'), Path('tree.json'), Path('pdf.png'))]
        with patch('scripts.visual_sweep_raster.rasterized_sources', return_value=iter(records)):
            actual = list(sweep.rasterized_sources(Path('.'), records, Path('png'), 1, 'webfont'))
        self.assertEqual(actual, records)

    def test_png_failure_replaces_previous_passed_summary(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ['source.hwp', 'oracle.pdf']:
                (root / name).write_bytes(b'input')
            out = root / 'out'
            sweep.update_root_summary(out, {'key': 'case', 'pr_review_gate': {'status': 'passed'}})
            with (patch.object(sweep, 'sweep_provenance', return_value={}),
                  patch.object(sweep, 'run_manifest_for_target', return_value={}),
                  patch.object(sweep, 'load_note_shape', return_value={}),
                  patch.object(sweep, 'default_sweep_font_paths', return_value=[]),
                  patch.object(png, 'export_native_png', side_effect=SystemExit('PNG 부분 출력')),
                  self.assertRaisesRegex(SystemExit, '부분 출력')):
                sweep.render_target(root, sweep.Target('case', Path('source.hwp'), Path('oracle.pdf')),
                                    out, 'rhwp', 96, 32, [1], resume=True,
                                    svg_rasterizer='webfont', rasterizer='png')
            saved = json.loads((out / 'summary.json').read_text())[0]
            self.assertEqual(saved['pr_review_gate']['status'], 're_review_required')
            self.assertEqual(saved['pr_review_gate']['reason'], 'raster_export_failed')


class EmbeddedFontCatalogueTests(unittest.TestCase):
    def test_selected_ttc_face_keeps_document_family_alias(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            sources = root / 'source-fonts'
            sources.mkdir()
            path = sources / 'embedded.ttc'
            shutil.copyfile(Path(__file__).parents[2] / 'tests/fixtures/fonts/RHWPExactFaceSmoke.ttc', path)
            requests = root / 'requests.json'
            requests.write_text(json.dumps([{'path': str(path), 'faceIndex': 1,
                                            'families': ['문서 논리 글꼴']}]))
            catalogue = json.loads(png.embedded_font_catalog(requests, root).read_text())
            self.assertEqual(len(catalogue['faces']), 1)
            face = catalogue['faces'][0]
            self.assertEqual(face['faceIndex'], 1)
            self.assertEqual(face['sourceKind'], 'document-embedded')
            self.assertIn('문서 논리 글꼴', face['families'])
            self.assertIn('RHWP Exact Face One', face['families'])
            from fontTools.ttLib import TTFont
            with TTFont(face['path']) as font:
                self.assertTrue(font.getBestCmap())
