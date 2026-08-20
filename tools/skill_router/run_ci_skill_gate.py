#!/usr/bin/env python3
"""Local mimic of .github/workflows/skill-router-gate.yml.

Same seven job steps, PYTHONUTF8=1, fail-fast on the first non-zero.
stdlib only.

    python tools/skill_router/run_ci_skill_gate.py

Exit 0 pass, 1 fail (CI-equivalent).
"""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
GATE_PY = "tools/skill_router/gate_new_skill.py"
GATE_REPEATS = 3
ROUTE_UNITTEST = "tools.skill_router.test_route"
GATE_UNITTEST = "scripts.tests.test_skill_router_gate"
CATALOG_UNITTEST = "tools.skill_router.test_catalog_routes"
AUTHOR_UNITTEST = "tools.skill_router.test_author_skill"
CATALOG_SYNC_PY = "tools/skill_router/check_catalog_sync.py"
PROBES_UNITTEST = "tools.skill_router.test_probes"
PRECOMMIT_UNITTEST = "tools.skill_router.test_precommit"


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is None:
            continue
        try:
            reconfigure(encoding="utf-8", errors="replace")
        except (OSError, ValueError):
            pass


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def _run(cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=str(REPO), env=_cli_env(), check=False)
    return int(proc.returncode)


def main() -> int:
    _configure_stdio()
    os.environ["PYTHONUTF8"] = "1"
    os.environ["PYTHONIOENCODING"] = "utf-8"
    started = time.monotonic()

    py = sys.executable
    steps: list[tuple[str, list[str]]] = []
    for i in range(1, GATE_REPEATS + 1):
        steps.append(
            (
                f"gate_new_skill.py run {i}/{GATE_REPEATS}",
                [py, GATE_PY],
            )
        )
    steps.append(
        (
            f"python -m unittest {ROUTE_UNITTEST}",
            [py, "-m", "unittest", ROUTE_UNITTEST],
        )
    )
    steps.append(
        (
            f"python -m unittest {GATE_UNITTEST}",
            [py, "-m", "unittest", GATE_UNITTEST],
        )
    )
    steps.append(
        (
            f"python -m unittest {CATALOG_UNITTEST}",
            [py, "-m", "unittest", CATALOG_UNITTEST],
        )
    )
    steps.append(
        (
            f"python -m unittest {AUTHOR_UNITTEST}",
            [py, "-m", "unittest", AUTHOR_UNITTEST],
        )
    )
    steps.append(
        (
            f"python {CATALOG_SYNC_PY}",
            [py, CATALOG_SYNC_PY],
        )
    )
    steps.append(
        (
            f"python -m unittest {PROBES_UNITTEST}",
            [py, "-m", "unittest", PROBES_UNITTEST],
        )
    )
    steps.append(
        (
            f"python -m unittest {PRECOMMIT_UNITTEST}",
            [py, "-m", "unittest", PRECOMMIT_UNITTEST],
        )
    )

    passed = 0
    failed_label = ""
    failed_code = 0
    for label, cmd in steps:
        print(label, flush=True)
        code = _run(cmd)
        if code != 0:
            failed_label = label
            failed_code = code
            break
        passed += 1

    elapsed = time.monotonic() - started
    if failed_label:
        print(
            f"FAIL: {failed_label} exit {failed_code} "
            f"({passed}/{len(steps)} steps ok, {elapsed:.1f}s)",
            flush=True,
        )
        return 1
    print(
        f"PASS: gate {GATE_REPEATS}/{GATE_REPEATS}, "
        f"unittest {ROUTE_UNITTEST}, unittest {GATE_UNITTEST}, "
        f"unittest {CATALOG_UNITTEST}, unittest {AUTHOR_UNITTEST}, "
        f"{CATALOG_SYNC_PY}, unittest {PROBES_UNITTEST}, "
        f"unittest {PRECOMMIT_UNITTEST} "
        f"({elapsed:.1f}s)",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
