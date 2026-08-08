"""계획 빌더 — 문법 검사와 직렬화.

빌더는 **문법만** 검사한다(값 타입·필수 인자). 실행 가능성은 rhwp 의 선검증이
판정한다 — 판정자를 두 곳에 두면 반드시 어긋난다.
"""

from __future__ import annotations

from typing import Any, List

import pytest

import rhwp
import rhwp.commands
import rhwp.plan
from rhwp.plan import Plan, PlanResult, clear_plan_capability_cache


@pytest.fixture(autouse=True)
def _reset_dry_run_cache() -> None:
    # [트랙 G R61 D-4] _dry_run_support 는 모듈 전역 캐시라 시험 간 오염을 막는다.
    clear_plan_capability_cache()


def test_builder_produces_contract_shaped_plan() -> None:
    plan = (
        Plan("서식.hwp", "제출본.hwp")
        .fill_fields({"성명": "홍길동"})
        .replace_text("2025년", "2026년")
        .set_cell(1, 0, 0, "값")
        .set_checkbox(1)
        .verify()
    )
    payload = plan.to_dict()

    assert payload["planVersion"] == "1.0"
    assert payload["input"] == "서식.hwp"
    assert payload["output"] == "제출본.hwp"
    assert payload["assertions"]["verify"] is True
    actions = [s["action"] for s in payload["steps"]]
    assert actions == ["fill_fields", "replace_text", "set_cell", "set_checkbox"]


def test_dry_run_flag_lives_in_the_plan_itself() -> None:
    """계획서가 dryRun 을 실으므로 MCP 경로도 인자 추가 없이 같은 계약을 얻는다."""
    plan = Plan("a.hwp", "b.hwp").fill_fields({"이름": "값"})
    assert "dryRun" not in plan.to_dict()
    assert plan.to_dict(dry_run=True)["dryRun"] is True


def test_occurrence_is_serialized_only_when_given() -> None:
    with_occ = Plan("a.hwp", "b.hwp").replace_text("가", "나", occurrence=2).to_dict()
    assert with_occ["steps"][0]["occurrence"] == 2

    without = Plan("a.hwp", "b.hwp").replace_text("가", "나").to_dict()
    assert "occurrence" not in without["steps"][0]


def test_empty_plan_is_rejected() -> None:
    with pytest.raises(ValueError) as caught:
        Plan("a.hwp", "b.hwp").to_dict()
    assert "step" in str(caught.value)


def test_fill_fields_rejects_empty_mapping() -> None:
    with pytest.raises(ValueError):
        Plan("a.hwp", "b.hwp").fill_fields({})


def test_replace_text_rejects_empty_find() -> None:
    with pytest.raises(ValueError):
        Plan("a.hwp", "b.hwp").replace_text("", "값")


def test_replace_text_rejects_non_string_replace() -> None:
    with pytest.raises(TypeError):
        Plan("a.hwp", "b.hwp").replace_text("가", 123)  # type: ignore[arg-type]


def test_set_cell_rejects_newlines() -> None:
    """셀은 한 줄 값이다 — CLI 선검증과 같은 규칙을 빌더에서도 즉시 잡는다."""
    with pytest.raises(ValueError) as caught:
        Plan("a.hwp", "b.hwp").set_cell(0, 0, 0, "두\n줄")
    assert "줄바꿈" in str(caught.value)


@pytest.mark.parametrize("bad", [-1, "0", 1.5])
def test_set_cell_rejects_bad_coordinates(bad: object) -> None:
    with pytest.raises((ValueError, TypeError)):
        Plan("a.hwp", "b.hwp").set_cell(bad, 0, 0, "값")  # type: ignore[arg-type]


def test_set_checkbox_rejects_negative_occurrence() -> None:
    with pytest.raises(ValueError):
        Plan("a.hwp", "b.hwp").set_checkbox(-1)


def test_chaining_returns_same_builder() -> None:
    plan = Plan("a.hwp", "b.hwp")
    assert plan.fill_fields({"a": "b"}) is plan


# ── PlanResult 판정 ─────────────────────────────────────────────────────


def test_result_reports_violations_as_data() -> None:
    """위반은 예외가 아니라 결과다 — 고쳐서 다시 검사하는 것이 정상 흐름."""
    result = PlanResult(
        {
            "schemaVersion": "1.0",
            "invalid": [
                {"step": 0, "action": "fill_fields", "reason": "필드 '없음' 이(가) 없습니다"},
                {"step": 1, "action": "replace_text", "reason": "'X' 일치 0건"},
            ],
        }
    )
    assert not result.ok
    assert len(result.violations) == 2
    described = result.describe_violations()
    assert "step 0" in described
    assert "일치 0건" in described


def test_result_ok_when_no_violations() -> None:
    result = PlanResult({"schemaVersion": "1.0", "steps": [{"step": 0}]})
    assert result.ok
    assert result.describe_violations() == "위반 없음"


def test_dry_run_result_exposes_preview_not_steps() -> None:
    result = PlanResult(
        {
            "schemaVersion": "1.0",
            "dryRun": True,
            "invalid": [],
            "preview": [{"step": 0, "action": "replace_text", "matches": 7, "willReplace": 7}],
        }
    )
    assert result.is_dry_run
    assert result.ok
    assert len(result.preview) == 1
    assert result.preview[0].matches == 7
    assert result.steps == []


def test_run_result_exposes_steps_and_verify() -> None:
    result = PlanResult(
        {
            "schemaVersion": "1.0",
            "steps": [{"step": 0, "action": "fill_fields", "filledCount": 1}],
            "verify": {"identical": True, "diffCount": 0},
            "changedPages": [0],
        }
    )
    assert not result.is_dry_run
    assert result.steps[0].filled_count == 1
    verify = result.verify
    assert verify is not None and verify.identical
    assert result.changed_pages == [0]


# ── check() 의 --dry-run 지원 게이트 (트랙 G R61 D-4) ─────────────────────


def _fake_capabilities(*, supports_dry_run: bool):  # type: ignore[no-untyped-def]
    def fake_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        flags = ["--json", "--plan-json"]
        if supports_dry_run:
            flags.append("--dry-run")
        return {
            "schemaVersion": "1.0",
            "commands": [{"name": "run", "flags": flags}],
        }

    return fake_run_json


def test_check_proceeds_when_binary_declares_dry_run_support(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: List[List[Any]] = []

    def fake_capabilities_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        calls.append(list(args))
        return _fake_capabilities(supports_dry_run=True)(args, **kwargs)

    def fake_plan_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        calls.append(list(args))
        return {"schemaVersion": "1.0", "dryRun": True, "invalid": [], "preview": []}

    monkeypatch.setattr(rhwp.commands, "run_json", fake_capabilities_run_json)
    monkeypatch.setattr(rhwp.plan, "run_json", fake_plan_run_json)

    result = Plan("a.hwp", "b.hwp").fill_fields({"이름": "값"}).check()
    assert result.is_dry_run
    assert len(calls) == 2  # capabilities 1회 + run --plan-json 1회


def test_check_raises_and_does_not_execute_when_dry_run_unsupported(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # [D-4] 옛 바이너리(#3759 이전)는 dryRun 필드를 무시하고 진짜로 실행할 수
    # 있다 — 확인 없이 넘기면 "검사만 한 줄" 알았던 호출이 문서를 편집한다.
    plan_run_json_calls: List[List[Any]] = []

    monkeypatch.setattr(
        rhwp.commands, "run_json", _fake_capabilities(supports_dry_run=False)
    )

    def fake_plan_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        plan_run_json_calls.append(list(args))
        return {"schemaVersion": "1.0"}

    monkeypatch.setattr(rhwp.plan, "run_json", fake_plan_run_json)

    with pytest.raises(rhwp.RhwpError, match="dry-run"):
        Plan("a.hwp", "b.hwp").fill_fields({"이름": "값"}).check()

    assert plan_run_json_calls == []  # 프로세스를 아예 안 띄웠다


def test_check_caches_capability_lookup_across_calls(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    capability_calls: List[List[Any]] = []

    def fake_capabilities_run_json(args, **kwargs):  # type: ignore[no-untyped-def]
        capability_calls.append(list(args))
        return _fake_capabilities(supports_dry_run=True)(args, **kwargs)

    monkeypatch.setattr(rhwp.commands, "run_json", fake_capabilities_run_json)
    monkeypatch.setattr(
        rhwp.plan,
        "run_json",
        lambda args, **kwargs: {
            "schemaVersion": "1.0",
            "dryRun": True,
            "invalid": [],
            "preview": [],
        },
    )

    Plan("a.hwp", "b.hwp").fill_fields({"이름": "값"}).check()
    Plan("a.hwp", "b.hwp").fill_fields({"이름": "값2"}).check()
    assert len(capability_calls) == 1  # 두 번째 호출은 캐시를 쓴다

    clear_plan_capability_cache()
    Plan("a.hwp", "b.hwp").fill_fields({"이름": "값3"}).check()
    assert len(capability_calls) == 2  # 캐시를 비우면 다시 묻는다
