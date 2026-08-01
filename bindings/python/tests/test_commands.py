"""명령 래퍼 — 각 함수가 CLI 인자를 정확히 조립하는지.

봉투를 읽는 것은 다른 테스트가 본다. 여기서는 **인자 조립**만 검증한다.
플래그 하나가 빠지면 도구는 조용히 다른 일을 하고, 그건 봉투를 봐도 모른다.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, List

import pytest

import rhwp
from rhwp import _process


@pytest.fixture
def captured(monkeypatch: pytest.MonkeyPatch) -> List[List[Any]]:
    """실제 실행 대신 인자만 가로챈다."""
    seen: List[List[Any]] = []

    def fake_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        seen.append(list(args))
        return {"schemaVersion": "1.0"}

    def fake_run_ndjson(args, **kwargs):  # type: ignore[no-untyped-def]
        seen.append(list(args))
        return []

    monkeypatch.setattr(rhwp.commands, "run_json", fake_run_json)
    monkeypatch.setattr(rhwp.commands, "run_ndjson", fake_run_ndjson)
    return seen


def _as_strings(args: List[Any]) -> List[str]:
    return [str(a) for a in args]


# ── 조회 ────────────────────────────────────────────────────────────────


def test_info_builds_minimal_command(captured: List[List[Any]]) -> None:
    rhwp.info("a.hwp")
    assert _as_strings(captured[0]) == ["info", "a.hwp", "--json"]


def test_export_text_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_text("a.hwp")
    assert _as_strings(captured[0]) == ["export-text", "a.hwp", "--json"]


def test_export_structure_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_structure("a.hwp")
    assert _as_strings(captured[0]) == ["export-structure", "a.hwp", "--json"]


def test_export_tables_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_tables("a.hwpx")
    assert _as_strings(captured[0]) == ["export-tables", "a.hwpx", "--json"]


def test_fields_builds_command(captured: List[List[Any]]) -> None:
    rhwp.fields("a.hwp")
    assert _as_strings(captured[0]) == ["fields", "a.hwp", "--json"]


def test_search_uses_double_dash_for_query(captured: List[List[Any]]) -> None:
    """``-`` 로 시작하는 검색어도 값으로 읽히도록 구분자를 쓴다."""
    rhwp.search("a.hwp", "-예산")
    args = _as_strings(captured[0])
    assert args[-2:] == ["--", "-예산"]


def test_search_flags(captured: List[List[Any]]) -> None:
    rhwp.search("a.hwp", "예산", case_sensitive=False, limit=5)
    args = _as_strings(captured[0])
    assert "--ignore-case" in args
    assert args[args.index("--limit") + 1] == "5"


def test_search_omits_ignore_case_when_sensitive(captured: List[List[Any]]) -> None:
    rhwp.search("a.hwp", "예산")
    assert "--ignore-case" not in _as_strings(captured[0])


def test_digest_flags(captured: List[List[Any]]) -> None:
    rhwp.digest("a.hwp", sections=True, pages="1-3")
    args = _as_strings(captured[0])
    assert "--sections" in args
    assert args[args.index("--pages") + 1] == "1-3"


def test_digest_without_options(captured: List[List[Any]]) -> None:
    rhwp.digest("a.hwp")
    args = _as_strings(captured[0])
    assert "--sections" not in args
    assert "--pages" not in args


def test_capabilities_mcp_flag(captured: List[List[Any]]) -> None:
    rhwp.capabilities()
    assert _as_strings(captured[0]) == ["capabilities"]
    rhwp.capabilities(mcp=True)
    assert _as_strings(captured[1]) == ["capabilities", "--mcp"]


# ── 산출 ────────────────────────────────────────────────────────────────


@pytest.mark.parametrize(
    ("func", "command"),
    [
        (rhwp.export_svg, "export-svg"),
        (rhwp.export_pdf, "export-pdf"),
        (rhwp.export_markdown, "export-markdown"),
        (rhwp.export_hml, "export-hml"),
        (rhwp.export_doclang, "export-doclang"),
        (rhwp.thumbnail, "thumbnail"),
    ],
)
def test_output_commands_pass_out_path(
    captured: List[List[Any]], func: Any, command: str
) -> None:
    func("a.hwp", out="out/dir")
    args = _as_strings(captured[0])
    assert args[0] == command
    assert args[args.index("-o") + 1] == "out/dir"
    assert args[-1] == "--json"


def test_output_commands_omit_out_when_absent(captured: List[List[Any]]) -> None:
    rhwp.export_pdf("a.hwp")
    assert "-o" not in _as_strings(captured[0])


def test_export_svg_page_flag(captured: List[List[Any]]) -> None:
    rhwp.export_svg("a.hwp", page=2)
    args = _as_strings(captured[0])
    assert args[args.index("-p") + 1] == "2"


def test_extract_pages_requires_range(captured: List[List[Any]]) -> None:
    rhwp.extract_pages("a.hwp", "2-4", out="b.hwp")
    args = _as_strings(captured[0])
    assert args[args.index("--pages") + 1] == "2-4"


def test_build_from_ingest(captured: List[List[Any]]) -> None:
    rhwp.build_from_ingest("spec.json", out="new.hwp")
    args = _as_strings(captured[0])
    assert args[0] == "build-from-ingest"
    assert args[1] == "spec.json"


# ── 변환·검증 ───────────────────────────────────────────────────────────


def test_export_hwpx_verify_flags(captured: List[List[Any]]) -> None:
    rhwp.export_hwpx("a.hwp", out="b.hwpx", verify=True, verify_pages=True)
    args = _as_strings(captured[0])
    assert "--verify" in args
    assert "--verify-pages" in args


def test_export_hwpx_without_verify(captured: List[List[Any]]) -> None:
    rhwp.export_hwpx("a.hwp")
    args = _as_strings(captured[0])
    assert "--verify" not in args


def test_convert_verify_flag(captured: List[List[Any]]) -> None:
    rhwp.convert("a.hwpx", out="b.hwp", verify=True)
    args = _as_strings(captured[0])
    assert args[0] == "convert"
    assert "--verify" in args


def test_ir_diff_takes_two_paths(captured: List[List[Any]]) -> None:
    rhwp.ir_diff("a.hwp", "b.hwp")
    assert _as_strings(captured[0]) == ["ir-diff", "a.hwp", "b.hwp", "--json"]


# ── 편집 ────────────────────────────────────────────────────────────────


def test_fill_fields_serializes_data_as_json(captured: List[List[Any]]) -> None:
    rhwp.fill_fields("a.hwp", {"성명": "홍길동"}, out="b.hwp")
    args = _as_strings(captured[0])
    payload = json.loads(args[args.index("--data") + 1])
    assert payload == {"성명": "홍길동"}
    # ensure_ascii=False 여야 한글이 그대로 간다 (CLI 가 UTF-8 을 받는다).
    assert "홍길동" in args[args.index("--data") + 1]


def test_fill_fields_all_flags(captured: List[List[Any]]) -> None:
    rhwp.fill_fields("a.hwp", {"a": "b"}, out="c.hwp", dry_run=True, verify=True)
    args = _as_strings(captured[0])
    assert "--dry-run" in args
    assert "--verify" in args


def test_replace_text_occurrence_only_when_given(captured: List[List[Any]]) -> None:
    rhwp.replace_text("a.hwp", "가", "나", out="b.hwp")
    assert "--occurrence" not in _as_strings(captured[0])

    rhwp.replace_text("a.hwp", "가", "나", out="b.hwp", occurrence=3)
    args = _as_strings(captured[1])
    assert args[args.index("--occurrence") + 1] == "3"


def test_replace_text_ignore_case(captured: List[List[Any]]) -> None:
    rhwp.replace_text("a.hwp", "가", "나", ignore_case=True)
    assert "--ignore-case" in _as_strings(captured[0])


def test_set_cell_coordinates(captured: List[List[Any]]) -> None:
    rhwp.set_cell("a.hwpx", 1, 2, 3, "값", out="b.hwpx")
    args = _as_strings(captured[0])
    assert args[args.index("--table") + 1] == "1"
    assert args[args.index("--row") + 1] == "2"
    assert args[args.index("--col") + 1] == "3"
    assert args[args.index("--text") + 1] == "값"


def test_set_cell_keep_style(captured: List[List[Any]]) -> None:
    rhwp.set_cell("a.hwpx", 0, 0, 0, "값", keep_style=True)
    assert "--keep-style" in _as_strings(captured[0])


# ── 대량 ────────────────────────────────────────────────────────────────


def test_batch_streams_paths_through_stdin(monkeypatch: pytest.MonkeyPatch) -> None:
    seen: Dict[str, Any] = {}

    def fake_run_ndjson(args, *, stdin=None, **kwargs):  # type: ignore[no-untyped-def]
        seen["args"] = list(args)
        seen["stdin"] = stdin
        return []

    monkeypatch.setattr(rhwp.commands, "run_ndjson", fake_run_ndjson)
    rhwp.batch("export-text", ["a.hwp", "b.hwp"])

    assert seen["args"][:2] == ["batch", "export-text"]
    assert seen["args"][-1] == "--json"
    assert seen["stdin"] == "a.hwp\nb.hwp\n"


def test_batch_rejects_empty_input() -> None:
    with pytest.raises(ValueError) as caught:
        rhwp.batch("export-text", [])
    assert "최소 1개" in str(caught.value)


def test_batch_accepts_path_objects(monkeypatch: pytest.MonkeyPatch) -> None:
    seen: Dict[str, Any] = {}

    def fake_run_ndjson(args, *, stdin=None, **kwargs):  # type: ignore[no-untyped-def]
        seen["stdin"] = stdin
        return []

    monkeypatch.setattr(rhwp.commands, "run_ndjson", fake_run_ndjson)
    rhwp.batch("export-text", [Path("a.hwp"), Path("b.hwp")])
    assert "a.hwp" in seen["stdin"]


# ── 판정 전달 ───────────────────────────────────────────────────────────


def test_raise_on_verdict_is_forwarded(monkeypatch: pytest.MonkeyPatch) -> None:
    """판정 예외 옵션이 실행 계층까지 전달돼야 한다."""
    seen: Dict[str, Any] = {}

    def fake_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        seen.update(kwargs)
        return {"schemaVersion": "1.0"}

    monkeypatch.setattr(rhwp.commands, "run_json", fake_run_json)
    rhwp.export_hwpx("a.hwp", verify=True, raise_on_verdict=True)
    assert seen["raise_on_verdict"] is True


def test_timeout_is_forwarded(monkeypatch: pytest.MonkeyPatch) -> None:
    seen: Dict[str, Any] = {}

    def fake_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        seen.update(kwargs)
        return {"schemaVersion": "1.0"}

    monkeypatch.setattr(rhwp.commands, "run_json", fake_run_json)
    rhwp.info("a.hwp", timeout=12.5)
    assert seen["timeout"] == 12.5


def test_default_timeout_is_generous() -> None:
    """대형 문서 렌더가 수십 초 걸릴 수 있다 — 기본값이 짧으면 오탐이 난다."""
    assert _process.DEFAULT_TIMEOUT is not None
    assert _process.DEFAULT_TIMEOUT >= 60
