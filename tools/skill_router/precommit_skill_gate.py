#!/usr/bin/env python3
"""Pre-commit gate for skill-path changes.

If staged or working-tree paths include `.claude/skills/`, `.agents/skills/`,
or `tools/skill_router/`, run `gate_new_skill.py` three times and
`python -m unittest tools.skill_router.test_route` once. Unrelated changes
exit 0 without blocking.

    python tools/skill_router/precommit_skill_gate.py

Exit 0 pass or skip, 1 fail. stdlib only.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
GATE_PY = HERE / "gate_new_skill.py"
GATE_REPEAT = 3
RUN_TIMEOUT_SEC = 300
ROUTE_UNITTEST_MODULE = "tools.skill_router.test_route"

TRIGGER_PREFIXES = (
    ".claude/skills/",
    ".agents/skills/",
    "tools/skill_router/",
)
TRIGGER_EXACT = tuple(prefix.rstrip("/") for prefix in TRIGGER_PREFIXES)


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


def _say(line: str) -> None:
    print(line, flush=True)


def _normalize_path(raw: str) -> str:
    text = raw.strip().replace("\\", "/").lstrip("./")
    if text.startswith("\""):
        text = text.strip("\"")
    return text


def _is_trigger(path: str) -> bool:
    if not path:
        return False
    if path in TRIGGER_EXACT:
        return True
    return any(path.startswith(prefix) for prefix in TRIGGER_PREFIXES)


def _git_lines(args: list[str]) -> list[str]:
    try:
        proc = subprocess.run(
            ["git", *args],
            cwd=str(REPO),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=30,
            env=_cli_env(),
            check=False,
        )
    except FileNotFoundError as exc:
        raise RuntimeError("git is not on PATH") from exc
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f"git {' '.join(args)} hung after 30s") from exc
    if proc.returncode != 0:
        err = (proc.stderr or "").strip()[:400]
        raise RuntimeError(f"git {' '.join(args)} exit {proc.returncode} stderr={err!r}")
    lines: list[str] = []
    for raw in (proc.stdout or "").splitlines():
        path = _normalize_path(raw)
        if path:
            lines.append(path)
    return lines


def changed_paths() -> list[str]:
    """Staged, unstaged, and untracked paths. `git diff` omits new files."""
    seen: set[str] = set()
    ordered: list[str] = []
    groups = (
        ["diff", "--cached", "--name-only"],
        ["diff", "--name-only"],
        ["ls-files", "--others", "--exclude-standard"],
    )
    for args in groups:
        for path in _git_lines(args):
            if path in seen:
                continue
            seen.add(path)
            ordered.append(path)
    return ordered


def trigger_paths(paths: list[str]) -> list[str]:
    return [path for path in paths if _is_trigger(path)]


def _run(cmd: list[str], label: str) -> int:
    _say(f"skill-gate: {label}")
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(REPO),
            timeout=RUN_TIMEOUT_SEC,
            env=_cli_env(),
            check=False,
        )
    except subprocess.TimeoutExpired:
        _say(f"skill-gate: FAIL: {label} hung after {RUN_TIMEOUT_SEC}s")
        return 1
    if proc.returncode != 0:
        _say(f"skill-gate: FAIL: {label} exit {proc.returncode}")
        return proc.returncode
    return 0


def run_gate() -> int:
    if not GATE_PY.is_file():
        _say(f"skill-gate: FAIL: missing {GATE_PY}")
        return 1
    python = sys.executable
    for run in range(1, GATE_REPEAT + 1):
        code = _run(
            [python, str(GATE_PY)],
            f"gate_new_skill.py [{run}/{GATE_REPEAT}]",
        )
        if code != 0:
            return 1
    code = _run(
        [python, "-m", "unittest", ROUTE_UNITTEST_MODULE],
        f"unittest {ROUTE_UNITTEST_MODULE}",
    )
    if code != 0:
        return 1
    _say("skill-gate: OK")
    return 0


def main() -> int:
    _configure_stdio()
    try:
        paths = changed_paths()
    except RuntimeError as exc:
        _say(f"skill-gate: FAIL: {exc}")
        return 1
    hits = trigger_paths(paths)
    if not hits:
        _say("skill-gate: skip (no skill-path changes)")
        return 0
    _say("skill-gate: triggered by:")
    for path in hits:
        _say(f"  {path}")
    return run_gate()


if __name__ == "__main__":
    sys.exit(main())
