"""Release packaging must consume the same standard WASM the browser tested."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from scripts.verify_release_wasm import verify

ROOT = Path(__file__).resolve().parents[2]


class ReleaseWasmIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.package = self.root / "pkg"
        self.package.mkdir()
        self.dist = self.root / "dist"
        self.dist.mkdir()
        self.sha = "a" * 40
        self.manifest_path = self.root / "build.json"
        artifacts = {}
        for name, data in (("rhwp.js", b"glue"), ("rhwp_bg.wasm", b"\0asm")):
            for directory in (self.package, self.dist):
                (directory / name).write_bytes(data)
            artifacts[name] = {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
        self.manifest = {
            "success": True, "source_dirty": False, "source_sha": self.sha,
            "profile": "release",
            "build_configuration": {
                "profile": "release", "build_mode": "standard", "cdylib_only": "0",
                "crate_types": "rlib+cdylib", "lto_policy": "manifest",
            },
            "tool_commands": {"wasm-opt": ["wasm-opt", "-O"]}, "artifacts": artifacts,
        }
        expected = artifacts["rhwp_bg.wasm"]["sha256"]
        self.consumption = {
            "source_sha": self.sha, "profile": "release", "expected_sha256": expected,
            "matched": True, "received": [{"status": 200, "sha256": expected}],
        }
        (self.root / "consumption").mkdir()
        self.record_path = self.root / "consumption/browser.json"
        self.write()

    def write(self):
        self.manifest_path.write_text(json.dumps(self.manifest))
        self.record_path.write_text(json.dumps(self.consumption))

    def check(self):
        verify(self.manifest_path, [self.package, self.dist], self.sha, True)

    def test_tested_package_can_be_copied_into_distribution(self):
        self.check()

    def test_rebuilt_or_replaced_distribution_is_rejected(self):
        for name in ("rhwp.js", "rhwp_bg.wasm"):
            with self.subTest(name=name):
                (self.dist / name).write_bytes(b"replaced")
                with self.assertRaisesRegex(ValueError, "artifact changed"):
                    self.check()
                (self.dist / name).write_bytes((self.package / name).read_bytes())

    def test_failed_dirty_stale_dev_and_experimental_builds_are_rejected(self):
        original = copy.deepcopy(self.manifest)
        changes = [
            ("success", False), ("source_dirty", True), ("source_sha", "b" * 40),
            ("profile", "dev"), ("tool_commands", {}),
            ("build_configuration", {**original["build_configuration"], "build_mode": "cdylib"}),
            ("build_configuration", {**original["build_configuration"], "lto_policy": "override-false"}),
        ]
        for key, value in changes:
            with self.subTest(key=key, value=value):
                self.manifest = {**original, key: value}
                self.write()
                with self.assertRaises(ValueError):
                    self.check()

    def test_missing_browser_observation_is_rejected(self):
        self.record_path.unlink()
        with self.assertRaisesRegex(ValueError, "actual browser"):
            self.check()
        # Before the browser runs, only source and package identity are checked.
        verify(self.manifest_path, [self.package], self.sha)

    def test_matched_flag_alone_cannot_hide_wrong_source_or_response(self):
        original = copy.deepcopy(self.consumption)
        changes = [
            ("matched", False), ("source_sha", "b" * 40), ("profile", "dev"),
            ("expected_sha256", "0" * 64), ("received", []),
            ("received", [{"status": 404, "sha256": original["expected_sha256"]}]),
            ("received", [{"status": 200, "sha256": "0" * 64}]),
        ]
        for key, value in changes:
            with self.subTest(key=key):
                self.consumption = {**original, key: value}
                self.write()
                with self.assertRaisesRegex(ValueError, "consumption mismatch"):
                    self.check()


class ReleaseWasmWorkflowTests(unittest.TestCase):
    def test_release_consumers_verify_before_upload_without_rebuilding(self):
        for name, upload in (("deploy-pages", "Upload Pages artifact"),
                             ("npm-publish", "Upload WASM artifacts")):
            with self.subTest(workflow=name):
                workflow = (ROOT / f".github/workflows/{name}.yml").read_text()
                build = "python3 scripts/measure_render_diff_wasm.py --profile release --build-mode standard"
                check = "uses: ./.github/actions/verify-release-wasm"
                self.assertEqual(workflow.count(build), 1)
                self.assertEqual(workflow.count(check), 1)
                self.assertLess(workflow.index(build), workflow.index(check))
                self.assertLess(workflow.index(check), workflow.index(f"name: {upload}"))
                self.assertNotIn("wasm-pack build", workflow)
        pages = (ROOT / ".github/workflows/deploy-pages.yml").read_text()
        self.assertLess(pages.index("npx vite build"), pages.index("--package-dir rhwp-studio/dist"))
        self.assertLess(pages.index("--package-dir rhwp-studio/dist"), pages.index("name: Upload Pages artifact"))
        self.assertIn("github.event_name == 'push' && github.ref == 'refs/heads/main'", pages)
        package = (ROOT / ".github/workflows/npm-publish.yml").read_text()
        self.assertIn("needs: validate-release-source", package)
        self.assertIn("default: false", package.split("permissions:", 1)[0])

    def test_release_gate_fails_closed_and_never_builds_another_wasm(self):
        action = (ROOT / ".github/actions/verify-release-wasm/action.yml").read_text()
        self.assertNotIn("wasm-pack", action)
        self.assertNotIn("measure_render_diff_wasm.py", action)
        self.assertNotIn("continue-on-error", action)
        self.assertIn("RHWP_RENDER_DIFF_DIRECT_PDF_GATE: '1'", action)
        self.assertIn("RHWP_WASM_BUILD_MANIFEST:", action)
        self.assertIn("--require-consumption", action)
        self.assertIn("if: ${{ always() }}", action)
        self.assertLess(action.index("npm run e2e:render-diff:ci"),
                        action.index("--require-consumption"))


if __name__ == "__main__":
    unittest.main()
