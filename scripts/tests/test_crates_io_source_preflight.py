"""Local contracts for immutable source candidates, not registry publication."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

from scripts.crates_io_source_preflight import PreflightError, build_report


class SourcePreflightTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Source Contract")
        self.git("config", "user.email", "source@example.invalid")
        self.write(".gitignore", "output/\n")
        self.write("LICENSE", "fixture license text\n")
        self.write("Cargo.toml", '[package]\nname="root-fixture"\nversion="0.1.0"\nlicense="MIT"\n[dependencies]\nleaf={path="leaf"}\n')
        self.write("src/lib.rs", 'pub const INPUT: &[u8] = include_bytes!("../assets/runtime.bin");\n')
        self.write("assets/runtime.bin", "fixture runtime data\n")
        self.write("leaf/Cargo.toml", '[package]\nname="leaf"\nversion="0.1.0"\nlicense="MIT"\npublish=false\n')
        self.write("leaf/src/lib.rs", "pub fn leaf() {}\n")
        self.commit()
        self.policy = {
            "schema_version": 1, "status": "proposal", "packages": [
                {"source": "leaf", "published_name": "leaf", "version": "0.1.0",
                 "include": ["Cargo.toml", "src/**"], "extra_repository_files": {"LICENSE": "LICENSE"}, "local_dependencies": []},
                {"source": ".", "published_name": "root-fixture", "version": "0.9.0",
                 "include": ["Cargo.toml", "LICENSE", "src/**", "assets/runtime.bin"], "local_dependencies": ["leaf"]},
            ],
        }
        self.runtime = {"schema_version": 1, "files": [{"consumer": "src/lib.rs", "path": "assets/runtime.bin"}]}

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.repo).decode().strip()

    def write(self, name, content):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture")

    def run_check(self, destination="output/run"):
        return build_report(self.repo, self.policy, self.runtime, self.repo / destination)

    def test_reproducible_archive_retains_original_manifest_and_never_claims_publish_ready(self):
        first = self.run_check("output/first")
        second = self.run_check("output/second")
        self.assertEqual(first["packages"], second["packages"])
        self.assertFalse(first["publish_ready"])
        self.assertFalse(first["cargo_verified"])
        self.assertEqual(first["dependency_order"], ["leaf", "root-fixture"])
        self.assertEqual({x["kind"] for x in first["manifest_blockers"]}, {"publish_false", "path_version_missing"})
        with tarfile.open(self.repo / "output/first/root-fixture.source.tar.gz") as archive:
            manifest = archive.extractfile("root-fixture-source/Cargo.toml").read()
        self.assertIn(b'version="0.1.0"', manifest)
        self.assertEqual(self.git("status", "--porcelain"), "")

    def test_dirty_source_and_untracked_secrets_do_not_enter_commit_snapshot(self):
        first = self.run_check("output/first")
        self.write("src/lib.rs", "dirty workspace text\n")
        self.write("src/untracked.env", "SECRET=do-not-package\n")
        second = self.run_check("output/second")
        self.assertEqual(first["packages"], second["packages"])
        with tarfile.open(self.repo / "output/second/root-fixture.source.tar.gz") as archive:
            self.assertNotIn("root-fixture-source/src/untracked.env", archive.getnames())
            self.assertIn(b"include_bytes!", archive.extractfile("root-fixture-source/src/lib.rs").read())

    def test_missing_runtime_input_is_rejected(self):
        self.policy["packages"][1]["include"].remove("assets/runtime.bin")
        with self.assertRaisesRegex(PreflightError, "runtime source omitted"):
            self.run_check()

    def test_new_notice_cannot_be_silently_dropped(self):
        self.write("leaf/NOTICE", "required notice\n")
        self.commit()
        with self.assertRaisesRegex(PreflightError, "provenance source omitted"):
            self.run_check()

    def test_license_copy_is_required(self):
        self.policy["packages"][0]["extra_repository_files"] = {}
        with self.assertRaisesRegex(PreflightError, "license text missing"):
            self.run_check()

    def test_parent_archive_path_is_rejected(self):
        self.policy["packages"][0]["extra_repository_files"] = {"../LICENSE": "LICENSE"}
        with self.assertRaisesRegex(PreflightError, "unsafe"):
            self.run_check()

    def test_tracked_private_source_is_rejected(self):
        self.write("mydocs/privacy/secret", "private fixture\n")
        self.commit()
        self.policy["packages"][1]["include"].append("mydocs/privacy/**")
        with self.assertRaisesRegex(PreflightError, "private/generated/corpus"):
            self.run_check()

    def test_symlink_input_is_rejected(self):
        os.symlink("../LICENSE", self.repo / "src/link.rs")
        self.commit()
        with self.assertRaisesRegex(PreflightError, "non-regular Git entry"):
            self.run_check()

    def test_output_outside_ignored_directory_is_rejected_without_mutation(self):
        original = (self.repo / "Cargo.toml").read_bytes()
        with self.assertRaisesRegex(PreflightError, "below repository output"):
            self.run_check("Cargo.toml")
        self.assertEqual((self.repo / "Cargo.toml").read_bytes(), original)

    def test_output_symlink_cannot_overwrite_repository_source(self):
        out = self.repo / "output/run"
        out.mkdir(parents=True)
        os.symlink(self.repo / "Cargo.toml", out / "leaf.source.tar.gz")
        original = (self.repo / "Cargo.toml").read_bytes()
        with self.assertRaisesRegex(PreflightError, "unsafe output file"):
            self.run_check()
        self.assertEqual((self.repo / "Cargo.toml").read_bytes(), original)

    def test_cycle_in_actual_source_dependencies_is_rejected(self):
        with (self.repo / "leaf/Cargo.toml").open("a") as manifest:
            manifest.write('[dependencies]\nroot-fixture={path="..",version="0.1"}\n')
        self.commit()
        self.policy["packages"][0]["local_dependencies"] = ["root-fixture"]
        with self.assertRaisesRegex(PreflightError, "cycle"):
            self.run_check()

    def test_declared_graph_cannot_hide_a_real_path_dependency(self):
        self.policy["packages"][1]["local_dependencies"] = []
        with self.assertRaisesRegex(PreflightError, "graph mismatch"):
            self.run_check()

    def test_unknown_schema_is_rejected(self):
        self.policy["schema_version"] = 2
        with self.assertRaisesRegex(PreflightError, "unsupported source policy"):
            self.run_check()

    def test_workspace_inherited_path_dependency_is_not_hidden(self):
        self.write("Cargo.toml", '[package]\nname="root-fixture"\nversion="0.1.0"\nlicense="MIT"\n[workspace.dependencies]\nleaf={path="leaf",version="0.1"}\n[dependencies]\nleaf={workspace=true}\n')
        self.commit()
        result = self.run_check()
        self.assertEqual(result["dependency_order"], ["leaf", "root-fixture"])
        self.assertNotIn("path_version_missing", {r["kind"] for r in result["manifest_blockers"]})

    def test_root_patch_is_reported_without_being_applied(self):
        with (self.repo / "Cargo.toml").open("a") as manifest:
            manifest.write('[patch.crates-io]\nleaf={path="leaf"}\n')
        self.commit()
        result = self.run_check()
        self.assertIn("root_patch_not_propagated", {r["kind"] for r in result["manifest_blockers"]})

    def test_relocated_runtime_input_is_rejected(self):
        self.policy["packages"][1]["include"].remove("assets/runtime.bin")
        self.policy["packages"][1]["extra_repository_files"] = {"relocated.bin": "assets/runtime.bin"}
        with self.assertRaisesRegex(PreflightError, "not preserved within one package"):
            self.run_check()

    def test_font_provenance_must_travel_with_the_asset(self):
        payload = b"synthetic font bytes"
        (self.repo / "assets/font.otf").write_bytes(payload)
        self.write("assets/font-source.txt", "fixture provenance\n")
        self.write("assets/fonts/licenses/font-notices.json", json.dumps({
            "repositoryOnlyFonts": [{"repositoryPath": "assets/font.otf",
                "sha256": hashlib.sha256(payload).hexdigest(), "licenseFiles": ["LICENSE"],
                "provenance": "assets/font-source.txt"}],
        }))
        self.commit()
        self.policy["packages"][1]["include"].append("assets/font.otf")
        with self.assertRaisesRegex(PreflightError, "font license/provenance omitted"):
            self.run_check()
        self.policy["packages"][1]["include"].append("assets/font-source.txt")
        result = self.run_check()
        self.assertEqual(len(result["font_assets_checked"]), 1)

    def test_cli_failure_replaces_a_previous_pass_report(self):
        self.write("policy.json", json.dumps(self.policy))
        self.write("runtime.json", json.dumps(self.runtime))
        tool = Path(__file__).resolve().parents[1] / "crates_io_source_preflight.py"
        command = [sys.executable, str(tool), "--repo", str(self.repo),
                   "--policy", "policy.json", "--runtime-files", "runtime.json"]
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = self.repo / "output/pr-review/issue8174/source-preflight/source-preflight.json"
        self.assertEqual(json.loads(report.read_text())["source_check"], "pass")
        self.policy["packages"][1]["include"].remove("assets/runtime.bin")
        self.write("policy.json", json.dumps(self.policy))
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(report.read_text())["source_check"], "failed")
        self.assertFalse(json.loads(report.read_text())["publish_ready"])


if __name__ == "__main__":
    unittest.main()
