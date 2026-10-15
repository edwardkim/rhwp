#!/usr/bin/env python3
"""Packaging contracts with temporary files; no Cargo, native library or Qt required."""

import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "package-native-host.py"
SPEC = importlib.util.spec_from_file_location("rhwp_package_native_host", SCRIPT)
packager = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packager)
CAPABILITIES = {
    "ok": True, "abiVersion": 1, "nativeSkia": True, "hostRequests": True,
    "worker": True, "threadAffine": True, "maxSessions": 64,
}
EXAMPLE = '''import argparse, json
from pathlib import Path
parser = argparse.ArgumentParser()
parser.add_argument("--library", required=True)
parser.add_argument("--input")
parser.add_argument("--output")
args = parser.parse_args()
assert Path(args.library).read_bytes() == b"fixture native library"
print(json.dumps(vars(args)))
'''


class PackageNativeHostContracts(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="rhwp-native-package-contract-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.source = self.root / "source"
        self.source.mkdir()
        for relative in packager.SOURCE_FILES.values():
            path = self.source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture\n", encoding="utf-8")
        for name in ("native_host.py", "qt_native_host.py"):
            (self.source / "bindings/Native/examples" / name).write_text(EXAMPLE, encoding="utf-8")
        self.library = self.root / packager.LIBRARY_NAMES[packager.platform.system()]
        self.library.write_bytes(b"fixture native library")
        self.output = self.root / "package"
        self.revision = "1234567890abcdef" * 2 + "12345678"

    def package(self):
        return packager.package(self.library, self.output, self.revision, source_root=self.source)

    def build(self):
        with patch.object(packager, "probe_library", return_value=dict(CAPABILITIES)):
            return self.package()

    def launch(self, directory=None, *arguments):
        return subprocess.run(
            [sys.executable, str((directory or self.output) / "run.py"), *arguments],
            cwd=self.root, capture_output=True, text=True, check=False,
        )

    def test_manifest_identifies_copied_bytes_and_actual_capabilities(self):
        def probe(path):
            self.assertNotEqual(path, self.library)
            self.assertEqual(path.read_bytes(), self.library.read_bytes())
            return dict(CAPABILITIES)

        with patch.object(packager, "probe_library", side_effect=probe) as mocked:
            manifest = self.package()
        mocked.assert_called_once()
        self.assertEqual(manifest, json.loads((self.output / "manifest.json").read_text()))
        self.assertEqual(manifest["capabilities"], CAPABILITIES)
        self.assertEqual(manifest["declaredSourceRevision"], self.revision)
        self.assertIn("not verified", manifest["sourceRevisionEvidence"])
        self.assertEqual(manifest["platform"], packager.platform.system())
        self.assertEqual(manifest["architecture"], packager.architecture())
        expected = {*packager.SOURCE_FILES, "run.py", manifest["artifact"]["path"]}
        self.assertEqual(set(manifest["files"]), expected)
        self.assertIn("docs/QT_NATIVE_HOST.md", manifest["files"])
        for relative, digest in manifest["files"].items():
            self.assertEqual(digest, hashlib.sha256((self.output / relative).read_bytes()).hexdigest())
        self.assertEqual(manifest["artifact"]["sha256"], packager.sha256(self.library))
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])

    def test_missing_or_wrong_capabilities_fail_without_output(self):
        for name, value in (
            ("ok", False), ("nativeSkia", False), ("hostRequests", False),
            ("worker", False), ("worker", 1), ("abiVersion", 2), ("abiVersion", True),
        ):
            with self.subTest(name=name, value=value):
                caps = {**CAPABILITIES, name: value}
                with patch.object(packager, "probe_library", return_value=caps):
                    with self.assertRaises(ValueError):
                        self.package()
                self.assertFalse(self.output.exists())
        caps = dict(CAPABILITIES)
        del caps["worker"]
        with self.assertRaises(ValueError):
            packager.validate_capabilities(caps)
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])

    def test_existing_directory_file_and_dangling_symlink_are_not_overwritten(self):
        with patch.object(packager, "probe_library") as probe:
            self.output.mkdir()
            with self.assertRaises(FileExistsError):
                self.package()
            self.assertEqual(list(self.output.iterdir()), [])
            self.output.rmdir()
            self.output.write_bytes(b"preserved")
            with self.assertRaises(FileExistsError):
                self.package()
            self.assertEqual(self.output.read_bytes(), b"preserved")
            self.output.unlink()
            try:
                self.output.symlink_to(self.root / "absent")
            except OSError:
                # Windows may not permit symlink creation for this account.
                pass
            else:
                with self.assertRaises(FileExistsError):
                    self.package()
                self.assertTrue(self.output.is_symlink())
            probe.assert_not_called()

    def test_bad_source_revision_or_library_does_not_create_output(self):
        with patch.object(packager, "probe_library") as probe:
            self.revision = "HEAD"
            with self.assertRaisesRegex(ValueError, "40-digit"):
                self.package()
            self.revision = "a" * 40
            missing = self.source / "bindings/Native/examples/qt_native_host.py"
            missing.unlink()
            with self.assertRaisesRegex(ValueError, "missing package source"):
                self.package()
            missing.write_text(EXAMPLE)
            self.library = self.root / "wrong.extension"
            self.library.write_bytes(b"fixture")
            with self.assertRaisesRegex(ValueError, "explicit regular"):
                self.package()
            probe.assert_not_called()
        self.assertFalse(self.output.exists())

    def test_copy_failure_removes_only_private_staging(self):
        original_copy = shutil.copyfile
        calls = 0

        def fail_second_copy(source, destination):
            nonlocal calls
            calls += 1
            if calls == 2:
                Path(destination).write_bytes(b"partial")
                raise OSError("injected copy failure")
            return original_copy(source, destination)

        with patch.object(packager.shutil, "copyfile", side_effect=fail_second_copy):
            with self.assertRaisesRegex(OSError, "injected"):
                self.build()
        self.assertFalse(self.output.exists())
        self.assertEqual(self.library.read_bytes(), b"fixture native library")
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])

    def test_publication_failure_removes_new_partial_output(self):
        original_rename = Path.rename
        calls = 0

        def fail_second_rename(source, destination):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected publication failure")
            return original_rename(source, destination)

        with patch.object(Path, "rename", fail_second_rename):
            with self.assertRaisesRegex(OSError, "publication failure"):
                self.build()
        self.assertFalse(self.output.exists())
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])

    def test_output_created_during_probe_is_preserved(self):
        def probe(_):
            self.output.mkdir()
            (self.output / "owner.txt").write_text("other writer")
            return dict(CAPABILITIES)

        with patch.object(packager, "probe_library", side_effect=probe):
            with self.assertRaises(FileExistsError):
                self.package()
        self.assertEqual((self.output / "owner.txt").read_text(), "other writer")
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])

    def test_relocated_launcher_uses_packaged_library_for_both_modes(self):
        self.build()
        moved = self.root / "moved package with spaces"
        self.output.rename(moved)
        shutil.rmtree(self.source)
        self.library.unlink()
        for mode in ("headless", "window"):
            with self.subTest(mode=mode):
                result = self.launch(moved, mode, "--input", "source.hwp", "--output", "new.hwpx")
                self.assertEqual(result.returncode, 0, result.stderr)
                response = json.loads(result.stdout)
                self.assertEqual(Path(response["library"]).parent, moved / "lib")
                self.assertEqual(response["input"], "source.hwp")
                self.assertEqual(response["output"], "new.hwpx")
        override = self.launch(moved, "headless", "--output", "new.hwpx", "--lib", "other.so")
        self.assertNotEqual(override.returncode, 0)

    def test_launcher_rejects_digest_path_escape_and_wrong_platform(self):
        manifest = self.build()
        library = self.output / manifest["artifact"]["path"]
        library.write_bytes(b"modified")
        result = self.launch(None, "headless", "--output", "new.hwpx")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("digest mismatch", result.stderr)
        library.write_bytes(b"fixture native library")
        manifest["files"]["../outside"] = "0" * 64
        (self.output / "manifest.json").write_text(json.dumps(manifest))
        result = self.launch(None, "headless", "--output", "new.hwpx")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("package-relative path", result.stderr)
        del manifest["files"]["../outside"]
        manifest["platform"] = "different OS"
        (self.output / "manifest.json").write_text(json.dumps(manifest))
        result = self.launch(None, "headless", "--output", "new.hwpx")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match", result.stderr)

    def test_invalid_native_library_probe_and_missing_cli_arguments_fail(self):
        with self.assertRaisesRegex(ValueError, "probe failed"):
            self.package()
        self.assertFalse(self.output.exists())
        self.assertEqual(list(self.root.glob(".rhwp-native-package-*")), [])
        result = subprocess.run([sys.executable, str(SCRIPT)], capture_output=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"requires --library", result.stderr)


if __name__ == "__main__":
    unittest.main()
