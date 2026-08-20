#!/usr/bin/env python3
"""Install a git pre-commit hook that runs precommit_skill_gate.py.

Writes `<git-dir>/hooks/pre-commit`. Linked worktrees get the hook in that
worktree's git-dir, not the main repo's common hooks.

Does not overwrite an existing pre-commit unless it already contains the
marker `RHWP_SKILL_GATE`. Re-install is idempotent.

    python tools/skill_router/install_git_hook.py

Exit 0 installed or already current, 1 refuse/fail. stdlib only.
"""

from __future__ import annotations

import os
import stat
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
GATE_SCRIPT = HERE / "precommit_skill_gate.py"
MARKER = "RHWP_SKILL_GATE"


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


def _git_one(args: list[str]) -> str:
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
    return (proc.stdout or "").strip()


def _resolve_git_path(raw: str) -> Path:
    path = Path(raw)
    if not path.is_absolute():
        path = (REPO / path).resolve()
    else:
        path = path.resolve()
    return path


def _posix(path: Path) -> str:
    return path.resolve().as_posix()


def hook_body(python: Path, script: Path) -> str:
    py = _posix(python).replace('"', '\\"')
    gate = _posix(script).replace('"', '\\"')
    return (
        "#!/bin/sh\n"
        f"# {MARKER}\n"
        "# Installed by tools/skill_router/install_git_hook.py.\n"
        "export PYTHONUTF8=1\n"
        "export PYTHONIOENCODING=utf-8\n"
        f'exec "{py}" "{gate}" "$@"\n'
    )


def _chmod_executable(path: Path) -> None:
    mode = path.stat().st_mode
    path.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def install() -> int:
    if not GATE_SCRIPT.is_file():
        _say(f"install-git-hook: FAIL: missing {GATE_SCRIPT}")
        return 1
    git_dir = _resolve_git_path(_git_one(["rev-parse", "--git-dir"]))
    common_dir = _resolve_git_path(_git_one(["rev-parse", "--git-common-dir"]))
    worktree = git_dir != common_dir
    hooks_dir = git_dir / "hooks"
    hook_path = hooks_dir / "pre-commit"
    _say(f"install-git-hook: git-dir={_posix(git_dir)}")
    _say(f"install-git-hook: git-common-dir={_posix(common_dir)}")
    if worktree:
        _say("install-git-hook: linked worktree — writing this worktree's git-dir hooks")
        _say("install-git-hook: not writing the main repo common hooks")
    body = hook_body(Path(sys.executable), GATE_SCRIPT)
    if hook_path.is_file():
        existing = hook_path.read_text(encoding="utf-8", errors="replace")
        if MARKER not in existing:
            _say(
                f"install-git-hook: FAIL: {hook_path} exists without {MARKER}; "
                "refusing to overwrite"
            )
            return 1
        if existing == body:
            _say(f"install-git-hook: already current at {hook_path}")
            return 0
    hooks_dir.mkdir(parents=True, exist_ok=True)
    hook_path.write_text(body, encoding="utf-8", newline="\n")
    _chmod_executable(hook_path)
    _say(f"install-git-hook: wrote {hook_path}")
    return 0


def main() -> int:
    _configure_stdio()
    try:
        return install()
    except RuntimeError as exec_err:
        _say(f"install-git-hook: FAIL: {exec_err}")
        return 1
    except OSError as exc:
        _say(f"install-git-hook: FAIL: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
