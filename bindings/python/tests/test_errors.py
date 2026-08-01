"""종료 코드 → 예외 매핑 계약.

이 파일이 지키는 핵심 구분: **판정 실패는 고장이 아니다.**
exit 3/4 를 기본으로 예외로 만들면 호출자가 봉투의 판정 근거를 안 읽는다.
"""

from __future__ import annotations

import pytest

from rhwp.errors import (
    EXIT_OK,
    EXIT_RUNTIME,
    EXIT_USAGE,
    EXIT_VERIFY,
    EXIT_VERIFY_PAGES,
    RhwpRuntimeError,
    UsageError,
    VerdictFailed,
    raise_for_exit,
)


def test_success_raises_nothing() -> None:
    raise_for_exit(EXIT_OK, argv=["rhwp", "info", "a.hwp"])


def test_usage_error_maps_to_usage_exception() -> None:
    with pytest.raises(UsageError) as caught:
        raise_for_exit(EXIT_USAGE, argv=["rhwp", "expot-svg"], stderr="오류: 알 수 없는 명령")
    assert caught.value.exit_code == EXIT_USAGE


def test_runtime_error_maps_to_runtime_exception() -> None:
    with pytest.raises(RhwpRuntimeError):
        raise_for_exit(EXIT_RUNTIME, argv=["rhwp", "info", "없음.hwp"], stderr="오류: 읽기 실패")


@pytest.mark.parametrize("code", [EXIT_VERIFY, EXIT_VERIFY_PAGES])
def test_verdict_is_not_an_exception_by_default(code: int) -> None:
    """판정 실패의 기본은 조용한 통과 — 호출자가 봉투를 읽어 판단한다."""
    raise_for_exit(code, argv=["rhwp", "export-hwpx", "a.hwp", "--verify"])


@pytest.mark.parametrize("code", [EXIT_VERIFY, EXIT_VERIFY_PAGES])
def test_verdict_raises_when_explicitly_requested(code: int) -> None:
    envelope = {"verify": {"identical": False, "diffCount": 7}}
    with pytest.raises(VerdictFailed) as caught:
        raise_for_exit(
            code,
            argv=["rhwp", "export-hwpx", "a.hwp", "--verify"],
            envelope=envelope,
            raise_on_verdict=True,
        )
    # 판정 근거가 예외에 실려 있어야 한다 — 없으면 왜 실패했는지 알 수 없다.
    assert caught.value.envelope == envelope
    assert caught.value.is_page_count_mismatch == (code == EXIT_VERIFY_PAGES)


def test_unknown_exit_code_is_not_silently_passed() -> None:
    """모르는 코드를 통과시키면 실패한 작업이 성공으로 보고된다."""
    with pytest.raises(RhwpRuntimeError) as caught:
        raise_for_exit(42, argv=["rhwp", "info", "a.hwp"])
    assert "42" in str(caught.value)


def test_usage_error_extracts_did_you_mean_hint() -> None:
    """did-you-mean 힌트를 구조화해 꺼낼 수 있어야 한다."""
    with pytest.raises(UsageError) as caught:
        raise_for_exit(
            EXIT_USAGE,
            argv=["rhwp", "expot-svg"],
            stderr="오류: 알 수 없는 명령입니다\n힌트: 가장 가까운 명령은 'export-svg' 입니다",
        )
    assert caught.value.suggestion == "가장 가까운 명령은 'export-svg' 입니다"


def test_usage_error_without_hint_returns_none() -> None:
    with pytest.raises(UsageError) as caught:
        raise_for_exit(EXIT_USAGE, argv=["rhwp"], stderr="오류: 인자가 필요합니다")
    assert caught.value.suggestion is None


def test_error_carries_reproducible_command() -> None:
    """버그 리포트에 그대로 붙일 수 있는 명령 문자열."""
    with pytest.raises(RhwpRuntimeError) as caught:
        raise_for_exit(
            EXIT_RUNTIME, argv=["rhwp", "info", "공백 있는 파일.hwp", "--json"]
        )
    command = caught.value.command
    assert "rhwp info" in command
    # 공백이 있는 인자는 따옴표로 감싸야 붙여넣기가 가능하다.
    assert '"공백 있는 파일.hwp"' in command


def test_error_str_includes_last_stderr_line() -> None:
    """가장 구체적인 진단(마지막 줄)이 메시지에 보여야 한다."""
    with pytest.raises(RhwpRuntimeError) as caught:
        raise_for_exit(
            EXIT_RUNTIME,
            argv=["rhwp", "info", "a.hwp"],
            stderr="첫 줄\n오류: 진짜 사유는 여기",
        )
    assert "진짜 사유는 여기" in str(caught.value)
