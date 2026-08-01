"""NDJSON 스트리밍 — 대량 배치에서 메모리를 아낀다.

`run_ndjson` 은 전량을 모으고, `iter_ndjson` 은 나오는 대로 넘긴다. 후자의 계약은
**소비자가 중간에 멈춰도 자식 프로세스가 남지 않는다**는 것이다. 남으면 파일을
잡고 있어 다음 작업이 막힌다.
"""

from __future__ import annotations

from pathlib import Path
from typing import List

import pytest

from rhwp import _binary
from rhwp._process import iter_ndjson, run_ndjson
from rhwp.errors import ProtocolError


@pytest.fixture(autouse=True)
def _wire_fake(fake_binary: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(_binary.ENV_VAR, str(fake_binary))
    _binary.clear_cache()


def test_iter_yields_records_lazily() -> None:
    seen: List[int] = []
    for record in iter_ndjson(["ndjson"]):
        seen.append(record["pageCount"])
    assert seen == [1, 2, 3]


def test_iter_can_stop_early_without_leaking() -> None:
    """break 로 빠져나가도 자식이 정리돼야 한다."""
    for record in iter_ndjson(["ndjson"]):
        assert record["pageCount"] == 1
        break  # 첫 레코드만 보고 중단
    # 여기까지 왔으면 finally 절이 프로세스를 정리했다는 뜻이다.


def test_iter_surfaces_malformed_lines() -> None:
    with pytest.raises(ProtocolError) as caught:
        list(iter_ndjson(["garbage"]))
    assert "JSON" in str(caught.value)


def test_iter_and_collect_agree() -> None:
    """스트리밍과 일괄 수집이 같은 결과를 내야 한다."""
    streamed = [r["source"] for r in iter_ndjson(["ndjson"])]
    collected = [r["source"] for r in run_ndjson(["ndjson"])]
    assert streamed == collected


def test_iter_includes_error_records() -> None:
    """실패 레코드도 스트림에 남는다 — 조용히 사라지면 누락을 알 수 없다."""
    records = list(iter_ndjson(["ndjson-partial"]))
    assert len(records) == 2
    assert any("error" in r for r in records)


def test_empty_stream_is_not_an_error() -> None:
    """처리할 것이 없는 것과 실패는 다르다."""
    records = list(iter_ndjson(["empty"]))
    assert records == []


def test_collect_returns_dicts_not_envelopes() -> None:
    """배치 레코드는 dict 다 — 봉투로 감싸면 error 키 검사가 번거로워진다."""
    records = run_ndjson(["ndjson"])
    assert all(isinstance(r, dict) for r in records)
