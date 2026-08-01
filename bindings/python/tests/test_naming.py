"""camelCase ↔ snake_case 기계 변환.

수기 개명을 금지하는 것이 요점이다. 규칙이 코드로 고정돼 있어야 봉투에 필드가
늘어도 바인딩이 자동으로 따라온다.
"""

from __future__ import annotations

import pytest

from rhwp._naming import camel_keys, snake_keys, to_camel, to_snake


@pytest.mark.parametrize(
    ("camel", "snake"),
    [
        ("pageCount", "page_count"),
        ("changedPages", "changed_pages"),
        ("schemaVersion", "schema_version"),
        ("sourceA", "source_a"),
        ("sourceB", "source_b"),
        ("pageCountMismatch", "page_count_mismatch"),
        ("irSchemaVersion", "ir_schema_version"),
        ("dryRun", "dry_run"),
        ("notFoundEmpty", "not_found_empty"),
        ("maxDisp", "max_disp"),
        ("structTextrunPm1", "struct_textrun_pm1"),
        ("outputFormat", "output_format"),
        ("filledCount", "filled_count"),
        ("replacedCount", "replaced_count"),
        ("oldText", "old_text"),
        ("keepStyle", "keep_style"),
        ("caseSensitive", "case_sensitive"),
        ("didYouMean", "did_you_mean"),
        ("nextCall", "next_call"),
        # 이미 snake 면 그대로.
        ("already_snake", "already_snake"),
        # 단일 단어.
        ("format", "format"),
        ("pages", "pages"),
    ],
)
def test_to_snake(camel: str, snake: str) -> None:
    assert to_snake(camel) == snake


def test_to_snake_handles_acronym_runs() -> None:
    """연속 대문자 약어의 경계를 살린다."""
    assert to_snake("HTMLPage") == "html_page"
    assert to_snake("exportPDF") == "export_pdf"


@pytest.mark.parametrize(
    ("snake", "camel"),
    [
        ("page_count", "pageCount"),
        ("dry_run", "dryRun"),
        ("changed_pages", "changedPages"),
        ("case_sensitive", "caseSensitive"),
        ("alreadyCamel", "alreadyCamel"),
        ("single", "single"),
    ],
)
def test_to_camel(snake: str, camel: str) -> None:
    assert to_camel(snake) == camel


def test_round_trip_is_stable_for_typical_keys() -> None:
    """봉투 키는 왕복해도 같아야 한다 — 아니면 계획서를 되돌려 보낼 수 없다."""
    for key in ["pageCount", "dryRun", "changedPages", "notFoundEmpty", "caseSensitive"]:
        assert to_camel(to_snake(key)) == key


def test_snake_keys_walks_nested_structures() -> None:
    source = {
        "schemaVersion": "1.0",
        "changedPages": [0, 1],
        "verify": {"diffCount": 0, "identical": True},
        "steps": [{"filledCount": 1, "oldText": "값"}],
    }
    result = snake_keys(source)
    assert result["schema_version"] == "1.0"
    assert result["changed_pages"] == [0, 1]
    assert result["verify"]["diff_count"] == 0
    assert result["steps"][0]["filled_count"] == 1


def test_snake_keys_leaves_values_untouched() -> None:
    """필드 *이름*만 규약을 따른다 — 내용은 봉투 그대로다."""
    source = {"oldText": "camelCase 라는 값", "data": {"회사명": "테스트"}}
    result = snake_keys(source)
    assert result["old_text"] == "camelCase 라는 값"
    # 한글 키는 사용자 데이터다 — 변환 대상이 아니다.
    assert result["data"]["회사명"] == "테스트"


def test_camel_keys_reverses_for_outgoing_payloads() -> None:
    plan = {"plan_version": "1.0", "dry_run": True, "steps": [{"case_sensitive": False}]}
    result = camel_keys(plan)
    assert result["planVersion"] == "1.0"
    assert result["dryRun"] is True
    assert result["steps"][0]["caseSensitive"] is False


def test_empty_string_is_safe() -> None:
    assert to_snake("") == ""
    assert to_camel("") == ""
