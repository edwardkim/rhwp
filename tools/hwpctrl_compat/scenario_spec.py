"""시나리오가 스스로 선언하는 계약 — 경로 이식성·기대 반환·기대 예외.

두 러너와 두 판정자(`run_gate.validate_rhwp_output`, `compare.py`)가 **같은 규칙**을 읽어야
해서 여기에 모았다. 규칙을 한쪽만 고치면 diff 가 구현 차이가 아니라 하니스 차이가 된다.

## 왜 생겼나 (#4274 리뷰)

셋 다 같은 뿌리다 — **시나리오가 Windows 한 대를 전제로 적혀 있었다.**

1. 경로를 Windows 절대 경로로 박아 놨다. Linux 에서 `C:\\...\\s1.jpg` 는 못 여는 파일이 아니라
   **그냥 그런 이름의 상대 경로**다. `InsertPicture` 가 조용히 실패하고 뒤따르는 여덟 호출이
   `MissingApi` 로 무너졌다. `C:\\없는폴더xyz\\a.bmp` 는 더 나쁘다 — "없는 폴더"를 재려던 자리가
   Linux 에서는 **멀쩡한 파일 이름**이라 `false` 여야 할 것이 `true` 가 되고, 자체 검사는
   반환값을 안 보니 그대로 `OK` 로 셌다. 게다가 저장소 작업본에 그 이름의 쓰레기 파일이 남았다.
2. 일부러 죽는 호출(`SetCurFieldName` 인자 부족 따위)이 공식 시나리오에 그냥 들어 있었다.
   Linux 자체 검사는 호출 오류를 하나라도 보면 `CALL_ERROR` 를 내므로 게이트가 붉어진다.
   **검사를 무르게 하는 것은 답이 아니다** — 그러면 진짜 오류도 함께 통과한다.

그래서 시나리오가 **미리 선언**하게 했다. 선언하지 않은 오류는 여전히 실패다.

## 스키마

```jsonc
{
  "paths": {                         // 이름 → 플랫폼별 실제 경로
    "src": {
      "win":   "{repo}\\samples\\s1.jpg", // 현재 worktree의 fixture
      "posix": "{repo}/samples/s1.jpg"   // {repo}·{out} 토큰을 해석한다
    }
  },
  "calls": [
    ["InsertPicture", [{"$path": "src"}, true, 0]],         // 러너가 제 플랫폼 값으로 바꾼다
    ["CreatePageImage", [{"$path": "noDir"}, 0], {"expect": false}],
    ["SetCurFieldName", ["새이름"], {"expectError": {
        "rhwp": "필수 매개 변수입니다",   // rhwp 오류 문구 — 반드시 이것을 담아야 한다
        "ocx":  null,                     // 오라클 문구 미측정: "죽는다"까지만 요구한다
        "why":  "규격은 뒤 셋을 선택으로 적지만 실물은 넷을 다 요구한다(계획서 §4.71)"
    }}]
  ]
}
```

- `expect` 는 **정규화한 반환값**과 그대로 대조한다. 자체 검사(Linux)와 오라클 대조(Windows)가
  **같은 한 값**을 본다 — 플랫폼마다 인자가 갈려도 정답은 하나라는 것을 이 자리가 못박는다.
- `expectError.rhwp` 는 **필수**고 `MissingApi` 를 쓸 수 없다. "아직 안 만들었다"를 계약으로
  선언하면 리뷰가 막았던 그 구멍이 이름만 바꿔 돌아온다.
- `expectError.ocx` 는 키가 **필수**다. 문구를 잰 적이 없으면 `null` 을 적는다 — 안 잰 것을
  적어 두는 것과 안 잰 것을 빈칸으로 두는 것은 다르다. `null` 이면 오라클이 **죽었는가**만
  본다. 문구를 재고 나면 그 자리에 넣으면 그때부터 문구까지 대조한다.
"""

from __future__ import annotations

import io
import json
from pathlib import Path

MISSING_API = "MissingApi"
PATH_KEYS = ("win", "posix")


def load_scenario(path: Path) -> dict:
    with io.open(path, encoding="utf-8") as fh:
        return json.load(fh)


def platform_path_key(system_name: str) -> str:
    """실행 호스트에 맞는 경로 갈래. 오라클 러너는 Windows 에서만 도니 항상 `win` 이다."""
    return "win" if system_name.lower().startswith("win") else "posix"


def resolve_args(args: list, definition: dict, key: str, repo: Path, out_dir: Path) -> list:
    """인자 속 `{"$path": 이름}` 을 이 플랫폼의 실제 경로로 바꾼다.

    `{repo}`·`{out}` 을 넓혀 준다. 산출 경로를 `{out}` 아래로 적으면 시나리오가 저장소 작업본에
    쓰레기를 남기지 않는다 — Linux 에서 실제로 남겼다.

    **넘겨받는 `repo`·`out_dir` 은 절대경로여야 한다** — 러너가 그렇게 넘긴다. 상대 경로를
    한글에 넘기면 답도 오류도 없이 **멈춘다**(`SaveAs("output/…/x.hwp")` 하나로 오라클이 십
    분을 넘겨 죽었다). 여기서 절대화하지 않는 것은 이 함수가 두 플랫폼의 경로를 같이 다루기
    때문이다 — 자체 검사가 POSIX 경로를 넣는데 Windows 에서 `resolve()` 하면 드라이브가 붙는다.
    """
    table = definition.get("paths") or {}
    out = []
    for arg in args:
        if isinstance(arg, dict) and "$path" in arg:
            name = arg["$path"]
            if name not in table:
                raise ValueError(f"시나리오에 없는 경로 이름입니다: {name}")
            variants = table[name]
            if key not in variants:
                raise ValueError(f"경로 '{name}' 에 '{key}' 갈래가 없습니다")
            out.append(str(variants[key]).replace("{repo}", str(repo)).replace("{out}", str(out_dir)))
        else:
            out.append(arg)
    return out


def call_contract(call: list) -> dict:
    """호출 하나가 선언한 계약. 없으면 빈 계약(오류 금지·반환값 무검사)이다."""
    spec = call[2] if len(call) > 2 else {}
    if not isinstance(spec, dict):
        raise ValueError(f"호출 계약은 객체여야 합니다: {call[0]}")
    unknown = set(spec) - {"expect", "expectError"}
    if unknown:
        raise ValueError(f"모르는 계약 키입니다({call[0]}): {sorted(unknown)}")
    expected_error = spec.get("expectError")
    if expected_error is not None:
        if not isinstance(expected_error, dict):
            raise ValueError(f"expectError 는 객체여야 합니다: {call[0]}")
        if "rhwp" not in expected_error or "ocx" not in expected_error:
            raise ValueError(f"expectError 는 'rhwp' 와 'ocx' 를 모두 적어야 합니다: {call[0]}")
        rhwp = expected_error["rhwp"]
        if not isinstance(rhwp, str) or not rhwp:
            raise ValueError(f"expectError.rhwp 는 빈 문자열이 아니어야 합니다: {call[0]}")
        if MISSING_API in rhwp:
            raise ValueError(f"expectError 로 {MISSING_API} 를 선언할 수 없습니다: {call[0]}")
    return spec


def contracts(definition: dict) -> list[dict]:
    """`Open` 을 포함한 호출 순서에 맞춘 계약 목록 — 러너가 기록하는 차례와 같다."""
    rows = [{}] if definition.get("open") else []
    rows += [call_contract(call) for call in definition.get("calls", [])]
    return rows


def check_call(contract: dict, record: dict, side: str) -> str | None:
    """호출 하나가 계약을 지켰는지. 어긋난 이유를 글로, 지켰으면 `None`.

    `side` 는 `rhwp` 또는 `ocx` — 기대 오류 문구가 러너마다 다르기 때문이다.
    """
    name = record.get("call", "?")
    error = record.get("error")
    expected_error = contract.get("expectError")

    if expected_error is None:
        if error:
            return f"{name}: 선언하지 않은 오류 — {error}"
    else:
        if not error:
            return f"{name}: 죽어야 하는데 값이 돌아왔다 — {json.dumps(record.get('value'), ensure_ascii=False)}"
        if MISSING_API in str(error):
            # 오류를 선언했다고 해서 "아직 안 만들었다"까지 통과시키지 않는다.
            return f"{name}: 기대한 오류가 아니라 {MISSING_API} 다 — {error}"
        wanted = expected_error.get(side)
        if wanted is not None and wanted not in str(error):
            return f"{name}: 기대 오류 문구 '{wanted}' 가 없다 — {error}"
        return None

    if "expect" in contract and record.get("value") != contract["expect"]:
        got = json.dumps(record.get("value"), ensure_ascii=False)
        want = json.dumps(contract["expect"], ensure_ascii=False)
        return f"{name}: 기대 반환 {want} 인데 {got} 이다"
    return None
