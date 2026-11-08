"""Release surfaces must not silently lose credits or disagree on product version."""
import json
import hashlib
from pathlib import Path
import re
import tomllib
import unittest
import tempfile

from scripts.prepare_release_notices import prepare

ROOT = Path(__file__).resolve().parents[2]
RECORDS = ('CHANGELOG.md', 'CHANGELOG_EN.md', 'mydocs/release/release_notes_v0.9.0.md',
           'mydocs/release/release_notes_v0.9.0_EN.md')


class Release090PreparationTests(unittest.TestCase):
    def test_crate_source_policy_has_one_entry_per_package_and_dependency(self):
        packages = json.loads((ROOT / 'packaging/crates-io/source-policy.json').read_text())['packages']
        for key in ('source', 'published_name'):
            values = [p[key] for p in packages]
            self.assertEqual(len(values), len(set(values)), key)
        for package in packages:
            dependencies = package['local_dependencies']
            self.assertEqual(len(dependencies), len(set(dependencies)), package['published_name'])
        jpeg = next(p for p in packages if p['published_name'] == 'rhwp-jpeg-encoder')
        self.assertIn('upstream/Cargo.toml.orig', jpeg['include'])
        original = ROOT / jpeg['source'] / 'upstream/Cargo.toml.orig'
        provenance = json.loads((ROOT / jpeg['source'] / 'UPSTREAM.json').read_text())
        self.assertEqual(hashlib.sha256(original.read_bytes()).hexdigest(),
                         provenance['files']['Cargo.toml.orig'])

    def test_release_credits_match_resolved_ledger_without_duplicates(self):
        ledger = json.loads((ROOT / 'mydocs/tech/investigations/release-v0.9.0/contributor-ledger.json').read_text())
        self.assertEqual(ledger['unresolvedNumbers'], [])
        self.assertEqual(ledger['unresolvedGitIdentities'], [])
        handles = [p['handle'] for p in ledger['contributors']]
        self.assertEqual(len(handles), len(set(h.casefold() for h in handles)))
        self.assertEqual(len(handles), ledger['counts']['people'])
        for name in RECORDS:
            with self.subTest(name=name):
                text = (ROOT / name).read_text()
                self.assertIn(ledger['range']['headSha'], text)
                self.assertIn(f"{ledger['range']['commitCount']:,}", text)
                start = '<!-- release-contributors:v0.9.0:start -->'
                end = '<!-- release-contributors:v0.9.0:end -->'
                self.assertEqual(text.count(start), 1)
                self.assertEqual(text.count(end), 1)
                block = text.split(start)[1].split(end)[0]
                self.assertEqual(re.findall(r'^- @([^\s]+)$', block, re.M), handles)

    def test_editor_package_retains_project_license_notice(self):
        self.assertEqual((ROOT / "npm/editor/LICENSE").read_bytes(), (ROOT / "LICENSE").read_bytes())

    def test_binary_and_wasm_notices_preserve_originals_without_font_programs(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory)
            copied = prepare(destination)
            for name in ('THIRD_PARTY_LICENSES.md', 'assets/licenses/Unicode-3.0.txt',
                         'licenses/pdfium/LICENSE', 'licenses/intel-mediasdk/LICENSE',
                         'data/font-metrics/nimbus-roman/URW-FONT-NOTICE.txt', 'vendor/rhwp-qcms/COPYING',
                         'vendor/rhwp-pdf-import-backend/src/fonts/assets/LICENSE-DejaVu'):
                self.assertIn(name, copied)
            for name in copied:
                self.assertEqual((destination / name).read_bytes(), (ROOT / name).read_bytes())
                self.assertNotIn(Path(name).suffix.lower(), {'.ttf', '.otf', '.woff', '.woff2', '.rs'})

    def test_notice_symlink_cannot_read_outside_source_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'source'
            root.mkdir()
            private = Path(directory) / 'private.txt'
            private.write_text('local-only')
            (root / 'THIRD_PARTY_LICENSES.md').symlink_to(private)
            with self.assertRaisesRegex(ValueError, 'outside'):
                prepare(Path(directory) / 'package', root)

    def test_product_manifests_and_lock_roots_match(self):
        cargo = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        self.assertEqual(cargo['package']['version'], '0.9.0')
        lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
        self.assertEqual([p['version'] for p in lock['package'] if p['name'] == 'rhwp'], ['0.9.0'])
        for directory in ('rhwp-studio', 'rhwp-vscode', 'npm/editor', 'rhwp-chrome', 'rhwp-firefox'):
            for filename in ('package.json', 'package-lock.json'):
                path = ROOT / directory / filename
                if not path.exists() and filename == 'package-lock.json':
                    continue  # editor has no install-time dependencies or lockfile
                with self.subTest(path=str(path)):
                    value = json.loads(path.read_text())
                    self.assertEqual(value['version'], '0.9.0')
                    if filename == 'package-lock.json':
                        self.assertEqual(value['packages']['']['version'], '0.9.0')
        for filename in ('rhwp-chrome/manifest.json', 'rhwp-firefox/manifest.json', 'rhwp-safari/src/manifest.json'):
            with self.subTest(filename=filename):
                self.assertEqual(json.loads((ROOT / filename).read_text())['version'], '0.9.0')


if __name__ == '__main__':
    unittest.main()
