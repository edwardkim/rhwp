---
kind: guide
status: active
canonical: gym/docs/release_diff.md
last_verified: 2026-08-18
---

# gym 릴리스 간 차등 규약

이 문서는 `gym/tools/release_diff.py` 의 **분류 삼원**, **관측 kind**, **예외
경로 계약**을 고정한다. 작업 기록은
[`mydocs/working/gym_release_diff.md`](../../mydocs/working/gym_release_diff.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_release_diff.py` 가 기계로 고정한다.

릴리스 게이트(`release_gate.py`)는 이 도구의 분류를 파이프라인 판정으로 묶는다.
게이트는 **regression 만 차단**한다. surface-changed 는 리뷰 신호이지 자동
차단이 아니다. 이 문서가 다루는 것은 게이트가 아니라 차등 오라클 자체다.

## 1. 왜 이 기둥이 필요한가

운동장 채점기는 정답을 골든으로 박제하지 않고, 채점 시점에 rhwp 로 다시 계산한다
(#4653). 그러니 같은 제출물을 두 바이너리로 돌리면 관측이 같아야 한다. 갈리면
그 사이 릴리스에서 동작이 바뀐 것이다. #4658 교차형식 차등이 형식축에서 검증한
원리를 시간축으로 돌린 것뿐이다. 새 메커니즘은 없다.

리더보드 총점은 통과/실패의 이진값이라 둔감하다. 쪽수·표수·필드값·해시·판정
문자열처럼 봉투에서 길어낸 **관측 raw** 를 대조한다. 골든 없이, 관측이 갈리는
지점이 곧 회귀 후보다.

이 도구는 "무엇이 바뀌었나" 를 가리키지 "어느 쪽이 옳은가" 를 판정하지 않는다.
한컴 정답지가 없다. 판정은 사람이 한다.

## 2. 사용

```bash
python gym/tools/release_diff.py --old <구 바이너리> --new <신 바이너리>
python gym/tools/release_diff.py --old old/rhwp --new target/debug/rhwp --pack core-cli
python gym/tools/release_diff.py --old old/rhwp --new new/rhwp -o gym/release-diff.json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--old` | (필수) | 구 rhwp 바이너리. `runner.find_bin` 으로 해석한다. |
| `--new` | (필수) | 신 rhwp 바이너리. |
| `--agent` | `claude-fable-5` | 관측에 쓸 제출물 루트. |
| `--pack` | 전체 탐색 | 반복 지정 가능. 지정하면 그 pack 만 본다. |
| `-o` / `--out` | `gym/release-diff.json` | JSON 보고 경로. UTF-8 · BOM 없음 · LF. |
| `--digest-timeout` | 30 | capabilities 프로브 초. 0 이하는 무제한. |

프로브 명령은 항상 `rhwp capabilities` 이다. 관측 명령은 각 과제의 `cmd` 다.
파일 연산자(`file_exists` 등)는 CLI 를 부르지 않는다.

## 3. 분류 삼원 — 정직 조항

`classify(surface, divergences)` 는 오직 세 값만 낸다.

| 표면이 바뀌었나 | 관측이 갈렸나 | 분류 | exit | ok | reviewRequired |
|---|---|---|---|---|---|
| 아니오 | 아니오 | `stable` | 0 | 참 | 거짓 |
| 아니오 | 예 | `regression` | 3 | 거짓 | 거짓 |
| 예 | 아니오 | `surface-changed` | 2 | 거짓 | 참 |
| 예 | 예 | `surface-changed` | 2 | 거짓 | 참 |

규칙:

1. **표면이 회귀보다 앞선다.** capabilities digest 가 다르면 관측 분기 유무와
   무관하게 `surface-changed` 다. 의도된 명령 추가를 회귀로 오신고하지 않는다.
2. **표면이 같고 관측이 갈리면 `regression`.** 순수 동작 변화다.
3. **둘 다 같으면 `stable`.** 자기-대조에서 분기가 나오면 관측에 비결정성이 있다.
4. **`classify` 는 `probe-failed` 를 내지 않는다.** 그 값은 분류가 아니라 도구
   실패 상태다. `CLASSIFICATIONS` 튜플에 넣지 않는다.

`divergences` 인자는 목록·건수·bool 을 모두 받는다. 파이썬 진릿값으로만 본다.

- 거짓으로 접히는 값(`None`, `0`, `""`, `[]`, `{}`, `False`) → 분기 없음.
- 그 외(`1`, `[행]`, `"x"`, `True`) → 분기 있음.

표면 인자도 진릿값이다. 빈 문자열 표면은 거짓이라 `stable`/`regression` 쪽으로
간다. 프로브가 실패한 `None` digest 를 여기 넣지 말라.

## 4. 표면을 모를 때 — probe-failed

두 바이너리의 capabilities digest 를 둘 다 문자열로 얻지 못하면 **분류하지
않는다.** 한쪽 digest 가 `None` 인 상태를 `surface-changed` 로 부르면 거짓말이다.
양쪽이 `None` 인 상태를 `stable` 로 부르면 더 큰 거짓말이다.

그래서 보고 상태는 `probe-failed` 다.

| 필드 | 값 | 왜 |
|---|---|---|
| `classification` | `probe-failed` | 삼원이 아니다 |
| `exit` | 1 | 도구 실패. 0/2/3 과 겹치지 않는다 |
| `ok` | 거짓 | 안정을 위장하지 않는다 |
| `reviewRequired` | 거짓 | 사람 판정(표면 변경)을 위장하지 않는다 |
| `surfaceChanged` | 거짓 | 표면을 모르면 바뀐 것이 아니다 |
| `probeFailed` | 참 | 표지 |
| `probeErrors` | 역할별 오류 목록 | old/new 각각 |

`classify()` 를 고쳐 `probe-failed` 를 내게 하지 않는다. `can_classify_surface` 가
막아서 `classify_or_probe_failed` 또는 `build_report` 가 갈라 받는다.

프로브가 잡는 실패:

| kind | 예외 | 의미 |
|---|---|---|
| `missing-bin` | `FileNotFoundError` | 바이너리가 없다 |
| `permission` | `PermissionError` | 실행 권한이 없다 |
| `timeout` | `TimeoutExpired` / `TimeoutError` | capabilities 가 시간 안에 안 끝났다 |
| `os-error` | 그 외 `OSError` | 파이프·경로·윈도 오류 |
| `decode-error` | `UnicodeError` | 출력 디코드 실패 |
| `unexpected` | 그 외 | 카탈로그 밖. 그래도 도구는 죽지 않는다 |

`KeyboardInterrupt` · `SystemExit` · `MemoryError` · `GeneratorExit` 는 삼키지
않는다. 사용자가 끊었는데 안정 보고를 내면 거짓말이다.

## 5. 관측 kind

한 검사의 결과는 판정이 아니라 관측이다. 구/신이 같은 kind·같은 값이면 분기가
아니다.

| kind | 언제 | 표시 |
|---|---|---|
| `value` | 허용 exit + JSON + 경로 성공 | raw 값 |
| `exit` | 허용하지 않은 종료 코드 | `exitN` |
| `nojson` | 허용 exit 인데 봉투가 없다 | `nojson` |
| `digfail` | 경로 평가 실패 | `digfail` |
| `no-cmd` | 검사에 `cmd` 가 없다 | `no-cmd` |
| `resolve-error` | `resolve_args` 의 제출물 부재(`FileNotFoundError`) | `resolve-error` |
| `cli-error` | `run_cli` 의 `RuntimeError` | `cli-error` |
| `timeout` | CLI 시간초과 | `timeout` |
| `missing-bin` | CLI 실행 파일이 없다 | `missing-bin` |
| `permission` | CLI 권한 거부 | `permission` |
| `os-error` | CLI/경로의 그 외 OS 오류 | `os-error` |
| `type-error` | 검사 형식이 dict 가 아니거나 TypeError | `type-error` |
| `value-error` | 비정수 인덱스 등 ValueError | `value-error` / `digfail` |
| `decode-error` | 유니코드 오류 | `decode-error` |
| `unexpected` | 카탈로그 밖 | `unexpected` |

`observation_from_result` 의 경로 실패(KeyError · IndexError · TypeError ·
ValueError · AttributeError)는 모두 `digfail` 로 접는다. 한 칸을 못 읽었다고
차등 도구 전체를 죽이면, 빈 제출·깨진 경로가 있는 한 릴리스를 비교할 수 없다.

같은 오류가 구/신 양쪽에 나면 분기가 아니다. 한쪽만 나면 분기다. 분류는 그때
표면이 같은지 보고 `regression` 또는 `surface-changed` 를 고른다. 오류 관측이
분류 규칙을 바꾸지 않는다.

## 6. 관측 동일성

`observations_equal` / `_values_equal` 계약:

| 왼쪽 | 오른쪽 | 같은가 | 왜 |
|---|---|---|---|
| `6` | `6.0` | 예 | JSON 숫자의 int/float 요동 |
| `True` | `1` | 아니오 | bool 을 int 로 접지 않는다 |
| `False` | `0` | 아니오 | 위와 같음 |
| `{b:1,a:2}` | `{a:2.0,b:1}` | 예 | 키 순서 무관, 숫자 정규화 |
| `{"kind":"nojson"}` | `{"kind":"value","value":"nojson"}` | 아니오 | 종류가 다르면 표시가 같아도 다르다 |
| `{"kind":"exit","code":1}` | `{"kind":"value","value":"exit1"}` | 아니오 | 위와 같음 |
| `NaN` | `NaN` | 예 | 둘 다 비숫자. 회귀로 오신고하지 않는다 |
| `+inf` | `-inf` | 아니오 | 부호가 다른 무한대 |
| `[1,[2,3.0]]` | `[1.0,[2,3]]` | 예 | 중첩 숫자 정규화 |
| `b"ab"` | `"ab"` | 아니오 | 바이트와 문자열은 다르다 |

`cell_text_eq` 는 표 전체가 아니라 `(table, row, col)` 칸의 `text` 만 본다.
칸이 없으면 값 `None` 이다(digfail 이 아니다). 표 인덱스가 범위를 벗어나면
`digfail` / `IndexError` 다.

## 7. 파일 연산자는 관측이 아니다

다음 op 는 CLI 를 부르지 않고 raw 대조에서도 뺀다.

- `file_exists`
- `same_hash`
- `differs_from_input`
- `files_differ`

존재/동일성은 릴리스와 무관하게 흔들릴 수 있는 자리다. 산출 파일 크기·경로를
관측으로 넣으면 같은 바이너리의 자기-대조도 갈린다.

`xml_root_eq` · `json_value_eq` 같은 제출물 검사도 이 도구의 관측 대상이 되려면
`cmd` 가 있어야 한다. `cmd` 가 없으면 `no-cmd` 관측이고 CLI 는 부르지 않는다.

## 8. JSON 봉투

`kind=gymReleaseDiff`, `schemaVersion=1.0`. 키 집합은 시험이 `REPORT_KEYS` 로
고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymReleaseDiff` |
| `schemaVersion` | str | 항상 `1.0` |
| `old` / `new` | obj | `{bin, capabilitiesSha256}` |
| `surfaceChanged` | bool | digest 가 다른가. 프로브 실패면 거짓 |
| `tasksCompared` | int | 실제로 본 과제 수 |
| `observationsCompared` | int | 파일 연산을 뺀 검사 수 |
| `observationsSkipped` | int | 파일 연산 수 |
| `divergences` | int | `diffs` 길이 |
| `classification` | str | 삼원 또는 `probe-failed` |
| `classificationReason` | str | 사람이 읽는 이유 |
| `exit` | int | 0 / 1 / 2 / 3 |
| `ok` | bool | `stable` 과만 참 |
| `reviewRequired` | bool | `surface-changed` 와만 참 |
| `diffs` | list | 분기 행 |

부가 키:

| 키 | 언제 |
|---|---|
| `probeFailed` | 표면을 못 쟀을 때 |
| `probeErrors` | old/new 프로브 실패 줄 |
| `packErrors` | pack 읽기 실패. 분류를 바꾸지 않는다 |
| `taskErrors` | 한 과제 루프의 예외. 분류를 바꾸지 않는다 |
| `writeError` | JSON 쓰기 실패. 종료 코드는 분류를 따른다 |

`validate_report` 가 정직 계약을 검사한다.

- 삼원일 때 `ok` · `reviewRequired` · `surfaceChanged` · `exit` 가 표와 같아야
  한다. `probeFailed` 가 참이면 안 된다.
- `stable` 인데 `divergences > 0` 이면 거짓말이다.
- `regression` 인데 `divergences == 0` 이면 거짓말이다.
- `probe-failed` 인데 `ok` 또는 `reviewRequired` 또는 `surfaceChanged` 가 참이면
  거짓말이다.

## 9. 예외 경로 — 도구가 죽지 않는 자리

감사기(이 차등 도구) 자신은 한 검사·한 pack 의 예외로 멈추지 않는다. 빈
제출물, 없는 바이너리, 권한, 시간초과, 깨진 pack JSON 은 관측/오류 목록으로
남기고 다음으로 간다.

| 자리 | 잡는 것 | 접는 곳 |
|---|---|---|
| `resolve_args` | `FileNotFoundError` 등 | 관측 `resolve-error` / 대응 kind |
| `run_cli` | timeout · missing-bin · permission · OSError | 관측 |
| `dig` / `find_cell` | KeyError · IndexError · TypeError · ValueError | `digfail` |
| `probe_capabilities` | 위 + 빈 경로 | 프로브 봉투. digest=None |
| `load_pack` | OSError · JSON 오류 | `packErrors` |
| `discover_packs` | OSError | 빈 목록 + packErrors |
| `find_bin` | OSError | 경로 유지 + 오류 문자열 |
| `write_report` | OSError | `writeError` |
| `diff_task` 한 검사 | 그 외 예외 | 그 행만 건너뜀. 거짓 분기를 만들지 않음 |

한 pack 을 못 읽어도 다른 pack 은 계속 비교한다. pack 오류는 분류를 뒤집지
않는다. 비교한 과제만으로 삼원을 고른다. 비교를 하나도 못 했더라도 digest 가
있으면 그 digest 로 분류한다(관측 0 · 분기 0 → 표면이 같으면 `stable`).

자기-대조에서 `stable` 이 아니면 관측이 비결정적이거나 프로브가 실패한
것이다. 커밋된 `gym/release-diff.json` 이 있으면 시험이 그 점을 확인한다.

## 10. 종료 코드

| exit | 의미 | 게이트 |
|---|---|---|
| 0 | `stable` | pass |
| 1 | `probe-failed` | 도구 실패. 분류를 위장하지 않음 |
| 2 | `surface-changed` | review. 자동 차단 아님 |
| 3 | `regression` | block |

`exit_for` 는 삼원만 받는다. `skipped` 같은 미지 값은 `KeyError` 다.
`probe-failed` 는 `status_exit` 가 1 을 낸다. `EXIT_BY_CLASS` 에 넣지 않아
기존 게이트 계약(`CLASSIFICATIONS` ↔ `EXIT_BY_CLASS`)이 그대로다.

## 11. 보고 쓰기 형식

`write_report` 는 UTF-8, BOM 없음, LF, 마지막 개행 하나, `ensure_ascii=False`,
`indent=2` 다. 같은 입력이면 바이트가 같다. 시험이 BOM/`\r\n` 부재를 고정한다.

쓰기 실패는 예외로 도구를 죽이지 않고 `writeError` 에 남긴다. 종료 코드는
이미 계산한 분류를 따른다. 디스크가 가득 찼다고 회귀를 안정으로 바꾸지 않는다.

## 12. 오검출 관문 요약

도구가 거짓말하지 않도록 지키는 문:

1. **명령 표면 대조.** digest 가 같아야 관측 변화를 회귀로 부른다.
2. **판정성 종료 코드 허용.** `expect_exits` 에 3 이 있으면 exit 3 은 실패가
   아니라 값이다.
3. **비결정 관측 배제.** 파일 존재/동일성은 raw 대조에서 뺀다.
4. **표면을 모르면 분류하지 않는다.** `probe-failed` 는 삼원이 아니다.
5. **오류도 관측이다.** 제출물 부재·CLI 실패를 예외로 올리지 않고 구/신이
   같은 실패인지를 본다.
6. **한 pack 의 파싱 실패는 전 저장소 비교를 죽이지 않는다.** 오류 목록에
   남긴다.

## 13. 시험이 고정하는 것

`python -m unittest scripts.tests.test_gym_release_diff`

바이너리 없이 돈다. `run_cli` · `subprocess.run` · `load_pack` 을 목으로
갈아끼운다.

고정하는 축:

- 분류 행렬과 확장 진릿값 표.
- `classify` 가 `probe-failed` 를 내지 않음.
- digest 부재를 삼원으로 위장하지 않음.
- 관측 동일성(숫자·bool·kind·NaN·중첩).
- 예외 kind 카탈로그와 context 분기(resolve 의 FileNotFound 는 resolve-error).
- 프로브 성공/실패 봉투.
- `validate_report` 정직 계약.
- `main` 의 exit 0/1/2/3.
- 커밋된 자기-대조 리포트가 있으면 divergences=0.

## 14. 이 도구가 하지 않는 것

- pack JSON 을 고치지 않는다. 과제의 기대값을 바꾸지 않는다.
- 한컴 문서가 맞는지 틀리는지 말하지 않는다.
- 어느 바이너리가 "더 옳은지" 고르지 않는다.
- 표면이 바뀐 릴리스를 자동으로 막지 않는다. 그건 사람 몫이다.
- 파일 산출의 바이트 동일성을 릴리스 회귀로 부르지 않는다.
- 치명 예외(키보드 중단 등)를 삼켜 성공인 척하지 않는다.

## 15. 관련 기둥

| 기둥 | 도구 | 질문 |
|---|---|---|
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` | 마지막 스텝을 빼도 통과하나? |
| 도구 강건성 | `robustness.py` | 손상 입력에 rhwp 가 패닉·행 하나? |
| 릴리스 차등 | `release_diff.py` | 두 바이너리가 같은 관측을 내나? |
| 릴리스 게이트 | `release_gate.py` | 차등 + 리더보드를 파이프라인 판정으로 묶나? |

차등은 오라클이다. 게이트는 그 오라클을 읽는다. 오라클이 표면을 모르는데
안정이라고 쓰면 게이트도 속는다. 그래서 `probe-failed` 를 삼원 밖에 둔다.

## 16. 봉투 표본

아래는 시험이 조립하는 최소 표본이다. 필드의 참/거짓이 분류와 어긋나면
`validate_report` 가 거부한다.

### 16.1 stable

```json
{
  "kind": "gymReleaseDiff",
  "schemaVersion": "1.0",
  "old": {"bin": "rhwp", "capabilitiesSha256": "aaa"},
  "new": {"bin": "rhwp", "capabilitiesSha256": "aaa"},
  "surfaceChanged": false,
  "tasksCompared": 10,
  "observationsCompared": 20,
  "observationsSkipped": 3,
  "divergences": 0,
  "classification": "stable",
  "classificationReason": "명령 표면과 관측이 같다",
  "exit": 0,
  "ok": true,
  "reviewRequired": false,
  "diffs": []
}
```

자기-대조(같은 바이너리를 `--old` 와 `--new` 에 넣음)는 이 모양이어야 한다.
`divergences` 가 0 이 아니면 관측에 비결정 자리가 남아 있는 것이다.

### 16.2 regression

```json
{
  "classification": "regression",
  "surfaceChanged": false,
  "ok": false,
  "reviewRequired": false,
  "exit": 3,
  "divergences": 1,
  "diffs": [
    {
      "pack": "core-cli",
      "task": "T01",
      "check": "쪽수",
      "op": "value_eq",
      "path": "pageCount",
      "old": {"kind": "value", "value": 6},
      "new": {"kind": "value", "value": 7}
    }
  ]
}
```

표면이 같은데 쪽수가 6→7 이면 순수 동작 변화다. 어느 쪽이 한컴과 맞는지는
이 도구가 말하지 않는다.

### 16.3 surface-changed

```json
{
  "classification": "surface-changed",
  "surfaceChanged": true,
  "ok": false,
  "reviewRequired": true,
  "exit": 2,
  "divergences": 0
}
```

명령이 늘거나 빠진 릴리스다. 관측이 같아도 사람 판정이 필요하다. 관측이
갈려도 분류는 그대로 `surface-changed` 다. 회귀로 접지 않는다.

### 16.4 probe-failed

```json
{
  "classification": "probe-failed",
  "surfaceChanged": false,
  "ok": false,
  "reviewRequired": false,
  "exit": 1,
  "probeFailed": true,
  "probeErrors": [
    {
      "role": "old",
      "bin": "rhwp",
      "kind": "missing-bin",
      "error": "FileNotFoundError",
      "head": "..."
    }
  ]
}
```

구 바이너리가 없을 때 `stable` 이라고 쓰면 게이트가 pass 한다. `surface-changed`
라고 쓰면 사람 리뷰를 요구한다. 둘 다 표면을 모르는 상태를 아는 척하는
것이다. exit 1 은 그 거짓말을 막는다.

## 17. 한 검사의 예외가 분류를 바꾸지 않는 예

같은 제출물, 같은 표면 digest.

| old 관측 | new 관측 | 분기? | 분류 |
|---|---|---|---|
| value 6 | value 6 | 아니오 | stable |
| value 6 | value 7 | 예 | regression |
| timeout | timeout (같은 페이로드) | 아니오 | stable |
| timeout | value 6 | 예 | regression |
| missing-bin | missing-bin | 아니오 | stable |
| resolve-error | value "ok" | 예 | regression |
| digfail KeyError | digfail KeyError | 아니오 | stable |
| digfail KeyError | digfail IndexError | 예 | regression |

표면 digest 가 다르면 위 표의 마지막 칸은 전부 `surface-changed` 다.
오류 관측은 값을 대신할 뿐, 오검출 관문의 순서를 바꾸지 않는다.

## 18. pack 오류와 분류

`packErrors` 가 있어도 분류는 이미 비교한 과제만 본다.

- pack A 를 읽지 못하고 pack B 의 관측이 같으면 → `stable` + packErrors.
- pack A 를 읽지 못하고 pack B 의 관측이 갈리면 → `regression` + packErrors.
- 모든 pack 을 읽지 못했고 digest 는 같다 → 과제 0 · 분기 0 → `stable`.
  "비교할 것이 없었다" 이지 "도구가 실패했다" 가 아니다.
- digest 자체를 못 얻으면 → 과제 수와 무관하게 `probe-failed`.

마지막 두 줄을 섞지 말라. pack 부재와 바이너리 부재는 다른 거짓말이다.
