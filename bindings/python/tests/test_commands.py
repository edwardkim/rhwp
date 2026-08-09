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


def test_export_text_page_and_max_chars(captured: List[List[Any]]) -> None:
    """[트랙 G R61 D-12] page/max_chars 가 CLI 에 있었지만 래퍼에 없었다."""
    rhwp.export_text("a.hwp", page=2, max_chars=500)
    args = _as_strings(captured[0])
    assert args[args.index("-p") + 1] == "2"
    assert args[args.index("--max-chars") + 1] == "500"


def test_export_structure_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_structure("a.hwp")
    assert _as_strings(captured[0]) == ["export-structure", "a.hwp", "--json"]


def test_export_structure_mode(captured: List[List[Any]]) -> None:
    """[트랙 G R61 D-12] mode 가 CLI 에 있었지만 래퍼에 없었다."""
    rhwp.export_structure("a.hwp", mode="outline")
    args = _as_strings(captured[0])
    assert args[args.index("--mode") + 1] == "outline"


def test_export_tables_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_tables("a.hwpx")
    assert _as_strings(captured[0]) == ["export-tables", "a.hwpx", "--json"]


def test_table_to_csv_builds_command(captured: List[List[Any]]) -> None:
    rhwp.table_to_csv("a.hwpx", table=7, out="table.csv", bom=True)
    assert _as_strings(captured[0]) == [
        "table-to-csv", "a.hwpx", "--table", "7", "-o", "table.csv", "--bom", "--json",
    ]


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


def test_digest_max_chars(captured: List[List[Any]]) -> None:
    """[트랙 G R61 D-12] max_chars 가 CLI 에 있었지만 래퍼에 없었다."""
    rhwp.digest("a.hwp", max_chars=300)
    args = _as_strings(captured[0])
    assert args[args.index("--max-chars") + 1] == "300"


def test_digest_without_options(captured: List[List[Any]]) -> None:
    rhwp.digest("a.hwp")
    args = _as_strings(captured[0])
    assert "--sections" not in args
    assert "--pages" not in args


def test_extract_data_flags(captured: List[List[Any]]) -> None:
    rhwp.extract_data("a.hwp", kind="amount", limit=5)
    assert _as_strings(captured[0]) == [
        "extract-data", "a.hwp", "--kind", "amount", "--limit", "5", "--json",
    ]


def test_capabilities_mcp_flag(captured: List[List[Any]]) -> None:
    rhwp.capabilities()
    assert _as_strings(captured[0]) == ["capabilities"]
    rhwp.capabilities(mcp=True)
    assert _as_strings(captured[1]) == ["capabilities", "--mcp"]


def test_export_provenance_map_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_provenance_map()
    assert _as_strings(captured[0]) == ["export-provenance-map", "--json"]


def test_explain_builds_command(captured: List[List[Any]]) -> None:
    rhwp.explain("a.hwp")
    assert _as_strings(captured[0]) == ["explain", "a.hwp", "--json"]


def test_export_plan_schema_flags(captured: List[List[Any]]) -> None:
    rhwp.export_plan_schema()
    assert _as_strings(captured[0]) == ["export-plan-schema", "--json"]
    rhwp.export_plan_schema(bare=True, out="plan.json")
    assert _as_strings(captured[1]) == [
        "export-plan-schema", "--bare", "-o", "plan.json", "--json",
    ]


def test_export_agent_manifest_flags(captured: List[List[Any]]) -> None:
    rhwp.export_agent_manifest()
    assert _as_strings(captured[0]) == ["export-agent-manifest", "--json"]
    rhwp.export_agent_manifest(bare=True)
    assert _as_strings(captured[1]) == ["export-agent-manifest", "--bare", "--json"]


def test_export_ontology_builds_command(captured: List[List[Any]]) -> None:
    rhwp.export_ontology()
    assert _as_strings(captured[0]) == ["export-ontology", "--json"]
    rhwp.export_ontology(bare=True, out="onto.jsonld")
    assert _as_strings(captured[1]) == [
        "export-ontology", "--bare", "-o", "onto.jsonld", "--json",
    ]


def test_inspect_builds_each_supported_command(captured: List[List[Any]]) -> None:
    rhwp.inspect("a.hwp", "hidden-text", threshold_pt=0.5, include_offpage=True)
    rhwp.inspect("a.hwp", "injection", min_confidence="high", include_fields=True)
    rhwp.inspect("a.hwp", "unicode", kind="bidi")
    assert _as_strings(captured[0]) == [
        "inspect", "hidden-text", "a.hwp", "--threshold-pt", "0.5", "--include-offpage", "--json",
    ]
    assert _as_strings(captured[1]) == [
        "inspect", "injection", "a.hwp", "--min-confidence", "high", "--include-fields", "--json",
    ]
    assert _as_strings(captured[2]) == [
        "inspect", "unicode", "a.hwp", "--kind", "bidi", "--json",
    ]


def test_inspect_rejects_options_for_another_subcommand(captured: List[List[Any]]) -> None:
    with pytest.raises(ValueError, match="hidden-text"):
        rhwp.inspect("a.hwp", "hidden-text", kind="bidi")


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


def test_export_hwpx_out_is_positional_not_a_flag(captured: List[List[Any]]) -> None:
    # [트랙 G R61 D-1] CLI 는 `export-hwpx <입력> [출력] ...` — 위치 인자다.
    # `-o` 플래그로 조립하면 CLI 가 알 수 없는 옵션으로 거부한다(exit 2).
    rhwp.export_hwpx("a.hwp", out="b.hwpx")
    assert _as_strings(captured[0]) == ["export-hwpx", "a.hwp", "b.hwpx", "--json"]


def test_export_hwpx_without_verify(captured: List[List[Any]]) -> None:
    rhwp.export_hwpx("a.hwp")
    args = _as_strings(captured[0])
    assert "--verify" not in args
    assert "-o" not in args


def test_convert_verify_flag(captured: List[List[Any]]) -> None:
    rhwp.convert("a.hwpx", out="b.hwp", verify=True)
    args = _as_strings(captured[0])
    assert args[0] == "convert"
    assert "--verify" in args


def test_convert_out_is_positional_not_a_flag(captured: List[List[Any]]) -> None:
    # [트랙 G R61 D-1] 같은 결함이 convert 에도 있었다 — `convert <입력> <출력> ...`.
    rhwp.convert("a.hwpx", out="b.hwp")
    assert _as_strings(captured[0]) == ["convert", "a.hwpx", "b.hwp", "--json"]


def test_convert_without_out_raises_usage_error(captured: List[List[Any]]) -> None:
    # [트랙 G R61 D-1] convert 는 산출 경로가 필수다(기본 경로 없음) — Node
    # 바인딩(assertDryRunSupported 와 같은 계열의 선검증)과 동일하게, 프로세스를
    # 띄우기도 전에 UsageError 로 무엇이 빠졌는지 이름으로 알려야 한다.
    with pytest.raises(rhwp.UsageError):
        rhwp.convert("a.hwpx")
    assert captured == []  # 프로세스를 아예 안 띄웠다


def test_ir_diff_takes_two_paths(captured: List[List[Any]]) -> None:
    rhwp.ir_diff("a.hwp", "b.hwp")
    assert _as_strings(captured[0]) == ["ir-diff", "a.hwp", "b.hwp", "--json"]


def test_render_diff_self_roundtrip_when_path_b_omitted(
    captured: List[List[Any]],
) -> None:
    """[트랙 G R61 D-2] Node에는 있었지만 파이썬 바인딩에 없던 명령."""
    rhwp.render_diff("a.hwp")
    assert _as_strings(captured[0]) == ["render-diff", "a.hwp", "--json"]


def test_render_diff_before_after_with_options(captured: List[List[Any]]) -> None:
    rhwp.render_diff("before.hwp", "after.hwp", via="svg", page=2, max_disp=0.5)
    args = _as_strings(captured[0])
    assert args[:3] == ["render-diff", "before.hwp", "after.hwp"]
    assert args[args.index("--via") + 1] == "svg"
    assert args[args.index("-p") + 1] == "2"
    assert args[args.index("--max-disp") + 1] == "0.5"


def test_ir_diff_section_and_paragraph(captured: List[List[Any]]) -> None:
    """[트랙 G R61 D-12] section/paragraph(-s/-p) 가 CLI 에 있었지만 래퍼에 없었다."""
    rhwp.ir_diff("a.hwp", "b.hwp", section=1, paragraph=3)
    args = _as_strings(captured[0])
    assert args[args.index("-s") + 1] == "1"
    assert args[args.index("-p") + 1] == "3"


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


def test_csv_to_table_builds_command(captured: List[List[Any]]) -> None:
    rhwp.csv_to_table(
        "a.hwpx", "values.csv", 7, out="edited.hwpx", dry_run=True, verify=True
    )
    assert _as_strings(captured[0]) == [
        "csv-to-table", "a.hwpx", "--csv", "values.csv", "--table", "7",
        "-o", "edited.hwpx", "--dry-run", "--verify", "--json",
    ]


# ── 대량 ────────────────────────────────────────────────────────────────


def test_scan_builds_command(captured: List[List[Any]]) -> None:
    rhwp.scan("폴더")
    assert _as_strings(captured[0]) == ["scan", "폴더", "--json"]
    rhwp.scan("a", "b", probe=True, max_depth=2, limit=100)
    assert _as_strings(captured[1]) == [
        "scan", "a", "b", "--probe", "--max-depth", "2", "--limit", "100", "--json",
    ]


def test_scan_rejects_empty_input() -> None:
    with pytest.raises(ValueError) as caught:
        rhwp.scan()
    assert "최소 1개" in str(caught.value)


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
