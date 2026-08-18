from __future__ import annotations

import unittest

from support import PKG  # noqa: F401
from lineage_chain.hexutil import is_sha256_hex, normalize_sha256_hex, sha_defect


class HexContractTests(unittest.TestCase):
    GOOD = "3c1c839b9b750e90a88239cf7052f46858ebefac8d4e2f985d4fac7699c7a5b3"

    def test_good(self) -> None:
        self.assertTrue(is_sha256_hex(self.GOOD))
        self.assertIsNone(sha_defect(self.GOOD))

    def test_uppercase_normalizes(self) -> None:
        self.assertEqual(normalize_sha256_hex(self.GOOD.upper()), self.GOOD)
        self.assertTrue(is_sha256_hex(self.GOOD.upper()))
        self.assertIsNone(sha_defect(self.GOOD.upper()))

    def test_missing(self) -> None:
        self.assertEqual(sha_defect(None), "missing")
        self.assertEqual(sha_defect(""), "missing")
        self.assertFalse(is_sha256_hex(""))

    def test_length(self) -> None:
        self.assertEqual(sha_defect("ab"), "length")
        self.assertEqual(sha_defect(self.GOOD + "aa"), "length")

    def test_nonhex(self) -> None:
        self.assertEqual(sha_defect("g" * 64), "nonhex")

    def test_prefixed(self) -> None:
        self.assertEqual(sha_defect("0x" + self.GOOD), "prefixed")

    def test_whitespace(self) -> None:
        self.assertEqual(sha_defect(" " + self.GOOD), "whitespace")


if __name__ == "__main__":
    unittest.main()
