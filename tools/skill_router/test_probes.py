#!/usr/bin/env python3
"""Assert every gate_new_skill.PROBES request routes to that skill id.

Loads PROBES from gate_new_skill.py (do not edit that file from this
test). Each tuple member is sent to route.py --json. Selected skill is
skillSelection[0].id and must equal the PROBES key.

저장소 루트:

    python -m unittest tools.skill_router.test_probes
"""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
ROUTE_PY = HERE / "route.py"
GATE_PY = HERE / "gate_new_skill.py"
CLI_TIMEOUT_SEC = 20


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def _load_probes() -> dict[str, tuple[str, ...]]:
    if not GATE_PY.is_file():
        raise AssertionError(f"missing {GATE_PY}")
    spec = importlib.util.spec_from_file_location(
        "gate_new_skill_probes_under_test", GATE_PY
    )
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load {GATE_PY}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    probes = getattr(mod, "PROBES", None)
    if not isinstance(probes, dict) or not probes:
        raise AssertionError("gate_new_skill.PROBES is missing or empty")
    return probes


def _iter_probes(
    probes: dict[str, Any],
) -> list[tuple[str, int, str]]:
    rows: list[tuple[str, int, str]] = []
    for skill_id, raw in sorted(probes.items()):
        if isinstance(raw, str):
            text = raw.strip()
            if text:
                rows.append((str(skill_id), 1, text))
            continue
        if not isinstance(raw, (tuple, list)):
            continue
        for probe_i, item in enumerate(raw, start=1):
            if not isinstance(item, str):
                continue
            text = item.strip()
            if text:
                rows.append((str(skill_id), probe_i, text))
    return rows


def _parse_one_json(stdout: str) -> Any:
    text = stdout.strip()
    if not text:
        raise AssertionError("stdout was empty; expected one JSON object")
    decoder = json.JSONDecoder()
    try:
        obj, idx = decoder.raw_decode(text)
    except json.JSONDecodeError as exc:
        preview = text[:240].replace("\n", "\\n")
        raise AssertionError(f"stdout is not JSON: {exc}: {preview!r}") from exc
    leftover = text[idx:].strip()
    if leftover:
        preview = leftover[:120].replace("\n", "\\n")
        raise AssertionError(
            f"stdout had trailing non-JSON after one value: {preview!r}"
        )
    return obj


def _run_cli(request: str) -> subprocess.CompletedProcess[str]:
    if not ROUTE_PY.is_file():
        raise AssertionError(
            "tools/skill_router/route.py is missing; failing immediately (no hang)"
        )
    try:
        return subprocess.run(
            [sys.executable, str(ROUTE_PY), request, "--json"],
            cwd=str(REPO),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=CLI_TIMEOUT_SEC,
            env=_cli_env(),
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise AssertionError(
            f"route.py CLI hung after {CLI_TIMEOUT_SEC}s for {request!r}"
        ) from exc


def _route_cli(request: str) -> dict[str, Any]:
    proc = _run_cli(request)
    if proc.returncode != 0:
        err = (proc.stderr or "").strip()[:400]
        out = (proc.stdout or "").strip()[:200]
        raise AssertionError(
            f"route.py exit {proc.returncode} for {request!r}. "
            f"stderr={err!r} stdout={out!r}"
        )
    obj = _parse_one_json(proc.stdout)
    if not isinstance(obj, dict):
        raise AssertionError(f"envelope is {type(obj).__name__}, expected object")
    return obj


def _selected_skill_id(selection: Any) -> str:
    """Match gate_new_skill.probe_router: skillSelection[0].id."""
    if isinstance(selection, list) and selection:
        first = selection[0]
        if isinstance(first, dict):
            return str(first.get("id") or "")
        if isinstance(first, str):
            return first
    if isinstance(selection, dict):
        return str(selection.get("id") or selection.get("skill") or "")
    if isinstance(selection, str):
        return selection
    return ""


class ProbeRouteTests(unittest.TestCase):
    def test_every_probe_routes_to_its_skill_id(self) -> None:
        probes = _load_probes()
        rows = _iter_probes(probes)
        self.assertTrue(rows, "gate_new_skill.PROBES has no route requests")
        failed: list[str] = []
        for skill_id, probe_i, request in rows:
            with self.subTest(skill=skill_id, n=probe_i, request=request):
                try:
                    envelope = _route_cli(request)
                    selected = _selected_skill_id(envelope.get("skillSelection"))
                except AssertionError as exc:
                    failed.append(
                        f"{skill_id} [{probe_i}] {request!r}: {exc}"
                    )
                    self.fail(str(exc))
                    continue
                if selected != skill_id:
                    failed.append(
                        f"{skill_id} [{probe_i}] {request!r} "
                        f"selected {selected!r}"
                    )
                self.assertEqual(
                    selected,
                    skill_id,
                    f"{skill_id} probe {request!r} selected {selected!r}",
                )
        if failed:
            self.fail(f"failing probes ({len(failed)}): {failed}")


if __name__ == "__main__":
    unittest.main()
