"""cmap 재사용이 변경·손상된 글꼴을 정상 글꼴로 오판하지 않는지 검사한다."""

import io
import base64
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont

from scripts import visual_sweep_font_cmap as cmap
from scripts import visual_sweep as sweep


def font_bytes(characters: str, remove_cmap: bool = False) -> bytes:
    builder = FontBuilder(1000, isTTF=True)
    glyphs = [".notdef", *[f"g{ord(ch)}" for ch in characters]]
    builder.setupGlyphOrder(glyphs)
    builder.setupCharacterMap({ord(ch): f"g{ord(ch)}" for ch in characters})
    builder.setupGlyf({glyph: TTGlyphPen(None).glyph() for glyph in glyphs})
    builder.setupHorizontalMetrics({glyph: (500, 0) for glyph in glyphs})
    builder.setupHorizontalHeader(ascent=800, descent=-200)
    builder.setupNameTable({"familyName": "CacheContract", "styleName": "Regular"})
    builder.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
    builder.setupPost()
    if remove_cmap:
        del builder.font["cmap"]
    output = io.BytesIO()
    builder.save(output)
    return output.getvalue()


class EmbeddedFontCmapCacheTests(unittest.TestCase):
    def setUp(self):
        cmap._UNICODE_COUNTS.clear()
        cmap._RULE_RECORDS.clear()

    def tearDown(self):
        cmap._UNICODE_COUNTS.clear()
        cmap._RULE_RECORDS.clear()

    def test_identical_font_bytes_are_parsed_once(self):
        data = font_bytes("AB")
        with patch("fontTools.ttLib.TTFont", wraps=TTFont) as parser:
            first = cmap.embedded_font_unicode_count(data)
            self.assertEqual(cmap.embedded_font_unicode_count(bytes(data)), first)
            self.assertEqual(parser.call_count, 1)
        self.assertEqual(first[1], 2)

    def test_changed_and_missing_cmap_bytes_are_not_accepted_from_cache(self):
        first = cmap.embedded_font_unicode_count(font_bytes("A"))
        second = cmap.embedded_font_unicode_count(font_bytes("AB"))
        self.assertNotEqual(first[0], second[0])
        self.assertEqual((first[1], second[1]), (1, 2))
        invalid = font_bytes("A", remove_cmap=True)
        for _ in range(2):
            with self.assertRaisesRegex(ValueError, "Unicode cmap"):
                cmap.embedded_font_unicode_count(invalid)
        with self.assertRaises(Exception):
            cmap.embedded_font_unicode_count(b"not a font")
        self.assertEqual(len(cmap._UNICODE_COUNTS), 2)

    def test_cache_does_not_retain_an_unbounded_set_of_fonts(self):
        with patch.object(cmap, "_CACHE_LIMIT", 2):
            for characters in ["A", "AB", "ABC"]:
                self.assertEqual(cmap.embedded_font_unicode_count(font_bytes(characters))[1], len(characters))
        self.assertEqual(len(cmap._UNICODE_COUNTS), 2)

    def test_svg_payload_keeps_unicode_whitespace_and_strict_base64_validation(self):
        payload = base64.b64encode(font_bytes("AB")).decode("ascii")
        wrapped = "\n\t \u2003\u0085".join(payload[i:i + 40] for i in range(0, len(payload), 40))
        with tempfile.TemporaryDirectory() as directory:
            svg = Path(directory) / "font.svg"
            def inspect(encoded):
                svg.write_text(f'<svg><style>@font-face{{font-family:Fixture;src:url(data:font/ttf;base64,{encoded})}}</style></svg>', encoding='utf-8')
                return sweep.inspect_svg_embedded_fonts(svg)[0]
            valid = inspect(wrapped)
            self.assertEqual(valid['status'], 'passed')
            self.assertEqual(valid['unicode_mapping_count'], 2)
            self.assertEqual(inspect(wrapped + '@')['status'], 'failed')

    def rule(self, data, family="Fixture"):
        payload = base64.b64encode(data).decode("ascii")
        return f'@font-face{{font-family:{family};src:url(data:font/ttf;base64,{payload})}}'

    def test_identical_rules_across_pages_decode_once_and_keep_each_report(self):
        rule = self.rule(font_bytes("AB"))
        with tempfile.TemporaryDirectory() as directory:
            pages = [Path(directory) / f'page_{number}.svg' for number in (1, 2)]
            for number, page in enumerate(pages):
                page.write_text(f'<svg><style>{rule}{rule}</style><text>{number}</text></svg>')
            report = Path(directory) / 'report.json'
            with patch.object(cmap.base64, 'b64decode', wraps=base64.b64decode) as decode:
                sweep.check_sweep_embedded_fonts(pages, report)
            self.assertEqual(decode.call_count, 1)
            record = json.loads(report.read_text())
            self.assertEqual(record['embedded_font_count'], 4)
            self.assertEqual([len(page['fonts']) for page in record['pages']], [2, 2])
            self.assertEqual(record['pages'][0]['fonts'], record['pages'][1]['fonts'])

    def test_cached_family_does_not_hide_changed_payload_header_or_alias(self):
        original = self.rule(font_bytes("A"))
        first = cmap.inspect_embedded_font_rule(original)[0]
        changed = cmap.inspect_embedded_font_rule(self.rule(font_bytes("AB")))[0]
        self.assertNotEqual(first['sha256'], changed['sha256'])
        self.assertEqual(changed['unicode_mapping_count'], 2)
        alias = cmap.inspect_embedded_font_rule(original.replace('Fixture', 'Alias'))[0]
        self.assertEqual(alias['family'], 'Alias')
        invalid_header = cmap.inspect_embedded_font_rule(original.replace(';base64', ''))[0]
        self.assertEqual(invalid_header['status'], 'failed')
        for invalid in (font_bytes("A", remove_cmap=True), b'not a font'):
            broken = self.rule(invalid)
            with patch.object(cmap.base64, 'b64decode', wraps=base64.b64decode) as decode:
                for _ in range(2):
                    result = cmap.inspect_embedded_font_rule(broken)[0]
                    self.assertEqual(result['status'], 'failed')
                    self.assertIn('sha256', result)
            self.assertEqual(decode.call_count, 2)

    def test_rule_cache_is_bounded_and_reports_cannot_mutate_it(self):
        rule = self.rule(font_bytes("A"))
        result = cmap.inspect_embedded_font_rule(rule)
        result[0]['status'] = 'failed'
        self.assertEqual(cmap.inspect_embedded_font_rule(rule)[0]['status'], 'passed')
        with patch.object(cmap, '_CACHE_LIMIT', 2):
            for family in ('One', 'Two', 'Three'):
                cmap.inspect_embedded_font_rule(self.rule(font_bytes("A"), family))
        self.assertEqual(len(cmap._RULE_RECORDS), 2)

    def test_rule_scanner_preserves_case_whitespace_nested_and_unclosed_rules(self):
        rule = self.rule(font_bytes("A"))
        cases = [
            rule * 2,
            rule.replace('@font-face{', '@FONT-FACE\n\t\u2003{'),
            '@font-face{broken {' + rule + '}}' + rule,
            '@font-face{unclosed',
            '@font-face{}@font-face{bad{} ' + rule + '}tail',
            '<svg><style>' + rule + '</style><text>' + rule + '</text></svg>',
            '@font-faceX{}' + rule,
        ]
        for svg in cases:
            with self.subTest(svg=svg[:80]):
                expected = re.findall(r'@font-face\s*\{[^{}]*\}', svg, re.IGNORECASE)
                self.assertEqual(list(cmap.embedded_font_rules(svg)), expected)


if __name__ == "__main__":
    unittest.main()
