"""[#4655] 공식 배포 표면을 v0.8.2 채널로 고정한다.

새 배포 채널은 파일 하나만 추가돼도 태그나 릴리스 이벤트에서 실제 게시를
시도할 수 있다. 따라서 철회한 채널의 실행 자산이 다시 생기지 않는지와 기존
채널의 핵심 workflow가 남아 있는지를 함께 검사한다.
"""

from __future__ import annotations

import json
import importlib.util
import os
import re
import struct
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
RELEASE_BINARY_WORKFLOW = REPO_ROOT / ".github/workflows/release-binary.yml"

ABI_SPEC = importlib.util.spec_from_file_location(
    "rhwp_verify_linux_abi", REPO_ROOT / "scripts/verify-linux-abi.py"
)
linux_abi = importlib.util.module_from_spec(ABI_SPEC)
ABI_SPEC.loader.exec_module(linux_abi)

PRESERVED = [
    ".github/workflows/deploy-pages.yml",
    ".github/workflows/npm-publish.yml",
    ".github/workflows/release-binary.yml",
    "Dockerfile",  # 공식 배포 채널이 아니라 기존 WASM 빌드 환경
]

WITHDRAWN = [
    ".github/workflows/action-selftest.yml",
    ".github/workflows/docker-publish.yml",
    ".github/workflows/node-binding.yml",
    ".github/workflows/python-binding.yml",
    ".github/workflows/release-installers.yml",
    ".github/workflows/release-packages.yml",
    "Dockerfile.cli",
    "action.yml",
    "bindings/node",
    "bindings/python",
    "contrib/install",
    "contrib/packaging",
    "server.json",
    "tools/set_package_version.py",
    "tools/update_channel_manifests.py",
]

FORBIDDEN_WORKFLOW_MARKERS = [
    "docker/build-push-action",
    "docker/login-action",
    "ghcr.io",
    "maturin publish",
    "pypi",
    "bindings/node",
    "bindings/python",
    "cargo deb",
    "cargo generate-rpm",
    "cargo binstall",
    "wix",
    "contrib/install",
    "contrib/packaging",
    "server.json",
]


def tracked_path(path: str) -> bool:
    """작업자의 무시된 로컬 산출물은 배포 표면 재도입으로 보지 않는다."""
    result = subprocess.run(
        ["git", "ls-files", "--", path],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return bool(result.stdout.strip())


def release_binary_matrix() -> dict[str, dict[str, str]]:
    """Release Binary의 include matrix를 사람이 검토할 수 있는 계약으로 읽는다."""
    workflow = RELEASE_BINARY_WORKFLOW.read_text(encoding="utf-8")
    block = workflow.split("      matrix:\n", maxsplit=1)[1].split(
        "\n    steps:", maxsplit=1
    )[0]
    entries: dict[str, dict[str, str]] = {}
    current: dict[str, str] | None = None
    for raw_line in block.splitlines():
        line = raw_line.strip()
        if line.startswith("- target:"):
            target = line.split(":", maxsplit=1)[1].strip(" '\"")
            current = {"target": target}
            if target in entries:
                raise AssertionError(f"release target 중복: {target}")
            entries[target] = current
            continue
        if current is None or ":" not in line or line.startswith("#"):
            continue
        key, value = line.split(":", maxsplit=1)
        if key in {"runner", "archive", "archive_suffix", "binary_name"}:
            current[key] = value.strip(" '\"")
    return entries


class ReleaseChannelPolicyWorkflowTests(unittest.TestCase):
    def test_full_renderer_budget_keeps_all_capture_commands(self):
        workflow = (REPO_ROOT / ".github/workflows/full-renderer-sweep.yml").read_text()
        self.assertIn("timeout-minutes: 90", workflow)
        self.assertIn("--profiles screen,print,high-quality,fast-preview --include-pdf", workflow)
        self.assertIn("--canvaskit-surface webgpu --skip-native", workflow)
        self.assertIn("--canvaskit-surface software --skip-native", workflow)

    def test_linux_release_runtime_is_pinned_to_verified_lts(self):
        entries = release_binary_matrix()
        self.assertEqual(entries["x86_64-unknown-linux-gnu"]["runner"], "ubuntu-24.04")
        self.assertEqual(entries["aarch64-unknown-linux-gnu"]["runner"], "ubuntu-24.04-arm")

    def test_runner_switch_keeps_native_build_cache_separate(self):
        workflow = RELEASE_BINARY_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("key: ${{ matrix.target }}-${{ matrix.runner }}-cargo-", workflow)
        restore = workflow.split("restore-keys:", 1)[1].split("- name:", 1)[0]
        self.assertIn("${{ matrix.target }}-${{ matrix.runner }}-cargo-", restore)

    def test_packaged_cli_has_png_backend_on_every_release_target(self):
        workflow = RELEASE_BINARY_WORKFLOW.read_text(encoding="utf-8")
        build = workflow.split("      - name: Build release binary\n", 1)[1].split(
            "\n      - name:", 1
        )[0]
        self.assertIn("--locked", build)
        self.assertIn("--features native-skia", build)
        self.assertIn("--target ${{ matrix.target }}", build)
        self.assertNotIn("if:", build)
        smoke = workflow.split("      - name: Verify packaged PNG export\n", 1)[1].split(
            "\n      - name:", 1
        )[0]
        self.assertNotIn("if:", smoke, "PNG export must be verified on all five targets")
        self.assertIn("scripts/verify_release_binary.py", smoke)
        self.assertIn('--archive "${ARCHIVE_NAME}"', smoke)
        self.assertIn("--sample samples/issue2527_empty_linesegs.hwpx", smoke)
        self.assertLess(workflow.index("Verify packaged PNG export"), workflow.index("Upload artifact"))

    def test_release_binary_matrix_includes_linux_aarch64(self):
        entries = release_binary_matrix()
        expected_targets = {
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc",
        }
        self.assertEqual(
            set(entries),
            expected_targets,
            "공식 CLI release matrix는 기존 네 target과 Linux AArch64를 포함해야 한다",
        )

        linux_arm = entries["aarch64-unknown-linux-gnu"]
        self.assertEqual(linux_arm.get("runner"), "ubuntu-24.04-arm")
        self.assertEqual(linux_arm.get("archive"), "tar.gz")
        self.assertEqual(linux_arm.get("archive_suffix"), "linux-aarch64")
        self.assertEqual(linux_arm.get("binary_name"), "rhwp")

        suffixes = [entry.get("archive_suffix") for entry in entries.values()]
        self.assertNotIn(None, suffixes, "archive suffix가 없는 release target이 있다")
        self.assertEqual(len(suffixes), len(set(suffixes)), "archive suffix가 중복됐다")

    def test_user_visible_versions_match_release_version(self):
        cargo = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        release_version = cargo["package"]["version"]

        def package_version(path: str) -> str:
            return json.loads((REPO_ROOT / path).read_text(encoding="utf-8"))["version"]

        visible_versions = {
            "rhwp-studio About": package_version("rhwp-studio/package.json"),
            "VS Code extension": package_version("rhwp-vscode/package.json"),
            "npm editor": package_version("npm/editor/package.json"),
            "Chrome and Edge extension": package_version("rhwp-chrome/manifest.json"),
            "Chrome package": package_version("rhwp-chrome/package.json"),
            "Firefox extension": package_version("rhwp-firefox/manifest.json"),
            "Firefox package": package_version("rhwp-firefox/package.json"),
            "Safari extension": package_version("rhwp-safari/src/manifest.json"),
        }
        self.assertRegex(release_version, r"^\d+\.\d+\.\d+$")
        self.assertEqual(
            set(visible_versions.values()),
            {release_version},
            f"사용자 표시 버전이 릴리스 버전과 다르다: {visible_versions}",
        )

        lock_versions = {
            path: json.loads((REPO_ROOT / path).read_text(encoding="utf-8"))["packages"][
                ""
            ]["version"]
            for path in (
                "rhwp-studio/package-lock.json",
                "rhwp-vscode/package-lock.json",
                "rhwp-chrome/package-lock.json",
                "rhwp-firefox/package-lock.json",
            )
        }
        self.assertEqual(
            set(lock_versions.values()),
            {release_version},
            f"root package-lock 버전이 릴리스 버전과 다르다: {lock_versions}",
        )

        cargo_lock = tomllib.loads(
            (REPO_ROOT / "Cargo.lock").read_text(encoding="utf-8")
        )
        rhwp_packages = [
            package for package in cargo_lock["package"] if package["name"] == "rhwp"
        ]
        self.assertEqual(len(rhwp_packages), 1)
        self.assertEqual(rhwp_packages[0]["version"], release_version)

        release_document_markers = {
            "README.md": f"**v{release_version} — v1.0 조판 엔진 체계화**",
            "README_EN.md": f"**v{release_version} — systematizing the v1.0 typesetting engine**",
            "THIRD_PARTY_LICENSES.md": f"`rhwp` v{release_version}",
            "rhwp-vscode/CHANGELOG.md": f"## {release_version} — ",
        }
        stale_release_documents = [
            path
            for path, marker in release_document_markers.items()
            if marker not in (REPO_ROOT / path).read_text(encoding="utf-8")
        ]
        self.assertEqual(
            stale_release_documents,
            [],
            f"사용자 안내 버전이 릴리스와 다르다: {stale_release_documents}",
        )

        display_wiring = {
            "rhwp-studio/src/ui/about-dialog.ts": "Version ${__APP_VERSION__}",
            "rhwp-studio/vite.config.ts": "__APP_VERSION__: JSON.stringify(pkg.version)",
            "rhwp-chrome/vite.config.ts": "__APP_VERSION__: JSON.stringify(studioPkg.version)",
            "rhwp-chrome/options.js": "chromeApi.runtime.getManifest().version",
            "rhwp-firefox/vite.config.ts": "__APP_VERSION__: JSON.stringify(studioPkg.version)",
            "rhwp-firefox/options.js": "browser.runtime.getManifest().version",
        }
        missing = [
            path
            for path, marker in display_wiring.items()
            if marker not in (REPO_ROOT / path).read_text(encoding="utf-8")
        ]
        self.assertEqual(missing, [], f"사용자 표시 버전의 단일 출처 배선이 끊겼다: {missing}")

    def test_v082_distribution_channels_remain(self):
        missing = [path for path in PRESERVED if not (REPO_ROOT / path).exists()]
        self.assertEqual(missing, [], f"v0.8.2 공식 배포 자산이 사라졌다: {missing}")

    def test_withdrawn_distribution_surfaces_do_not_return(self):
        present = [path for path in WITHDRAWN if tracked_path(path)]
        self.assertEqual(
            present,
            [],
            "#4655에서 철회한 배포·바인딩 표면이 다시 추가됐다. 신규 공식 채널은 "
            f"메인테이너의 명시적 채택과 안전 검증이 먼저다: {present}",
        )

    def test_workflows_do_not_publish_withdrawn_channels(self):
        workflow_text = "\n".join(
            path.read_text(encoding="utf-8").lower()
            for path in sorted((REPO_ROOT / ".github/workflows").glob("*.yml"))
        )
        found = [marker for marker in FORBIDDEN_WORKFLOW_MARKERS if marker in workflow_text]
        self.assertEqual(found, [], f"철회한 채널의 게시·패키징 명령이 workflow에 남았다: {found}")

    def test_npm_workflow_is_limited_to_v082_packages_and_extensions(self):
        workflow = (REPO_ROOT / ".github/workflows/npm-publish.yml").read_text(
            encoding="utf-8"
        )
        for expected in [
            "Publish @rhwp/core",
            "working-directory: pkg",
            "Publish @rhwp/editor",
            "working-directory: npm/editor",
            "npx vsce publish",
            "npx ovsx publish",
        ]:
            self.assertIn(expected, workflow)
        self.assertEqual(workflow.count("npm publish --access public"), 2)


class LinuxRunnerBaselineWorkflowTests(unittest.TestCase):
    """[#9154] Product producers use a declared OS; metadata jobs are independent."""

    @staticmethod
    def workflow(name):
        return (REPO_ROOT / ".github/workflows" / name).read_text(encoding="utf-8")

    @staticmethod
    def job(source, name):
        match = re.search(r"(?ms)^  " + re.escape(name)
                          + r":\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)", source)
        if match is None:
            raise AssertionError("missing job: " + name)
        return match.group()

    def test_product_execution_jobs_use_supported_lts(self):
        producers = {
            "ci.yml": ["lint", "native-skia-tests", "frontend-unit-gates",
                       "frontend-package-gates", "wasm-build"],
            "build-nextest-archives.yml": ["build"],
            "run-nextest-archives.yml": ["run"],
            "codeql.yml": ["analyze"],
            "render-diff.yml": ["canvas-visual-diff"],
            "adapter-diff.yml": ["adapter-diff"],
            "proptest-roundtrip.yml": ["proptest-roundtrip"],
            "deploy-pages.yml": ["build"],
            "npm-publish.yml": ["build-wasm", "build-vsix"],
            "oracle-public-advisory.yml": ["advisory"],
            "layout-anomaly-advisory.yml": ["advisory"],
        }
        for name, jobs in producers.items():
            for job in jobs:
                with self.subTest(workflow=name, job=job):
                    block = self.job(self.workflow(name), job)
                    self.assertRegex(block, r"(?m)^    runs-on: ubuntu-24\.04$")
                    self.assertNotIn("runs-on: ubuntu-latest", block)

    def test_manual_os_choice_is_closed_and_schedule_uses_baseline(self):
        selector = ("${{ github.event_name == 'workflow_dispatch' && "
                    "inputs.runner_image == 'ubuntu-26.04' && "
                    "'ubuntu-26.04' || 'ubuntu-24.04' }}")
        for name in ("fuzz-smoke.yml", "full-renderer-sweep.yml"):
            with self.subTest(workflow=name):
                source = self.workflow(name)
                choice = source.split("      runner_image:\n", 1)[1].split(
                    "      max_total_time:" if name == "fuzz-smoke.yml" else "      corpus:", 1
                )[0]
                self.assertIn("default: ubuntu-24.04", choice)
                self.assertIn("type: choice", choice)
                self.assertEqual(re.findall(r"(?m)^          - (\S+)$", choice),
                                 ["ubuntu-24.04", "ubuntu-26.04"])
                self.assertIn("runs-on: " + selector, source)
                self.assertIn("RHWP_RUNNER_IMAGE: " + selector, source)
                self.assertLess(source.index("Validate runner selection"),
                                source.index("actions/checkout@"))
        fuzz = self.workflow("fuzz-smoke.yml")
        self.assertIn('cron: "41 2 * * *"', fuzz)
        self.assertIn("-rss_limit_mb=2048", fuzz)
        self.assertIn("-timeout=30", fuzz)
        self.assertIn('default: "60"', fuzz)

    def test_sanitizer_build_has_its_own_stage_without_changing_fuzzer_limits(self):
        source = self.workflow("fuzz-smoke.yml")
        header = source.split("  fuzz-smoke:\n", 1)[1].split("    strategy:\n", 1)[0]
        self.assertIn("timeout-minutes: 60", header)
        self.assertLess(source.index("      - name: Build sanitizer target "),
                        source.index("      - name: Smoke-run "))
        build = source.split("      - name: Build sanitizer target ", 1)[1].split(
            "      - name: Smoke-run ", 1)[0]
        smoke = source.split("      - name: Smoke-run ", 1)[1].split(
            "      - name: Upload crash artifacts", 1)[0]
        self.assertIn('/usr/bin/time -v cargo +nightly fuzz build --codegen-units 16 "${TARGET}"', build)
        self.assertIn('free -m', build)
        self.assertIn('nproc', build)
        self.assertNotIn("continue-on-error:", build)
        self.assertNotIn("if: always()", build + smoke)
        self.assertIn('cargo +nightly fuzz run --codegen-units 16 "${TARGET}" --', smoke)
        self.assertIn('-max_total_time="${MAX_TOTAL_TIME}"', smoke)
        self.assertIn("-rss_limit_mb=2048", smoke)
        self.assertIn("-timeout=30", smoke)
        self.assertNotIn("timeout-minutes:", smoke)
        for weakening_flag in ("--dev", "--release", "--no-default-features",
                               "--no-trace-compares", "--no-cfg-fuzzing",
                               "--sanitizer none"):
            self.assertNotIn(weakening_flag, build + smoke)

    def test_actual_runner_validation_rejects_unknown_mismatch_and_shell_input(self):
        for name in ("fuzz-smoke.yml", "full-renderer-sweep.yml"):
            source = self.workflow(name)
            block = source.split("      - name: Validate runner selection\n", 1)[1].split(
                "\n      - ", 1
            )[0]
            raw = block.split("        run: |\n", 1)[1]
            script = "\n".join(line[10:] for line in raw.splitlines())
            with tempfile.TemporaryDirectory() as directory:
                sentinel = Path(directory) / "must-not-exist"
                cases = [
                    ("ubuntu-24.04", "ubuntu-24.04", True),
                    ("ubuntu-26.04", "ubuntu-26.04", True),
                    ("ubuntu-latest", "ubuntu-24.04", False),
                    ("ubuntu-26.04", "ubuntu-24.04", False),
                    (f"ubuntu-24.04; touch {sentinel}", "ubuntu-24.04", False),
                ]
                for requested, actual, passed in cases:
                    with self.subTest(workflow=name, requested=requested):
                        result = subprocess.run(
                            ["bash", "-c", script], capture_output=True, text=True,
                            env={**os.environ, "REQUESTED_RUNNER_IMAGE": requested,
                                 "RHWP_RUNNER_IMAGE": actual}, check=False,
                        )
                        self.assertEqual(result.returncode == 0, passed)
                        self.assertFalse(sentinel.exists())

    def test_native_package_abi_gate_precedes_loading_and_packaging(self):
        source = self.workflow("native-host-package.yml")
        linux = source.split("          - platform: linux-x86_64\n", 1)[1].split(
            "          - platform:", 1
        )[0]
        self.assertIn("runner: ubuntu-24.04", linux)
        gate = source.split("      - name: Verify Linux ABI baseline\n", 1)[1].split(
            "      - name:", 1
        )[0]
        self.assertIn("if: runner.os == 'Linux'", gate)
        self.assertIn("scripts/verify-linux-abi.py", gate)
        self.assertIn("--max-glibc 2.39", gate)
        self.assertLess(source.index("Build native Skia C ABI"), source.index("Verify Linux ABI baseline"))
        self.assertLess(source.index("Verify Linux ABI baseline"), source.index("Probe actual library capabilities"))
        self.assertLess(source.index("Verify Linux ABI baseline"), source.index("Package explicit library"))
        self.assertIn("Exercise packaged headless host", source)
        self.assertIn("rhwp-native-host-${{ matrix.platform }}", source)

    def test_native_cache_keys_do_not_restore_foreign_os_baselines(self):
        families = {
            "ci.yml": "frontend-wasm-cargo-",
            "render-diff.yml": "render-diff-cargo-",
            "deploy-pages.yml": "wasm-",
            "npm-publish.yml": "wasm-",
        }
        for name, family in families.items():
            with self.subTest(workflow=name):
                source = self.workflow(name)
                self.assertIn("${{ runner.os }}-ubuntu-24.04-" + family, source)
                self.assertNotIn("${{ runner.os }}-" + family, source)
        sweep = self.workflow("full-renderer-sweep.yml")
        self.assertIn("${{ runner.os }}-${{ env.RHWP_RUNNER_IMAGE }}-full-renderer-sweep-cargo-", sweep)
        self.assertNotIn("${{ runner.os }}-cargo-", sweep)
        fuzz = self.workflow("fuzz-smoke.yml")
        self.assertIn("shared-key: fuzz-smoke-${{ matrix.target }}-${{ env.RHWP_RUNNER_IMAGE }}", fuzz)


class LinuxAbiRequirementTests(unittest.TestCase):
    """Version-needs fixtures are independent ABI contracts, not renderer golden data."""

    NEEDS = ("Version needs section '.gnu.version_r' contains 1 entry:\n"
             "  File: libc.so.6 Cnt: 2\n"
             "  0x0010: Name: GLIBC_2.34 Flags: none Version: 2\n"
             "  0x0020: Name: GLIBC_2.2.5 Flags: none Version: 3\n")

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.artifact = Path(self.directory.name) / "fixture.so"
        header = bytearray(64)
        header[:6] = b"\x7fELF\x02\x01"
        struct.pack_into("<HH", header, 16, 3, 62)
        self.artifact.write_bytes(header)

    def verify_output(self, text, *, maximum="2.39", stderr="", code=0):
        process = subprocess.CompletedProcess([], code, stdout=text, stderr=stderr)
        with patch.object(linux_abi.subprocess, "run", return_value=process) as call:
            result = linux_abi.verify(self.artifact, maximum)
            self.assertEqual(call.call_args.kwargs["env"]["LC_ALL"], "C")
            self.assertNotIn("shell", call.call_args.kwargs)
            return result

    def test_supported_glibc_requirement_and_numeric_order(self):
        report = self.verify_output(self.NEEDS)
        self.assertTrue(report["ok"])
        self.assertEqual(report["maxRequiredGlibc"], "2.34")
        self.assertEqual(report["requiredGlibcVersions"], ["2.2.5", "2.34"])
        self.assertEqual(len(report["sha256"]), 64)
        self.assertLess(linux_abi.version_tuple("2.9"), linux_abi.version_tuple("2.39"))

    def test_too_new_glibc_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "2.39 exceeded: 2.43"):
            self.verify_output(self.NEEDS.replace("GLIBC_2.34", "GLIBC_2.43"))

    def test_exported_versions_and_cxx_abi_are_not_glibc_requirements(self):
        text = ("Version definition section '.gnu.version_d' contains 1 entry:\n"
                "  0x0000: Name: GLIBC_99.0\n" + self.NEEDS
                + "  0x0030: Name: GLIBCXX_3.4.30 Flags: none Version: 4\n")
        self.assertEqual(self.verify_output(text)["maxRequiredGlibc"], "2.34")

    def test_unrecognized_abi_contract_does_not_become_success(self):
        with self.assertRaisesRegex(ValueError, "dot-separated numbers"):
            self.verify_output(self.NEEDS.replace("GLIBC_2.34", "GLIBC_PRIVATE"))
        with self.assertRaisesRegex(ValueError, "recognizable"):
            self.verify_output("unexpected readelf output")
        with self.assertRaisesRegex(ValueError, "dot-separated numbers"):
            self.verify_output(self.NEEDS, maximum="latest")

    def test_tool_failure_and_warning_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "readelf failed"):
            self.verify_output("", code=1, stderr="invalid ELF")
        with self.assertRaisesRegex(ValueError, "diagnostic"):
            self.verify_output(self.NEEDS, stderr="corrupt section")

    def test_invalid_elf_never_calls_tool(self):
        for content in (b"not ELF", bytes(64)):
            self.artifact.write_bytes(content)
            with patch.object(linux_abi.subprocess, "run") as call:
                with self.assertRaisesRegex(ValueError, "supported ELF"):
                    linux_abi.verify(self.artifact, "2.39")
                call.assert_not_called()

    def test_unversioned_elf_has_no_declared_glibc_requirement(self):
        report = self.verify_output("No version information found in this file.\n")
        self.assertIsNone(report["maxRequiredGlibc"])


if __name__ == "__main__":
    unittest.main()
