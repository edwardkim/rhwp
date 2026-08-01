"""봉투 패리티 — 자기서술과 실제 봉투가 어긋나지 않는지.

`capabilities` 는 "에이전트가 도구 정의를 자동 생성하는 원천"이다. 그 선언과 실제
봉투가 어긋나면, 선언을 읽고 만든 코드가 실행 시점에 깨진다. 바인딩은 그 선언의
첫 번째 소비자이므로 여기서 어긋남을 잡는 것이 자연스럽다.

전부 통합 테스트다 — 실물 봉투를 봐야 대조할 수 있다.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Set

import pytest

import rhwp

pytestmark = pytest.mark.integration


def _declared_commands(caps: rhwp.Envelope) -> Dict[str, Dict[str, Any]]:
    return {c["name"]: c for c in caps.raw["commands"]}


def test_every_json_command_declares_record_fields(wired_binary: Path) -> None:
    """`--json` 을 낸다고 선언했으면 어떤 필드가 나오는지도 말해야 한다."""
    caps = rhwp.capabilities()
    missing: List[str] = []
    for name, cmd in _declared_commands(caps).items():
        if not cmd.get("json"):
            continue
        fields = cmd.get("recordFields")
        if not isinstance(fields, list) or not fields:
            missing.append(name)
    assert not missing, f"recordFields 를 선언하지 않은 json 명령: {missing}"


def test_declared_fields_actually_appear(wired_binary: Path, sample_hwp: Path) -> None:
    """선언한 필드가 실제 봉투에 나오는지 — 대표 명령 몇 개로 대조한다.

    전 명령을 도는 대신 대표를 고른 이유: 산출 명령은 파일을 만들고, 변환 명령은
    시간이 오래 걸린다. 조회 계열이 계약 검증에 충분하다.
    """
    caps = rhwp.capabilities()
    declared = _declared_commands(caps)

    checks = [
        ("info", rhwp.info(sample_hwp)),
        ("export-text", rhwp.export_text(sample_hwp)),
        ("fields", rhwp.fields(sample_hwp)),
        ("export-structure", rhwp.export_structure(sample_hwp)),
    ]

    problems: List[str] = []
    for name, envelope in checks:
        spec = declared.get(name)
        if spec is None:
            problems.append(f"{name}: capabilities 에 없음")
            continue
        actual: Set[str] = set(envelope.raw)
        for field in spec.get("recordFields") or []:
            # 중첩 경로(`steps[].confusable` 같은)는 최상위 대조 대상이 아니다.
            if "[" in field or "." in field:
                continue
            if field not in actual:
                problems.append(f"{name}: 선언한 '{field}' 가 봉투에 없음 (실제: {sorted(actual)})")
    assert not problems, "\n".join(problems)


def test_every_envelope_carries_schema_version(
    wired_binary: Path, sample_hwp: Path
) -> None:
    """봉투 계약의 최소 조건 — 버전 없이는 소비자가 진화를 따라갈 수 없다."""
    for envelope in (
        rhwp.info(sample_hwp),
        rhwp.export_text(sample_hwp),
        rhwp.fields(sample_hwp),
        rhwp.export_structure(sample_hwp),
        rhwp.capabilities(),
    ):
        assert envelope.schema_version == rhwp.SUPPORTED_SCHEMA_VERSION, (
            f"바인딩이 검증한 버전({rhwp.SUPPORTED_SCHEMA_VERSION})과 다르다: "
            f"{envelope.schema_version}"
        )


def test_declared_flags_are_accepted_by_the_tool(wired_binary: Path, sample_hwp: Path) -> None:
    """선언한 플래그가 실제로 먹히는지 — 조회 계열 대표로 확인한다.

    선언에만 있고 구현에 없는 플래그는 "쓸 수 있다"고 읽혀 호출자를 오도한다.
    """
    from rhwp._process import run_raw

    caps = rhwp.capabilities()
    declared = _declared_commands(caps)
    spec = declared.get("info")
    assert spec is not None

    for flag in spec.get("flags") or []:
        if flag != "--json":
            continue
        result = run_raw(["info", sample_hwp, flag], check=False)
        assert result.exit_code == 0, f"선언한 플래그 {flag} 가 거부됐다: {result.stderr}"


def test_exit_code_dictionary_covers_binding_constants(wired_binary: Path) -> None:
    """바인딩이 매핑하는 코드를 도구가 전부 설명해야 한다."""
    codes = rhwp.capabilities()["exitCodes"]
    for constant in (
        rhwp.EXIT_OK,
        rhwp.EXIT_RUNTIME,
        rhwp.EXIT_USAGE,
        rhwp.EXIT_VERIFY,
        rhwp.EXIT_VERIFY_PAGES,
    ):
        description = codes[str(constant)]
        assert isinstance(description, str) and description.strip(), (
            f"exit {constant} 설명이 비어 있다"
        )


def test_mcp_tools_point_at_real_commands(wired_binary: Path) -> None:
    """MCP 도구가 가리키는 CLI 명령이 실존해야 한다 (선언-실행 단일 출처)."""
    caps = rhwp.capabilities()
    mcp = rhwp.capabilities(mcp=True)
    command_names = set(_declared_commands(caps))

    dangling: List[str] = []
    for tool in mcp.raw["tools"]:
        target = (tool.get("cli") or {}).get("command")
        if target and target not in command_names:
            dangling.append(f"{tool['name']} → {target}")
    assert not dangling, f"실존하지 않는 명령을 가리키는 도구: {dangling}"


def test_session_tools_are_not_duplicated_in_stateless_manifest(
    wired_binary: Path,
) -> None:
    """세션 도구와 무상태 도구는 다른 층이다 — 섞이면 소비자가 혼동한다."""
    mcp = rhwp.capabilities(mcp=True)
    stateless = {t["name"] for t in mcp.raw["tools"]}
    session_prefixes = ("hwp_open", "hwp_close", "hwp_doc_")
    leaked = [n for n in stateless if n.startswith(session_prefixes)]
    assert not leaked, f"세션 도구가 무상태 매니페스트에 섞였다: {leaked}"


def test_binding_error_mapping_matches_tool_behaviour(wired_binary: Path) -> None:
    """실제 실패 상황이 우리가 매핑한 예외로 올라오는지."""
    # 없는 파일 → 런타임 실패 (인자를 고쳐도 안 풀린다)
    with pytest.raises(rhwp.RhwpRuntimeError):
        rhwp.info("존재하지-않는-파일-parity.hwp")

    # 알 수 없는 명령 → 사용법 오류 (호출 조립 버그)
    with pytest.raises(rhwp.UsageError):
        rhwp.run_json(["존재하지않는명령", "--json"])


def test_unknown_field_is_reported_not_raised(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    """편집 계층과 계획 계층의 계약 차이 — 섞으면 오작동한다.

    `edit fill-fields` 는 없는 누름틀을 **오류가 아니라 `notFound` 로 보고**한다
    (일부만 채우는 것이 유효한 사용이기 때문). 반면 계획 실행기는 같은 상황을
    **선검증 위반**으로 잡아 실행 자체를 막는다 — 계획은 전부 아니면 전무이므로.

    바인딩은 두 계약을 그대로 전달하고 하나로 통일하지 않는다.
    """
    out = tmp_path / "부분채움.hwp"
    result = rhwp.fill_fields(sample_hwp, {"절대로존재하지않는필드XYZ": "값"}, out=out)
    assert "절대로존재하지않는필드XYZ" in result.not_found

    # 같은 상황을 계획으로 돌리면 실행 전에 막힌다.
    plan_result = rhwp.Plan(sample_hwp, tmp_path / "안나옴.hwp").fill_fields(
        {"절대로존재하지않는필드XYZ": "값"}
    ).run()
    assert not plan_result.ok, "계획은 선검증에서 막아야 한다"
    assert not (tmp_path / "안나옴.hwp").exists()


def test_ir_schema_version_is_declared_in_capabilities(wired_binary: Path) -> None:
    """IR 스키마 명령이 자기서술에 등재돼 있어야 바인딩이 찾을 수 있다."""
    caps = rhwp.capabilities()
    spec = _declared_commands(caps).get("export-ir-schema")
    assert spec is not None, "export-ir-schema 가 capabilities 에 없다"
    assert spec.get("json") is True
    fields = spec.get("recordFields") or []
    assert "irSchemaVersion" in fields
    assert "schema" in fields

    # MCP 도구로도 노출돼야 한다 — 에이전트가 문서 모델을 물어볼 수 있어야 하고,
    # 드리프트 가드(`capabilities_mcp_covers_every_json_command`)도 이를 요구한다.
    mcp = rhwp.capabilities(mcp=True)
    tool = next(
        (t for t in mcp.raw["tools"] if t["name"] == "hwp_export_ir_schema"), None
    )
    assert tool is not None, "hwp_export_ir_schema MCP 도구가 없다"
    assert tool["cli"]["command"] == "export-ir-schema"
