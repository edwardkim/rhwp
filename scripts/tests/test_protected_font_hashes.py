import json
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

    def test_excluded_hash_cannot_be_selected_from_a_font_set_or_pair(self):
        files = [{"sha256": "installed"}, {"sha256": "excluded"}]
        pairs = {"equation.ttf": ["installed", "excluded"]}
        manifest = {
            "excluded_sha256": ["excluded"],
            "accepted_sha256_pairs": pairs,
            "font_sets": {"shared": {"files": files}},
        }
        document = {
            "backends": {"native": {"font_set": "shared"}},
            "accepted_sha256_pairs": pairs,
        }
        for scope in (None, document):
            self.assertEqual(supply_hashes(manifest, "installed", scope), ["installed"])
            self.assertEqual(supply_hashes(manifest, "excluded", scope), [])

    def test_active_equation_fonts_use_the_approved_version(self):
        manifest_path = (
            Path(__file__).resolve().parents[2]
            / "mydocs/verification/protected-font-supply/manifest.json"
        )
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        approved = "a3a3cd992a89ac38e7707b58143ebdb1da9511d3c462f019cda7f92c30467807"
        forbidden = {
            "c4f768c176ce26de91d6115bfb30c2d87c43c03d054f79e909fd340b1e46740a",
            "f92ff78fe7eaf2914e9879966e6a8383d3c525242339b781a117b755df70034a",
        }
        self.assertLessEqual(forbidden, set(manifest["excluded_sha256"]))
        for document in manifest["documents"]:
            for backend in document["backends"].values():
                for entry in manifest["font_sets"][backend["font_set"]]["files"]:
                    self.assertNotIn(entry["sha256"], forbidden)
                    if entry["filename"].casefold() == "hyhwpeq.ttf":
                        self.assertEqual(supply_hashes(manifest, approved, document), [approved])
                        self.assertEqual(entry["sha256"], approved)
