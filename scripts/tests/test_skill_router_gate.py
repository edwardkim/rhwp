"""Repeat-run the skill-router gate and route contract tests.

Repo root is the parent of scripts/. stdlib only.

    python -m unittest scripts.tests.test_skill_router_gate
    python scripts/tests/test_skill_router_gate.py
"""

from __future__ import annotations

import os
import subprocess
import sys
import unittest
from pathlib import Path

# scripts/tests/this.py → parent of scripts/ is the repo root
REPO_ROOT = Path(__file__).resolve().parents[2]
GATE_PY = REPO_ROOT / "tools" / "skill_router" / "gate_new_skill.py"
TEST_ROUTE_PY = REPO_ROOT / "tools" / "skill_router" / "test_route.py"
TEST_CATALOG_PY = REPO_ROOT / "tools" / "skill_router" / "test_catalog_routes.py"
TEST_AUTHOR_PY = REPO_ROOT / "tools" / "skill_router" / "test_author_skill.py"
REPEAT = 3
RUN_TIMEOUT_SEC = 120
ROUTE_UNITTEST_MODULE = "tools.skill_router.test_route"
ROUTE_UNITTEST_FILE = "tools/skill_router/test_route.py"
CATALOG_UNITTEST_MODULE = "tools.skill_router.test_catalog_routes"
CATALOG_UNITTEST_FILE = "tools/skill_router/test_catalog_routes.py"
AUTHOR_UNITTEST_MODULE = "tools.skill_router.test_author_skill"
AUTHOR_UNITTEST_FILE = "tools/skill_router/test_author_skill.py"


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def _run(cmd: list[str], label: str) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            cmd,
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=RUN_TIMEOUT_SEC,
            env=_cli_env(),
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise AssertionError(
            f"{label} hung after {RUN_TIMEOUT_SEC}s: {cmd!r}"
        ) from exc


def _assert_exit_zero(
    test: unittest.TestCase, proc: subprocess.CompletedProcess[str], label: str
) -> None:
    if proc.returncode == 0:
        return
    err = (proc.stderr or "").strip()[:800]
    out = (proc.stdout or "").strip()[:400]
    test.fail(f"{label} exit {proc.returncode}. stderr={err!r} stdout={out!r}")


def _unittest_cmd(module: str) -> list[str]:
    """Prefer dotted module form; fall back to the file path."""
    return [sys.executable, "-m", "unittest", module]


class SkillRouterGateTests(unittest.TestCase):
    def test_gate_new_skill_exits_zero_three_times(self) -> None:
        self.assertTrue(
            GATE_PY.is_file(),
            f"missing tools/skill_router/gate_new_skill.py at {GATE_PY}",
        )
        cmd = [sys.executable, str(GATE_PY)]
        for run in range(REPEAT):
            with self.subTest(run=run + 1):
                label = f"gate_new_skill.py run {run + 1}/{REPEAT}"
                proc = _run(cmd, label)
                _assert_exit_zero(self, proc, label)

    def test_route_unittest_exits_zero_three_times(self) -> None:
        self.assertTrue(
            TEST_ROUTE_PY.is_file(),
            f"missing tools/skill_router/test_route.py at {TEST_ROUTE_PY}",
        )
        cmd = _unittest_cmd(ROUTE_UNITTEST_MODULE)
        file_cmd = [sys.executable, "-m", "unittest", ROUTE_UNITTEST_FILE]
        for run in range(REPEAT):
            with self.subTest(run=run + 1):
                label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                proc = _run(cmd, label)
                if proc.returncode != 0 and cmd != file_cmd:
                    # Dotted module may fail without a package; try the file path.
                    cmd = file_cmd
                    label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                    proc = _run(cmd, label)
                _assert_exit_zero(self, proc, label)

    def test_catalog_routes_unittest_exits_zero_three_times(self) -> None:
        self.assertTrue(
            TEST_CATALOG_PY.is_file(),
            f"missing tools/skill_router/test_catalog_routes.py at {TEST_CATALOG_PY}",
        )
        cmd = _unittest_cmd(CATALOG_UNITTEST_MODULE)
        file_cmd = [sys.executable, "-m", "unittest", CATALOG_UNITTEST_FILE]
        for run in range(REPEAT):
            with self.subTest(run=run + 1):
                label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                proc = _run(cmd, label)
                if proc.returncode != 0 and cmd != file_cmd:
                    cmd = file_cmd
                    label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                    proc = _run(cmd, label)
                _assert_exit_zero(self, proc, label)

    def test_author_skill_unittest_exits_zero_three_times(self) -> None:
        self.assertTrue(
            TEST_AUTHOR_PY.is_file(),
            f"missing tools/skill_router/test_author_skill.py at {TEST_AUTHOR_PY}",
        )
        cmd = _unittest_cmd(AUTHOR_UNITTEST_MODULE)
        file_cmd = [sys.executable, "-m", "unittest", AUTHOR_UNITTEST_FILE]
        for run in range(REPEAT):
            with self.subTest(run=run + 1):
                label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                proc = _run(cmd, label)
                if proc.returncode != 0 and cmd != file_cmd:
                    cmd = file_cmd
                    label = f"unittest {cmd[-1]} run {run + 1}/{REPEAT}"
                    proc = _run(cmd, label)
                _assert_exit_zero(self, proc, label)


if __name__ == "__main__":
    unittest.main()
