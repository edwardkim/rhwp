from __future__ import annotations

import unittest

from support import PKG  # noqa: F401
from oracle_vs_self.versions import parse_versions


class VersionParseTests(unittest.TestCase):
    def test_none_tokens(self) -> None:
        for token in ("", "none", "-", "unmatched", None):
            parsed = parse_versions(token)
            self.assertEqual(parsed.kind, "none", token)

    def test_unknown_tokens(self) -> None:
        for token in ("unknown", "unparsed", "?"):
            self.assertEqual(parse_versions(token).kind, "unknown")

    def test_single_year(self) -> None:
        parsed = parse_versions("2022")
        self.assertEqual(parsed.kind, "agree")
        self.assertEqual(parsed.years, ("2022",))
        self.assertTrue(parsed.agree)

    def test_agree_join(self) -> None:
        parsed = parse_versions("2018+2020+2024")
        self.assertEqual(parsed.kind, "agree")
        self.assertEqual(parsed.years, ("2018", "2020", "2024"))
        self.assertEqual(parsed.canonical, "2018+2020+2024")

    def test_disagree_join(self) -> None:
        parsed = parse_versions("2010!2020!2024")
        self.assertEqual(parsed.kind, "disagree")
        self.assertFalse(parsed.agree)
        self.assertEqual(parsed.canonical, "2010!2020!2024")

    def test_legacy_2010_is_allowed(self) -> None:
        self.assertEqual(parse_versions("2010").kind, "agree")

    def test_invalid_year(self) -> None:
        parsed = parse_versions("2025")
        self.assertEqual(parsed.kind, "invalid")
        self.assertEqual(parsed.out_of_contract, ("2025",))

    def test_mixed_valid_and_invalid(self) -> None:
        parsed = parse_versions("2018+2025")
        self.assertEqual(parsed.kind, "invalid")
        self.assertEqual(parsed.years, ("2018",))
        self.assertEqual(parsed.out_of_contract, ("2025",))

    def test_comma_join_is_agree(self) -> None:
        parsed = parse_versions("2020,2022")
        self.assertEqual(parsed.kind, "agree")
        self.assertEqual(parsed.years, ("2020", "2022"))

    def test_plus_and_bang_is_disagree(self) -> None:
        parsed = parse_versions("2018+2020!2024")
        self.assertEqual(parsed.kind, "disagree")


if __name__ == "__main__":
    unittest.main()
