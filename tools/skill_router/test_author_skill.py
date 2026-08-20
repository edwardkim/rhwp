#!/usr/bin/env python3
"""CAP-5706 author-skill route contract.

Drives `tools/skill_router/route.py --json` for skill-author phrases.
Does not import private helpers from test_route.py.

저장소 루트:

    python -m unittest tools.skill_router.test_author_skill
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
ROUTE_PY = REPO / "tools" / "skill_router" / "route.py"
CLI_TIMEOUT_SEC = 20

ENVELOPE_KEYS = (
    "schemaVersion",
    "request",
    "intent",
    "requiredCapabilities",
    "skillSelection",
    "executionGraph",
)

AUTHOR_REQUESTS = (
    "새 스킬 만들어",
    "create a skill",
    "SKILL.md 작성",
)


def _fail_if_router_missing() -> None:
    if not ROUTE_PY.is_file():
        raise unittest.SkipTest(
            "tools/skill_router/route.py is missing (CAP-5706 router "
            "not ready). Skipping instead of hanging."
        )


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
        raise AssertionError(f"stdout had trailing non-JSON after one value: {preview!r}")
    return obj


def _run_cli(request: str) -> subprocess.CompletedProcess[str]:
    _fail_if_router_missing()
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


def _node_haystack(node: Any) -> str:
    if isinstance(node, str):
        return node
    if not isinstance(node, dict):
        return str(node)
    parts: list[str] = []
    for key in ("id", "action", "command"):
        val = node.get(key)
        if val is not None:
            parts.append(str(val))
    return " ".join(parts)


def _graph_nodes(envelope: dict[str, Any]) -> list[Any]:
    graph = envelope.get("executionGraph")
    if isinstance(graph, list):
        return graph
    if isinstance(graph, dict):
        nodes = graph.get("nodes")
        if isinstance(nodes, list):
            return nodes
        raise AssertionError(
            f"executionGraph has no nodes list; keys={sorted(graph.keys())}"
        )
    raise AssertionError(
        f"executionGraph is {type(graph).__name__}, expected object or list"
    )


def _blob(value: Any) -> str:
    if isinstance(value, str):
        return value
    return json.dumps(value, ensure_ascii=False)


def _norm_token(value: str) -> str:
    return re.sub(r"[\s_]+", "-", value.strip().lower())


def _intent_tokens(intent: Any) -> list[str]:
    tokens: list[str] = []
    if isinstance(intent, str):
        tokens.append(intent)
    elif isinstance(intent, dict):
        for key in ("id", "name", "intent", "kind", "type", "slug"):
            val = intent.get(key)
            if isinstance(val, str):
                tokens.append(val)
        tokens.append(_blob(intent))
    else:
        tokens.append(_blob(intent))
    return tokens


def _has_intent(envelope: dict[str, Any], expected: str) -> bool:
    want = _norm_token(expected)
    for token in _intent_tokens(envelope.get("intent")):
        if want == _norm_token(token) or want in _norm_token(token):
            return True
    return False


def _has_capability(value: Any, capability: str) -> bool:
    return capability in _blob(value)


def _graph_blob(nodes: list[Any]) -> str:
    parts = [_node_haystack(node) for node in nodes]
    parts.append(_blob(nodes))
    return " ".join(parts)


def _is_gate_node(text: str) -> bool:
    if "gate_new_skill" in text:
        return True
    return bool(re.search(r"(?i)\bgate\b", text))


def _mentions_three_pass(blob: str, nodes: list[Any]) -> bool:
    if re.search(r"(?i)3[-_ ]?(pass|x)|three[-_ ]pass|세\s*번", blob):
        return True
    if blob.count("gate_new_skill") >= 3:
        return True
    gate_at = [i for i, node in enumerate(nodes) if _is_gate_node(_node_haystack(node))]
    return len(gate_at) >= 3


class _RouterTests(unittest.TestCase):
    def setUp(self) -> None:
        _fail_if_router_missing()


class AuthorSkillRouteTests(_RouterTests):
    def test_author_skill_phrases_route_to_rhwp_skill_author(self) -> None:
        for request in AUTHOR_REQUESTS:
            with self.subTest(request=request):
                envelope = _route_cli(request)
                missing = [key for key in ENVELOPE_KEYS if key not in envelope]
                self.assertEqual(missing, [], f"missing envelope keys: {missing}")
                self.assertEqual(envelope.get("request"), request)
                self.assertTrue(
                    _has_intent(envelope, "author-skill"),
                    f"intent was {envelope.get('intent')!r}, expected author-skill",
                )
                self.assertTrue(
                    _has_capability(
                        envelope.get("skillSelection"), "rhwp-skill-author"
                    ),
                    envelope.get("skillSelection"),
                )
                nodes = _graph_nodes(envelope)
                self.assertTrue(nodes, "executionGraph has no nodes")
                haystacks = [_node_haystack(node) for node in nodes]
                blob = _graph_blob(nodes)
                self.assertIn(
                    "gate_new_skill",
                    blob,
                    f"graph has no gate_new_skill: {haystacks!r}",
                )
                self.assertRegex(
                    blob,
                    r"(?i)SKILL\.md",
                    f"graph has no SKILL.md scaffold: {haystacks!r}",
                )
                self.assertTrue(
                    _mentions_three_pass(blob, nodes),
                    "graph needs a 3-pass mention or three gate nodes: "
                    f"{haystacks!r}",
                )


if __name__ == "__main__":
    unittest.main()
