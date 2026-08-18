"""RFC 4180 codec — independent of the rhwp binary."""

from __future__ import annotations

import unittest

from table_exchange.csv_codec import read_csv, write_csv


class CsvCodecTests(unittest.TestCase):
    def test_plain_rectangle(self) -> None:
        text = write_csv([["제목", "담당자", "세부 내용"], ["", "", ""], ["", "", ""]])
        parsed = read_csv(text)
        self.assertTrue(parsed.ok)
        self.assertEqual(len(parsed.records), 3)
        self.assertEqual(parsed.records[0], ("제목", "담당자", "세부 내용"))

    def test_quoted_comma_and_quote(self) -> None:
        text = write_csv([['가,나"다', "x"]])
        self.assertTrue(text.startswith('"가,나""다"'))
        parsed = read_csv(text)
        self.assertTrue(parsed.ok)
        self.assertEqual(parsed.records[0][0], '가,나"다')

    def test_unclosed_quote_is_csv_parse(self) -> None:
        parsed = read_csv('"닫히지 않은 따옴표')
        self.assertFalse(parsed.ok)
        self.assertIn("따옴표", parsed.message or "")

    def test_bom_is_stripped_on_read(self) -> None:
        parsed = read_csv("\ufeffa,b\r\n1,2\r\n")
        self.assertTrue(parsed.ok)
        self.assertEqual(parsed.records[0][0], "a")

    def test_crlf_record_separator(self) -> None:
        text = write_csv([["a", "b"], ["c", "d"]])
        self.assertIn("\r\n", text)
        self.assertTrue(text.endswith("\r\n"))
