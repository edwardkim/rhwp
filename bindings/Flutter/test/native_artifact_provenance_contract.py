#!/usr/bin/env python3
"""Portable archive/source rejection controls; never SDK or Native execution."""

import copy
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
import native_artifact_provenance as provenance


class NativeArtifactContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repository"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Portable qualification")
        self.git("config", "user.email", "portable@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "core.filemode", "false")
        for name in ("Cargo.toml", "Cargo.lock", "src/lib.rs", "bindings/Native/Cargo.toml",
                     "vendor/crate/lib.rs", "assets/font.bin", "bindings/swift/Sources/CRhwpNative/api.h",
                     "mydocs/compile-time.md", "bindings/Flutter/lib/view.dart",
                     ".github/workflows/desktop-platform-qualification.yml"):
            self.write(self.repo / name, "original " + name)
        self.commit()
        self.old = self.git("rev-parse", "HEAD").strip()
        self.archive = self.root / "artifact"
        self.archive.mkdir()
        self.configure("windows")

    def tearDown(self):
        self.temporary.cleanup()

    def git(self, *args):
        return subprocess.check_output(["git", "-c", "gc.auto=0", *args], cwd=self.repo,
                                       stderr=subprocess.DEVNULL, text=True)

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture input")

    @staticmethod
    def write(path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(data)

    def save(self, filename, value):
        self.write(self.archive / filename, json.dumps(value))

    def configure(self, platform):
        self.platform = platform
        self.name = "rhwp_native_ffi.dll" if platform == "windows" else "librhwp_native_ffi.dylib"
        self.native = self.archive / "native-target" / self.name
        self.native.parent.mkdir(exist_ok=True)
        self.native.write_bytes(b"portable artifact identity, never executable")
        self.fingerprint = hashlib.sha256(self.native.read_bytes()).hexdigest()
        if platform == "windows":
            self.inner = {"source_head": self.old, "build": "passed",
                          "artifacts": {self.name: self.fingerprint}}
            self.outer = {"platform": platform, "source_head": self.old, "build": "failed",
                          "steps": [{"command": ["rustup", "run", "1.93.1", "cargo", "build", "--release",
                                                  "-p", "rhwp-native-ffi", "--features", "native-skia-d3d",
                                                  "--target", "x86_64-pc-windows-msvc"], "exitCode": 0},
                                    {"command": ["python", "consumer.py"], "exitCode": 1}]}
            self.save("native-build-receipt.json", self.inner)
            self.save("windows-sdk-receipt.json", self.outer)
        else:
            self.outer = {"platform": platform, "source_head": self.old, "build": "passed",
                          "targetSdkCompileLinkVerified": True, "metalProducerReadbackVerified": True,
                          "architecture": "arm64", "rustTarget": "aarch64-apple-darwin",
                          "artifacts": {"native": {"path": "/old-ci/native-target/" + self.name,
                                                   "sha256": self.fingerprint, "bytes": self.native.stat().st_size}},
                          "steps": [{"command": ["rustup", "run", "1.93.1", "cargo", "build", "--locked",
                                                  "--manifest-path", "/old-ci/source/bindings/Native/Cargo.toml",
                                                  "--features", "native-skia-metal", "--lib", "--target",
                                                  "aarch64-apple-darwin"], "exitCode": 0}]}
            self.producer = {"platform": platform, "source_head": self.old, "status": "passed",
                             "producerReadbackVerified": True, "nativeSha256": self.fingerprint}
            self.save("macos-sdk-receipt.json", self.outer)
            self.save("metal-producer.json", self.producer)

    def load(self, source=None):
        return provenance.load_native_artifact(self.archive, self.repo, self.platform, source or self.old)

    def test_windows_native_success_survives_later_consumer_failure(self):
        value = self.load()
        self.assertEqual(value["native"], self.native)
        self.assertEqual(value["nativeReceipt"], self.archive / "native-build-receipt.json")
        self.assertFalse(value["proof"]["nativeRebuilt"])
        self.assertTrue(value["proof"]["sourceEquivalenceVerified"])
        self.assertEqual(value["proof"]["currentSourceHead"], self.old)
        self.assertEqual(value["proof"]["sourceManifestSha256"], value["proof"]["currentSourceManifestSha256"])
        json.dumps(value["proof"])

    def test_macos_requires_apple_sdk_and_the_same_native_producer(self):
        shutil.rmtree(self.archive)
        self.archive.mkdir()
        self.configure("macos")
        value = self.load()
        self.assertEqual(value["nativeReceipt"], self.archive / "macos-sdk-receipt.json")
        self.assertEqual(value["proof"]["nativeTarget"], "aarch64-apple-darwin")
        for field, replacement in (("build", "failed"), ("metalProducerReadbackVerified", False),
                                    ("targetSdkCompileLinkVerified", False), ("architecture", "x86_64")):
            candidate = dict(self.outer, **{field: replacement})
            self.save("macos-sdk-receipt.json", candidate)
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.load()
        self.save("macos-sdk-receipt.json", self.outer)
        for field, replacement in (("status", "failed"), ("producerReadbackVerified", False),
                                    ("nativeSha256", "0" * 64), ("source_head", "0" * 40),
                                    ("platform", "windows")):
            self.save("metal-producer.json", dict(self.producer, **{field: replacement}))
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.load()

    def test_committed_changes_in_exactly_the_three_exclusions_are_allowed(self):
        for name in ("bindings/Flutter/lib/view.dart", "mydocs/pr/assets/issue8701/new/evidence.json",
                     ".github/workflows/desktop-platform-qualification.yml"):
            self.write(self.repo / name, "new Flutter/qualification input")
        self.commit()
        proof = self.load()["proof"]
        self.assertNotEqual(proof["currentSourceHead"], proof["nativeSourceHead"])
        self.assertEqual(len(proof["excludedChangedPaths"]), 3)
        self.assertEqual(proof["sourceManifestSha256"], proof["currentSourceManifestSha256"])

    def test_every_other_git_input_is_conservatively_rejected(self):
        for name in ("Cargo.toml", "Cargo.lock", "src/lib.rs", "bindings/Native/Cargo.toml",
                     "vendor/crate/lib.rs", "assets/font.bin", "bindings/swift/Sources/CRhwpNative/api.h",
                     "mydocs/compile-time.md", "unknown/new.file", "bindings/Flutter-other/new.rs",
                     "mydocs/pr/assets/issue87010/evidence.json",
                     ".github/workflows/desktop-platform-qualification.yml.extra"):
            self.git("reset", "--hard", self.old)
            self.write(self.repo / name, "changed Native or unknown input")
            self.commit()
            with self.subTest(path=name), self.assertRaisesRegex(ValueError, "Native build inputs differ"):
                self.load()

    def test_committed_file_mode_change_is_not_a_blob_only_equivalence(self):
        self.git("update-index", "--chmod=+x", "src/lib.rs")
        self.git("commit", "-qm", "mode changed")
        self.assertFalse(self.git("status", "--porcelain"))
        with self.assertRaisesRegex(ValueError, "Native build inputs differ"):
            self.load()

    def test_dirty_current_including_allowed_flutter_is_rejected(self):
        for name, staged in (("Cargo.toml", False), ("bindings/Flutter/lib/view.dart", True),
                             ("untracked.rs", False)):
            self.git("reset", "--hard", self.old)
            self.write(self.repo / name, "dirty input")
            if staged:
                self.git("add", name)
            with self.subTest(path=name), self.assertRaisesRegex(ValueError, "clean"):
                self.load()
            if name == "untracked.rs":
                (self.repo / name).unlink()

    def test_missing_or_nonliteral_old_source_never_falls_back_to_head(self):
        for source in ("HEAD", self.old[:8], "0" * 40, "Z" * 40):
            inner = dict(self.inner, source_head=source)
            outer = dict(self.outer, source_head=source)
            self.save("native-build-receipt.json", inner)
            self.save("windows-sdk-receipt.json", outer)
            with self.subTest(source=source), self.assertRaises(ValueError):
                self.load(source)

    def test_wrong_source_platform_or_native_build_cannot_be_accepted(self):
        for filename, base, field, replacement in (
                ("native-build-receipt.json", self.inner, "source_head", "0" * 40),
                ("native-build-receipt.json", self.inner, "build", "failed"),
                ("native-build-receipt.json", self.inner, "platform", "macos"),
                ("windows-sdk-receipt.json", self.outer, "platform", "macos"),
                ("windows-sdk-receipt.json", self.outer, "source_head", "0" * 40)):
            self.save("native-build-receipt.json", self.inner)
            self.save("windows-sdk-receipt.json", self.outer)
            self.save(filename, dict(base, **{field: replacement}))
            with self.subTest(field=field, file=filename), self.assertRaises(ValueError):
                self.load()

    def test_windows_native_cargo_step_needs_real_launcher_package_feature_target_and_success(self):
        original = self.outer["steps"][0]
        changes = []
        for value in (1, False, None):
            changes.append(dict(original, exitCode=value))
        for token, replacement in (("rustup", "echo"), ("build", "check"),
                                   ("rhwp-native-ffi", "another-package"),
                                   ("native-skia-d3d", "native-skia-metal"),
                                   ("x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu")):
            command = list(original["command"])
            command[command.index(token)] = replacement
            changes.append(dict(original, command=command))
        for candidate in changes:
            self.save("windows-sdk-receipt.json", dict(self.outer, steps=[candidate]))
            with self.subTest(step=candidate), self.assertRaises(ValueError):
                self.load()
        self.save("windows-sdk-receipt.json", dict(self.outer, steps=[original, original]))
        with self.assertRaises(ValueError):
            self.load()

    def test_native_hash_and_declared_bytes_are_authoritative(self):
        self.native.write_bytes(b"modified Native bytes")
        with self.assertRaisesRegex(ValueError, "hash"):
            self.load()
        self.native.write_bytes(b"portable artifact identity, never executable")
        candidate = copy.deepcopy(self.inner)
        candidate["artifacts"][self.name] = {"sha256": self.fingerprint, "bytes": True}
        self.save("native-build-receipt.json", candidate)
        with self.assertRaises(ValueError):
            self.load()

    def test_duplicate_native_or_receipt_names_are_ambiguous_even_with_identical_bytes(self):
        duplicate = self.archive / "duplicate" / self.name.upper()
        duplicate.parent.mkdir()
        shutil.copyfile(self.native, duplicate)
        with self.assertRaisesRegex(ValueError, "exactly one"):
            self.load()
        duplicate.unlink()
        shutil.copyfile(self.archive / "native-build-receipt.json", duplicate.with_name("native-build-receipt.json"))
        with self.assertRaisesRegex(ValueError, "exactly one"):
            self.load()

    def test_missing_outer_sdk_receipt_is_not_an_unattested_native_success(self):
        (self.archive / "windows-sdk-receipt.json").unlink()
        with self.assertRaises(ValueError):
            self.load()

    def test_all_archive_symlinks_including_unrelated_external_directories_are_rejected(self):
        link = self.archive / "linked"
        try:
            link.symlink_to(self.repo, target_is_directory=True)
        except OSError:
            self.skipTest("host does not permit creating a symlink")
        with self.assertRaises(ValueError):
            self.load()
        link.unlink()
        link.symlink_to(self.native)
        with self.assertRaises(ValueError):
            self.load()
        link.unlink()
        root_link = self.root / "artifact-link"
        root_link.symlink_to(self.archive, target_is_directory=True)
        with self.assertRaises(ValueError):
            provenance.load_native_artifact(root_link, self.repo, self.platform, self.old)

    def test_external_hardlink_alias_is_rejected(self):
        alias = self.root / "external-alias"
        os.link(self.native, alias)
        with self.assertRaises(ValueError):
            self.load()

    def test_windows_direntry_fast_stat_zero_identity_does_not_reject_a_regular_file(self):
        # Windows DirEntry.stat may leave these identity fields at zero. The
        # authoritative path/fd metadata has a real link count and must agree.
        real_scandir = os.scandir
        fast_stat_calls = []
        class FastEntry:
            def __init__(self, entry):
                self.entry, self.path = entry, entry.path
            def stat(self, *, follow_symlinks=True):
                fast_stat_calls.append(self.path)
                info = self.entry.stat(follow_symlinks=follow_symlinks)
                return SimpleNamespace(st_mode=info.st_mode, st_dev=0, st_ino=0, st_nlink=0,
                                       st_size=info.st_size, st_mtime_ns=info.st_mtime_ns,
                                       st_ctime_ns=info.st_ctime_ns,
                                       st_file_attributes=getattr(info, "st_file_attributes", 0))
        @contextmanager
        def fast_scandir(directory):
            with real_scandir(directory) as entries:
                yield (FastEntry(entry) for entry in entries)
        with mock.patch.object(provenance.os, "scandir", side_effect=fast_scandir):
            result = self.load()
        self.assertEqual(result["native"], self.native)
        self.assertFalse(fast_stat_calls, "archive identity must use path/fd metadata")

    def test_authoritative_zero_link_count_is_still_rejected_with_bounded_entry_context(self):
        real_lstat = Path.lstat
        def zero_link_count(path, *args, **kwargs):
            info = real_lstat(path, *args, **kwargs)
            if path != self.native:
                return info
            return SimpleNamespace(st_mode=info.st_mode, st_dev=info.st_dev, st_ino=info.st_ino,
                                   st_nlink=0, st_size=info.st_size, st_mtime_ns=info.st_mtime_ns,
                                   st_ctime_ns=info.st_ctime_ns,
                                   st_file_attributes=getattr(info, "st_file_attributes", 0))
        with mock.patch.object(Path, "lstat", zero_link_count):
            with self.assertRaisesRegex(ValueError, 'entry="native-target/rhwp_native_ffi.dll", type=regular, nlink=0') as caught:
                self.load()
        self.assertNotIn(str(self.root), str(caught.exception))
        self.assertLess(len(str(caught.exception)), 300)

    def test_reparse_boundary_and_entry_diagnostic_remain_strict_and_bounded(self):
        info = SimpleNamespace(st_mode=stat.S_IFREG, st_file_attributes=0x400, st_nlink=1)
        with self.assertRaisesRegex(ValueError, "reparse"):
            provenance._signature(info)
        description = provenance._entry_description("long/" + "\x1b\n\N{HANGUL SYLLABLE GA}" * 3000, info)
        self.assertLess(len(description), 230)
        self.assertNotIn("\x1b", description)
        self.assertNotIn("\n", description)
        self.assertIn("type=reparse, nlink=1", description)

    def test_json_duplicate_keys_and_nonfinite_values_are_rejected(self):
        for content in ('{"source_head":"' + self.old + '","source_head":"' + self.old + '"}',
                        '{"value":NaN}', '[]'):
            self.write(self.archive / "native-build-receipt.json", content)
            with self.subTest(content=content), self.assertRaises(ValueError):
                self.load()

    def test_empty_native_and_archive_receipt_tree_bounds_fail_closed(self):
        self.native.write_bytes(b"")
        with self.assertRaises(ValueError):
            self.load()
        self.native.write_bytes(b"portable artifact identity, never executable")
        for constant, value in (("MAX_ENTRIES", 1), ("MAX_ARCHIVE_BYTES", 1),
                                 ("MAX_NATIVE_BYTES", 1), ("MAX_RECEIPT_BYTES", 1),
                                 ("MAX_TREE_ENTRIES", 1), ("MAX_TREE_BYTES", 1)):
            with mock.patch.object(provenance, constant, value):
                with self.subTest(bound=constant), self.assertRaises(ValueError):
                    self.load()

    def test_promisor_metadata_is_usable_but_missing_tree_cannot_launch_a_fetch(self):
        marker_script = self.root / "fetch.py"
        marker = marker_script.with_suffix(".ran")
        self.write(marker_script, 'from pathlib import Path; Path(__file__).with_suffix(".ran").write_text("unexpected fetch")')
        def argument(path):
            return str(path).replace("%", "%%").replace(" ", "% ")
        self.git("config", "remote.private.url", "ext::" + argument(sys.executable) + " " + argument(marker_script))
        self.git("config", "remote.private.promisor", "true")
        self.git("config", "extensions.partialClone", "private")
        self.git("config", "protocol.ext.allow", "always")
        self.load()  # A promisor marker alone must not reject complete metadata.
        tree = self.git("rev-parse", "HEAD:src").strip()
        object_path = self.repo / ".git/objects" / tree[:2] / tree[2:]
        object_path.unlink()  # Only this disposable fixture's private loose tree.
        with self.assertRaisesRegex(ValueError, "metadata unavailable"):
            self.load()
        self.assertFalse(marker.exists(), "blocked Git metadata read executed a remote helper")


if __name__ == "__main__":
    unittest.main()
