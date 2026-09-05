#!/usr/bin/env python3
"""Fail-closed sync: .agents/skills folders == catalog.json ids ==
intents.py skills == graph.py builders.

    python tools/skill_router/check_catalog_sync.py
    python tools/skill_router/check_catalog_sync.py --json

Exit 0 pass, 1 mismatch/unreadable, 2 usage. stdlib only.
Does not edit catalog.json or SKILL.md.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
SKILLS_DIR = REPO / ".agents" / "skills"
CATALOG_JSON = HERE / "catalog.json"
INTENTS_PY = HERE / "intents.py"
GRAPH_PY = HERE / "graph.py"

SCHEMA_VERSION = "1.0"
USAGE = "usage: python tools/skill_router/check_catalog_sync.py [--json]"

SKILL_ID_RE = re.compile(r"^rhwp-[a-z0-9-]+$")
INTENT_SKILL_RE = re.compile(r'"skill"\s*:\s*"([^"]+)"')
BUILDER_KEY_RE = re.compile(r'^\s*"(rhwp-[a-z0-9-]+)"\s*:', re.MULTILINE)


class SyncFail(Exception):
    """A sync check failed. exit 1."""


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


def _sorted_unique(ids: list[str]) -> list[str]:
    return sorted(set(ids))


def _fmt_ids(ids: list[str]) -> str:
    return ", ".join(ids) if ids else "(none)"


def _read_text(path: Path, label: str) -> str:
    if not path.is_file():
        raise SyncFail(f"missing {label}: {path}")
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise SyncFail(f"{label} unreadable: {exc}") from exc


def _extract_balanced(text: str, start: int, opener: str, closer: str) -> str:
    """Return the body between the first opener at/after start and its match."""
    idx = text.find(opener, start)
    if idx < 0:
        raise SyncFail(f"expected {opener!r} after assignment")
    depth = 0
    in_str = False
    escape = False
    quote = ""
    for pos in range(idx, len(text)):
        ch = text[pos]
        if in_str:
            if escape:
                escape = False
                continue
            if ch == "\\":
                escape = True
                continue
            if ch == quote:
                in_str = False
            continue
        if ch in ('"', "'"):
            in_str = True
            quote = ch
            continue
        if ch == opener:
            depth += 1
            continue
        if ch == closer:
            depth -= 1
            if depth == 0:
                return text[idx + 1 : pos]
    raise SyncFail(f"unbalanced {opener}{closer}")


def list_skill_dirs() -> list[str]:
    if not SKILLS_DIR.is_dir():
        raise SyncFail(f"missing skills dir: {SKILLS_DIR}")
    ids: list[str] = []
    for child in SKILLS_DIR.iterdir():
        if not child.is_dir():
            continue
        if (child / "SKILL.md").is_file():
            ids.append(child.name)
    if not ids:
        raise SyncFail("no .agents/skills/*/SKILL.md folders")
    return _sorted_unique(ids)


def load_catalog_ids() -> list[str]:
    raw_text = _read_text(CATALOG_JSON, "catalog.json")
    try:
        raw = json.loads(raw_text)
    except json.JSONDecodeError as exc:
        raise SyncFail(f"catalog.json unreadable: {exc}") from exc
    payload: Any = raw
    if isinstance(raw, dict) and "skills" in raw:
        payload = raw["skills"]
    ids: list[str] = []
    if isinstance(payload, dict):
        for key, value in payload.items():
            if key in ("schemaVersion", "catalogVersion", "version"):
                continue
            skill_id = key
            if isinstance(value, dict):
                skill_id = str(value.get("id") or value.get("skill") or value.get("name") or key)
            elif isinstance(value, str) and SKILL_ID_RE.match(value):
                skill_id = value
            if not skill_id:
                raise SyncFail(f"catalog skill {key!r} missing id")
            ids.append(str(skill_id))
    elif isinstance(payload, list):
        if not payload:
            raise SyncFail("catalog.json has empty skills list")
        for item in payload:
            if not isinstance(item, dict):
                raise SyncFail("catalog skills list contains a non-object")
            skill_id = item.get("id") or item.get("skill") or item.get("name")
            if not skill_id:
                raise SyncFail("catalog skill entry missing id")
            ids.append(str(skill_id))
    else:
        raise SyncFail("catalog.json has no skills map or list")
    if not ids:
        raise SyncFail("catalog.json skills have no ids")
    dupes = sorted({sid for sid in ids if ids.count(sid) > 1})
    if dupes:
        raise SyncFail(f"duplicate catalog ids: {', '.join(dupes)}")
    return _sorted_unique(ids)


def parse_intent_skills() -> list[str]:
    text = _read_text(INTENTS_PY, "intents.py")
    marker = "INTENT_SPECS"
    start = text.find(marker)
    if start < 0:
        raise SyncFail("INTENT_SPECS not found in intents.py")
    eq = text.find("=", start)
    if eq < 0:
        raise SyncFail("INTENT_SPECS assignment not found")
    body = _extract_balanced(text, eq, "(", ")")
    ids = [m.group(1) for m in INTENT_SKILL_RE.finditer(body)]
    if not ids:
        raise SyncFail("INTENT_SPECS has no skill fields")
    return _sorted_unique(ids)


def parse_graph_skills() -> list[str]:
    text = _read_text(GRAPH_PY, "graph.py")
    marker = "_BUILDERS"
    start = text.find(marker)
    if start < 0:
        raise SyncFail("_BUILDERS not found in graph.py")
    eq = text.find("=", start)
    if eq < 0:
        raise SyncFail("_BUILDERS assignment not found")
    body = _extract_balanced(text, eq, "{", "}")
    ids = BUILDER_KEY_RE.findall(body)
    if not ids:
        raise SyncFail("_BUILDERS has no rhwp-* skill keys")
    return _sorted_unique(ids)


def _diff(left: list[str], right: list[str]) -> list[str]:
    return sorted(set(left) - set(right))


def check_sync() -> dict[str, Any]:
    dir_ids = list_skill_dirs()
    catalog_ids = load_catalog_ids()
    intent_ids = parse_intent_skills()
    graph_ids = parse_graph_skills()

    extras_in_dir = _diff(dir_ids, catalog_ids)
    missing_from_catalog = _diff(dir_ids, catalog_ids)
    extras_in_catalog = _diff(catalog_ids, dir_ids)
    missing_from_dir = _diff(catalog_ids, dir_ids)

    extras_in_intents = _diff(intent_ids, catalog_ids)
    missing_from_intents = _diff(catalog_ids, intent_ids)
    extras_in_graph = _diff(graph_ids, catalog_ids)
    missing_from_graph = _diff(catalog_ids, graph_ids)

    extras_in_intents_vs_dir = _diff(intent_ids, dir_ids)
    missing_from_intents_vs_dir = _diff(dir_ids, intent_ids)
    extras_in_graph_vs_dir = _diff(graph_ids, dir_ids)
    missing_from_graph_vs_dir = _diff(dir_ids, graph_ids)

    mismatched = any(
        (
            extras_in_dir,
            missing_from_catalog,
            extras_in_catalog,
            missing_from_dir,
            extras_in_intents,
            missing_from_intents,
            extras_in_graph,
            missing_from_graph,
            extras_in_intents_vs_dir,
            missing_from_intents_vs_dir,
            extras_in_graph_vs_dir,
            missing_from_graph_vs_dir,
        )
    )
    ok = (
        not mismatched
        and dir_ids == catalog_ids == intent_ids == graph_ids
    )
    return {
        "schemaVersion": SCHEMA_VERSION,
        "kind": "catalogSync",
        "ok": ok,
        "exit": 0 if ok else 1,
        "counts": {
            "dir": len(dir_ids),
            "catalog": len(catalog_ids),
            "intents": len(intent_ids),
            "graph": len(graph_ids),
        },
        "dir": dir_ids,
        "catalog": catalog_ids,
        "intents": intent_ids,
        "graph": graph_ids,
        "extras-in-dir": extras_in_dir,
        "missing-from-catalog": missing_from_catalog,
        "extras-in-catalog": extras_in_catalog,
        "missing-from-dir": missing_from_dir,
        "extras-in-intents": extras_in_intents,
        "missing-from-intents": missing_from_intents,
        "extras-in-graph": extras_in_graph,
        "missing-from-graph": missing_from_graph,
    }


def _print_human(envelope: dict[str, Any], json_mode: bool) -> None:
    _say(json_mode, f"extras-in-dir: {_fmt_ids(envelope['extras-in-dir'])}")
    _say(json_mode, f"missing-from-catalog: {_fmt_ids(envelope['missing-from-catalog'])}")
    _say(json_mode, f"extras-in-catalog: {_fmt_ids(envelope['extras-in-catalog'])}")
    _say(json_mode, f"missing-from-dir: {_fmt_ids(envelope['missing-from-dir'])}")
    _say(json_mode, f"extras-in-intents: {_fmt_ids(envelope['extras-in-intents'])}")
    _say(json_mode, f"missing-from-intents: {_fmt_ids(envelope['missing-from-intents'])}")
    _say(json_mode, f"extras-in-graph: {_fmt_ids(envelope['extras-in-graph'])}")
    _say(json_mode, f"missing-from-graph: {_fmt_ids(envelope['missing-from-graph'])}")
    counts = envelope["counts"]
    if envelope["ok"]:
        n = counts["dir"]
        _say(
            json_mode,
            f"OK: {n} skills synced (dir=catalog=intents=graph)",
        )
        return
    _say(
        json_mode,
        "FAIL: catalog sync mismatch "
        f"(dir={counts['dir']}, catalog={counts['catalog']}, "
        f"intents={counts['intents']}, graph={counts['graph']})",
    )


def _parse_argv(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="check_catalog_sync.py",
        description="Fail-closed check that skill folders match catalog/intents/graph.",
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
        envelope = check_sync()
        _print_human(envelope, json_mode)
        exit_code = 0 if envelope["ok"] else 1
        envelope["exit"] = exit_code
    except SyncFail as exc:
        _say(json_mode, f"FAIL: {exc}")
        envelope = {
            "schemaVersion": SCHEMA_VERSION,
            "kind": "catalogSync",
            "ok": False,
            "exit": 1,
            "error": str(exc),
            "extras-in-dir": [],
            "missing-from-catalog": [],
            "extras-in-catalog": [],
            "missing-from-dir": [],
        }
        exit_code = 1
    if json_mode and envelope is not None:
        sys.stdout.write(json.dumps(envelope, ensure_ascii=False, indent=2))
        sys.stdout.write("\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
