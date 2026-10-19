"""원본 글꼴 준비의 단위 계약. 문서 Visual Sweep의 대체 증거가 아니다."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

try:
    import pymupdf
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.cffLib import PrivateDict
except ImportError:
    pymupdf = None


@unittest.skipIf(pymupdf is None, '선택 도구에는 PyMuPDF와 fonttools가 필요합니다')
class OriginalHftFontTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = Path(__file__).resolve().parents[1] / 'prepare_hft_hangul_font.py'
        spec = importlib.util.spec_from_file_location('prepare_hft_hangul_font', path)
        cls.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.tool)

    def cmap(self, body):
        class Document:
            def xref_stream(self, reference):
                return body.encode('ascii')
        return self.tool.mapped_characters(Document(), 1)

    def test_explicit_and_consecutive_unicode_maps_preserve_codes(self):
        self.assertEqual(self.cmap('1 beginbfchar <01> <ac00> endbfchar '
                                  '1 beginbfrange <03> <05> <ac01> endbfrange'),
                         [(1, 0xAC00), (3, 0xAC01), (4, 0xAC02), (5, 0xAC03)])

    def test_array_range_is_not_assumed_to_be_consecutive(self):
        self.assertEqual(self.cmap('1 beginbfrange <03> <05> [<ac04> <b098> <d06c>] endbfrange'),
                         [(3, 0xAC04), (4, 0xB098), (5, 0xD06C)])

    def test_range_with_missing_destination_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'missing destinations'):
            self.cmap('1 beginbfrange <03> <05> [<ac04> <b098>] endbfrange')

    def test_reversed_range_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'Invalid character mapping range'):
            self.cmap('1 beginbfrange <05> <03> <ac00> endbfrange')

    def test_original_cubic_points_and_fractional_em_values_are_preserved(self):
        width, charstring = self.tool.restore_outline(
            '1000 0 0 -200 1000 1000 d1 10.25 80 m '
            '20 40.5 60 30 90 80 c 70 90 l h h f')
        pen = RecordingPen()
        charstring.private = PrivateDict()
        charstring.draw(pen)
        self.assertEqual(width, 1000)
        self.assertEqual(pen.value, [
            ('moveTo', ((10.25, 80),)),
            ('curveTo', ((20, 40.5), (60, 30), (90, 80))),
            ('lineTo', ((70, 90),)),
            ('closePath', ()),
        ])

    def test_unhandled_transform_cannot_silently_change_glyphs(self):
        with self.assertRaisesRegex(ValueError, 'Unsupported glyph operator'):
            self.tool.restore_outline('1000 0 0 -200 1000 1000 d1 2 0 0 2 0 0 cm')

    def test_width_only_type3_cannot_supply_an_original_glyph_outline(self):
        # 한컴 Print는 실제 글자를 별도 이미지로 그리고 검색용 Type3에는
        # 전진폭·문자 매핑만 남길 수 있다. 이 스트림으로 빈 글꼴을 만들면
        # 실제 원본 글꼴을 공급했다고 오판하고 정상 한글까지 지워 버린다.
        with self.assertRaisesRegex(ValueError, 'Empty source glyph outline'):
            self.tool.restore_outline('1000 0 0 -200 1000 1000 d1 f')

    def test_horizontal_bearing_keeps_original_outline_origin(self):
        for left, expected in [('10.25', 10), ('-12.75', -13)]:
            width, charstring = self.tool.restore_outline(
                f'1000 0 -100 -200 1000 1000 d1 {left} 80 m '
                '90 80 l 90 90 l h f')
            charstring.private = PrivateDict()
            self.assertEqual(self.tool.source_horizontal_metrics(width, charstring),
                             (1000, expected))
            pen = RecordingPen()
            charstring.draw(pen)
            self.assertEqual(pen.value[0], ('moveTo', ((float(left), 80),)))

    def test_paint_without_advance_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'Invalid move'):
            self.tool.restore_outline('10 80 m 70 90 l h f')

    def test_target_document_pdf_is_not_accepted_as_font_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = pymupdf.open()
            document.new_page().insert_text((30, 30), 'Ordinary target document')
            source = root / 'target.pdf'
            document.save(source)
            output = root / 'should-not-exist.otf'
            for include_bullets in [False, True]:
                with self.assertRaisesRegex(ValueError, 'independent complete alphabet'):
                    self.tool.restore(source, output, include_bullets)
            self.assertFalse(output.exists())

    def test_arrows_require_the_complete_independent_alphabet_and_all_requested_symbols(self):
        alphabet = ''.join(chr(i) for i in range(0xAC00, 0xD7A4))
        self.assertEqual(self.tool.specimen_characters(), alphabet)
        self.assertEqual(self.tool.specimen_characters(True), alphabet + '·•∙○◦')
        self.assertEqual(self.tool.specimen_characters(True, True), alphabet + '·•∙○◦←↑→↓➜')
        template = Path(__file__).resolve().parents[2] / 'samples/hwpx/blank_hwpx.hwpx'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            specimen = root / 'independent-arrows.hwpx'
            with contextlib.redirect_stdout(io.StringIO()):
                self.tool.create_specimen(template, specimen, True, True)
            with zipfile.ZipFile(specimen) as archive:
                import xml.etree.ElementTree as ET
                section = ET.fromstring(archive.read('Contents/section0.xml'))
                text = ''.join(n.text or '' for n in section.iter() if n.tag.endswith('}t'))
                self.assertEqual(text, alphabet + '·•∙○◦←↑→↓➜')

            class Page:
                def get_text(self):
                    # 전체 한글이 있어도 요청한 마지막 화살표 하나가 빠지면 거부한다.
                    return alphabet + '·•∙○◦←↑→↓'

            with patch.object(self.tool.pymupdf, 'open', return_value=[Page()]):
                with self.assertRaisesRegex(ValueError, 'independent complete alphabet'):
                    self.tool.restore(root / 'incomplete.pdf', root / 'bad.otf', True, True, True)
            self.assertFalse((root / 'bad.otf').exists())

    def test_bullet_mode_keeps_complete_alphabet_and_rejects_missing_symbol(self):
        with self.assertRaisesRegex(ValueError, 'complete alphabet-with-bullets'):
            self.tool.restore(Path('not-opened.pdf'), Path('not-created.otf'), symbols_only=True)
        template = Path(__file__).resolve().parents[2] / 'samples/hwpx/blank_hwpx.hwpx'
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'alphabet-bullets.hwpx'
            with contextlib.redirect_stdout(io.StringIO()):
                self.tool.create_specimen(template, output, True)
            with zipfile.ZipFile(output) as archive:
                import xml.etree.ElementTree as ET
                section = ET.fromstring(archive.read('Contents/section0.xml'))
                text = ''.join(n.text or '' for n in section.iter() if n.tag.endswith('}t'))
                self.assertEqual(text[:11172], ''.join(chr(i) for i in range(0xAC00, 0xD7A4)))
                self.assertEqual(text[11172:], '·•∙○◦')
            class Page:
                def get_text(self):
                    return text[:-1]
            with patch.object(self.tool.pymupdf, 'open', return_value=[Page()]):
                with self.assertRaisesRegex(ValueError, 'independent complete alphabet'):
                    self.tool.restore(Path('incomplete.pdf'), Path(directory) / 'bad.otf', True)
            self.assertFalse((Path(directory) / 'bad.otf').exists())

    def test_specimen_has_entire_alphabet_hft_face_and_no_saved_lines(self):
        template = Path(__file__).resolve().parents[2] / 'samples/hwpx/blank_hwpx.hwpx'
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'alphabet.hwpx'
            with contextlib.redirect_stdout(io.StringIO()):
                self.tool.create_specimen(template, output)
            with zipfile.ZipFile(output) as archive:
                import xml.etree.ElementTree as ET
                header = ET.fromstring(archive.read('Contents/header.xml'))
                section = ET.fromstring(archive.read('Contents/section0.xml'))
                fonts = [node for node in header.iter() if node.tag.endswith('}font')]
                self.assertTrue(fonts)
                self.assertTrue(all(font.attrib['face'] == '한양신명조'
                                    and font.attrib['type'] == 'HFT' for font in fonts))
                characters = ''.join(node.text or '' for node in section.iter() if node.tag.endswith('}t'))
                self.assertEqual(characters, ''.join(chr(i) for i in range(0xAC00, 0xD7A4)))
                self.assertFalse(any(node.tag.endswith('}linesegarray') for node in section.iter()))

    def test_requested_face_is_not_renamed_to_the_default_face(self):
        template = Path(__file__).resolve().parents[2] / 'samples/hwpx/blank_hwpx.hwpx'
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'alphabet.hwpx'
            with contextlib.redirect_stdout(io.StringIO()):
                self.tool.create_specimen(template, output, 'HY신명조')
            with zipfile.ZipFile(output) as archive:
                import xml.etree.ElementTree as ET
                header = ET.fromstring(archive.read('Contents/header.xml'))
                fonts = [node for node in header.iter() if node.tag.endswith('}font')]
                self.assertTrue(fonts)
                self.assertTrue(all(node.attrib['face'] == 'HY신명조'
                                    and node.attrib['type'] == 'HFT' for node in fonts))
            provenance = json.loads(output.with_suffix('.provenance.json').read_text())
            self.assertEqual(provenance['requestedFace'], 'HY신명조')
            self.assertEqual(provenance['characters'], 11172)

    def test_invalid_face_cannot_be_used_to_label_a_font(self):
        for face in ['', '  ', 'HFT\x00Other', 'HFT\nOther', 'a' * 257]:
            with self.subTest(face=repr(face)):
                with self.assertRaisesRegex(ValueError, 'HFT face'):
                    self.tool.validate_face(face)


if __name__ == '__main__':
    unittest.main()
