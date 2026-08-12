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


class ReleaseChannelPolicyWorkflowTests(unittest.TestCase):
    def test_user_visible_versions_match_release_version(self):
        cargo = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        release_version = cargo["package"]["version"]

        def package_version(path: str) -> str:
            return json.loads((REPO_ROOT / path).read_text(encoding="utf-8"))["version"]

        visible_versions = {
            "rhwp-studio About": package_version("rhwp-studio/package.json"),
            "Chrome and Edge extension": package_version("rhwp-chrome/manifest.json"),
            "Firefox extension": package_version("rhwp-firefox/manifest.json"),
        }
        self.assertEqual(release_version, "0.8.4")
        self.assertEqual(
            set(visible_versions.values()),
            {release_version},
            f"사용자 표시 버전이 릴리스 버전과 다르다: {visible_versions}",
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
