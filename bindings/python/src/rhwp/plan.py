"""계획 실행기 (L4) — 의도를 선언하면 rhwp 가 안전을 보장한다.

도구를 체이닝하는 대신 계획서 하나를 만든다. rhwp 가 **정적 선검증(실행 0)** →
**원자 실행**(전 step 인메모리 적용) → **사후 단언 통과 시에만 단 한 번 저장**
순으로 처리하므로, 중간 실패가 반쪽 편집 문서를 남기지 않는다.

```python
plan = (
    rhwp.Plan("서식.hwp", "제출본.hwp")
    .fill_fields({"성명": "홍길동"})
    .replace_text("2025년", "2026년")
    .set_checkbox(1)
    .verify()
)

preview = plan.check()          # 디스크 무변경 — 실행 전 검사
if preview.ok:
    journal = plan.run()
    assert journal.verify.identical
```
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Union

from ._process import DEFAULT_TIMEOUT, run_json
from .models import Envelope

__all__ = ["Plan", "PlanResult", "run_plan"]

PathLike = Union[str, Path]


class PlanResult(Envelope):
    """계획 실행/검사 결과 저널."""

    __slots__ = ()

    @property
    def ok(self) -> bool:
        """위반 없이 통과했는가 (검사·실행 공통)."""
        return not self.violations

    @property
    def violations(self) -> List[Envelope]:
        """선검증 위반 목록. 통과했으면 빈 리스트."""
        raw = self.raw.get("invalid")
        if not isinstance(raw, list):
            return []
        return [Envelope(item) for item in raw if isinstance(item, Mapping)]

    @property
    def is_dry_run(self) -> bool:
        """검사 전용 실행이었는가 (디스크 무변경)."""
        return bool(self.raw.get("dryRun"))

    @property
    def preview(self) -> List[Envelope]:
        """검사 모드의 step 별 미리보기. 실행 모드면 빈 리스트."""
        raw = self.raw.get("preview")
        if not isinstance(raw, list):
            return []
        return [Envelope(item) for item in raw if isinstance(item, Mapping)]

    @property
    def steps(self) -> List[Envelope]:
        """실행 모드의 step 별 결과. 검사 모드면 빈 리스트."""
        raw = self.raw.get("steps")
        if not isinstance(raw, list):
            return []
        return [Envelope(item) for item in raw if isinstance(item, Mapping)]

    def describe_violations(self) -> str:
        """위반을 사람이 읽을 여러 줄로 — 로그·오류 메시지에 그대로 쓴다."""
        if not self.violations:
            return "위반 없음"
        lines = []
        for v in self.violations:
            raw = v.raw
            step = raw.get("step", "?")
            action = raw.get("action", "?")
            reason = raw.get("reason", "(사유 없음)")
            lines.append(f"  step {step} ({action}): {reason}")
        return "\n".join(lines)


class Plan:
    """계획서 빌더 — 체이닝으로 step 을 쌓는다.

    빌더는 **문법만** 검사한다(값 타입·필수 인자). 실제 실행 가능성은 rhwp 의
    선검증이 판정한다 — 판정자를 두 곳에 두면 어긋난다.
    """

    def __init__(self, input_path: PathLike, output_path: PathLike) -> None:
        self._input = str(input_path)
        self._output = str(output_path)
        self._steps: List[Dict[str, Any]] = []
        self._assertions: Dict[str, bool] = {}

    # ── step 추가 ────────────────────────────────────────────────────────
    def fill_fields(self, data: Mapping[str, Any]) -> "Plan":
        """누름틀 채우기. ``{"이름#1": "값"}`` 으로 동명 순번 지정."""
        if not isinstance(data, Mapping) or not data:
            raise ValueError("fill_fields 는 비어 있지 않은 {필드: 값} 매핑이 필요합니다")
        self._steps.append({"action": "fill_fields", "data": dict(data)})
        return self

    def replace_text(
        self,
        find: str,
        replace: str,
        *,
        occurrence: Optional[int] = None,
        case_sensitive: bool = True,
    ) -> "Plan":
        """문자열 치환. ``occurrence`` 를 주면 그 순번 하나만."""
        if not find:
            raise ValueError("replace_text 의 find 는 비어 있을 수 없습니다")
        if not isinstance(replace, str):
            raise TypeError("replace 는 문자열이어야 합니다")
        step: Dict[str, Any] = {
            "action": "replace_text",
            "find": find,
            "replace": replace,
            "caseSensitive": case_sensitive,
        }
        if occurrence is not None:
            if occurrence < 0:
                raise ValueError("occurrence 는 0 이상이어야 합니다")
            step["occurrence"] = occurrence
        self._steps.append(step)
        return self

    def set_cell(
        self, table: int, row: int, col: int, text: str, *, keep_style: bool = False
    ) -> "Plan":
        """표 셀 기록. 좌표는 ``export_tables`` 로 확인한다."""
        for label, value in (("table", table), ("row", row), ("col", col)):
            if not isinstance(value, int) or value < 0:
                raise ValueError(f"{label} 은 0 이상의 정수여야 합니다 (받음: {value!r})")
        if any(ch in text for ch in "\r\n\t"):
            raise ValueError("셀 값에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록)")
        step: Dict[str, Any] = {
            "action": "set_cell", "table": table, "row": row, "col": col, "text": text,
        }
        if keep_style:
            step["keepStyle"] = True
        self._steps.append(step)
        return self

    def set_checkbox(self, occurrence: int) -> "Plan":
        """빈 체크박스(□) 중 ``occurrence`` 번째를 표시(☑)한다."""
        if not isinstance(occurrence, int) or occurrence < 0:
            raise ValueError("occurrence 는 0 이상의 정수여야 합니다")
        self._steps.append({"action": "set_checkbox", "occurrence": occurrence})
        return self

    # ── 단언 ─────────────────────────────────────────────────────────────
    def verify(self, enabled: bool = True) -> "Plan":
        """저장 직후 자기검증을 요구한다 (실패 시 저장 없이 exit 3)."""
        self._assertions["verify"] = enabled
        return self

    def require_all_fields_found(self, enabled: bool = True) -> "Plan":
        """채우지 못한 필드가 하나도 없어야 한다고 단언한다."""
        self._assertions["notFoundEmpty"] = enabled
        return self

    # ── 직렬화·실행 ──────────────────────────────────────────────────────
    def to_dict(self, *, dry_run: bool = False) -> Dict[str, Any]:
        """계획서 JSON 구조를 돌려준다 (검토·저장·전송용)."""
        if not self._steps:
            raise ValueError("step 이 하나도 없는 계획은 실행할 수 없습니다")
        plan: Dict[str, Any] = {
            "planVersion": "1.0",
            "input": self._input,
            "output": self._output,
            "steps": list(self._steps),
        }
        if self._assertions:
            plan["assertions"] = dict(self._assertions)
        if dry_run:
            plan["dryRun"] = True
        return plan

    def check(self, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> PlanResult:
        """**실행하지 않고** 검사만 한다 — 디스크 무변경, step 별 미리보기 반환.

        위반이 있으면 예외가 아니라 ``result.violations`` 로 돌려준다. 계획을
        고쳐서 다시 검사하는 것이 정상 흐름이기 때문이다.
        """
        return _execute(self.to_dict(dry_run=True), timeout=timeout)

    def run(self, *, timeout: Optional[float] = DEFAULT_TIMEOUT) -> PlanResult:
        """실행한다. 단언이 실패하면 **저장 없이** 판정이 담긴 저널을 돌려준다."""
        return _execute(self.to_dict(), timeout=timeout)

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        actions = ", ".join(s["action"] for s in self._steps)
        return f"Plan({self._input} → {self._output}: [{actions}])"


def _execute(plan: Mapping[str, Any], *, timeout: Optional[float]) -> PlanResult:
    """계획서를 인라인으로 넘겨 실행한다.

    선검증 위반은 exit 2 라 기본 규약대로면 :class:`UsageError` 가 된다. 하지만
    계획 실행에서 위반은 **정상적인 결과**다 — 계획을 고쳐 다시 검사하는 것이
    설계된 흐름이므로, ``invalid[]`` 를 담은 봉투는 예외 대신 값으로 돌려준다.
    ``invalid`` 가 없는 exit 2 는 진짜 호출 조립 버그이므로 그대로 올린다.
    """
    import json as _json

    from .errors import UsageError

    args = ["run", "--plan-json", _json.dumps(plan, ensure_ascii=False), "--json"]
    try:
        envelope = run_json(args, timeout=timeout)
    except UsageError as exc:
        if exc.envelope is not None and "invalid" in exc.envelope:
            return PlanResult(exc.envelope)
        raise
    return PlanResult(envelope)


def run_plan(
    plan: Mapping[str, Any], *, timeout: Optional[float] = DEFAULT_TIMEOUT
) -> PlanResult:
    """이미 만들어 둔 계획서(dict)를 그대로 실행한다.

    빌더를 쓰지 않고 JSON 파일에서 읽어온 계획을 돌릴 때 쓴다. 선검증 위반은
    :func:`_execute` 와 같은 규약으로 예외가 아니라 결과로 돌아온다.
    """
    return _execute(plan, timeout=timeout)
