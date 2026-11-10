#!/usr/bin/env python3
"""Portable runtime bundle contracts; header fixtures are never executable code."""

import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tool'))
import runtime_bundle as bundle

SOURCE = '1' * 40


def header(platform, architecture='x86_64', shared=False):
    data = bytearray(128)
    if platform == 'linux':
        data[:7] = b'\x7fELF\x02\x01\x01'
        struct.pack_into('<HH', data, 16, 3 if shared else 2, 62 if architecture == 'x86_64' else 183)
    elif platform == 'windows':
        data[:2] = b'MZ'
        struct.pack_into('<I', data, 60, 64)
        data[64:68] = b'PE\0\0'
        struct.pack_into('<H', data, 68, 0x8664 if architecture == 'x86_64' else 0xaa64)
        struct.pack_into('<H', data, 86, 0x2000 if shared else 0)
        struct.pack_into('<H', data, 88, 0x20b)
    else:
        data[:4] = b'\xcf\xfa\xed\xfe'
        struct.pack_into('<I', data, 4, 0x1000007 if architecture == 'x86_64' else 0x100000c)
        struct.pack_into('<I', data, 12, 6 if shared else 2)
    return bytes(data)


class RuntimeBundleContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='rhwp-runtime-bundle-')
        self.root = Path(self.temp.name)
        self.runtime = self.root / 'runtime'
        self.runtime.mkdir()
        (self.runtime / 'empty').mkdir()
        (self.runtime / 'data').mkdir()
        (self.runtime / 'data' / 'asset.bin').write_bytes(b'assets belong to the supplied Flutter build')
        (self.runtime / 'data' / 'empty.bin').write_bytes(b'')
        self.notices = self.root / 'notices'
        self.notices.mkdir()
        for name in bundle.NOTICES:
            (self.notices / name).write_text(name + '\n')
        (self.notices / 'licenses').mkdir()
        (self.notices / 'licenses' / 'dependency.txt').write_text('supplied dependency notice\n')
        self.native = self.root / 'native-input'
        self.dispatcher = self.root / 'owner-input'
        self.output = self.root / 'bundle'
        self.configure('linux')

    def tearDown(self):
        self.temp.cleanup()

    def configure(self, platform, architecture='x86_64'):
        self.platform, self.architecture = platform, architecture
        (self.runtime / 'app').write_bytes(header(platform, architecture))
        (self.runtime / 'app').chmod(0o755)
        self.native.write_bytes(header(platform, architecture, True))
        self.dispatcher.write_bytes(header(platform, architecture, True))

    def package(self, **changes):
        args = dict(runtime=self.runtime, executable='app', native=self.native,
                    dispatcher=self.dispatcher, notices=self.notices, output=self.output,
                    platform=self.platform, architecture=self.architecture,
                    sources={name: SOURCE for name in ('runtime', 'native', 'dispatcher')})
        args.update(changes)
        return bundle.package(**args)

    def save_manifest(self, value):
        (self.output / 'rhwp-runtime.json').write_text(json.dumps(value))

    def test_all_targets_and_architectures_are_read_without_execution(self):
        for target in bundle.NAMES:
            for architecture in ('x86_64', 'arm64'):
                self.configure(target, architecture)
                manifest = self.package()
                self.assertEqual(manifest['architecture'], architecture)
                self.assertEqual(bundle.verify(self.output), manifest)
                self.assertTrue((self.output / 'app' / 'empty').is_dir())
                self.assertEqual((self.output / 'app/data/empty.bin').read_bytes(), b'')
                self.assertEqual((self.output / 'notices/licenses/dependency.txt').read_text(), 'supplied dependency notice\n')
                # Caller machine paths are not serialized as provenance.
                self.assertNotIn(str(self.root), (self.output / 'rhwp-runtime.json').read_text())
                shutil.rmtree(self.output)

    def test_relocation_and_cli_verification(self):
        self.package()
        relocated = self.root / 'with spaces' / 'relocated bundle'
        relocated.parent.mkdir()
        shutil.move(self.output, relocated)
        shutil.rmtree(self.runtime)
        self.native.unlink(); self.dispatcher.unlink()
        self.assertEqual(bundle.verify(relocated)['runtime'], 'app/app')
        process = subprocess.run([sys.executable, '-B', str(Path(bundle.__file__)), 'verify',
                                  '--bundle', str(relocated), '--platform', 'linux',
                                  '--architecture', 'x86_64'], capture_output=True, text=True, timeout=10)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertFalse(json.loads(process.stdout)['nativeLoaded'])
        self.assertFalse(json.loads(process.stdout)['runtimeLaunched'])

    def test_corruption_unlisted_missing_and_wrong_target_are_rejected(self):
        manifest = self.package()
        asset = self.output / 'app/data/asset.bin'; original = asset.read_bytes()
        asset.write_bytes(b'x' * len(original))
        with self.assertRaisesRegex(ValueError, 'mismatch'): bundle.verify(self.output)
        asset.write_bytes(original)
        injected = self.output / 'app/unlisted.dll'; injected.write_bytes(b'bad')
        with self.assertRaisesRegex(ValueError, 'incomplete'): bundle.verify(self.output)
        injected.unlink(); asset.unlink()
        with self.assertRaisesRegex(ValueError, 'incomplete'): bundle.verify(self.output)
        asset.write_bytes(original)
        for changes in ({'expected_platform': 'windows'}, {'expected_architecture': 'arm64'}):
            with self.assertRaisesRegex(ValueError, 'mismatch'): bundle.verify(self.output, **changes)
        wrong = copy.deepcopy(manifest); wrong['platform'] = 'windows'; self.save_manifest(wrong)
        with self.assertRaises(ValueError): bundle.verify(self.output)

    def test_roles_sources_schema_and_abi_are_required(self):
        manifest = self.package()
        cases = [dict(schemaVersion=True), dict(schemaVersion=2), dict(native='app/app'),
                 dict(dispatcher='app/app'), dict(runtime='../app'),
                 dict(abi={'native': 1, 'dispatcher': 1}), dict(abi={'native': True, 'dispatcher': 2}),
                 dict(sources={'runtime': SOURCE}),
                 dict(sources={name: '/private/path' for name in manifest['sources']})]
        for changes in cases:
            self.save_manifest(dict(manifest, **changes))
            with self.subTest(changes=changes), self.assertRaises(ValueError): bundle.verify(self.output)
        self.save_manifest(manifest)
        body = (self.output / 'rhwp-runtime.json').read_text()
        (self.output / 'rhwp-runtime.json').write_text(body.replace('"kind":', '"kind":"wrong", "kind":', 1))
        with self.assertRaisesRegex(ValueError, 'duplicate'): bundle.verify(self.output)

    def test_manifest_bound_rejects_before_json_decode(self):
        self.package()
        (self.output / 'rhwp-runtime.json').write_bytes(b' ' * (bundle.MAX_MANIFEST_BYTES + 1))
        with self.assertRaises(ValueError): bundle.verify(self.output)

    def test_executable_header_role_mismatch_rejects_before_output_exists(self):
        for role in ('runtime', 'native', 'dispatcher'):
            self.configure('linux')
            path = self.runtime / 'app' if role == 'runtime' else getattr(self, role)
            path.write_bytes(header('windows', shared=role != 'runtime'))
            with self.subTest(role=role), self.assertRaisesRegex(ValueError, 'platform/architecture'): self.package()
            self.assertFalse(self.output.exists())
        self.configure('linux'); self.native.write_bytes(header('linux', shared=False))
        with self.assertRaisesRegex(ValueError, 'library type'): self.package()
        self.assertFalse(self.output.exists())

    @unittest.skipIf(os.name == 'nt', 'symlink privilege is not assumed on Windows')
    def test_internal_framework_links_preserved_and_escape_rejected(self):
        (self.runtime / 'Versions').mkdir()
        (self.runtime / 'Versions/A').mkdir()
        (self.runtime / 'Versions/A/framework').write_bytes(b'framework-resource')
        (self.runtime / 'Versions/Current').symlink_to('A', target_is_directory=True)
        (self.runtime / 'framework').symlink_to('Versions/Current/framework')
        manifest = self.package()
        self.assertEqual(manifest['files']['app/framework']['target'], 'Versions/Current/framework')
        self.assertTrue((self.output / 'app/framework').is_symlink())
        shutil.rmtree(self.output)
        (self.runtime / 'escape').symlink_to(self.notices / 'LICENSE')
        with self.assertRaises(ValueError): self.package()
        self.assertFalse(self.output.exists())
        (self.runtime / 'escape').unlink()
        (self.runtime / 'escape').symlink_to('../notices/LICENSE')
        with self.assertRaises(ValueError): self.package()

    @unittest.skipIf(os.name == 'nt', 'portable Unix hardlink test')
    def test_hardlinks_and_executable_role_symlinks_are_rejected(self):
        os.link(self.native, self.runtime / 'linked')
        with self.assertRaises(ValueError): self.package()
        (self.runtime / 'linked').unlink()
        self.native.unlink(); self.native.symlink_to(self.dispatcher)
        with self.assertRaises(ValueError): self.package()

    def test_output_is_exclusive_and_failed_input_preserves_existing_data(self):
        self.output.mkdir(); marker = self.output / 'user.txt'; marker.write_text('keep')
        with self.assertRaises(FileExistsError): self.package()
        self.assertEqual(marker.read_text(), 'keep')

    def test_missing_notice_and_case_collision_rejected(self):
        (self.notices / 'THIRD_PARTY.md').unlink()
        with self.assertRaises(OSError): self.package()
        self.assertFalse(self.output.exists())
        (self.notices / 'THIRD_PARTY.md').write_text('restored')
        (self.runtime / 'APP').write_bytes(b'case-collision')
        with self.assertRaisesRegex(ValueError, 'case-colliding'): self.package()

    def test_fat_macho_accepts_real_slices_and_rejects_overlaps(self):
        self.configure('macos', 'arm64')
        fat = bytearray(256)
        fat[:8] = b'\xca\xfe\xba\xbe\0\0\0\2'
        for i, (arch, offset) in enumerate([('x86_64', 64), ('arm64', 192)]):
            struct.pack_into('>IIIII', fat, 8+i*20, 0x1000007 if arch == 'x86_64' else 0x100000c,
                             0, offset, 64, 0)
            fat[offset:offset+64] = header('macos', arch, True)[:64]
        self.native.write_bytes(fat)
        self.assertEqual(bundle.binary_identity(self.native, True), ('macos', {'x86_64', 'arm64'}))
        self.package()
        shutil.rmtree(self.output)
        struct.pack_into('>I', fat, 8+20+8, 64)
        self.native.write_bytes(fat)
        with self.assertRaisesRegex(ValueError, 'overlapping'): self.package()


if __name__ == '__main__':
    unittest.main()
