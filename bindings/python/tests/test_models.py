"""봉투 모델 — 세 가지 접근 방식과 "모름 vs 없음" 구분.

가장 중요한 계약: **오타가 조용한 ``None`` 이 되면 안 된다.** 없는 필드를 물으면
실패해야 한다. 그렇지 않으면 필드 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가
가장 찾기 어려운 버그가 된다.
"""

from __future__ import annotations

import pytest

from rhwp.models import Envelope, VerifyReport


def test_three_ways_to_reach_the_same_value() -> None:
    env = Envelope({"pageCount": 3, "schemaVersion": "1.0"})
    assert env.page_count == 3           # 속성 (snake)
    assert env["pageCount"] == 3         # 원문 키
    assert env["page_count"] == 3        # 변환 키


def test_missing_field_fails_loudly_not_silently() -> None:
    env = Envelope({"pageCount": 3})
    with pytest.raises(AttributeError) as caught:
        _ = env.page_conut  # 오타
    # 있는 필드를 알려줘야 사용자가 고칠 수 있다.
    assert "pageCount" in str(caught.value)

    with pytest.raises(KeyError):
        _ = env["없는필드"]


def test_envelope_is_read_only() -> None:
    """도구가 내놓은 판정을 호출자가 고치면 안 된다."""
    env = Envelope({"pageCount": 3})
    with pytest.raises(AttributeError):
        env.page_count = 5  # type: ignore[misc]


def test_nested_mappings_are_wrapped() -> None:
    env = Envelope({"verify": {"diffCount": 0, "identical": True}})
    assert isinstance(env["verify"], Envelope)
    assert env["verify"].diff_count == 0


def test_lists_of_objects_are_wrapped_per_item() -> None:
    env = Envelope({"pages": [{"pageNo": 0, "text": "가"}, {"pageNo": 1, "text": "나"}]})
    pages = env.pages
    assert all(isinstance(p, Envelope) for p in pages)
    assert pages[1].page_no == 1


def test_verify_property_distinguishes_absent_from_failed() -> None:
    """``None``(검증 안 함)과 실패를 섞으면 검증 없는 저장을 통과로 읽는다."""
    not_requested = Envelope({"output": "a.hwp", "verify": None})
    assert not_requested.verify is None

    failed = Envelope({"verify": {"identical": False, "diffCount": 2}})
    report = failed.verify
    assert isinstance(report, VerifyReport)
    assert report.identical is False
    assert report.diff_count == 2
    assert not report  # __bool__ 이 판정을 대변한다


def test_verify_reparse_error_is_surfaced() -> None:
    """재파싱 실패는 '판정 불가'가 아니라 실패로 보고된다."""
    env = Envelope({"verify": {"identical": False, "diffCount": None, "reparseError": "손상"}})
    report = env.verify
    assert report is not None
    assert report.identical is False
    assert report.diff_count is None
    assert report.reparse_error == "손상"


def test_verify_pages_property_distinguishes_absent_from_failed() -> None:
    """[트랙 G R61 D-10] verify 와 대칭인 verifyPages 접근자가 없었다."""
    from rhwp.models import VerifyPagesReport

    not_requested = Envelope({"output": "a.hwp", "verifyPages": None})
    assert not_requested.verify_pages is None

    failed = Envelope({"verifyPages": {"before": 5, "after": 4, "identical": False}})
    report = failed.verify_pages
    assert isinstance(report, VerifyPagesReport)
    assert report.before == 5
    assert report.after == 4
    assert report.identical is False
    assert not report  # __bool__ 이 판정을 대변한다

    passed = Envelope({"verifyPages": {"before": 5, "after": 5, "identical": True}})
    assert bool(passed.verify_pages)


def test_changed_pages_distinguishes_unknown_from_empty() -> None:
    """``None``(모름)과 ``[]``(바뀐 쪽 없음)은 다른 결론이다."""
    unknown = Envelope({"changedPages": None})
    assert unknown.changed_pages is None

    none_changed = Envelope({"changedPages": []})
    assert none_changed.changed_pages == []

    changed = Envelope({"changedPages": [0, 2]})
    assert changed.changed_pages == [0, 2]


def test_get_path_walks_dotted_paths() -> None:
    env = Envelope({"verify": {"identical": True}, "steps": [{"action": "fill_fields"}]})
    assert env.get_path("verify.identical") is True
    assert env.get_path("verify.없음", "기본값") == "기본값"
    assert env.get_path("없는.경로") is None


def test_raw_returns_a_copy_not_the_original() -> None:
    """원문을 돌려주되, 그걸 고쳐도 봉투는 안 바뀌어야 한다."""
    env = Envelope({"pageCount": 3})
    raw = env.raw
    raw["pageCount"] = 99
    assert env.page_count == 3


def test_mapping_protocol_works() -> None:
    env = Envelope({"a": 1, "b": 2})
    assert len(env) == 2
    assert set(env) == {"a", "b"}
    assert dict(env) == {"a": 1, "b": 2}


def test_non_mapping_input_is_rejected() -> None:
    with pytest.raises(TypeError):
        Envelope([1, 2, 3])  # type: ignore[arg-type]
