import unittest

from scripts.check_protected_font import accepted_hashes


class ProtectedFontHashesTest(unittest.TestCase):
    pairs = {"arial.ttf": ["regular-original", "regular-windows"],
             "arialbd.ttf": ["bold-original", "bold-windows"]}

    def test_either_registered_hash_can_satisfy_its_own_entry(self):
        self.assertEqual(accepted_hashes("regular-original", self.pairs),
                         ["regular-original", "regular-windows"])
        self.assertEqual(accepted_hashes("regular-windows", self.pairs),
                         ["regular-windows", "regular-original"])

    def test_style_and_other_versions_keep_separate_hashes(self):
        allowed = accepted_hashes("bold-original", self.pairs)
        self.assertNotIn("regular-windows", allowed)
        self.assertNotIn("bold-third-version", allowed)
        self.assertEqual(accepted_hashes("dotumche-original", self.pairs), ["dotumche-original"])
