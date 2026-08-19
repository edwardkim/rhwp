#!/usr/bin/env python3
"""Route a user request to one rhwp skill.

Pipeline: request → intent → requiredCapabilities → skillSelection → executionGraph.

    python tools/skill_router/route.py "<user request>" --json

Stdout is one JSON object. Errors go to stderr. Exit 0 success, 2 usage.
stdlib only. catalog.json is optional.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import graph as graphmod
import intents

SCHEMA_VERSION = "1.0"
USAGE = 'usage: python tools/skill_router/route.py "<request>" --json'

# Matches .claude/skills/<name>/SKILL.md so the router works with no catalog.json.
DEFAULT_SKILL_PATHS: dict[str, str] = {
    "rhwp-onboarding": ".claude/skills/rhwp-onboarding/SKILL.md",
    "rhwp-doc-triage": ".claude/skills/rhwp-doc-triage/SKILL.md",
    "rhwp-form-fill": ".claude/skills/rhwp-form-fill/SKILL.md",
    "rhwp-table-exchange": ".claude/skills/rhwp-table-exchange/SKILL.md",
    "rhwp-safe-edit": ".claude/skills/rhwp-safe-edit/SKILL.md",
    "rhwp-security-sweep": ".claude/skills/rhwp-security-sweep/SKILL.md",
    "rhwp-bulk-pipeline": ".claude/skills/rhwp-bulk-pipeline/SKILL.md",
    "rhwp-visual-regression": ".claude/skills/rhwp-visual-regression/SKILL.md",
    "rhwp-work-receipt": ".claude/skills/rhwp-work-receipt/SKILL.md",
    "rhwp-mcp-session": ".claude/skills/rhwp-mcp-session/SKILL.md",
    "rhwp-provenance": ".claude/skills/rhwp-provenance/SKILL.md",
    "rhwp-exam-ingest": ".claude/skills/rhwp-exam-ingest/SKILL.md",
    "rhwp-contributor": ".claude/skills/rhwp-contributor/SKILL.md",
    "rhwp-cli": ".claude/skills/rhwp-cli/SKILL.md",
    "rhwp-codex": ".claude/skills/rhwp-codex/SKILL.md",
    "rhwp-skill-router": ".claude/skills/rhwp-skill-router/SKILL.md",
}

ENVELOPE_KEYS = (
    "schemaVersion",
    "request",
    "intent",
    "requiredCapabilities",
    "skillSelection",
    "executionGraph",
    "untrustedContent",
    "untrustedFields",
)


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is None:
            continue
        try:
            reconfigure(encoding="utf-8", errors="replace")
        except (OSError, ValueError):
            pass


def _as_skill_entry(skill_id: str, payload: Any) -> dict[str, str] | None:
    if isinstance(payload, str):
        return {"id": skill_id, "path": payload, "capabilityId": skill_id}
    if not isinstance(payload, dict):
        return None
    path = payload.get("path") or payload.get("skillPath") or payload.get("skill")
    if not path:
        return None
    cap = payload.get("capabilityId") or payload.get("capability") or skill_id
    return {"id": str(payload.get("id") or skill_id), "path": str(path), "capabilityId": str(cap)}


def _normalize_catalog(raw: Any) -> dict[str, dict[str, str]]:
    """Accept a dict map, {skills: ...}, or a list of {id, path} objects."""
    out: dict[str, dict[str, str]] = {}
    if raw is None:
        return out
    if isinstance(raw, dict) and "skills" in raw:
        raw = raw["skills"]
    if isinstance(raw, dict):
        for key, value in raw.items():
            if key in ("schemaVersion", "catalogVersion", "version"):
                continue
            entry = _as_skill_entry(str(key), value)
            if entry:
                out[entry["id"]] = entry
        return out
    if isinstance(raw, list):
        for item in raw:
            if not isinstance(item, dict):
                continue
            skill_id = item.get("id") or item.get("skill") or item.get("name")
            if not skill_id:
                continue
            entry = _as_skill_entry(str(skill_id), item)
            if entry:
                out[entry["id"]] = entry
    return out


def _load_catalog_module(path: Path) -> Any | None:
    spec = importlib.util.spec_from_file_location("skill_router_catalog", path)
    if spec is None or spec.loader is None:
        return None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _catalog_payload_from_module(mod: Any, repo_root: Path) -> Any:
    for name in ("CATALOG", "SKILLS", "SKILL_MAP", "catalog", "skills", "SKILL_PATHS"):
        if hasattr(mod, name):
            value = getattr(mod, name)
            if not callable(value):
                return value
    for name in ("get_catalog", "load", "load_catalog"):
        fn = getattr(mod, name, None)
        if not callable(fn):
            continue
        try:
            return fn(repo_root)
        except TypeError:
            return fn()
    return None


def load_catalog(here: Path | None = None) -> dict[str, dict[str, str]]:
    """Load tools/skill_router/catalog.py or catalog.json when present.

    Missing catalog is not an error — DEFAULT_SKILL_PATHS is enough to run.
    """
    base = here or HERE
    repo_root = base.parent.parent
    merged: dict[str, dict[str, str]] = {
        skill_id: {"id": skill_id, "path": path, "capabilityId": skill_id}
        for skill_id, path in DEFAULT_SKILL_PATHS.items()
    }

    py_path = base / "catalog.py"
    if py_path.is_file():
        try:
            mod = _load_catalog_module(py_path)
            if mod is not None:
                loaded = _normalize_catalog(_catalog_payload_from_module(mod, repo_root))
                merged.update(loaded)
        except FileNotFoundError:
            pass
        except Exception as exc:  # catalog is optional; never hang the router
            print(f"skill_router: catalog.py ignored ({exc})", file=sys.stderr)

    json_path = base / "catalog.json"
    if json_path.is_file():
        try:
            raw = json.loads(json_path.read_text(encoding="utf-8"))
            merged.update(_normalize_catalog(raw))
        except (OSError, json.JSONDecodeError) as exc:
            print(f"skill_router: catalog.json ignored ({exc})", file=sys.stderr)

    return merged


def _skill_record(skill_id: str, catalog: dict[str, dict[str, str]]) -> dict[str, str]:
    if skill_id in catalog:
        return catalog[skill_id]
    path = DEFAULT_SKILL_PATHS.get(skill_id, f".claude/skills/{skill_id}/SKILL.md")
    return {"id": skill_id, "path": path, "capabilityId": skill_id}


def route(request: str, catalog: dict[str, dict[str, str]] | None = None) -> dict[str, Any]:
    """Run the five-stage pipeline and return the envelope."""
    cat = catalog if catalog is not None else load_catalog()
    classified = intents.classify(request)
    skill_id = classified["skill"]
    record = _skill_record(skill_id, cat)
    capability = record.get("capabilityId") or classified["capability"]

    envelope = {
        "schemaVersion": SCHEMA_VERSION,
        "request": request,
        "intent": {
            "id": classified["id"],
            "label": classified["label"],
            "confidence": classified["confidence"],
        },
        "requiredCapabilities": [capability],
        "skillSelection": [
            {
                "id": record["id"],
                "path": record["path"],
                "reason": classified["reason"],
            }
        ],
        "executionGraph": graphmod.build_graph(classified["id"], record["id"]),
        "untrustedContent": False,
        "untrustedFields": [],
    }
    # Exact key set, insertion order = pipeline order.
    return {key: envelope[key] for key in ENVELOPE_KEYS}


def _parse_argv(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="route.py",
        description="Route a user request to one rhwp skill (JSON envelope).",
    )
    parser.add_argument("request", nargs="?", help="user request text")
    parser.add_argument(
        "--json",
        action="store_true",
        help="write one JSON envelope to stdout (required)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    _configure_stdio()
    args = _parse_argv(argv)
    request = (args.request or "").strip()
    if not request or not args.json:
        print(USAGE, file=sys.stderr)
        return 2
    try:
        envelope = route(request)
    except ValueError as exc:
        print(f"skill_router: {exc}", file=sys.stderr)
        print(USAGE, file=sys.stderr)
        return 2
    sys.stdout.write(json.dumps(envelope, ensure_ascii=False, indent=2))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
