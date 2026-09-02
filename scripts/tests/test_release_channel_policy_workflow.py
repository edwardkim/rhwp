"""[#4655] 공식 배포 표면을 v0.8.2 채널로 고정한다.

새 배포 채널은 파일 하나만 추가돼도 태그나 릴리스 이벤트에서 실제 게시를
시도할 수 있다. 따라서 철회한 채널의 실행 자산이 다시 생기지 않는지와 기존
채널의 핵심 workflow가 남아 있는지를 함께 검사한다.
"""

from __future__ import annotations

import json
import subprocess
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RELEASE_BINARY_WORKFLOW = REPO_ROOT / ".github/workflows/release-binary.yml"

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
        self.assertEqual(release_version, "0.8.6")
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
            "README.md": "**v0.8.6 — v1.0 조판 엔진 체계화**",
            "README_EN.md": "**v0.8.6 — systematizing the v1.0 typesetting engine**",
            "THIRD_PARTY_LICENSES.md": "`rhwp` v0.8.6",
            "rhwp-vscode/CHANGELOG.md": "## 0.8.6 — 2026-09-02",
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


if __name__ == "__main__":
    unittest.main()
