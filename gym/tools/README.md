---
kind: guide
status: active
canonical: gym/tools/README.md
last_verified: 2026-08-18
---

# gym/tools — 라이브 오라클 프로브

이 문서는 `gym/tools/oracle_probe.py` 의 정본 설명이다. 새 CLI 바이너리는
없다. 프로브는 순수 Python 이고, gym 채점이 믿는 전제를 팩 픽스처 없이
감사한다.

관련 문서:

- 모듈 자체: [`oracle_probe.py`](oracle_probe.py)
- 단위 시험: [`../../scripts/tests/test_gym_oracle_probe.py`](../../scripts/tests/test_gym_oracle_probe.py)
- 실문서 과제: [`../packs/oracle-probe/README.md`](../packs/oracle-probe/README.md)
- 작업 노트: [`../../mydocs/working/gym_oracle_probe.md`](../../mydocs/working/gym_oracle_probe.md)
- 채점 규약: [`../README.md`](../README.md)
- 검사 연산자: [`../core/checks.py`](../core/checks.py)
- 기준 풀이: [`build_baseline.py`](build_baseline.py)

사용:

```bash
python gym/tools/oracle_probe.py --json
python gym/tools/oracle_probe.py --selftest
python gym/tools/oracle_probe.py --json --selftest
python -m unittest scripts/tests/test_gym_oracle_probe.py
```

종료 코드는 보고서의 `ok` 를 따른다. `ok=true` 이면 0, 아니면 1.

------------------------------------------------------------------------

## 1. 목적 — 라이브 오라클 이중 계산 프로브

gym 의 채점 규약은 한 문장이다.

> 정답은 골든 파일로 박제돼 있지 않다. `score.py` 가 채점 시점에
> rhwp 로 기대값을 재계산한다.

그 규약이 성립하려면 세 가지가 참이어야 한다.

1. **결정성.** 같은 오라클을 두 번 돌려도 정규화 결과가 같다.
2. **자리표.** `{input}` 과 한 문자열의 여러 `{sub:이름}` 이 기준 풀이
   (`build_baseline.resolve`) 와 같이 치환된다. 남은 `{sub:` 는 실패다.
3. **부재 보고.** 산출물 경로가 파일이 아니면 `ok=false` 이다. 부재를
   통과로 위장하지 않는다.

`oracle_probe.py` 는 이 세 가지를 **독립 도구** 로 감사한다.
`build_baseline.py` 를 import 하지 않는다. 경로 삽입·스트림 재설정·
러너 적재 같은 부수효과가 구조 자기점검을 더럽히기 때문이다. 자리표
규칙은 복제돼 있고, 테스트가 양쪽 출력을 대조한다.

이 도구가 실패하면 gym 채점의 전제가 무너진 것이다. 개별 과제의
점수가 아니라 **채점기 자체** 를 의심해야 한다.

------------------------------------------------------------------------

## 2. 비목적

이 도구가 하지 않는 일.

- rhwp 바이너리를 실행하지 않는다. `--json` / `--selftest` 는 팩·표본
  없이 돈다.
- 새 clap 명령, 새 `src/bin` 을 추가하지 않는다.
- `answer_eq` 의 기대값을 계산하지 않는다. 그건 `score.py` 의 일이다.
- 골든 숫자를 저장하지 않는다.
- 네트워크를 쓰지 않는다.
- 제출물을 채점하지 않는다. 채점은 `gym/score.py` 가 한다.

프로브는 채점기의 **도구 상자** 이지 채점기 자체가 아니다.

------------------------------------------------------------------------

## 3. gym 채점이 라이브 오라클을 쓰는 방식

과제의 `checks` 항목은 연산자와 명령을 고른다. 기대 숫자는 적지 않는다.

```json
{
  "name": "쪽수 라이브 대조",
  "op": "answer_eq",
  "answer": "pages",
  "cmd": ["info", "{input}", "--json"],
  "path": "pageCount"
}
```

채점 시각의 흐름은 이렇다.

1. 에이전트가 `answer.json` 에 `{"pages": N}` 을 낸다.
2. 채점기가 `rhwp info <입력> --json` 을 **지금** 실행한다.
3. 봉투에서 `pageCount` 를 읽는다.
4. `checks.norm` 으로 숫자 문자열과 숫자를 같게 보고 등호를 판정한다.

`N` 이 어제 맞았다고 오늘도 맞는 것은 아니다. 바이너리가 바뀌면
오라클이 바뀐다. 그것이 의도다. 픽스처가 진화하면 정답도 따라 진화한다.

`len_answer_eq` 는 스칼라가 아니라 배열 길이를 잰다.

```json
{
  "op": "len_answer_eq",
  "answer": "fieldLen",
  "cmd": ["fields", "{input}", "--json"],
  "path": "fields"
}
```

에이전트는 길이를 보고하고, 오라클은 배열을 다시 받아 `len()` 한다.
`fieldCount` 스칼라와 `len(fields)` 가 어긋나면 명령 봉투가 깨진 것이다.

`value_eq` 는 제출 키가 아니라 오라클 좌표를 상수와 비교한다. 편집
과제에서 많이 쓰지만, 이 프로브 pack 은 읽기 전용 조회에 집중한다.

기준 풀이(`reference/*.json`) 도 숫자를 박제하지 않는다. 같은 명령을
다시 적어 `build_baseline.py` 가 제출물을 만든다. 이중 계산은 과제와
기준 풀이와 채점기에 세 번 나타난다.

------------------------------------------------------------------------

## 4. 부재는 통과가 아니다

산출물이 없으면 채점은 실패여야 한다. "파일이 없으니 비교할 것이 없다"
를 통과로 읽으면, 아무 것도 제출하지 않은 에이전트가 만점을 받는다.

`probe_missing_artifact` 의 존재 이유가 이것이다.

| 경로 | status | ok | present |
|------|--------|----|---------|
| 일반 파일 | `present` | true | true |
| 없는 경로 | `absent` | false | false |
| 디렉터리 | `not-a-file` | false | false |
| `""` / `None` / 비문자 | `invalid` | false | false |

`ok=true` 는 `status=present` 일 때만 난다. 빈 파일(0바이트)은
존재하는 파일이므로 통과다. "내용이 올바른가" 는 다른 검사의 몫이다.

`probe_artifacts` 는 묶음이다. 하나라도 없으면 묶음은 실패다. 빈
목록도 실패다 (`bool([])` 이 거짓). "검사할 산출물이 없다" 를
통과로 위장하지 않는다.

구조 자기점검은 존재하지 않는 경로를 일부러 넣어, `ok is True` 이면
즉시 이슈를 남긴다.

```text
부재 산출물을 통과로 위장했다
```

이 문장이 보고서에 뜨면 프로브 자체가 거짓말이다.

------------------------------------------------------------------------

## 5. 봉투 계약

`--json` 출력은 항상 객체다. 배열이 아니다.

```json
{
  "kind": "gymOracleProbe",
  "schemaVersion": "1.0",
  "ok": true,
  "mode": "structural"
}
```

필수 키:

- `kind` — 항상 `"gymOracleProbe"`. 다른 gym 도구 봉투와 섞이지 않는다.
- `schemaVersion` — 지금 `"1.0"`. 필드가 깨지면 올린다.
- `ok` — 전체 판정.
- `mode` — `"structural"` 또는 `"selftest"`.

`envelope(**fields)` 가 이 키를 먼저 넣고 호출자 필드를 덮어쓴다.
호출자가 `kind=` 를 넘기면 덮인다. CLI 는 그렇게 부르지 않는다.

구조 자기점검(`mode=structural`) 추가 키:

- `exports` — 실제 호출 가능한 필수 함수 이름.
- `required` — `REQUIRED_EXPORTS` 사본.
- `issues` — 문자열 목록. 비어 있어야 `ok`.
- `issueCount` — `len(issues)`.
- `probes` — 결정성·자리표·부재의 요약.

자기점검(`mode=selftest`) 추가 키:

- `checks` — `{name, ok, detail?}` 목록.
- `failed` — 실패한 검사 이름.
- `checkCount` / `issueCount`.

사람이 읽는 출력(`render_human`) 은 첫 줄에 통과/실패와 kind·schema 를
쓰고, 모드에 따라 exports/probes/issues 또는 O/X 검사 목록을 붙인다.

------------------------------------------------------------------------

## 6. CLI

```
python gym/tools/oracle_probe.py [--json] [--selftest]
```

| 플래그 | 동작 |
|--------|------|
| (없음) | 구조 자기점검을 사람 문구로 낸다 |
| `--json` | 같은 보고서를 JSON 봉투로 낸다 |
| `--selftest` | 내장 프로브를 돌린다 |
| 둘 다 | 자기점검을 JSON 으로 낸다 |

`parse_args` 는 `argv` 를 받을 수 있다. `run(argv)` 는 종료 코드를
반환하고 stdout 에 쓴다. `main()` 은 `sys.argv[1:]` 를 넘긴다.

`__main__` 가드는 stdout/stderr 를 UTF-8 `errors=replace` 로
재설정한다. Windows 콘솔에서 한글 보고서가 깨지지 않게 하기 위함이다.

인자가 틀리면 argparse 가 사용법을 내고 2 로 죽는다. 그것은 프로브
실패가 아니라 사용 오류다.

------------------------------------------------------------------------

## 7. 공개 표면

`REQUIRED_EXPORTS` 는 구조 자기점검이 확인하는 최소 집합이다.

```
probe_determinism
probe_placeholders
probe_missing_artifact
resolve_placeholders
json_canonicalize
```

아래는 모듈이 실제로 내보내는 함수다. 각 항목은 행복 경로와 예외
경로를 단위 시험이 건드린다.

### 7.1 leftover_sub_names(text)

치환 뒤에 남은 `{sub:이름}` 의 이름들을 왼쪽부터 모은다.

- 문자열이 아니거나 `{sub:` 가 없으면 `[]`.
- 닫는 `}` 가 없으면 나머지 전체를 한 이름으로 넣는다.
- 중복 이름을 제거하지 않는다. 순서와 횟수가 남는다.

### 7.2 extract_sub_names(token)

`leftover_sub_names` 의 별칭이다. 치환 전 인벤토리로 쓴다.

### 7.3 is_exact_sub_token(token)

토큰 전체가 `{sub:이름}` 인지 본다.

참이려면:

- 문자열이고
- `{sub:` 로 시작하고
- `}` 로 끝나며
- `{` 가 정확히 한 번.

임베디드·연속 두 개·`{input}` 은 거짓이다.

### 7.4 resolve_placeholders(token, task, sub_dir, mkdir=True)

`build_baseline.resolve` 와 같은 규칙.

1. 토큰이 `{input}` 이면 `task["input"]`. 키가 없으면 `KeyError`.
2. 토큰이 정확한 `{sub:이름}` 이면 `join(sub_dir, 이름)`.
   `mkdir` 이면 `dirname` 만 만든다 (기준 풀이와 동일).
3. 문자열 안의 여러 `{sub:이름}` 은 **전부** 바꾼다. 경로의 `\` 는
   JSON 에 박힐 수 있으므로 `\\` 로 이스케이프한다 (#4664).
4. 박힌 `{input}` 은 바꾸지 않는다.
5. 그 외는 토큰 그대로.

`mkdir=False` 는 부모 폴더를 만들지 않는다. 프로브가 파일 시스템에
손대지 않고 치환만 보고 싶을 때 쓴다.

닫히지 않은 `{sub:` 는 `str.split("}", 1)` 이 `ValueError` 를 낸다.
`probe_placeholders` 가 잡아 실패 보고로 바꾼다.

### 7.5 resolve_cmd(cmd, task, sub_dir, mkdir=True)

argv 각 토큰을 치환한다. `cmd` 가 list/tuple 이 아니면 `TypeError`.

### 7.6 probe_placeholders(token, task, sub_dir)

한 토큰의 `{sub:}` 가 모두 사라졌는지 본다.

반환:

```
ok, token, resolved, leftover, names, inputExact?
error?
```

비문자열은 `ok=false` 이고 `error` 에 타입 이름을 남긴다.
`KeyError` / `OSError` / `TypeError` / `ValueError` 도 `ok=false`.
`task is None` 이면 `{}` 로 치환을 시도하므로 `{input}` 은 KeyError.

`ok` 는 `leftover == []` 이다. 치환이 예외 없이 끝났고 남은
자리표가 없어야 통과다.

### 7.7 probe_cmd_placeholders(cmd, task, sub_dir)

argv 전 토큰을 본다. 하나라도 실패하면 묶음 실패.
`cmd` 를 `list()` 할 수 없으면 `ok=false`, `tokens=[]`.

반환: `ok`, `tokens`, `count` (순회 성공 시).

### 7.8 norm_scalar(value)

`checks.norm` 과 같은 스칼라 정규화. bool 을 먼저 본다
(`bool` 은 `int` 의 하위형).

| 입력 | 출력 |
|------|------|
| `True` / `False` | 그대로 |
| `int` | `float` |
| `float` 유한 | 그대로 |
| `float` NaN/inf | `None` |
| 숫자 문자열 | `float` |
| `"NaN"` / `"inf"` 문자열 | 문자열 그대로 (float 로 읽히지만 비유한) |
| 그 외 문자열 | `strip()` |
| 그 외 | 그대로 |

### 7.9 json_ready(value)

비교 가능한 JSON 값으로 접는다.

- `None` / `bool` 은 그대로.
- `int` / `float` / `str` 은 `norm_scalar`.
- `dict` 는 키를 `str` 로 바꾸고 값을 재귀.
- `list` / `tuple` 은 리스트로 재귀.
- `set` 은 정규화 후 `json.dumps(..., sort_keys=True)` 키로 정렬.
- `bytes` 는 UTF-8 디코드, 실패하면 hex.
- 그 외는 `str(value)`.

### 7.10 json_canonicalize(value)

키 정렬·공백 없는 JSON 문자열. `allow_nan=False`.
이중 계산 등호의 **단일 출처**.

NaN/inf 는 `json_ready` 가 `None` 으로 접으므로 `"null"` 이 된다.
`True` 는 `"true"`, `1` 은 `"1.0"`. Python 에서 `True == 1.0` 이지만
정규화 문자열은 갈라진다.

### 7.11 snapshot(value)

`json.loads(json_canonicalize(value))`.
다음 호출이 원본을 변이시켜도 비교 값이 바뀌지 않는다.

### 7.12 probe_determinism(compute_fn)

`compute_fn` 을 두 번 호출하고 정규화 결과가 같은지 본다.

한 쪽이라도 예외이거나 직렬화에 실패하면 `ok=false`.
같은 예외를 두 번 내도 '결정적 성공'으로 위장하지 않는다.
오라클은 값을 내야 한다.

호출 가능하지 않으면 `error` 에 타입 이름을 남긴다.

성공 시 `canonicalFirst` / `canonicalSecond` 를 남긴다.
실패 시 `error` 와 `errors` 목록을 남긴다.

### 7.13 probe_determinism_n(compute_fn, n=2)

이중 계산의 일반화. `n` 은 2 이상 정수. `True` 는 `int` 하위형이고
값 1 이라 거부된다. 문자열 `"2"` 도 거부된다.

오류가 하나라도 있으면 `ok=false`, 첫 오류를 `error` 에 남긴다.
모두 성공하면 첫 스냅샷과 나머지가 같아야 한다.

### 7.14 classify_artifact(path)

`present` / `absent` / `not-a-file` / `invalid`.
`os.PathLike` 은 허용. `OSError` 는 `absent`.

### 7.15 probe_missing_artifact(path)

분류 후 보고서를 만든다. `present` 이면 크기를 읽고 `ok=true`.
크기 확인이 `OSError` 이면 status 를 `absent` 로 내린다.

### 7.16 probe_artifacts(paths)

여러 경로. `ok` 는 "하나 이상 있고 모두 present".

### 7.17 probe_live_oracle(compute_fn, token=None, task=None, sub_dir=None, artifacts=None)

한 건의 라이브 오라클: 결정성 + 선택 자리표 + 선택 산출물.
`ok` 는 켜진 축의 논리곱.

### 7.18 envelope(**fields)

`kind` / `schemaVersion` 을 심고 필드를 합친다.

### 7.19 structural_self_check()

팩·표본·바이너리 없이 모듈 표면과 핵심 경로를 확인한다.

- 필수 함수가 호출 가능한가.
- 안정 계산의 결정성이 통과하는가.
- 다중 `{sub:}` 가 남는가.
- 부재 산출물이 `ok=true` 로 위장되는가.

### 7.20 run_selftest()

통과해야 할 것과 실패해야 할 것을 모두 확인한다.

내장 검사 이름:

- `determinism-stable`
- `determinism-drift-detected`
- `determinism-exception-is-not-pass`
- `determinism-numeric-norm`
- `placeholders-multi-sub`
- `placeholders-exact-input`
- `placeholders-unclosed-is-not-pass`
- `artifact-present`
- `artifact-absent-is-not-pass`
- `artifact-directory-is-not-pass`
- `artifact-empty-path-is-not-pass`
- `artifact-none-is-not-pass`
- `determinism-n-stable`
- `determinism-n-rejects-one`

새 불변식을 넣으면 여기와 단위 시험에 같은 이름을 남겨라.

### 7.21 render_human / parse_args / run / main

출력과 프로세스 진입점. `run` 은 코드를 반환하고 `sys.exit` 하지
않는다. 테스트가 stdout 을 가로채기 쉽게 하기 위함이다.

------------------------------------------------------------------------

## 8. 불변식

프로브가 지키는 것. 깨지면 버그다.

**I1. 다중 `{sub:}` 는 전부 치환된다.**
한 문자열의 첫 자리표만 바꾸면 나머지가 리터럴 파일명이 된다.
다세대 계획서의 `input` / `output` 이 동시에 `{sub:}` 인 이유가
이것이다 (#4664).

**I2. 박힌 `{input}` 은 바꾸지 않는다.**
기준 풀이 조립기도 그렇게 한다. 토큰 전체가 `{input}` 일 때만
`task["input"]` 이다.

**I3. 이중 계산이 어긋나면 실패다.**
키 순서와 `"2"` / `2` 는 같게 본다. 값이 실제로 바뀌면 실패다.

**I4. 예외는 통과가 아니다.**
같은 `RuntimeError` 를 두 번 내도 결정적 성공이 아니다.
오라클은 JSON 값을 내야 한다.

**I5. 부재는 통과가 아니다.**
`status=absent|not-a-file|invalid` 는 모두 `ok=false`.

**I6. `--json` 은 픽스처 없이 봉투를 낸다.**
`kind=gymOracleProbe`, `schemaVersion=1.0`.
구조 자기점검의 부재 프로브 자체는 `ok=false` 여야 한다.

**I7. 정규화 등호의 출처는 하나다.**
`json_canonicalize` 만이 이중 계산 비교 문자열을 만든다.
다른 함수가 `json.dumps` 를 직접 쓰지 않는다 (CLI 출력 제외).

**I8. bool 은 int 보다 먼저 본다.**
`True` 를 `1.0` 으로 접지 않는다. 정규화 문자열은 `"true"`.

**I9. 집합은 정렬한다.**
해시 순서가 결정성을 깨지 못하게 `json_ready` 가 정렬한다.

**I10. 스냅샷은 원본 변이에 닫혀 있다.**
`snapshot` 이후 원본 리스트에 append 해도 비교 값이 바뀌지 않는다.

**I11. 빌드 베이스라인과 치환이 같다.**
단위 시험이 `{input}`, 다중 `{sub:}`, 정확한 `{sub:capsules/…}`,
리터럴, 박힌 `{input}` 다섯 토큰을 양쪽에서 대조한다.

**I12. n < 2 는 사용 오류다.**
한 번만 돌리고 결정적이라고 주장하지 못한다.

------------------------------------------------------------------------

## 9. 실패 모드

프로브가 실패로 보고하는 상황과, 그것이 의미하는 것.

### 9.1 결정성 드리프트

`compute_fn` 이 호출마다 다른 값을 낸다. 시계, 난수, 전역 카운터,
파일 시스템 경합이 원인이다. gym 오라클은 이런 함수를 쓰면 안 된다.

보고: `ok=false`, `equal=false`, `first` ≠ `second`.

### 9.2 오라클 예외

타임아웃, 깨진 JSON, 없는 키, 0 나누기. 두 번 같은 예외여도 실패.
보고: `error` 에 `TypeName: message`, `errors` 에 각 실행.

### 9.3 직렬화 실패

`json_canonicalize` 가 `TypeError` / `ValueError` 를 내면
`JSON 정규화 실패:` 접두로 남는다. `json_ready` 가 대부분을
문자열로 접어 이 경로는 드물다. NaN 은 `None` 으로 접힌다.

### 9.4 호출 불가능

`compute_fn` 이 함수가 아니다. `ok=false`.

### 9.5 자리표 잔존

닫히지 않은 `{sub:`, 치환 실패, 비문자열 토큰.
리터럴 파일명 사고의 전조다.

### 9.6 입력 키 부재

`{input}` 인데 `task` 에 `"input"` 이 없다. `KeyError`.
과제 JSON 이 깨졌거나 프로브 호출자가 빈 딕셔너리를 넘긴 것이다.

### 9.7 산출물 부재 / 디렉터리 / 무효 경로

제출 폴더에 파일이 없거나, 파일 대신 디렉터리를 가리키거나,
경로가 비었다. 모두 실패. "비교할 대상이 없다" 는 통과가 아니다.

### 9.8 묶음 실패

`probe_live_oracle` 은 켜진 축의 논리곱이다. 결정성이 통과해도
산출물이 없으면 묶음은 실패다. 반대도 같다.

### 9.9 구조 자기점검 이슈

필수 함수 없음, 안정 계산 실패, 다중 `{sub:}` 잔존, 부재 위장.
이 네 가지는 `--json` 기본 모드가 막는다.

### 9.10 사용 오류

`probe_determinism_n(..., 1)`, 비순회 `cmd`, argparse 오용.
프로브 버그가 아니라 호출자 버그로 보고한다.

------------------------------------------------------------------------

## 10. 프로브 분류 행렬

단위 시험 `ProbeClassificationMatrixTests` 가 이 표를 강제한다.

### 10.1 산출물

| 입력 | status | ok | present | error |
|------|--------|----|---------|-------|
| 존재하는 파일 | present | T | T | 없음 |
| 0바이트 파일 | present | T | T | 없음 |
| 없는 경로 | absent | F | F | 없음 |
| 디렉터리 | not-a-file | F | F | 있음 |
| `""` | invalid | F | F | 있음 |
| `None` | invalid | F | F | 있음 |
| `1` / `[]` | invalid | F | F | 있음 |
| PathLike 파일 | present | T | T | 없음 |

### 10.2 결정성

| 입력 | ok | equal | error |
|------|----|-------|-------|
| 안정 함수 | T | T | 없음 |
| 드리프트 | F | F | 없음 |
| 예외 | F | F | 있음 |
| 비호출 | F | F | 있음 |
| `None` fn | F | F | 있음 |
| `{}` / `[]` / `None` 값 | T | T | 없음 |

성공 보고에는 `error` 키가 없다. 실패 보고에는 `equal=false`.

### 10.3 자리표

| token | ok | inputExact |
|-------|----|------------|
| `{input}` | T | T |
| `plain` | T | F |
| `{sub:a.hwp}` | T | F |
| `{sub:` | F | F |
| `None` / `1` | F | F |
| `""` | T | F |

### 10.4 n-회 결정성

| n | 결과 |
|---|------|
| 2 이상 안정 | ok |
| 2 이상 드리프트 | 실패 |
| 0, 1, -1, 1.5, `"2"`, `None`, `True` | 사용 오류 |

------------------------------------------------------------------------

## 11. JSON 정규화가 등호를 여는 이유

라이브 오라클은 JSON 봉투를 낸다. 같은 논리 값이 다른 바이트로
올 수 있다.

- 키 순서: `{"b":1,"a":2}` 와 `{"a":2,"b":1}`.
- 숫자 문자열: `"2"` 와 `2`.
- 공백: pretty-print 와 compact.
- 집합: 해시 순서.

`json_canonicalize` 는 이 네 가지를 접는다. 접지 말아야 할 것은
접지 않는다.

- `True` 와 `1` 은 다른 값이다.
- `"hangul"` 과 `" hangul "` 은 strip 후 같다.
- `"NaN"` 문자열은 문자 그대로 남는다. float NaN 은 `null`.
- 한글은 `\uXXXX` 로 이스케이프하지 않는다 (`ensure_ascii=False`).

큰 페이로드(수백 키, 중첩 리스트)도 두 번 돌려 같아야 한다.
정규화가 비결정이면 큰 봉투에서 먼저 드러난다.

------------------------------------------------------------------------

## 12. 자리표와 기준 풀이

`build_baseline.resolve` 의 규칙을 이 파일에 복제한 이유.

1. `build_baseline` 을 import 하면 `gym.core.runner` 가 로드되고
   스트림을 재설정하며 경로를 삽입한다.
2. `--json` 구조 자기점검은 그 부수효과 없이 돌아야 한다.
3. 규칙이 갈라지면 테스트가 red 가 된다.

복제된 규칙의 함정:

- 정확한 `{sub:a/b.hwp}` 는 `dirname` 만 mkdir. 파일 자체는 안 만든다.
- 임베디드 `{sub:}` 는 `_maybe_mkdir(path, fallback=sub_dir)`.
  `dirname` 이 비면 `sub_dir` 을 만든다.
- Windows 경로의 `\` 는 임베디드 치환에서만 `\\` 로 이스케이프.
  정확한 토큰 분기는 이스케이프하지 않는다 (기준 풀이와 동일).

프로브가 기준 풀이보다 "똑똑해지면" 안 된다. 같아야 한다.

------------------------------------------------------------------------

## 13. oracle-probe pack 과의 관계

`gym/packs/oracle-probe/` 는 프로브가 감사하는 전제를 **실문서
과제** 로 드러낸다.

- 에이전트는 쪽수·검색 건수·필드 수를 보고한다.
- 채점기는 같은 rhwp 명령을 다시 실행한다.
- 숫자는 어디에도 박제돼 있지 않다.

프로브 도구는 그 pack 을 채점하지 않는다. pack 은 rhwp 가 필요하고,
프로브는 바이너리 없이 CI 에서 돈다.

```
CI
 ├─ python -m unittest scripts/tests/test_gym_oracle_probe.py
 ├─ python gym/tools/audit.py          # pack 스키마·짝 기준풀이
 └─ python gym/tools/oracle_probe.py --json --selftest
```

라이브 채점(`score.py --pack oracle-probe`)은 바이너리가 있는
작업자가 돌린다. CI 의 gym 단계는 순수 파일이고, 그것이 이 저장소의
결이다.

------------------------------------------------------------------------

## 14. 작업된 예

### 14.1 안정 오라클

```python
probe_determinism(lambda: {"pageCount": 2, "kind": "info"})
# ok=true, equal=true, runs=2
```

### 14.2 드리프트

```python
i = {"n": 0}
def drift():
    i["n"] += 1
    return {"n": i["n"]}
probe_determinism(drift)
# ok=false, first={"n": 1.0}, second={"n": 2.0}
```

### 14.3 키 순서와 숫자 문자열

```python
seq = [{"b": 1, "a": "2"}, {"a": 2, "b": "1"}]
probe_determinism(lambda: seq.pop(0))
# ok=true — 정규화가 둘을 같게 본다
```

### 14.4 다중 자리표

```python
token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
probe_placeholders(token, {"input": "in.hwp"}, sub_dir)
# leftover=[], names=["o1.hwp", "o2.hwp"]
```

### 14.5 부재

```python
probe_missing_artifact("/no/such/oracle-output.svg")
# ok=false, status="absent", present=false
```

### 14.6 묶음

```python
probe_live_oracle(
    lambda: {"pageCount": 2},
    token="{input}",
    task={"input": "in.hwp"},
    artifacts=["/tmp/answer.json"],
)
# artifacts 가 없으면 ok=false — 결정성이 좋아도 부재는 통과가 아니다
```

------------------------------------------------------------------------

## 15. 위협 모델 (요약)

자세한 내용은 `mydocs/working/gym_oracle_probe.md` 에 있다.

골든 파일이 썩는 방식:

1. 누군가 `pageCount: 17` 을 과제에 적는다.
2. 페이지네이션이 고쳐져 실제 쪽수가 18 이 된다.
3. 정직한 에이전트는 18 을 보고하고 실패한다.
4. 숫자를 외운 에이전트는 17 을 내고 통과한다.

라이브 오라클은 3 과 4 를 뒤집는다. 정직한 쪽이 통과한다.

프로브가 막는 두 번째 위협: 오라클 함수가 비결정이거나, 자리표가
남거나, 없는 산출물을 통과로 세는 것. 이 셋이 있으면 라이브 채점도
거짓이다.

프로브가 막지 않는 것:

- rhwp 바이너리 자체의 회귀. 그건 라이브 채점과 회귀 시험의 몫.
- 에이전트가 기준 풀이 폴더를 읽는 것. 규칙으로 막을 뿐 기술로
  막지 않는다. 채점은 정직하게 돌아간다.
- 명령 봉투 스키마 변경. `path: pageCount` 가 사라지면 과제가
  깨진다. 그건 pack 감사와 기준 풀이 왕복의 몫.

------------------------------------------------------------------------

## 16. 테스트 지도

`scripts/tests/test_gym_oracle_probe.py` 는 바이너리 없이 모듈을
로드한다. `importlib` 로 파일 경로에서 읽으므로 `gym.tools` 패키지
설치가 필요 없다.

클래스:

- `PlaceholderTests` — 다중 `{sub:}`, 베이스라인 대조, 타입 오류,
  키 부재, 미닫힘, 중복 이름, 백슬래시 이스케이프, argv.
- `DeterminismTests` — 안정, 드리프트, 예외, 타임아웃 봉투,
  비호출, n-회, 빈 객체.
- `NormalizeTests` — bool/int/NaN/inf, 집합 정렬, bytes, 유니코드,
  큰 페이로드, 스냅샷 격리.
- `MissingArtifactTests` — 분류 행렬, 묶음, 0바이트, PathLike.
- `LiveOracleBundleTests` — 축 논리곱.
- `EnvelopeAndCliTests` — `--json` / `--selftest` / 사람 문구 /
  `run` / `main`.
- `ProbeClassificationMatrixTests` — 닫힌 status 집합, 자리표 표.
- `PublicSurfaceSmokeTests` — 공개 함수가 모두 호출 가능.
- `UnicodeAndPayloadExceptionTests` — 한글 자리표, 깊은 중첩.
- `ErrorEnvelopeContractTests` — 실패 보고의 키 집합.
- `NumericEdgeTests` — bool≠1 정규화 문자열, 큰 정수.

새 공개 함수를 추가하면 `PublicSurfaceSmokeTests.PUBLIC` 과
해당 행복·예외 시험을 같이 넣어라.

------------------------------------------------------------------------

## 17. 다른 gym/tools 와의 자리

| 도구 | 하는 일 | 프로브와의 관계 |
|------|---------|-----------------|
| `audit.py` | pack 스키마·짝 기준풀이·ID 고유 | pack 등재 관문. 프로브 불변식은 안 봄 |
| `build_baseline.py` | 기준 풀이 실행 → 제출물 | 자리표 규칙의 원본. 프로브가 복제 |
| `score.py` (상위) | 라이브 채점 | 프로브가 전제를 감사 |
| `coverage.py` | pack 커버리지 | 무관 |
| `release_diff.py` | 릴리스 차이 | 무관 |
| `oracle_probe.py` | 이중 계산·자리표·부재 | 이 문서의 대상 |

`gym/tools/` 아래 다른 스크립트의 사용법은 각 파일의 모듈 문자열에
있다. 이 README 는 오라클 프로브의 정본이다.

------------------------------------------------------------------------

## 18. 자주 하는 실수

1. **골든 숫자를 과제에 적는다.** `value_eq` 의 `value: 17` 은
   "17쪽이어야 한다" 가 아니라 "이 좌표의 값이 17 이어야 한다" 이다.
   쪽수 자체는 `answer_eq` + 라이브 `info` 로 물어라.
2. **첫 `{sub:}` 만 바꾼다.** #4664 회귀. 프로브와 베이스라인 대조가
   막는다.
3. **예외를 결정적 성공으로 센다.** 두 번 같은 traceback 은 값이
   아니다.
4. **디렉터리를 산출물로 낸다.** `status=not-a-file`.
5. **빈 산출물 목록을 통과로 본다.** `probe_artifacts([])` 는 실패.
6. **`build_baseline` 을 프로브에서 import 한다.** 하지 마라.
7. **`--json` 이 배열을 낸다고 가정한다.** 객체다. `kind` 로 가른다.
8. **한글 자리표가 깨질 거라고 가정한다.** 이름은 그대로 경로에
   들어간다. 단위 시험이 확인한다.
9. **`True` 와 `1` 을 같다고 본다.** 정규화 문자열은 다르다.
10. **n=1 로 결정성을 주장한다.** 사용 오류.

------------------------------------------------------------------------

## 19. 변경을 넣을 때

1. 공개 함수의 행복·예외 경로를 단위 시험에 추가한다.
2. 불변식이면 `run_selftest` 에 이름을 붙인다.
3. `REQUIRED_EXPORTS` 를 건드리면 구조 자기점검이 따라간다.
4. 자리표 규칙을 바꾸면 `build_baseline.resolve` 와 같은 커밋에서
   맞춘다. 테스트가 양쪽을 대조한다.
5. pack 과제를 늘리면 `reference/` 짝과 `audit.py` 를 통과시킨다.
6. `cargo fmt --all` 을 돌리지 않는다. 이 변경은 Python 이다.
7. 새 CLI 바이너리를 추가하지 않는다.

------------------------------------------------------------------------

## 20. 버전

- `kind`: `gymOracleProbe`
- `schemaVersion`: `1.0`
- 도입: #5207 / PR #5214
- 도구 경로: `gym/tools/oracle_probe.py`
- 시험 경로: `scripts/tests/test_gym_oracle_probe.py`
- pack: `gym/packs/oracle-probe` (OP01–OP44)

스키마를 깨는 변경은 `schemaVersion` 을 올리고, 이 README 의
봉투 절을 같이 고친다. 필드 추가는 1.0 안에서 허용하되 필수 키를
빼지 않는다.

------------------------------------------------------------------------

## 21. 명령 치트시트

```bash
# 구조 자기점검 (CI 가 믿는 최소)
python gym/tools/oracle_probe.py --json

# 내장 프로브 (통과해야 할 것 + 실패해야 할 것)
python gym/tools/oracle_probe.py --json --selftest

# 사람 문구
python gym/tools/oracle_probe.py
python gym/tools/oracle_probe.py --selftest

# 단위 시험
python -m unittest scripts.tests.test_gym_oracle_probe

# pack 정합
python gym/tools/audit.py
python gym/tools/audit.py --json

# 기준 풀이 왕복 (바이너리 필요)
python gym/tools/build_baseline.py --agent oracle-probe-ref --pack oracle-probe

# 라이브 채점 (바이너리 필요)
python gym/score.py --agent oracle-probe-ref --pack oracle-probe
```

`--json` 의 `probes.missingArtifact.ok` 는 **false** 여야 한다.
그것이 부재 비통과 계약의 살아 있는 증거다.

------------------------------------------------------------------------

## 22. 모듈 상수

```
KIND            = "gymOracleProbe"
SCHEMA_VERSION  = "1.0"
INPUT_TOKEN     = "{input}"
SUB_MARK        = "{sub:"
REQUIRED_EXPORTS = (
    "probe_determinism",
    "probe_placeholders",
    "probe_missing_artifact",
    "resolve_placeholders",
    "json_canonicalize",
)
```

`SUB_MARK` 는 닫는 중괄호를 포함하지 않는다. 이름에 `}` 가 없다고
가정한다. 이름에 `}` 가 있으면 첫 `}` 에서 잘린다. 기준 풀이와
같은 한계이고, 과제 작성자가 `{sub:weird}name}` 을 쓰지 않으면
문제 없다.

------------------------------------------------------------------------

## 23. 오류 문자열 (안정 관측점)

테스트가 의존하는 메시지 조각. 바꾸면 시험을 같이 고친다.

- `자리표는 문자열이어야 한다:`
- `cmd 는 인자 목록이어야 한다:`
- `cmd 순회 실패:`
- `compute_fn 이 호출 가능하지 않다:`
- `JSON 정규화 실패:`
- `n 은 2 이상이어야 한다:`
- `경로가 비어 있거나 문자열이 아니다`
- `디렉터리이지 산출 파일이 아니다`
- `크기 확인 실패:`
- `필수 함수 없음:`
- `안정 계산의 결정성 프로브가 실패했다`
- `부재 산출물을 통과로 위장했다`

예외는 `TypeName: message` 형식으로 접는다.
`KeyError: 'input'` 처럼 표준 표현을 유지한다.

------------------------------------------------------------------------

## 24. 파일 배치

```
gym/
  README.md                 # 운동장 규약 — 채점은 라이브다
  score.py                  # 라이브 채점기
  core/checks.py            # answer_eq / norm
  core/schema.py            # pack·task 검증
  tools/
    README.md               # 이 문서
    oracle_probe.py         # 프로브
    build_baseline.py       # 기준 풀이 (자리표 원본)
    audit.py                # pack 정합
  packs/oracle-probe/
    pack.json
    README.md
    tasks/OP01.json … OP44.json
    reference/OP01.json … OP44.json
scripts/tests/test_gym_oracle_probe.py
mydocs/working/gym_oracle_probe.md
```

------------------------------------------------------------------------

## 25. 라이선스·기여

이 파일은 rhwp 저장소의 일부다. 기여 절차는 저장소 루트의
기여자 안내를 따른다. 이 프로브만의 추가 규칙은 §19 다.

질문을 이슈로 남길 때 제목에 `oracle_probe` 와 `#5207` 을 넣어라.
재현은 `--json` 원문과 단위 시험 이름을 붙이면 충분하다.

------------------------------------------------------------------------

## 26. 한 줄 요약

채점은 정답을 박제하지 않고 지금 rhwp 로 다시 계산한다.
그 계산이 두 번 같고, 자리표가 남기지 않고, 없는 파일을 통과로
세지 않는지 — 이 도구가 그걸 감사한다.

부재는 실패다. 예외는 실패다. 드리프트는 실패다.
통과는 값이 나왔고, 두 번 같았고, 파일이 있을 때만이다.
