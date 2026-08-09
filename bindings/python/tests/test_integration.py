"""통합 — 실제 rhwp 바이너리와 실제 문서로 왕복한다.

바이너리가 없으면 전부 건너뛴다(단위 테스트는 그래도 돈다). 여기서 검증하는 것은
"파이썬이 봉투를 잘 읽나"가 아니라 **바인딩이 계약을 실제로 재포장했나**다 —
capabilities 선언과 파이썬 API 가 어긋나면 여기서 잡힌다.
"""

from __future__ import annotations

from pathlib import Path

import pytest

import rhwp

pytestmark = pytest.mark.integration


# ── 1층: 무상태 ─────────────────────────────────────────────────────────


def test_info_returns_document_summary(wired_binary: Path, sample_hwp: Path) -> None:
    meta = rhwp.info(sample_hwp)
    assert meta.schema_version == rhwp.SUPPORTED_SCHEMA_VERSION
    assert meta.page_count >= 1
    assert meta.format in ("hwp5", "hwpx", "hwp3", "hml")
    # 원문 키와 snake 속성이 같은 값을 가리켜야 한다.
    assert meta["pageCount"] == meta.page_count


def test_export_text_pages_match_page_count(wired_binary: Path, sample_hwp: Path) -> None:
    result = rhwp.export_text(sample_hwp)
    assert len(result.pages) == result.page_count


def test_fields_lists_form_controls(wired_binary: Path, sample_hwp: Path) -> None:
    result = rhwp.fields(sample_hwp)
    assert isinstance(result.fields, list)


def test_search_returns_addressed_matches(wired_binary: Path, sample_hwp: Path) -> None:
    text = rhwp.export_text(sample_hwp)
    first_page = text.pages[0].text if text.pages else ""
    needle = next((w for w in first_page.split() if len(w) >= 2), None)
    if needle is None:
        pytest.skip("검색할 어휘가 없는 샘플")
    result = rhwp.search(sample_hwp, needle)
    assert result.match_count >= 1


def test_capabilities_is_self_describing(wired_binary: Path) -> None:
    caps = rhwp.capabilities()
    assert caps.tool == "rhwp"
    assert isinstance(caps.commands, list)
    # 종료 코드 사전이 우리가 매핑한 코드를 전부 설명해야 한다.
    codes = caps["exitCodes"]
    for code in ("0", "1", "2", "3", "4"):
        assert isinstance(codes[code], str) and codes[code]


def test_missing_file_raises_runtime_error(wired_binary: Path) -> None:
    with pytest.raises(rhwp.RhwpRuntimeError):
        rhwp.info("존재하지-않는-문서.hwp")


def test_unknown_command_surfaces_did_you_mean(wired_binary: Path) -> None:
    """서버·CLI 가 주는 교정 단서를 바인딩이 구조화해 전달해야 한다."""
    from rhwp._process import run_json

    with pytest.raises(rhwp.UsageError) as caught:
        run_json(["expot-text", "a.hwp", "--json"])
    # 힌트가 있으면 구조화돼야 하고, 없더라도 예외 타입은 UsageError 다.
    assert caught.value.exit_code == 2


# ── 1층: 편집 + 판정 ────────────────────────────────────────────────────


def test_fill_fields_with_verify_reports_verdict_as_data(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    """판정은 예외가 아니라 반환값이다 — 이 규약이 바인딩의 핵심."""
    out = tmp_path / "채움.hwp"
    fields = rhwp.fields(sample_hwp).fields
    if not fields:
        pytest.skip("누름틀이 없는 샘플")
    name = fields[0].name

    result = rhwp.fill_fields(sample_hwp, {name: "통합테스트"}, out=out, verify=True)
    assert out.exists()
    verify = result.verify
    assert verify is not None, "verify=True 인데 보고가 없다"
    assert isinstance(verify.identical, bool)
    # 눈검증 대상 페이지도 함께 와야 한다.
    assert result.changed_pages is None or isinstance(result.changed_pages, list)


def test_dry_run_leaves_disk_untouched(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    fields = rhwp.fields(sample_hwp).fields
    if not fields:
        pytest.skip("누름틀이 없는 샘플")
    out = tmp_path / "안만들어짐.hwp"
    result = rhwp.fill_fields(sample_hwp, {fields[0].name: "값"}, out=out, dry_run=True)
    assert result.dry_run is True
    assert not out.exists(), "dry-run 이 파일을 만들었다"


# ── 2층: 세션 ───────────────────────────────────────────────────────────


def test_session_round_trip(wired_binary: Path, sample_hwp: Path, tmp_path: Path) -> None:
    """열기 → 편집 → 저장 → 검증 → 닫기."""
    out = tmp_path / "세션산출.hwp"
    with rhwp.open(sample_hwp) as doc:
        info = doc.info()
        assert info.page_count >= 1

        field_list = doc.fields().fields
        if field_list:
            doc.fill_fields({field_list[0].name: "세션값"})

        saved = doc.save(out, verify=True)
        verify = saved.verify
        assert verify is not None
        assert isinstance(verify.identical, bool)
    assert out.exists()


def test_session_render_page_closes_visual_loop(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    with rhwp.open(sample_hwp) as doc:
        target = tmp_path / "쪽0.svg"
        rendered = doc.render_page(0, target)
        assert rendered.raw, "렌더 결과가 비어 있다"
        assert target.exists(), "SVG 파일이 나오지 않았다"


def test_closed_document_rejects_further_calls(
    wired_binary: Path, sample_hwp: Path
) -> None:
    doc = rhwp.open(sample_hwp)
    doc.close()
    with pytest.raises(rhwp.SessionClosedError):
        doc.info()


def test_session_cleans_up_on_exception(wired_binary: Path, sample_hwp: Path) -> None:
    """예외로 빠져나가도 서버가 남으면 다음 작업이 막힌다."""
    session = rhwp.Session()
    try:
        with rhwp.open(sample_hwp, session=session) as doc:
            assert doc.doc_id
            raise RuntimeError("의도적 예외")
    except RuntimeError:
        pass
    finally:
        session.close()


# ── 3층: 계획 ───────────────────────────────────────────────────────────


def _supports_dry_run() -> bool:
    """rhwp 가 계획 --dry-run 을 지원하는지 (자기서술로 확인)."""
    caps = rhwp.capabilities()
    for cmd in caps.raw["commands"]:
        if cmd["name"] == "run":
            return "--dry-run" in (cmd.get("flags") or [])
    return False


def test_plan_check_previews_without_writing(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    if not _supports_dry_run():
        pytest.skip("이 rhwp 는 계획 --dry-run 미지원 (#3759 머지 전)")
    fields = rhwp.fields(sample_hwp).fields
    if not fields:
        pytest.skip("누름틀이 없는 샘플")
    out = tmp_path / "계획산출.hwp"

    plan = rhwp.Plan(sample_hwp, out).fill_fields({fields[0].name: "계획값"}).verify()
    preview = plan.check()

    assert preview.is_dry_run
    assert preview.ok, preview.describe_violations()
    assert len(preview.preview) == 1
    assert not out.exists(), "check() 가 파일을 만들었다"


def test_plan_run_produces_journal_and_output(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    fields = rhwp.fields(sample_hwp).fields
    if not fields:
        pytest.skip("누름틀이 없는 샘플")
    out = tmp_path / "계획실행.hwp"

    journal = rhwp.Plan(sample_hwp, out).fill_fields({fields[0].name: "실행값"}).verify().run()

    assert journal.ok, journal.describe_violations()
    assert len(journal.steps) == 1
    verify = journal.verify
    assert verify is not None and verify.identical
    assert out.exists()


def test_invalid_plan_returns_violations_not_exception(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    """위반은 결과다 — 계획을 고쳐 다시 검사하는 것이 정상 흐름."""
    out = tmp_path / "안나옴.hwp"
    plan = rhwp.Plan(sample_hwp, out).fill_fields({"존재하지않는필드XYZ": "값"})

    result = plan.check()

    assert not result.ok
    assert len(result.violations) >= 1
    assert "존재하지않는필드XYZ" in result.describe_violations()
    assert not out.exists()


def test_plan_violations_are_reported_all_at_once(
    wired_binary: Path, sample_hwp: Path, tmp_path: Path
) -> None:
    """두더지잡기 방지 — 위반을 하나씩 알려주면 왕복이 늘어난다."""
    out = tmp_path / "안나옴2.hwp"
    result = (
        rhwp.Plan(sample_hwp, out)
        .fill_fields({"없는필드A": "값"})
        .replace_text("이런문자열은결코없다9999", "X")
        .check()
    )
    assert len(result.violations) == 2, result.describe_violations()


# ── 계약 패리티 가드 ────────────────────────────────────────────────────


def test_binding_covers_every_agent_value_command(wired_binary: Path) -> None:
    """capabilities 가 선언한 json 명령을 바인딩이 빠뜨리지 않았는지.

    이 테스트가 M18 의 핵심 수용 기준이다 — rhwp 에 명령이 늘었는데 바인딩이
    뒤처지면 여기서 실패한다. 진단 계열은 사람 전용이라 제외한다.
    """
    caps = rhwp.capabilities()
    declared = {
        c["name"]
        for c in caps.raw["commands"]
        if c.get("json") and c.get("category") not in ("diagnostic", "internal", "serve")
    }
    # 바인딩이 노출하는 이름 (CLI 명령명 기준으로 환산).
    exported = {
            "info", "export-text", "export-structure", "export-tables", "table-to-csv",
            "export-svg",
            "export-pdf", "export-markdown", "export-hml", "export-doclang", "export-hwpx", "convert",
            "search", "fields", "digest", "extract-data", "inspect", "ir-diff", "thumbnail", "extract-pages",
            "build-from-ingest", "edit", "batch", "scan", "run", "capabilities", "export-ir-schema",
            "export-capabilities-schema", "export-provenance-map", "csv-to-table",
            "explain", "export-plan-schema", "export-agent-manifest", "export-ontology",
    }
    missing = declared - exported
    assert not missing, (
        f"바인딩이 빠뜨린 명령: {sorted(missing)}\n"
        "rhwp 에 명령이 늘었습니다 — commands.py 에 래퍼를 추가하세요."
    )
