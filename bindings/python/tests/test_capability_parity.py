"""[트랙 G R61 D-19] 선언 → 래퍼 패리티 — capabilities() 자기서술과 파이썬 API 대조.

Node 바인딩의 `test/parity.integration.test.ts` 가 이미 증명한 설계를 그대로 가져온다.
목록을 손으로 유지하지 않는다 — `capabilities()` 가 단일 출처이고, 대조는 기계가 한다.
rhwp 에 명령이 늘면 이 테스트가 먼저 실패한다.

D-1(`export_hwpx`/`convert` 의 `-o` 플래그)과 D-4(`Plan.check()` 의 `--dry-run` 게이트)는
둘 다 "바인딩 래퍼가 CLI 실제 계약과 어긋났다"는 결함이었고, 기존 단위 테스트
(`test_commands.py`)는 argv 조립만 봤을 뿐 "이 명령이 선언한 대로 노출됐나"는 보지
않았다. 이 파일이 그 빈 자리다.

## 판정 기준 (Node 의 선례를 그대로 승계)

- `capabilities()` 가 ``json: true`` 로 선언한 명령만 대상이다. ``json`` 이 없으면
  (예: export-render-tree, mcp-serve, export-png(feature gate), hwp5-*·test-*·gen-*
  진단/내부 도구) 바인딩이 감쌀 대상이 아니다 — capabilities 자체가 "기계 계약 아님"
  이라고 말하고 있다.
- ``category`` 가 diagnostic/internal/serve 면 대상에서 뺀다
  (Node `test/parity.integration.test.ts:50` `NOT_WRAPPED`).
  **주의**: 이 축은 카테고리만으로 걸러지므로 `ir-diff`·`verify`·`render-diff` 처럼
  category=diagnostic 이지만 json:true 인 명령은 이 하드 게이트를 통과하지 못한다.
  Node 는 이 셋을 실제로 감쌌지만(`commands.ts:651,714,765`), 그건 "diagnostic 이니까
  반드시 감싸라"가 기계로 강제된 결과가 아니라 사람이 내린 판단이다 — Node 자신의
  패리티 테스트도 이 셋을 요구하지 않는다. 그래서 이 파일은 두 번째(소프트) 테스트로
  그 카테고리 배제가 만드는 사각지대를 실패시키지 않고 보고만 한다.
- `edit` 처럼 서브커맨드로 갈라지는 명령은 서브커맨드마다 함수가 있어야 한다
  (`SUBCOMMAND_WRAPPERS`) — Node 와 동일하게 문자열 디스패치를 허용하지 않는다.
  `insert-image`/`redact`/`sanitize` 는 이 표에 없다 — Node 도 아직 감싸지 않았다
  (`SUBCOMMAND_WRAPPERS` 가 `edit` 에 `fillFields`/`replaceText`/`setCell` 셋만 요구,
  `commands.ts` 에 insertImage/redact/sanitize 함수 자체가 없다). 두 바인딩 공통의
  미완이라 이 파일의 "파이썬만 뒤처졌다" 판정 대상이 아니다 — 별도 이슈로 다룰 문제다.
- `run` 은 3층 `Plan`/`run_plan` 이 감싼다(`HIGHER_LAYER`) — 1층 무상태 함수로 다시
  노출하면 계획 문법 검사와 `check()` 미리보기를 우회하게 된다(Node `commands.ts`
  동일 판단, `HIGHER_LAYER` 주석 참고).
- `export-ir-schema`/`export-capabilities-schema` 는 이름 규칙이 다르다(`ALIASES`) —
  `schema.py` 가 "export" 접두어를 떼고 `ir_schema`/`capabilities_schema` 로 노출한다.
  Node 는 접두어를 그대로 살려 `exportIrSchema`/`exportCapabilitiesSchema` 로 부른다
  (`commands.ts:278,323`) — 이 이름 이탈은 **파이썬에만 있는 관례**이고, 기계적
  ``name.replace("-", "_")`` 규칙만으로는 못 잡는다. 그래서 예외 표로 명시한다
  (수기 목록이 아니라 "규칙이 다른 자리"의 유지보수 가능한 기록 — 파이썬이 새 이름
  규칙을 또 만들면 여기 추가하고 이유를 적는다).

## 실측 (2026-08-09, rhwp 0.8.2, 68개 선언 명령)

하드 테스트: 0건 누락 — D-1·D-4 수정 이후 새 표류 없음.
소프트 보고: `verify`, `render-diff` — 둘 다 category=diagnostic 이라 하드 게이트를
통과하지 않지만 Node 는 감쌌고 파이썬은 아직 없다. `render-diff` 는 이미 D-2 로
문서화됐다. `verify`(독립 사후검증 게이트, #4113)는 이 실행에서 처음 드러난
자리다 — 사람 판단으로 감쌀지 정할 항목이다.
"""

from __future__ import annotations

from pathlib import Path
from typing import Dict, Mapping, Tuple

import pytest

import rhwp

pytestmark = pytest.mark.integration

# 카테고리로 통째로 뺀다 — Node NOT_WRAPPED 와 동일.
NOT_WRAPPED_CATEGORIES = {"diagnostic", "internal", "serve"}

# CLI 에서 서브커맨드로 갈라지는 명령 — 파이썬은 서브커맨드마다 함수를 둔다.
SUBCOMMAND_WRAPPERS: Mapping[str, Tuple[str, ...]] = {
    "edit": ("fill_fields", "replace_text", "set_cell"),
}

# 위층(Plan/run_plan)이 감싸는 명령 — 1층 무상태 함수로 다시 노출하지 않는다.
HIGHER_LAYER: Mapping[str, object] = {"run": rhwp.Plan}

# 이름 규칙이 CLI 명령명의 기계적 변환(hyphen→underscore)과 다른 자리.
ALIASES: Mapping[str, str] = {
    "export-ir-schema": "ir_schema",
    "export-capabilities-schema": "capabilities_schema",
}

# category=diagnostic 이라 하드 게이트를 통과하지 못하지만, Node 는 실제로 감싼 명령.
# 사람 판단이 필요한 자리라 이 목록은 테스트를 실패시키지 않고 보고만 한다.
NODE_WRAPS_BEYOND_FLOOR: Tuple[str, ...] = ("ir-diff", "verify", "render-diff")


def _declared_commands(caps: rhwp.Envelope) -> Dict[str, dict]:
    return {
        c["name"]: c
        for c in caps.raw.get("commands", [])
        if isinstance(c, dict) and isinstance(c.get("name"), str)
    }


def test_every_declared_json_command_has_a_python_wrapper(wired_binary: Path) -> None:
    """capabilities() 가 json:true 로 선언한 (비-diagnostic/internal/serve) 명령마다
    파이썬 바인딩에 대응하는 이름이 있다.
    """
    caps = rhwp.capabilities()
    declared = _declared_commands(caps)
    exported = set(vars(rhwp))
    missing = []

    for name, spec in declared.items():
        if spec.get("json") is not True:
            continue
        if spec.get("category") in NOT_WRAPPED_CATEGORIES:
            continue

        if name in ALIASES:
            if ALIASES[name] not in exported:
                missing.append(f"{name} → rhwp.{ALIASES[name]} (alias)")
            continue

        subs = SUBCOMMAND_WRAPPERS.get(name)
        if subs is not None:
            for wrapper in subs:
                if wrapper not in exported:
                    missing.append(f"{name} → rhwp.{wrapper}")
            continue

        higher = HIGHER_LAYER.get(name)
        if higher is not None:
            if not callable(higher):
                missing.append(f"{name} → {higher!r} (상위 계층이 호출 가능하지 않음)")
            continue

        # `export-tables` → `export_tables`. 파이썬 자신의 변환 규칙(hyphen→underscore)을
        # 쓴다 — 여기서 손으로 다시 매핑하면 규칙이 두 벌이 되고 언젠가 어긋난다.
        expected = name.replace("-", "_")
        if expected not in exported:
            missing.append(f"{name} → rhwp.{expected}")

    assert not missing, (
        "capabilities() 가 선언했지만 파이썬 바인딩에 대응 함수가 없는 명령:\n  "
        + "\n  ".join(missing)
        + "\nrhwp 에 명령이 늘었습니다 — bindings/python/src/rhwp/commands.py 에 래퍼를 "
        "추가하거나, 의도된 예외면 이 파일의 ALIASES/SUBCOMMAND_WRAPPERS/HIGHER_LAYER/"
        "NOT_WRAPPED_CATEGORIES 에 반영하세요."
    )


def test_diagnostic_category_gaps_vs_node_are_reported(
    wired_binary: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """category=diagnostic 배제가 만드는 사각지대를 실패시키지 않고 눈에 띄게 보고한다.

    Node 가 실제로 감싼 diagnostic 명령(ir-diff·verify·render-diff)을 파이썬이 아직
    감싸지 않았으면 여기 걸린다. 감쌀지 말지는 사람 판단(D-2 계열)이므로 이 테스트는
    통과/실패가 아니라 표준출력 보고다 — CI 로그에서 눈에 띄되 빌드를 막지 않는다.
    """
    caps = rhwp.capabilities()
    declared = _declared_commands(caps)
    exported = set(vars(rhwp))
    gaps = []

    for name in NODE_WRAPS_BEYOND_FLOOR:
        spec = declared.get(name)
        if spec is None:
            continue  # 이 rhwp 버전엔 아직 없는 명령
        expected = name.replace("-", "_")
        if expected not in exported:
            gaps.append(name)

    if gaps:
        # [Windows cp949 함정] 이 저장소 다른 도구에서도 겪은 문제 — 콘솔 인코딩이
        # cp949 면 em dash 등 비 ASCII 구두점에서 UnicodeEncodeError 가 난다(pytest
        # -s 로 실제 터미널에 쓸 때만 드러나고, capsys 캡처 모드에서는 안 보인다).
        # 정보성 출력이라 죽을 이유가 없으므로 ASCII 구두점만 쓴다.
        print(
            "[정보] category=diagnostic 이라 하드 게이트를 통과하지 못했지만 Node 는 감싼 "
            f"명령이 파이썬엔 없습니다: {', '.join(gaps)}. "
            "이 목록은 테스트를 실패시키지 않습니다(감쌀지는 사람 판단, D-2/D-12 계열)."
        )
