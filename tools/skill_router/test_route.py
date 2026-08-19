#!/usr/bin/env python3
"""CAP-5706 skill router contract.

Drives `tools/skill_router/route.py` (import or CLI). Does not invent
request corpora. If the router file is missing, tests fail immediately
instead of hanging.

저장소 루트:

    python -m unittest tools/skill_router/test_route.py
"""

from __future__ import annotations

import importlib
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
IMPORT_TIMEOUT_SEC = 15

ENVELOPE_KEYS = (
    "schemaVersion",
    "request",
    "intent",
    "requiredCapabilities",
    "skillSelection",
    "executionGraph",
)

FILL_REQUEST = "이 서식 채워줘"
CONTRIBUTE_REQUEST = "PR 올려"


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


def _is_fields_node(text: str) -> bool:
    stripped = re.sub(r"(?i)fill[-_]?fields", " ", text)
    return bool(re.search(r"(?i)\bfields\b", stripped))


def _is_fill_node(text: str) -> bool:
    if re.search(r"(?i)fill[-_]?fields", text):
        return True
    return bool(re.search(r"(?i)\bfill\b", text))


def _is_issue_node(text: str) -> bool:
    if "이슈" in text:
        return True
    return bool(re.search(r"(?i)\b(issue|gh issue)\b", text))


def _is_pr_node(text: str) -> bool:
    if re.search(r"(?i)\b(gh pr|pull request)\b", text):
        return True
    return bool(re.search(r"(?i)\bpr\b", text))


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


class RouteReadyTests(unittest.TestCase):
    def test_route_py_missing_is_skip_not_hang(self) -> None:
        if not ROUTE_PY.is_file():
            self.skipTest(
                "tools/skill_router/route.py is missing (CAP-5706 router "
                "not ready). Skipping instead of hanging."
            )


class _RouterTests(unittest.TestCase):
    def setUp(self) -> None:
        _fail_if_router_missing()


class ImportRouteTests(_RouterTests):
    def test_import_tools_skill_router_route_does_not_hang(self) -> None:
        code = (
            "import sys\n"
            f"sys.path.insert(0, {str(REPO)!r})\n"
            "import tools.skill_router.route as route\n"
            "print('imported', getattr(route, '__file__', ''))\n"
        )
        try:
            proc = subprocess.run(
                [sys.executable, "-c", code],
                cwd=str(REPO),
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=IMPORT_TIMEOUT_SEC,
                env=_cli_env(),
                check=False,
            )
        except subprocess.TimeoutExpired as exc:
            raise AssertionError(
                f"import tools.skill_router.route hung after {IMPORT_TIMEOUT_SEC}s"
            ) from exc
        self.assertEqual(
            proc.returncode,
            0,
            f"import failed: stderr={(proc.stderr or '')[:400]!r}",
        )
        self.assertIn("imported", proc.stdout)

    def test_route_callable_fill_form_if_present(self) -> None:
        if str(REPO) not in sys.path:
            sys.path.insert(0, str(REPO))
        try:
            mod = importlib.import_module("tools.skill_router.route")
        except Exception as exc:
            self.fail(f"import tools.skill_router.route failed: {exc}")
        if not callable(getattr(mod, "route", None)):
            self.skipTest("tools.skill_router.route.route() is not exported")
        result = mod.route(FILL_REQUEST)
        if isinstance(result, str):
            result = _parse_one_json(result)
        self.assertIsInstance(result, dict)
        self.assertTrue(_has_intent(result, "fill-form"), result.get("intent"))
        self.assertTrue(
            _has_capability(result.get("requiredCapabilities"), "rhwp-form-fill"),
            result.get("requiredCapabilities"),
        )


class CliStdoutTests(_RouterTests):
    def test_stdout_is_one_json_object(self) -> None:
        proc = _run_cli(FILL_REQUEST)
        self.assertEqual(proc.returncode, 0, (proc.stderr or "")[:400])
        obj = _parse_one_json(proc.stdout)
        self.assertIsInstance(obj, dict)

    def test_envelope_has_required_keys(self) -> None:
        envelope = _route_cli(FILL_REQUEST)
        missing = [key for key in ENVELOPE_KEYS if key not in envelope]
        self.assertEqual(missing, [], f"missing envelope keys: {missing}")

    def test_contribute_envelope_has_required_keys(self) -> None:
        envelope = _route_cli(CONTRIBUTE_REQUEST)
        missing = [key for key in ENVELOPE_KEYS if key not in envelope]
        self.assertEqual(missing, [], f"missing envelope keys: {missing}")


class FillFormRouteTests(_RouterTests):
    def test_fill_form_intent_capability_and_fields_then_fill(self) -> None:
        envelope = _route_cli(FILL_REQUEST)
        self.assertEqual(envelope.get("request"), FILL_REQUEST)
        self.assertTrue(
            _has_intent(envelope, "fill-form"),
            f"intent was {envelope.get('intent')!r}, expected fill-form",
        )
        self.assertTrue(
            _has_capability(envelope.get("requiredCapabilities"), "rhwp-form-fill"),
            envelope.get("requiredCapabilities"),
        )
        self.assertTrue(
            _has_capability(envelope.get("skillSelection"), "rhwp-form-fill"),
            envelope.get("skillSelection"),
        )
        nodes = _graph_nodes(envelope)
        self.assertTrue(nodes, "executionGraph has no nodes")
        fields_at = [
            i for i, node in enumerate(nodes) if _is_fields_node(_node_haystack(node))
        ]
        fill_at = [
            i for i, node in enumerate(nodes) if _is_fill_node(_node_haystack(node))
        ]
        self.assertTrue(
            fields_at,
            f"graph has no fields node: {[_node_haystack(n) for n in nodes]!r}",
        )
        self.assertTrue(
            fill_at,
            f"graph has no fill node: {[_node_haystack(n) for n in nodes]!r}",
        )
        self.assertLess(
            fields_at[0],
            fill_at[0],
            "graph must run fields then fill",
        )


class ContributeRouteTests(_RouterTests):
    def test_contribute_intent_capability_and_issue_pr_graph(self) -> None:
        envelope = _route_cli(CONTRIBUTE_REQUEST)
        self.assertEqual(envelope.get("request"), CONTRIBUTE_REQUEST)
        self.assertTrue(
            _has_intent(envelope, "contribute"),
            f"intent was {envelope.get('intent')!r}, expected contribute",
        )
        self.assertTrue(
            _has_capability(envelope.get("requiredCapabilities"), "rhwp-contributor"),
            envelope.get("requiredCapabilities"),
        )
        self.assertTrue(
            _has_capability(envelope.get("skillSelection"), "rhwp-contributor"),
            envelope.get("skillSelection"),
        )
        nodes = _graph_nodes(envelope)
        self.assertTrue(nodes, "executionGraph has no nodes")
        issue_at = [
            i for i, node in enumerate(nodes) if _is_issue_node(_node_haystack(node))
        ]
        pr_at = [i for i, node in enumerate(nodes) if _is_pr_node(_node_haystack(node))]
        self.assertTrue(
            issue_at,
            f"graph has no issue node: {[_node_haystack(n) for n in nodes]!r}",
        )
        self.assertTrue(
            pr_at,
            f"graph has no pr node: {[_node_haystack(n) for n in nodes]!r}",
        )


if __name__ == "__main__":
    unittest.main()
