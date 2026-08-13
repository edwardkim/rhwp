from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
COMPOSE_PATH = ROOT / "docker-compose.yml"
GUIDE_PATH = ROOT / "mydocs" / "manual" / "dev_environment_guide.md"


class DockerWasmComposeContractTests(unittest.TestCase):
    """#4089: Windows Docker WASM cache와 stale wasm-opt 방지 계약."""

    def setUp(self) -> None:
        self.compose = COMPOSE_PATH.read_text(encoding="utf-8")
        self.guide = GUIDE_PATH.read_text(encoding="utf-8")

    def test_wasm_target_is_a_named_volume_outside_the_host_mount(self) -> None:
        self.assertIn("CARGO_TARGET_DIR: /build-target", self.compose)
        self.assertIn("- wasm-target:/build-target", self.compose)
        self.assertIn("  wasm-target:\n", self.compose)
        self.assertNotIn("CARGO_TARGET_DIR: /app/target", self.compose)

    def test_stale_wasm_opt_is_removed_before_build(self) -> None:
        cleanup = self.compose.index("rm -f pkg/*-opt.wasm")
        build = self.compose.index("wasm-pack build --target web")
        self.assertLess(cleanup, build)

    def test_host_pkg_ownership_is_restored_even_after_a_failed_build(self) -> None:
        self.assertIn("user: \"0:0\"", self.compose)
        self.assertIn("HOST_UID: ${UID:-1000}", self.compose)
        self.assertIn("HOST_GID: ${GID:-1000}", self.compose)
        self.assertIn("trap 'exit 130' INT", self.compose)
        self.assertIn("trap 'exit 143' TERM", self.compose)
        self.assertIn("trap finish EXIT", self.compose)
        self.assertIn('chown -R "$${HOST_UID}:$${HOST_GID}" pkg', self.compose)

    def test_guide_makes_docker_primary_and_no_opt_diagnostic_only(self) -> None:
        docker = self.guide.index("docker compose --env-file .env.docker run --rm wasm")
        native = self.guide.index("wasm-pack build --target web --out-dir pkg --no-opt")
        self.assertLess(docker, native)
        self.assertIn("진단용", self.guide)
        self.assertIn("최적화된 배포 산출물을 대체하지 않는다", self.guide)


if __name__ == "__main__":
    unittest.main()
