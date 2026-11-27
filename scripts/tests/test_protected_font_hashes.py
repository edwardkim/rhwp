from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from scripts.check_protected_font import accepted_hashes, excluded_font_path, supply_hashes


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

    def test_mac_otf_exclusion_ignores_case_and_checks_link_target(self):
        suffixes = ["_MAC.otf"]
        self.assertTrue(excluded_font_path(Path("Hmkmm_mac.OTF"), suffixes))
        self.assertFalse(excluded_font_path(Path("HMKMM_MAC.TTF"), suffixes))
        self.assertFalse(excluded_font_path(Path("HMKMM.otf"), suffixes))
        with TemporaryDirectory() as directory:
            target = Path(directory) / "Hmkmm_mac.OTF"
            target.touch()
            alias = target.with_name("Hmkmm.otf")
            try:
                alias.symlink_to(target)
            except OSError as error:
                self.skipTest(f"symlinks unavailable: {error}")
            self.assertTrue(excluded_font_path(alias, suffixes))

    def test_document_pair_is_local_even_when_font_sets_are_shared(self):
        files = [{"sha256": "dotumche-original"}, {"sha256": "regular-original"}]
        manifest = {
            "accepted_sha256_pairs": self.pairs,
            "font_sets": {"shared": {"files": files}},
        }
        other = {"backends": {"native": {"font_set": "shared"}, "wasm": {"font_set": "shared"}}}
        pair = ["dotumche-original", "dotumche-local"]
        chemical = {**other, "accepted_sha256_pairs": {"dotumche.ttf": pair}}

        self.assertEqual(supply_hashes(manifest, "dotumche-original", chemical), pair)
        self.assertEqual(supply_hashes(manifest, "dotumche-local", chemical), list(reversed(pair)))
        for scope in (None, other):
            self.assertEqual(supply_hashes(manifest, "dotumche-original", scope), ["dotumche-original"])
            self.assertEqual(supply_hashes(manifest, "dotumche-local", scope), [])
        self.assertEqual(supply_hashes(manifest, "regular-windows", chemical),
                         ["regular-windows", "regular-original"])
        self.assertEqual(supply_hashes(manifest, "bold-windows", chemical), [])
