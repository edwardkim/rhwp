#!/usr/bin/env python3
"""Repeat-route every catalog skill (stdlib unittest).

For each skill id in catalog.json (or .agents/skills folder names if the
JSON is missing) run three Korean phrasings, three times each. Missing
route.py fails immediately; this module does not hang.

저장소 루트:

    py -3 -m unittest tools.skill_router.test_skills_repeat
    py -3 tools/skill_router/test_skills_repeat.py
"""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Any, Callable

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ROUTE_PY = HERE / "route.py"
CATALOG_JSON = HERE / "catalog.json"
SKILLS_DIR = REPO / ".agents" / "skills"
CLI_TIMEOUT_SEC = 20
REPEAT = 3

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

# Skills the router can actually classify (route.py DEFAULT_SKILL_PATHS).
# Catalog-only ids still need a valid envelope.
ROUTER_SKILLS: frozenset[str] = frozenset(
    {
        "rhwp-onboarding",
        "rhwp-doc-triage",
        "rhwp-form-fill",
        "rhwp-table-exchange",
        "rhwp-safe-edit",
        "rhwp-security-sweep",
        "rhwp-bulk-pipeline",
        "rhwp-visual-regression",
        "rhwp-exam-ingest",
        "rhwp-contributor",
        "rhwp-cli",
        "rhwp-codex",
    }
)

# Three Korean phrasings per skill. Router skills use intent-pattern hits.
# Extra catalog skills only need a valid envelope.
PHRASINGS: dict[str, tuple[str, str, str]] = {
    "rhwp-bug-hunter": (
        "버그 찾아줘",
        "정답지와 비교해줘",
        "버그 헌팅 플레이북 실행해줘",
    ),
    "rhwp-bulk-pipeline": (
        "폴더 전체 변환해줘",
        "대량 처리해줘",
        "여러 hwp 한꺼번에 처리해줘",
    ),
    "rhwp-chief": (
        "요청 큐 돌려줘",
        "needs-agent 수거해줘",
        "서비스 루프 감시해줘",
    ),
    "rhwp-cli": (
        "페이지네이션 확인해줘",
        "png로 내보내줘",
        "조판부호 덤프해줘",
    ),
    "rhwp-codex": (
        "코덱스 보여줘",
        "명령 교본 알려줘",
        "뭘 쓸지 모르겠어",
    ),
    "rhwp-contributor": (
        "PR 올려",
        "피알 올려줘",
        "기여 절차 알려줘",
    ),
    "rhwp-doc-triage": (
        "이 hwp 뭔 문서야",
        "내용 요약해줘",
        "목차 뽑아줘",
    ),
    "rhwp-exam-ingest": (
        "시험지 변환해줘",
        "시험문제 만들어줘",
        "한글 시험지로 바꿔줘",
    ),
    "rhwp-explore": (
        "이 문서로 뭘 할 수 있어?",
        "문서 탐색 해줘",
        "어떤 rhwp 도구를 써야 해?",
    ),
    "rhwp-fde": (
        "고객이 이 문서가 안 열린대",
        "증상 접수해줘",
        "고객 회신 초안 써줘",
    ),
    "rhwp-fidelity-compare": (
        "한컴 PDF와 비교해줘",
        "공식 출력 기준 대조해줘",
        "한컴이 뽑은 PDF랑 rhwp가 같은지 봐줘",
    ),
    "rhwp-form-fill": (
        "이 서식 채워줘",
        "양식 채워 주세요",
        "누름틀에 값 넣어줘",
    ),
    "rhwp-onboarding": (
        "온보딩 진행해줘",
        "rhwp 처음 써봐",
        "셋업부터 해줘",
    ),
    "rhwp-recipes": (
        "어떤 레시피로 가?",
        "레시피 07 보여줘",
        "레시피 골라줘",
    ),
    "rhwp-safe-edit": (
        "안전하게 편집해줘",
        "문구 일괄 치환해줘",
        "dry-run으로 먼저 해줘",
    ),
    "rhwp-security-sweep": (
        "이 문서 보내도 돼?",
        "배포 전 점검해줘",
        "숨긴 텍스트 검사해줘",
    ),
    "rhwp-skill-router": (
        "스킬 라우터로 요청 보내줘",
        "요청을 스킬로 라우팅해줘",
        "어떤 스킬 쓸지 라우터가 정해줘",
    ),
    "rhwp-strategist": (
        "이 문서들로 전략 보고서 만들어줘",
        "정부과제 수주 근거 모아줘",
        "근거 대장 작성해줘",
    ),
    "rhwp-table-exchange": (
        "표를 CSV로 뽑아줘",
        "엑셀로 뽑아줘",
        "표 셀 하나만 고쳐줘",
    ),
    "rhwp-visual-regression": (
        "시각 회귀 확인해줘",
        "편집 전후 화면 비교해줘",
        "레이아웃 회귀 있는지 봐줘",
    ),
}

_ROUTE_FN: Callable[[str], dict[str, Any]] | None = None


def _require_route_py() -> None:
    if not ROUTE_PY.is_file():
        raise AssertionError(
            "tools/skill_router/route.py is missing; failing immediately (no hang)"
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
        raise AssertionError(
            f"stdout had trailing non-JSON after one value: {preview!r}"
        )
    return obj


def _load_route_fn() -> Callable[[str], dict[str, Any]]:
    global _ROUTE_FN
    if _ROUTE_FN is not None:
        return _ROUTE_FN
    _require_route_py()
    spec = importlib.util.spec_from_file_location(
        "skill_router_route_under_test", ROUTE_PY
    )
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load {ROUTE_PY}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    fn = getattr(mod, "route", None)
    if not callable(fn):
        raise AssertionError(f"{ROUTE_PY} has no callable route()")
    _ROUTE_FN = fn
    return fn


def _route_subprocess(request: str) -> dict[str, Any]:
    _require_route_py()
    try:
        proc = subprocess.run(
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


def _route(request: str) -> dict[str, Any]:
    """Call route.route when importable; otherwise spawn route.py."""
    try:
        envelope = _load_route_fn()(request)
    except AssertionError:
        raise
    except Exception:
        envelope = _route_subprocess(request)
    if isinstance(envelope, str):
        envelope = _parse_one_json(envelope)
    if not isinstance(envelope, dict):
        raise AssertionError(f"envelope is {type(envelope).__name__}, expected object")
    return envelope


def _ids_from_catalog(raw: Any) -> list[str]:
    skip = {"schemaVersion", "catalogVersion", "version"}
    if isinstance(raw, dict) and "skills" in raw:
        raw = raw["skills"]
    ids: list[str] = []
    seen: set[str] = set()

    def _add(skill_id: str) -> None:
        if skill_id and skill_id not in seen and skill_id not in skip:
            seen.add(skill_id)
            ids.append(skill_id)

    if isinstance(raw, dict):
        for key, value in raw.items():
            if key in skip:
                continue
            if isinstance(value, dict):
                _add(str(value.get("id") or key))
            else:
                _add(str(key))
        return ids
    if isinstance(raw, list):
        for item in raw:
            if not isinstance(item, dict):
                continue
            skill_id = item.get("id") or item.get("skill") or item.get("name")
            if skill_id:
                _add(str(skill_id))
    return ids


def _load_skill_ids() -> list[str]:
    if CATALOG_JSON.is_file():
        raw = json.loads(CATALOG_JSON.read_text(encoding="utf-8"))
        ids = _ids_from_catalog(raw)
        if ids:
            return ids
    if SKILLS_DIR.is_dir():
        names = sorted(p.name for p in SKILLS_DIR.iterdir() if p.is_dir())
        if names:
            return names
    return sorted(ROUTER_SKILLS)


def _phrasings_for(skill_id: str) -> tuple[str, str, str]:
    if skill_id in PHRASINGS:
        return PHRASINGS[skill_id]
    return (
        f"{skill_id} 스킬로 처리해줘",
        f"{skill_id} 해줘",
        f"{skill_id} 작업 부탁해",
    )


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


def _assert_envelope(test: unittest.TestCase, envelope: dict[str, Any], request: str) -> None:
    missing = [key for key in ENVELOPE_KEYS if key not in envelope]
    test.assertEqual(missing, [], f"missing envelope keys: {missing}")
    test.assertEqual(envelope.get("request"), request)
    caps = envelope.get("requiredCapabilities")
    test.assertTrue(caps, f"requiredCapabilities empty: {caps!r}")
    graph = envelope.get("executionGraph")
    test.assertIsInstance(graph, dict, f"executionGraph is {type(graph).__name__}")
    assert isinstance(graph, dict)
    nodes = graph.get("nodes")
    edges = graph.get("edges")
    test.assertTrue(nodes, f"executionGraph.nodes empty: {nodes!r}")
    test.assertTrue(edges, f"executionGraph.edges empty: {edges!r}")
    test.assertIsInstance(nodes, list)
    test.assertIsInstance(edges, list)


class RoutePyPresentTests(unittest.TestCase):
    def test_route_py_missing_fails_immediately(self) -> None:
        _require_route_py()
        self.assertTrue(ROUTE_PY.is_file())


class SkillsRepeatTests(unittest.TestCase):
    def setUp(self) -> None:
        _require_route_py()

    def test_every_skill_three_phrasings_three_runs(self) -> None:
        skill_ids = _load_skill_ids()
        self.assertTrue(skill_ids, "no skill ids from catalog.json or .agents/skills")
        for skill_id in skill_ids:
            phrasings = _phrasings_for(skill_id)
            self.assertEqual(len(phrasings), 3, skill_id)
            for phrasing in phrasings:
                for run in range(REPEAT):
                    with self.subTest(skill=skill_id, phrasing=phrasing, run=run):
                        envelope = _route(phrasing)
                        _assert_envelope(self, envelope, phrasing)
                        if skill_id in ROUTER_SKILLS:
                            selected = _selected_skill_ids(
                                envelope.get("skillSelection")
                            )
                            self.assertIn(
                                skill_id,
                                selected,
                                f"{phrasing!r} selected {selected!r}, expected {skill_id}",
                            )

    def test_fill_form_always_selects_rhwp_form_fill(self) -> None:
        for run in range(REPEAT):
            with self.subTest(run=run):
                envelope = _route(FILL_REQUEST)
                _assert_envelope(self, envelope, FILL_REQUEST)
                selected = _selected_skill_ids(envelope.get("skillSelection"))
                self.assertIn(
                    "rhwp-form-fill",
                    selected,
                    f"{FILL_REQUEST!r} selected {selected!r}",
                )

    def test_contribute_always_selects_rhwp_contributor(self) -> None:
        for run in range(REPEAT):
            with self.subTest(run=run):
                envelope = _route(CONTRIBUTE_REQUEST)
                _assert_envelope(self, envelope, CONTRIBUTE_REQUEST)
                selected = _selected_skill_ids(envelope.get("skillSelection"))
                self.assertIn(
                    "rhwp-contributor",
                    selected,
                    f"{CONTRIBUTE_REQUEST!r} selected {selected!r}",
                )


if __name__ == "__main__":
    unittest.main()
