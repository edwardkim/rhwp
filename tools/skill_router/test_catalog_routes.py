#!/usr/bin/env python3
"""Route every catalog.json skill at least once via route.py --json.

Picks each skill's first trigger. When that string loses to a more
specific intent, a unique remaining trigger is used instead. A skill
that cannot win any request fails — never skipped.

저장소 루트:

    python -m unittest tools.skill_router.test_catalog_routes
"""

from __future__ import annotations

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
CATALOG_JSON = HERE / "catalog.json"
CLI_TIMEOUT_SEC = 20

ENVELOPE_KEYS = (
    "schemaVersion",
    "request",
    "intent",
    "requiredCapabilities",
    "skillSelection",
    "executionGraph",
)

# First catalog trigger loses to a more specific intent. Probe is the
# first remaining trigger from that skill that actually wins.
DEDICATED_PROBES: dict[str, str] = {
    # "HWPX로 만들어줘" hits no exam-ingest pattern → rhwp-codex fallback.
    "rhwp-exam-ingest": "한글 시험지로 변환",
    # "누름틀 채워" is fill-form (specificity 95) over safe-edit (70).
    "rhwp-safe-edit": "문구 일괄 치환",
}

_WIN: dict[str, str] = {}
_NO_WIN: list[str] = []


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


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


def _load_catalog_skills() -> list[dict[str, Any]]:
    if not CATALOG_JSON.is_file():
        raise AssertionError(f"missing catalog: {CATALOG_JSON}")
    raw = json.loads(CATALOG_JSON.read_text(encoding="utf-8"))
    skills = raw.get("skills") if isinstance(raw, dict) else raw
    if not isinstance(skills, list) or not skills:
        raise AssertionError("catalog.json has no skills list")
    out: list[dict[str, Any]] = []
    for item in skills:
        if not isinstance(item, dict):
            continue
        skill_id = item.get("id")
        if not skill_id:
            continue
        out.append(item)
    if not out:
        raise AssertionError("catalog.json skills have no ids")
    return out


def _catalog_ids() -> list[str]:
    return [str(item["id"]) for item in _load_catalog_skills()]


def _selected_skill_ids(selection: Any) -> list[str]:
    ids: list[str] = []
    if isinstance(selection, str):
        ids.append(selection)
    elif isinstance(selection, dict):
        val = selection.get("id") or selection.get("skill") or selection.get("name")
        if val:
            ids.append(str(val))
    elif isinstance(selection, list):
        for item in selection:
            ids.extend(_selected_skill_ids(item))
    return ids


def _probe_candidates(skill: dict[str, Any]) -> list[str]:
    skill_id = str(skill["id"])
    triggers = [
        t.strip()
        for t in (skill.get("triggers") or [])
        if isinstance(t, str) and t.strip()
    ]
    dedicated = DEDICATED_PROBES.get(skill_id)
    ordered: list[str] = []
    if dedicated:
        ordered.append(dedicated)
    for trig in triggers:
        if trig not in ordered:
            ordered.append(trig)
    if not ordered:
        raise AssertionError(
            f"{skill_id} has no triggers and no dedicated probe"
        )
    return ordered


def _assert_envelope_shape(
    test: unittest.TestCase, envelope: dict[str, Any], request: str
) -> None:
    missing = [key for key in ENVELOPE_KEYS if key not in envelope]
    test.assertEqual(missing, [], f"missing envelope keys: {missing}")
    test.assertEqual(envelope.get("request"), request)
    caps = envelope.get("requiredCapabilities")
    test.assertTrue(caps, f"requiredCapabilities empty: {caps!r}")
    graph = envelope.get("executionGraph")
    if isinstance(graph, dict):
        test.assertIn("nodes", graph, f"executionGraph keys={sorted(graph.keys())}")
        test.assertIn("edges", graph, f"executionGraph keys={sorted(graph.keys())}")
        test.assertTrue(graph.get("nodes"), f"executionGraph.nodes empty: {graph.get('nodes')!r}")
        test.assertTrue(graph.get("edges"), f"executionGraph.edges empty: {graph.get('edges')!r}")
    elif isinstance(graph, list):
        test.assertTrue(graph, "executionGraph list of nodes is empty")
    else:
        test.fail(
            f"executionGraph is {type(graph).__name__}, expected object or list"
        )


def _route_until_win(skill: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    skill_id = str(skill["id"])
    tried: list[str] = []
    losses: list[str] = []
    for request in _probe_candidates(skill):
        tried.append(request)
        envelope = _route_cli(request)
        selected = _selected_skill_ids(envelope.get("skillSelection"))
        if skill_id in selected:
            _WIN[skill_id] = request
            return request, envelope
        other = selected[0] if selected else "<none>"
        losses.append(f"{request!r} -> {other}")
    _NO_WIN.append(skill_id)
    raise AssertionError(
        f"skill {skill_id} cannot win any request; tried {tried!r}; "
        f"losses: {losses}. FAIL and list the skill — do not skip."
    )


class CatalogPresentTests(unittest.TestCase):
    def test_catalog_and_route_py_exist(self) -> None:
        self.assertTrue(CATALOG_JSON.is_file(), f"missing {CATALOG_JSON}")
        self.assertTrue(ROUTE_PY.is_file(), f"missing {ROUTE_PY}")
        self.assertTrue(_catalog_ids(), "catalog.json has no skill ids")


class CatalogRouteTests(unittest.TestCase):
    def test_every_catalog_skill_wins_one_route_py_json(self) -> None:
        skills = _load_catalog_skills()
        self.assertTrue(skills, "catalog.json has no skills")
        failed: list[str] = []
        for skill in skills:
            skill_id = str(skill["id"])
            with self.subTest(skill=skill_id):
                try:
                    request, envelope = _route_until_win(skill)
                except AssertionError as exc:
                    failed.append(skill_id)
                    self.fail(str(exc))
                    continue
                _assert_envelope_shape(self, envelope, request)
                selected = _selected_skill_ids(envelope.get("skillSelection"))
                self.assertIn(
                    skill_id,
                    selected,
                    f"skillSelection={selected!r} did not name {skill_id}",
                )
        if failed:
            self.fail(f"skills that failed to win any request: {failed}")

    def test_winning_skills_equal_catalog_ids(self) -> None:
        catalog_ids = _catalog_ids()
        catalog_set = set(catalog_ids)
        for skill in _load_catalog_skills():
            skill_id = str(skill["id"])
            if skill_id not in _WIN:
                try:
                    _route_until_win(skill)
                except AssertionError:
                    pass
        winners = set(_WIN)
        missing = sorted(catalog_set - winners)
        extra = sorted(winners - catalog_set)
        self.assertEqual(
            missing,
            [],
            f"skills that failed to win any request: {missing}",
        )
        self.assertEqual(
            extra,
            [],
            f"winner ids not in catalog: {extra}",
        )
        self.assertEqual(winners, catalog_set)
        self.assertEqual(len(winners), len(catalog_ids))
        self.assertFalse(
            _NO_WIN,
            f"skills that failed to win any request: {_NO_WIN}",
        )


if __name__ == "__main__":
    unittest.main()
