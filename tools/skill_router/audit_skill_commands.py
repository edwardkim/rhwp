#!/usr/bin/env python3
"""Audit `rhwp <cmd>` tokens in `.claude/skills/*/SKILL.md` against live rhwp.

Extracts `rhwp [a-z][a-z0-9_-]*` (and group subcommands such as
`edit fill-fields`) the same way tests/skills_contract.rs and
tools/skill_router/gate_new_skill.py do, then requires every token to
appear in `rhwp capabilities` ∪ `rhwp --help` ∪ {help}.

Re-scans every skill three times per invocation (running once == 3
passes). Missing rhwp (absent file or FileNotFoundError) skips the live
layer and exits 0.

    python tools/skill_router/audit_skill_commands.py
    python tools/skill_router/audit_skill_commands.py --json

Exit 0 pass (or skipped live layer), 1 fail, 2 usage. stdlib only.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
SKILLS_DIR = REPO / ".claude" / "skills"

SCHEMA_VERSION = "1.0"
SCAN_TIMES = 3
RHWP_TIMEOUT_SEC = 20
MIN_KNOWN_COMMANDS = 20
MIN_EDIT_SUBCOMMANDS = 4
USAGE = "usage: python tools/skill_router/audit_skill_commands.py [--json]"


class AuditFail(Exception):
    """A contract check failed. exit 1."""


class UsageError(Exception):
    """Bad argv. exit 2."""


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is None:
            continue
        try:
            reconfigure(encoding="utf-8", errors="replace")
        except (OSError, ValueError):
            pass


def _human_stream(json_mode: bool):
    return sys.stderr if json_mode else sys.stdout


def _say(json_mode: bool, line: str) -> None:
    print(line, file=_human_stream(json_mode), flush=True)


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def _is_token_char(ch: str) -> bool:
    # Job regex: [a-z][a-z0-9_-]* — underscore allowed; hyphen matches CI.
    return ch.isascii() and (ch.islower() or ch.isdigit() or ch in "-_")


def _is_token(s: str) -> bool:
    return bool(s) and not s.startswith("-") and all(_is_token_char(ch) for ch in s)


def referenced_commands(body: str) -> list[tuple[str, str | None]]:
    """Extract `rhwp <token> [subtoken]` the same way skills_contract.rs does.

    After each `rhwp `, take `[a-z0-9_-]+` that starts with a-z. Placeholders
    such as `rhwp <명령>` and Korean particles (`rhwp 를`) do not count.
    """
    pat = "rhwp "
    refs: list[tuple[str, str | None]] = []
    idx = 0
    while True:
        pos = body.find(pat, idx)
        if pos < 0:
            break
        start = pos + len(pat)
        tok_chars: list[str] = []
        for ch in body[start:]:
            if not _is_token_char(ch):
                break
            tok_chars.append(ch)
        tok = "".join(tok_chars)
        if tok and tok[0].islower():
            after = start + len(tok)
            sub: str | None = None
            if body[after : after + 1] == " ":
                sub_chars: list[str] = []
                for ch in body[after + 1 :]:
                    if not _is_token_char(ch):
                        break
                    sub_chars.append(ch)
                candidate = "".join(sub_chars)
                if _is_token(candidate):
                    sub = candidate
            refs.append((tok, sub))
        idx = start
    return refs


def format_ref(tok: str, sub: str | None) -> str:
    return f"{tok} {sub}" if sub else tok


def unique_refs(refs: list[tuple[str, str | None]]) -> list[tuple[str, str | None]]:
    seen: set[tuple[str, str | None]] = set()
    out: list[tuple[str, str | None]] = []
    for item in refs:
        if item in seen:
            continue
        seen.add(item)
        out.append(item)
    return out


def list_skill_dirs() -> list[Path]:
    if not SKILLS_DIR.is_dir():
        raise AuditFail(f"missing skills directory: {SKILLS_DIR}")
    dirs = sorted(
        path for path in SKILLS_DIR.iterdir() if path.is_dir() and not path.name.startswith(".")
    )
    if len(dirs) < 1:
        raise AuditFail("no skill folders under .claude/skills/")
    return dirs


def read_skill(dir_path: Path) -> str:
    md = dir_path / "SKILL.md"
    if not md.is_file():
        raise AuditFail(f"{dir_path.name}/SKILL.md missing")
    try:
        return md.read_text(encoding="utf-8")
    except OSError as exc:
        raise AuditFail(f"{dir_path.name}/SKILL.md read failed: {exc}") from exc
    except UnicodeDecodeError as exc:
        raise AuditFail(f"{dir_path.name}/SKILL.md is not utf-8: {exc}") from exc


def find_rhwp_binary() -> Path | None:
    """Prefer target/release/rhwp.exe, then PATH rhwp. None if neither exists."""
    release_exe = REPO / "target" / "release" / "rhwp.exe"
    if release_exe.is_file():
        return release_exe
    release = REPO / "target" / "release" / "rhwp"
    if release.is_file():
        return release
    which = shutil.which("rhwp")
    if which:
        return Path(which)
    return None


def _run_rhwp(binary: Path, args: list[str]) -> str:
    argv = [str(binary), *args]
    try:
        proc = subprocess.run(
            argv,
            cwd=str(REPO),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=RHWP_TIMEOUT_SEC,
            env=_cli_env(),
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise AuditFail(f"rhwp {' '.join(args)} hung after {RHWP_TIMEOUT_SEC}s") from exc
    if proc.returncode != 0:
        err = (proc.stderr or "").strip()[:400]
        raise AuditFail(f"rhwp {' '.join(args)} exit {proc.returncode} stderr={err!r}")
    return proc.stdout or ""


def _help_head_token(tok: str) -> bool:
    return bool(tok) and all(_is_token_char(ch) for ch in tok)


def parse_known_commands(caps_stdout: str, help_stdout: str) -> set[str]:
    """capabilities names ∪ --help two-space heads ∪ {help}. Same as skills_contract.rs."""
    known: set[str] = set()
    try:
        caps = json.loads(caps_stdout)
    except json.JSONDecodeError as exc:
        preview = caps_stdout.strip()[:240].replace("\n", "\\n")
        raise AuditFail(f"rhwp capabilities is not JSON: {exc}: {preview!r}") from exc
    commands = caps.get("commands") if isinstance(caps, dict) else None
    if not isinstance(commands, list):
        raise AuditFail("rhwp capabilities missing commands array")
    for item in commands:
        if not isinstance(item, dict):
            continue
        name = item.get("name")
        if not isinstance(name, str) or not name:
            continue
        known.add(name)
        head = name.split()[0] if name.split() else ""
        if head:
            known.add(head)
    for line in help_stdout.splitlines():
        if not line.startswith("  "):
            continue
        rest = line[2:]
        parts = rest.split()
        if not parts:
            continue
        tok = parts[0]
        if _help_head_token(tok):
            known.add(tok)
    known.add("help")
    if len(known) < MIN_KNOWN_COMMANDS:
        raise AuditFail(
            f"real command set too small ({len(known)}) — self-description parse regression"
        )
    return known


def parse_group_subcommands(help_stdout: str) -> dict[str, set[str]]:
    """Harvest `rhwp <head> <sub>` names from --help, including `<a|b>` lists."""
    groups: dict[str, set[str]] = {}
    for line in help_stdout.splitlines():
        if not line.startswith("  "):
            continue
        parts = line[2:].split()
        if len(parts) < 2:
            continue
        head, sub = parts[0], parts[1]
        if not _is_token(head):
            continue
        if _is_token(sub):
            groups.setdefault(head, set()).add(sub)
            continue
        if sub.startswith("<") and "|" in sub:
            for alt in sub.strip("<>").split("|"):
                if _is_token(alt):
                    groups.setdefault(head, set()).add(alt)
    edit_subs = groups.get("edit")
    if edit_subs is None or len(edit_subs) < MIN_EDIT_SUBCOMMANDS:
        raise AuditFail(f"edit subcommand harvest regression: {groups!r}")
    return groups


def load_live_rhwp(json_mode: bool) -> dict[str, Any]:
    """Load known commands. Absent / vanished binary → skipped, not failed."""
    binary = find_rhwp_binary()
    if binary is None:
        _say(json_mode, "rhwp binary: skip (absent)")
        return {
            "present": False,
            "ok": True,
            "skipped": "no rhwp binary",
            "known": set(),
            "groups": {},
            "knownCount": 0,
        }
    _say(json_mode, f"rhwp binary: {binary}")
    try:
        caps_stdout = _run_rhwp(binary, ["capabilities"])
        help_stdout = _run_rhwp(binary, ["--help"])
    except FileNotFoundError:
        _say(json_mode, "rhwp binary: skip (absent)")
        return {
            "present": False,
            "ok": True,
            "skipped": "no rhwp binary",
            "known": set(),
            "groups": {},
            "knownCount": 0,
        }
    known = parse_known_commands(caps_stdout, help_stdout)
    groups = parse_group_subcommands(help_stdout)
    _say(
        json_mode,
        f"rhwp known: {len(known)} commands, {len(groups)} group heads",
    )
    return {
        "present": True,
        "ok": True,
        "binary": str(binary),
        "known": known,
        "groups": groups,
        "knownCount": len(known),
    }


def unknown_for_refs(
    refs: list[tuple[str, str | None]],
    known: set[str],
    groups: dict[str, set[str]],
) -> list[str]:
    """Return display tokens that are not in the live command set."""
    dead: list[str] = []
    seen: set[str] = set()
    for tok, sub in refs:
        if tok not in known:
            msg = f"rhwp {tok}"
            if msg not in seen:
                seen.add(msg)
                dead.append(msg)
            continue
        if sub is None:
            continue
        subs = groups.get(tok)
        if subs is None or sub in subs:
            continue
        listed = ",".join(sorted(subs))
        msg = f"rhwp {tok} {sub} (real subs: {listed})"
        if msg not in seen:
            seen.add(msg)
            dead.append(msg)
    return dead


def audit_one_skill(
    name: str,
    body: str,
    live: dict[str, Any],
) -> dict[str, Any]:
    refs = referenced_commands(body)
    skipped = bool(live.get("skipped"))
    if skipped:
        unknown: list[str] = []
    else:
        unknown = unknown_for_refs(refs, live["known"], live["groups"])
    unique = unique_refs(refs)
    ok = skipped or not unknown
    return {
        "id": name,
        "ok": ok,
        "pass": "skip" if skipped else ("pass" if ok else "fail"),
        "refCount": len(refs),
        "uniqueCount": len(unique),
        "refs": [format_ref(tok, sub) for tok, sub in unique],
        "unknown": unknown,
        "skipped": skipped,
    }


def _print_skill(json_mode: bool, scan_i: int, rec: dict[str, Any]) -> None:
    status = str(rec["pass"]).upper()
    unknown = rec["unknown"]
    skip_note = " unknown=skip" if rec.get("skipped") else f" unknown={len(unknown)}"
    _say(
        json_mode,
        f"[{scan_i}/{SCAN_TIMES}] {rec['id']}: refs={rec['refCount']} "
        f"unique={rec['uniqueCount']}{skip_note} {status}",
    )
    ref_list = ", ".join(rec["refs"]) if rec["refs"] else "(none)"
    _say(json_mode, f"  refs: {ref_list}")
    if unknown:
        _say(json_mode, f"  unknown: {'; '.join(unknown)}")


def scan_and_check(json_mode: bool, live: dict[str, Any]) -> list[dict[str, Any]]:
    """Re-scan every skill SCAN_TIMES times. Fail on first mismatch or unknown."""
    snapshots: list[list[tuple[str, tuple[tuple[str, str | None], ...]]]] = []
    last_records: list[dict[str, Any]] = []
    any_unknown = False
    unknown_rows: list[str] = []
    for scan_i in range(1, SCAN_TIMES + 1):
        rows: list[tuple[str, tuple[tuple[str, str | None], ...]]] = []
        records: list[dict[str, Any]] = []
        for dir_path in list_skill_dirs():
            name = dir_path.name
            body = read_skill(dir_path)
            refs = referenced_commands(body)
            rec = audit_one_skill(name, body, live)
            _print_skill(json_mode, scan_i, rec)
            if rec["unknown"]:
                any_unknown = True
                for token in rec["unknown"]:
                    msg = f"{name}: `{token}`"
                    if msg not in unknown_rows:
                        unknown_rows.append(msg)
            rows.append((name, tuple(refs)))
            records.append(rec)
        if snapshots and rows != snapshots[0]:
            _say(json_mode, f"[{scan_i}/{SCAN_TIMES}] SCAN: FAIL: mismatch vs scan 1")
            raise AuditFail(f"scan {scan_i} disagrees with scan 1")
        snapshots.append(rows)
        last_records = records
        _say(json_mode, f"[{scan_i}/{SCAN_TIMES}] SCAN: pass")
    if any_unknown:
        raise AuditFail(
            "skills reference unknown rhwp commands:\n  " + "\n  ".join(unknown_rows)
        )
    return last_records


def run_audit(json_mode: bool) -> dict[str, Any]:
    envelope: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": "skillCommandAudit",
        "ok": False,
        "exit": 1,
        "scans": SCAN_TIMES,
        "skills": [],
        "rhwp": {"present": find_rhwp_binary() is not None, "ok": False},
    }
    live = load_live_rhwp(json_mode)
    envelope["skills"] = scan_and_check(json_mode, live)
    rhwp_out = {
        "present": live["present"],
        "ok": True,
        "knownCount": live.get("knownCount", 0),
        "checked": sum(rec["refCount"] for rec in envelope["skills"]),
        "dead": [],
    }
    if live.get("skipped"):
        rhwp_out["skipped"] = live["skipped"]
    if live.get("binary"):
        rhwp_out["binary"] = live["binary"]
    envelope["rhwp"] = rhwp_out
    envelope["ok"] = True
    envelope["exit"] = 0
    n_skills = len(envelope["skills"])
    if rhwp_out.get("present"):
        rhwp_summary = str(rhwp_out.get("knownCount", 0))
    else:
        rhwp_summary = "skip"
    failed = [rec["id"] for rec in envelope["skills"] if not rec["ok"]]
    _say(
        json_mode,
        f"OK: {n_skills} skills x {SCAN_TIMES} scans, "
        f"rhwp={rhwp_summary}, refs={rhwp_out['checked']}, fail={len(failed)}",
    )
    return envelope


def _parse_argv(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="audit_skill_commands.py",
        description="Audit SKILL.md rhwp command tokens against live rhwp.",
        add_help=False,
    )
    parser.add_argument("--json", action="store_true", help="write a summary envelope to stdout")
    parser.add_argument("-h", "--help", action="store_true", help="show usage")
    args, unknown = parser.parse_known_args(argv)
    if args.help or unknown:
        raise UsageError(USAGE)
    return args


def main(argv: list[str] | None = None) -> int:
    _configure_stdio()
    try:
        args = _parse_argv(argv)
    except UsageError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    json_mode = bool(args.json)
    envelope: dict[str, Any] | None = None
    exit_code = 1
    try:
        envelope = run_audit(json_mode)
        exit_code = 0
    except AuditFail as exc:
        _say(json_mode, f"FAIL: {exc}")
        envelope = {
            "schemaVersion": SCHEMA_VERSION,
            "kind": "skillCommandAudit",
            "ok": False,
            "exit": 1,
            "error": str(exc),
            "scans": SCAN_TIMES,
        }
        exit_code = 1
    if json_mode and envelope is not None:
        sys.stdout.write(json.dumps(envelope, ensure_ascii=False, indent=2, default=str))
        sys.stdout.write("\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
