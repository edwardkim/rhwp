"""프로세스 실행 계약 — 가짜 바이너리로 각 종료 코드 경로를 검증한다.

봉투 계약을 **신뢰하되 검증한다**. 계약이 깨졌을 때 조용히 넘기면 호출자는
빈 결과를 "차이 없음"으로 오독한다.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rhwp import _binary
from rhwp._process import run_json, run_ndjson, run_raw
from rhwp.errors import ProtocolError, RhwpRuntimeError, UsageError, VerdictFailed


@pytest.fixture(autouse=True)
def _wire_fake(fake_binary: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(_binary.ENV_VAR, str(fake_binary))
    _binary.clear_cache()


def test_success_returns_envelope() -> None:
    env = run_json(["ok"])
    assert env["schemaVersion"] == "1.0"
    assert env["ok"] is True


def test_usage_exit_raises_usage_error_with_hint() -> None:
    with pytest.raises(UsageError) as caught:
        run_json(["usage"])
    assert caught.value.suggestion == "가장 가까운 명령은 'export-svg' 입니다"


def test_runtime_exit_raises_runtime_error() -> None:
    with pytest.raises(RhwpRuntimeError):
        run_json(["runtime"])


def test_verdict_exit_returns_envelope_by_default() -> None:
    """exit 3 은 예외가 아니다 — 판정 근거가 담긴 봉투를 돌려준다."""
    env = run_json(["verdict"])
    assert env["verify"]["identical"] is False
    assert env["verify"]["diffCount"] == 3


def test_verdict_exit_raises_when_requested_and_keeps_envelope() -> None:
    with pytest.raises(VerdictFailed) as caught:
        run_json(["verdict"], raise_on_verdict=True)
    assert caught.value.envelope is not None
    assert caught.value.envelope["verify"]["diffCount"] == 3


def test_page_mismatch_exit_is_flagged() -> None:
    with pytest.raises(VerdictFailed) as caught:
        run_json(["pages"], raise_on_verdict=True)
    assert caught.value.is_page_count_mismatch


def test_non_json_stdout_is_protocol_error() -> None:
    with pytest.raises(ProtocolError) as caught:
        run_json(["garbage"])
    assert "순수 JSON" in str(caught.value)


def test_empty_stdout_on_success_is_protocol_error() -> None:
    """성공했는데 봉투가 없으면 계약 위반이다 — 빈 결과로 넘기면 안 된다."""
    with pytest.raises(ProtocolError) as caught:
        run_json(["empty"])
    assert "비어 있습니다" in str(caught.value)


def test_unknown_exit_code_is_reported() -> None:
    with pytest.raises(RhwpRuntimeError) as caught:
        run_json(["unknown-exit"])
    assert "42" in str(caught.value)


def test_ndjson_returns_every_record() -> None:
    records = run_ndjson(["ndjson"])
    assert len(records) == 3
    assert [r["pageCount"] for r in records] == [1, 2, 3]


def test_ndjson_partial_failure_keeps_successful_records() -> None:
    """부분 실패로 스트림을 통째로 버리면 성공분까지 잃는다."""
    records = run_ndjson(["ndjson-partial"])
    assert len(records) == 2
    assert records[0].get("error") is None
    assert records[1]["error"] == "읽기 실패"


def test_run_raw_exposes_exit_and_streams() -> None:
    result = run_raw(["runtime"], check=False)
    assert result.exit_code == 1
    assert "읽을 수 없습니다" in result.stderr
    assert result.stdout == ""


def test_boolean_argument_is_rejected() -> None:
    """불리언이 값 위치에 오면 CLI 가 못 읽는다 — 호출 조립 버그로 잡는다."""
    with pytest.raises(TypeError) as caught:
        run_raw(["ok", True], check=False)
    assert "플래그" in str(caught.value)


def test_path_arguments_are_stringified(tmp_path: Path) -> None:
    result = run_raw(["ok", tmp_path], check=False)
    assert str(tmp_path) in result.argv


def test_iter_ndjson_is_exported_at_package_root() -> None:
    """[트랙 G R61 D-13] iter_ndjson 이 __all__ 에 없어 공개 API가 아니었다."""
    import rhwp
    from rhwp import _process

    assert "iter_ndjson" in _process.__all__
    assert rhwp.iter_ndjson is _process.iter_ndjson
