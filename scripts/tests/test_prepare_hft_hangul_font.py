"""독립 한글 HWPX 표본의 내용과 글꼴 요청을 확인한다."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

class HangulSpecimenTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = Path(__file__).resolve().parents[1] / 'prepare_hft_hangul_font.py'
        spec = importlib.util.spec_from_file_location('prepare_hft_hangul_font', path)
        cls.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.tool)

    def test_specimen_contains_all_requested_bullets_and_arrows(self):
        alphabet = ''.join(chr(i) for i in range(0xAC00, 0xD7A4))
        self.assertEqual(self.tool.specimen_characters(), alphabet)
        self.assertEqual(self.tool.specimen_characters(True), alphabet + '·•∙○◦')
        self.assertEqual(self.tool.specimen_characters(True, True), alphabet + '·•∙○◦←↑→↓➜')
        template = Path(__file__).resolve().parents[2] / 'samples/hwpx/blank_hwpx.hwpx'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            specimen = root / 'independent-arrows.hwpx'
            with contextlib.redirect_stdout(io.StringIO()):
                self.tool.create_specimen(template, specimen, include_bullets=True, include_arrows=True)
            with zipfile.ZipFile(specimen) as archive:
                import xml.etree.ElementTree as ET
                section = ET.fromstring(archive.read('Contents/section0.xml'))
                text = ''.join(n.text or '' for n in section.iter() if n.tag.endswith('}t'))
                self.assertEqual(text, alphabet + '·•∙○◦←↑→↓➜')


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
