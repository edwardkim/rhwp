"""Check XCFramework command construction with fake Apple tools, not Apple artifacts."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class PackageSwiftXCFrameworkTests(unittest.TestCase):
    def invoke(self, native_skia, *, previous=None, common_git_dir=False):
        with tempfile.TemporaryDirectory(prefix="rhwp-xcframework-contract-") as directory:
            root = Path(directory)
            for relative in ["scripts/package-swift-xcframework.sh", "Cargo.toml", "LICENSE",
                             "bindings/Native/Cargo.toml", "bindings/swift/README.md",
                             "bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h"]:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, target)
            fake_tools = root / "fake-tools"
            fake_tools.mkdir()
            stub = fake_tools / "stub"
            stub.write_text("""#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
name = Path(sys.argv[0]).name
with open(os.environ['RHWP_COMMAND_LOG'], 'a') as log:
    log.write(json.dumps([name, *sys.argv[1:]]) + '\\n')
if name == 'uname': print('Darwin')
elif name == 'lipo': Path(sys.argv[sys.argv.index('-output') + 1]).write_bytes(b'mock library')
elif name == 'xcodebuild': Path(sys.argv[sys.argv.index('-output') + 1]).mkdir()
elif name == 'zip': Path(sys.argv[2]).write_bytes(b'mock archive')
""")
            stub.chmod(0o700)
            for name in ["uname", "cargo", "rustup", "lipo", "xcodebuild", "zip"]:
                (fake_tools / name).symlink_to(stub)
            log = root / "commands.jsonl"
            target_dir = root / "shared-target"
            env = dict(os.environ, PATH=f"{fake_tools}:{os.environ['PATH']}",
                       RHWP_NATIVE_SKIA=native_skia, CARGO_TARGET_DIR=str(target_dir),
                       RHWP_COMMAND_LOG=str(log))
            if common_git_dir:
                repository = root / "parent-repository"
                subprocess.run(["git", "init", "--quiet", str(repository)], check=True)
                (root / ".git").write_text(f"gitdir: {repository / '.git'}\n")
                target_dir = repository / "target/pr-review"
                del env["CARGO_TARGET_DIR"]
            command = ["bash", str(root / "scripts/package-swift-xcframework.sh")]
            if previous is not None:
                first = subprocess.run(command, env=dict(env, RHWP_NATIVE_SKIA=previous),
                                       text=True, capture_output=True, check=False)
                self.assertEqual(first.returncode, 0, first.stderr)
                log.unlink()
            completed = subprocess.run(command,
                                       env=env, text=True, capture_output=True, check=False)
            calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
            if previous is not None:
                for variant in ["default", "native-skia"]:
                    self.assertEqual(len(list((root / "dist/swift" / variant).glob("*.zip"))), 1)
            return completed, calls, str(target_dir)

    def test_default_build_has_no_skia_feature(self):
        result, calls, target_dir = self.invoke("0")
        self.assertEqual(result.returncode, 0, result.stderr)
        builds = [call for call in calls if call[0] == "cargo"]
        self.assertEqual(len(builds), 4)
        for call in builds:
            self.assertNotIn("--features", call)
            self.assertIn("--locked", call)
            self.assertEqual(call[call.index("--target-dir") + 1], target_dir)
        self.assertIn("-apple-xcframework.zip", result.stdout)

    def test_opt_in_build_propagates_feature_and_target_path(self):
        result, calls, target_dir = self.invoke("1")
        self.assertEqual(result.returncode, 0, result.stderr)
        builds = [call for call in calls if call[0] == "cargo"]
        self.assertEqual(len(builds), 4)
        for call in builds:
            self.assertEqual(call[call.index("--features") + 1], "native-skia")
        for call in calls:
            if call[0] == "lipo":
                self.assertTrue(all(value.startswith(target_dir) for value in call[2:call.index("-output")]))
        self.assertIn("-apple-xcframework-native-skia.zip", result.stdout)

    def test_unknown_feature_value_fails_before_any_build(self):
        result, calls, _ = self.invoke("yes")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])
        self.assertIn("RHWP_NATIVE_SKIA must be 0 or 1", result.stdout)

    def test_building_both_variants_preserves_both_archives(self):
        result, _, _ = self.invoke("1", previous="0")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_default_target_uses_common_repository_for_linked_checkout(self):
        result, calls, target_dir = self.invoke("0", common_git_dir=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        builds = [call for call in calls if call[0] == "cargo"]
        self.assertEqual(len(builds), 4)
        for call in builds:
            self.assertEqual(call[call.index("--target-dir") + 1], target_dir)


if __name__ == "__main__":
    unittest.main()
