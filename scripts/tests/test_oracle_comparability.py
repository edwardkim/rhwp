import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('oracle_comparability', ROOT / 'scripts/oracle_comparability.py')
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)

class ComparabilityReview(unittest.TestCase):
    def test_selected_pages_do_not_export_the_whole_document(self):
        with tempfile.TemporaryDirectory() as folder:
            seen = []
            def export_svg(command):
                page = int(command[command.index('-p') + 1])
                seen.append(page)
                out = Path(command[command.index('-o') + 1])
                out.mkdir()
                (out / 'page.svg').write_text(
                    f'<svg width="800" height="1000">'
                    f'<text x="10" y="20" font-size="12">page{page}</text></svg>'
                )
                return b''
            with patch.object(oracle, 'run', export_svg):
                output = oracle.read_rhwp(Path('rhwp'), Path('sample.hwp'), Path(folder), [3, 5])
            self.assertEqual(seen, [2, 4])
            self.assertEqual([line['key'] for line in output['lines']], ['page2', 'page4'])

    def test_selected_pages_cannot_judge_all_source_paragraphs(self):
        result = oracle.judge_lines([{'text': 'abcdefghijklmnopqrstuvwxyz', 'cuts': [0, 12]}], [], [], [3])
        self.assertEqual(result['status'], '미측정')
        report = {'paperBox': {'status': '일치'}, 'fontScale': {'status': '일치'},
                  'storedLines': result, 'declaredFaces': {'status': '일치'}}
        self.assertEqual(oracle.verdict(report)[0], '비교가능')

    def test_pdf_character_attributes_and_font_quotes_are_order_independent(self):
        for attributes in [
            'quad="10 10 20 10 10 20 20 20" c="가"',
            'c="가" quad="10 10 20 10 10 20 20 20"',
        ]:
            with self.subTest(attributes=attributes):
                xml = (
                    '<document><page width="595" height="842"><block><line>'
                    '<font size="12" name="Test"><char ' + attributes +
                    ' x="10" y="20"/></font></line></block></page></document>'
                ).encode()
                font_info = b'Type0 "INPILL+Gulim" Identity-H\nType0 \'INPILL+Batang\' Identity-H'
                with patch.object(oracle, 'run', side_effect=[xml, font_info]):
                    output = oracle.read_pdf(Path('sample.pdf'), [1])
                self.assertEqual(output['lines'][0]['key'], '가')
                self.assertEqual(output['lines'][0]['font_px'], 16)
                self.assertEqual(output['lines'][0]['y'], 20 * 96 / 72)
                self.assertEqual(output['embedded'], {'gulim', 'batang'})

    def test_svg_run_preserves_character_order(self):
        with tempfile.TemporaryDirectory() as folder:
            work = Path(folder)
            def export_svg(command):
                out = work / 'svg'
                out.mkdir()
                (out / 'page_001.svg').write_text('<svg width="800" height="1000"><text x="10" y="20" font-size="12">cab문단</text></svg>')
                return b''
            with patch.object(oracle, 'run', export_svg):
                output = oracle.read_rhwp(Path('rhwp'), Path('sample.hwp'), work, None)
            self.assertEqual(output['lines'][0]['key'], 'cab문단')

    def test_pdf_and_svg_split_distant_cells_on_the_same_baseline(self):
        xml = (
            '<document><page width="595" height="842"><block><line><font size="12">'
            '<char x="10" y="20" c="칸"/><char x="22" y="20" c="하나"/>'
            '<char x="200" y="20" c="칸"/><char x="212" y="20" c="둘"/>'
            '</font></line></block></page></document>'
        ).encode()
        with patch.object(oracle, 'run', side_effect=[xml, b'']):
            pdf_lines = oracle.read_pdf(Path('sample.pdf'), [1])['lines']
        with tempfile.TemporaryDirectory() as folder:
            work = Path(folder)
            def export_svg(command):
                out = work / 'svg'
                out.mkdir()
                (out / 'page_001.svg').write_text(
                    '<svg width="800" height="1000">'
                    '<text x="10" y="26.7" font-size="16">칸하나</text>'
                    '<text x="200" y="26.7" font-size="16">칸둘</text></svg>'
                )
                return b''
            with patch.object(oracle, 'run', export_svg):
                svg_lines = oracle.read_rhwp(Path('rhwp'), Path('sample.hwp'), work, None)['lines']
        self.assertEqual([line['key'] for line in pdf_lines], ['칸하나', '칸둘'])
        self.assertEqual([line['key'] for line in svg_lines], ['칸하나', '칸둘'])

    def test_svg_runs_sort_by_position_without_sorting_characters(self):
        with tempfile.TemporaryDirectory() as folder:
            work = Path(folder)
            def export_svg(command):
                out = work / 'svg'
                out.mkdir()
                (out / 'page_001.svg').write_text(
                    '<svg width="800" height="1000">'
                    '<text x="80" y="20" font-size="12">끝문장</text>'
                    '<text x="10" y="20" font-size="12">앞문장</text></svg>'
                )
                return b''
            with patch.object(oracle, 'run', export_svg):
                output = oracle.read_rhwp(Path('rhwp'), Path('sample.hwp'), work, None)
            self.assertEqual([line['key'] for line in output['lines']], ['앞문장', '끝문장'])

    def test_uncomparable_glyph_does_not_hide_a_different_saved_cut(self):
        prefix = 'abcdefghijkl'
        paragraph = {'text': prefix + '\uf53a' + '뒤문단', 'cuts': [0, len(prefix) + 1]}
        # 그 글자 직전에서 끊긴 출력은 저장 컷을 재현하지 않았다. PUA를 지우면
        # 두 글자열이 모두 prefix가 되어 거짓 양성이 된다.
        before_glyph = [{'key': prefix}]
        after_glyph = [{'key': prefix + '\uf53a'}]
        self.assertNotEqual(oracle.comparison_key(prefix + '\uf53a'), prefix)
        result = oracle.judge_lines([paragraph], before_glyph, after_glyph)
        self.assertEqual(result['status'], '미측정')
        self.assertEqual(result['excludedUncomparable'], 1)

    def test_ordinary_saved_cut_is_still_counted(self):
        paragraph = {'text': 'abcdefghijkl뒤문단', 'cuts': [0, 12]}
        lines = [{'key': 'abcdefghijkl'}]
        result = oracle.judge_lines([paragraph], lines, lines)
        self.assertEqual(result['storedMultilineParagraphs'], 1)
        self.assertEqual(result['oracleKeepsButRhwpMisses'], 0)
        self.assertEqual(result['excludedUncomparable'], 0)

    def test_x_scale_without_x_variance_is_unmeasured(self):
        source = [dict(key=f'line-text-unique-{i:03d}', font_px=12, x=100, y=100+i*10) for i in range(8)]
        reference = [dict(line, font_px=9.6, x=80, y=line['y']*.8) for line in source]
        self.assertEqual(oracle.judge_content_scale(source, reference, .8)['status'], '미측정')

    def test_x_scale_does_not_require_y_variance(self):
        source = [dict(key=f'line-text-unique-{i:03d}', font_px=12, x=100+i*10, y=100) for i in range(8)]
        reference = [dict(line, font_px=9.6, x=line['x']*.8, y=80) for line in source]
        self.assertEqual(oracle.judge_content_scale(source, reference, .8)['status'], '배율')

if __name__ == '__main__':
    unittest.main(verbosity=2)
