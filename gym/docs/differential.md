---
kind: guide
status: active
canonical: gym/docs/differential.md
last_verified: 2026-08-18
---

# gym 교차형식 차등 규약

이 문서는 `gym/tools/differential.py` 의 **오검출 관문**, **짝짓기**, **본문
해시**, **관측 kind**, **예외 경로 계약**을 고정한다. 작업 기록은
[`mydocs/working/gym_differential.md`](../../mydocs/working/archives/gym_differential.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_differential.py` 가 기계로 고정한다.

릴리스 차등(`release_diff.py`)은 같은 원리를 **시간축**(구/신 바이너리)으로
돌린다. 이 문서가 다루는 것은 형식축 — 같은 문서의 HWP 판과 HWPX 판이다.
두 도구의 분류 삼원을 섞지 말라. 이쪽의 심각도는 `contradiction` /
`review` / `other-doc` 이고, 저쪽의 분류는 `stable` / `regression` /
`surface-changed` 다.

## 1. 왜 이 기둥이 필요한가

운동장 채점기는 정답을 골든으로 박제하지 않고, 채점 시점에 rhwp 로 다시
계산한다. 그러니 같은 문서의 두 형식에 같은 관측을 물리면 답이 같아야 한다.
갈리면 둘 중 하나의 읽기 경로가 틀린 것이다. 기대값을 아무도 적어두지 않은
자리까지 훑는 차등 테스트가 공짜로 생긴다.

저장소에 쌍둥이 픽스처가 139쌍 있다. 관측 6종이면 즉시 834건의 판정이 된다.
다만 이름이 같다고 같은 문서라는 보장은 없다. 개정판을 각각 저장한 쌍이
있다. 관문이 없으면 도구가 거짓말을 한다.

이 도구는 "무엇이 갈렸나" 를 가리키지 "어느 형식이 옳은가" 를 판정하지
않는다. 한컴 정답지가 없다. 판정은 사람이 한다.

## 2. 사용

```bash
python gym/tools/differential.py
python gym/tools/differential.py --limit 20
python gym/tools/differential.py --bin target/debug/rhwp -o gym/differential-report.json
python gym/tools/differential.py --cli-timeout 30
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--limit` | `0` | 검사할 쌍 수. `0` 이하면 전부. 접두는 정렬된 입력의 앞부분이라 결정적이다. |
| `--bin` | `runner.find_bin` | rhwp 바이너리. 상대경로는 러너가 절대화한다. |
| `-o` / `--out` | `gym/differential-report.json` | JSON 보고 경로. UTF-8 · BOM 없음 · LF. |
| `--cli-timeout` | `0` | 관측 CLI 초. `0` 이하는 무제한(기존 동작). |

프로브 명령은 없다. 관측 명령은 아래 표의 여섯 줄이다. 본문 해시는
`export-text --json`, IR 동일성은 `ir-diff --json` 이다.

## 3. 오검출 관문 — 정직 조항

`classify_pair(body_same, ir_identical, diverged)` 는 네 칸만 낸다.

| 관측이 갈렸나 | 본문 해시가 같나 | IR 동일인가 | 라벨 | findings 에 넣나 |
|---|---|---|---|---|
| 아니오 | * | * | `None` | 아니오 |
| 예 | 아니오 | * | `other-doc` | 아니오 |
| 예 | 예 | 아니오 | `review` | 예 |
| 예 | 예 | 예 | `contradiction` | 예 |

규칙:

1. **갈림이 없으면 침묵.** 본문·IR 을 보지도 않는다. 같은 관측은 결함이
   아니다.
2. **본문 해시가 다르면 다른 문서다.** 이름이 같아도 개정판이다. 결함으로
   부르지 않는다. `sameNameDifferentDocument` 만 올린다.
3. **본문이 같고 IR 이 다른데 관측이 갈리면 `review`.** rhwp 자신도 구조가
   다르다고 말한 자리. 사람 판정 몫.
4. **본문이 같고 IR 이 같은데 관측이 갈리면 `contradiction`.** 내부 모순.
   우선순위가 높다.
5. **본문 해시를 못 구하면 동일 문서로 치지 않는다.** `None == None` 은
   거짓이다. 양쪽 export-text 가 같이 죽어도 `contradiction` 이 아니다.
6. **IR 을 못 구하면 `identical=False` 다.** 못 본 것을 `contradiction`
   으로 부르지 않는다. 본문이 같으면 `review`.

`diverged` 인자는 목록·건수·bool 을 모두 받는다. 파이썬 진릿값으로만 본다.

- 거짓으로 접히는 값(`None`, `0`, `""`, `[]`, `{}`, `False`) → 갈림 없음.
- 그 외(`1`, `[행]`, `"x"`, `True`) → 갈림 있음.

`classify_pair` 는 예외를 관측으로 접지 않는다. 호출자가 해시·IR 을 정직하게
넣어야 한다.

## 4. 짝짓기 — 결정적 대표 경로

한 줄기에 `.hwp` / `.hwpx` 가 여러 개일 수 있다. `pick_twin_paths` 가
대표 한 쌍을 고른다. walk 순서에 의존하지 않는다.

1. 경로를 `/` 로 정규화한 뒤 **디렉터리** 를 본다.
2. 같은 디렉터리에 양쪽이 있으면 그 짝을 쓴다. 그런 디렉터리가 여러 개면
   디렉터리 경로가 사전순으로 앞선 것.
3. 같은 디렉터리 짝이 없으면 **얕고 사전순** 인 경로(`path_rank` =
   `(슬래시 수, 정규화 경로)`).
4. 한쪽만 있으면 쌍이 아니다. 지어내지 않는다.
5. 비문자·빈 경로는 순위에서 뺀다. 남는 유효 경로의 순위 규칙은 그대로다.

확장자는 대소문자를 접는다(`.HWP` = `.hwp`). 줄기는 `os.path.splitext` 의
앞부분이다. `find_twins_in` 은 줄기 사전순, 그다음 hwp 경로, 그다음 hwpx
경로로 정렬한다.

디렉터리가 없거나 `os.walk` 가 `OSError` 로 죽으면 빈 목록이다. 없는 쌍을
지어내지 않는다.

`--limit` 는 이 정렬된 목록의 접두다. 음수·변환 불능은 0 과 같다(전부).

## 5. 본문 해시 — 없음은 동일이 아니다

1차 관문은 공백을 무시한 본문의 SHA-256 이다.

| 봉투 | 해시 | `same_body_hash` |
|---|---|---|
| `{"pages":[{"text":"가 나"},{"text":"다"}]}` | `hash("가나다")` | 같은 글자면 참 |
| `{"pages":[{"text":"가나다"}]}` | 위와 같음 | 참 |
| `{"pages":[]}` | 빈 문자열의 해시 | 빈 본문끼리 참 |
| `{"pages":None}` / 키 없음 / 비-dict | `None` | 거짓 |
| export-text 예외 | `None` | 거짓 |
| 양쪽 다 `None` | — | **거짓** |

규칙:

- `pages_text` 가 `None` 을 내면 해시 함수를 부르지 않는다. 빈 문자열로
  위장하지 않는다.
- `normalize_body` 는 `str.split()` 이 지우는 공백(개행·탭·CR)만 접는다.
  글자·문장부호는 접지 않는다.
- `hash_text(None)` 은 `None` 이다. 빈 문자열의 해시로 바꾸지 않는다.
- `same_body_hash(None, None)` 은 거짓이다. 양쪽이 같이 실패한 것을
  "같은 문서" 로 부르면, 바이너리가 없을 때 전 쌍이 침묵한다.

실측 근거: 표본 25쌍에서 어긋난 2건 중 1건은 본문 해시부터 달랐다. 다른
개정판이었다. 결함이 아니었다.

## 6. 관측 kind

한 자리의 결과는 판정이 아니라 관측이다. HWP/HWPX 가 같은 kind·같은 값이면
갈림이 아니다.

| kind | 언제 | 표시 |
|---|---|---|
| `value` | JSON 봉투에 키가 있다 | raw 값 |
| `nojson` | 종료는 됐는데 봉투가 없다 | `exitN` |
| `badenv` | 봉투가 dict 가 아니다 | `badenv` |
| `missing` | 봉투에 키가 없다 | `None` |
| `timeout` | CLI 시간초과 | `timeout` |
| `missing-bin` | 실행 파일이 없다 | `missing-bin` |
| `permission` | 실행 권한이 없다 | `permission` |
| `os-error` | 그 외 OS 오류 | `os-error` |
| `type-error` | 주입 run 의 형식이 틀렸다 | `type-error` |
| `value-error` | JSON/값 오류 | `value-error` |
| `decode-error` | 유니코드 오류 | `decode-error` |
| `cli-error` | `RuntimeError` | `cli-error` |
| `unexpected` | 카탈로그 밖 | `unexpected` |

`observation_from_result` 는 종료 코드를 문자열로 붙이지 않는다. `nojson`
의 표시 `exit1` 과 값 `"exit1"` 은 다른 관측이다. 시험이 이 충돌을 고정한다.

같은 오류가 양쪽 에 나면 갈림이 아니다. 한쪽만 나면 갈림이다. 그다음 관문이
본문 해시와 IR 을 본다. 오류 관측이 관문의 순서를 바꾸지 않는다.

## 7. 관측 동일성

`observations_equal` / `_values_equal` 계약:

| 왼쪽 | 오른쪽 | 같은가 | 왜 |
|---|---|---|---|
| `6` | `6.0` | 예 | JSON 숫자의 int/float 요동 |
| `True` | `1` | 아니오 | bool 을 int 로 접지 않는다 |
| `False` | `0` | 아니오 | 위와 같음 |
| `{b:1,a:2}` | `{a:2.0,b:1}` | 예 | 키 순서 무관, 숫자 정규화 |
| `{"kind":"nojson","code":1}` | `{"kind":"value","value":"exit1"}` | 아니오 | 종류가 다르면 표시가 같아도 다르다 |
| `NaN` | `NaN` | 예 | 둘 다 비숫자. 차등으로 오신고하지 않는다 |
| `+inf` | `-inf` | 아니오 | 부호가 다른 무한대 |
| `[1,[2,3.0]]` | `[1.0,[2,3]]` | 예 | 중첩 숫자 정규화 |
| `b"ab"` | `"ab"` | 아니오 | 바이트와 문자열은 다르다 |
| `timeout` 관측 | `timeout` 같은 페이로드 | 예 | 같은 실패는 갈림이 아니다 |

## 8. JSON 봉투

`kind=gymDifferential`, `schemaVersion=1.0`. 키 집합은 시험이 `REPORT_KEYS` 로
고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymDifferential` |
| `schemaVersion` | str | 항상 `1.0` |
| `ok` | bool | `contradictions == 0` 과만 참 |
| `runner` | obj | `{bin}` |
| `pairs` | int | 검사한 쌍 수 |
| `observationsCompared` | int | 관측 대조 건수 |
| `sameNameDifferentDocument` | int | 본문 해시가 달라 제외한 쌍 |
| `findings` | list | `review` / `contradiction` 만. 줄기 순 |
| `contradictions` | int | `severity==contradiction` 집계 |
| `reviews` | int | `severity==review` 집계 |
| `exit` | int | 0 / 1 / 3 |
| `toolFailed` | bool | 도구 자리 오류가 있나 |

부가 키:

| 키 | 언제 |
|---|---|
| `toolErrors` | find-bin / walk / compare 실패. 심각도를 뒤집지 않는다 |
| `pairErrors` | 한 쌍 루프의 예외. 그 쌍만 건너뜀 |
| `writeError` | JSON 쓰기 실패. 종료 코드는 이미 계산한 상태를 따른다 |

`validate_report` 가 정직 계약을 검사한다.

- `ok` 는 `contradictions==0` 과만 같다.
- `contradictions` / `reviews` 는 findings 의 severity 집계와 같다.
- `contradiction` 행은 `irIdentical` 이 참이어야 한다.
- `review` 행은 `irIdentical` 이 거짓이어야 한다.
- `other-doc` 은 findings 에 없다.
- `toolFailed` 가 참이면 `exit` 는 1. 그래도 심각도 집계는 뒤집지 않는다.
- finding 의 `diverged` 는 비면 안 된다.

## 9. 예외 경로 — 도구가 죽지 않는 자리

감사기(이 차등 도구) 자신은 한 쌍의 CLI 예외로 멈추지 않는다. 없는
바이너리, 권한, 시간초과, 깨진 stdout, walk 실패는 관측/오류 목록으로
남기고 다음으로 간다.

| 자리 | 잡는 것 | 접는 곳 |
|---|---|---|
| `run_cli` | timeout · missing-bin · permission · OSError · decode | 관측 |
| `observe_with_run` | 주입 run 의 예외·형식 오류 | 관측 |
| `body_hash` / `body_hash_with_run` | 위 | `None` 해시. 동일 문서 아님 |
| `ir_identity_with_run` | 위 | `(False, None)`. contradiction 금지 |
| `find_twins_in` | 디렉터리 부재 · walk OSError | 빈 목록 |
| `find_bin` | OSError | 경로 유지 + toolErrors |
| `write_report` | OSError | `writeError` |
| 한 쌍 루프 | 그 외 예외 | `pairErrors`. 그 쌍만 건너뜀 |

한 쌍을 못 봐도 다른 쌍은 계속 비교한다. 쌍 오류는 심각도 집계를 뒤집지
않는다. 비교한 쌍만으로 모순/리뷰를 센다.

`KeyboardInterrupt` · `SystemExit` · `MemoryError` · `GeneratorExit` 는
삼키지 않는다. 사용자가 끊었는데 모순 0건이라고 쓰면 거짓말이다.

## 10. 종료 코드

| exit | 의미 |
|---|---|
| 0 | 모순 없음 (`ok`) 그리고 도구 실패 없음 |
| 1 | 도구 실패 (`toolFailed`). 모순 집계는 봉투에 남긴다 |
| 3 | IR 동일 모순 (`contradictions ≥ 1`) |

도구 실패가 모순보다 앞선다. 바이너리를 못 찾았는데 exit 0 을 내면, 못 본
것을 본 척하는 것이다. contradiction 이 있어도 도구 실패면 1 이다. 모순
목록은 그대로 남긴다.

`ok` 의 뜻은 바뀌지 않는다. `ok` 는 여전히 `contradictions==0` 이다.
리뷰만 있는 보고는 `ok=true`, exit 0 이다. 리뷰는 사람 몫이지 자동 차단이
아니다.

쓰기 실패는 예외로 도구를 죽이지 않고 `writeError` 에 남긴다. 종료 코드는
이미 계산한 상태를 따른다. 디스크가 가득 찼다고 모순을 없던 일로 바꾸지
않는다.

## 11. 보고 쓰기 형식

`write_report` 는 UTF-8, BOM 없음, LF, 마지막 개행 하나, `ensure_ascii=False`,
`indent=2` 다. 같은 입력이면 바이트가 같다. 시험이 BOM/`\r\n` 부재를 고정한다.

## 12. 오검출 관문 요약

도구가 거짓말하지 않도록 지키는 문:

1. **이름이 같다고 같은 문서가 아니다.** 본문 해시가 1차 관문이다.
2. **해시 부재는 동일 문서가 아니다.** `None==None` 을 참으로 부르지 않는다.
3. **IR 을 못 보면 contradiction 이 아니다.**
4. **같은 오류 양쪽은 갈림이 아니다.** 한쪽만 오류면 갈림이고, 그다음
   관문이 본문·IR 을 본다.
5. **한 쌍의 예외는 전수 스윕을 죽이지 않는다.** 그 쌍만 건너뛴다.
6. **치명 예외는 삼키지 않는다.**
7. **짝짓기는 walk 순서에 의존하지 않는다.** 같은 디렉터리, 그다음 얕고
   사전순.

## 13. 시험이 고정하는 것

`python -m unittest scripts.tests.test_gym_differential`

바이너리 없이 돈다. `run` · `subprocess.run` · `find_bin` 을 목으로
갈아끼운다.

고정하는 축:

- 짝짓기 표(같은 디렉터리 우선, 역순 입력 동일).
- 본문 해시 표(공백 접힘, 글자 불변, `None==None` 거짓).
- 관문 진릿값 표.
- 관측 동일성(숫자·bool·kind·NaN·중첩).
- 예외 kind 카탈로그와 치명 예외 표지.
- `compare_twins` 가 CLI 예외를 관측으로 접음.
- IR 예외 → review. 해시 예외 → other-doc.
- `validate_report` 정직 계약.
- `main` 의 exit 0/1/3.

## 14. 이 도구가 하지 않는 것

- 한컴 문서가 맞는지 틀리는지 말하지 않는다.
- 어느 형식이 "더 옳은지" 고르지 않는다.
- 이름만 같은 다른 문서를 결함으로 부르지 않는다.
- 본문 해시를 못 구했는데 내부 모순이라고 하지 않는다.
- IR 을 못 구했는데 내부 모순이라고 하지 않는다.
- 치명 예외를 삼켜 성공인 척하지 않는다.
- 짝짓기 순위를 walk 순서나 파일 시각으로 바꾸지 않는다.
- 공백 이외의 글자를 정규화하지 않는다.

## 15. 관련 기둥

| 기둥 | 도구 | 질문 |
|---|---|---|
| 교차형식 차등 | `differential.py` | 같은 문서의 두 형식이 같은 관측을 내나? |
| 릴리스 차등 | `release_diff.py` | 두 바이너리가 같은 관측을 내나? |
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` | 마지막 스텝을 빼도 통과하나? |
| 도구 강건성 | `robustness.py` | 손상 입력에 rhwp 가 패닉·행 하나? |

교차형식 차등은 오라클이다. 골든 파일이 없다. 오라클이 해시를 못 봤는데
모순이라고 쓰면 사람 시간이 샌다. 그래서 `None` 해시는 `other-doc` 이다.

## 16. 봉투 표본

아래는 시험이 조립하는 최소 표본이다. 필드의 참/거짓이 집계와 어긋나면
`validate_report` 가 거부한다.

### 16.1 침묵 (갈림 없음)

```json
{
  "kind": "gymDifferential",
  "schemaVersion": "1.0",
  "ok": true,
  "runner": {"bin": "rhwp"},
  "pairs": 4,
  "observationsCompared": 24,
  "sameNameDifferentDocument": 0,
  "findings": [],
  "contradictions": 0,
  "reviews": 0,
  "exit": 0,
  "toolFailed": false
}
```

관측이 같으면 본문 해시와 IR 을 보지 않는다. findings 는 비어 있다.

### 16.2 이름만 같은 다른 문서

관측은 갈렸지만 본문 해시가 다르다. findings 에 넣지 않는다.
`sameNameDifferentDocument` 만 1 올린다. `ok` 는 참이다.

### 16.3 review

```json
{
  "ok": true,
  "contradictions": 0,
  "reviews": 1,
  "findings": [
    {
      "stem": "doc",
      "hwp": "doc.hwp",
      "hwpx": "doc.hwpx",
      "irIdentical": false,
      "irDiffCount": 4,
      "severity": "review",
      "diverged": [
        {
          "observation": "pageCount",
          "hwp": {"kind": "value", "value": 6},
          "hwpx": {"kind": "value", "value": 7}
        }
      ]
    }
  ]
}
```

본문은 같은데 IR 이 다르고 쪽수가 갈렸다. 사람 판정 몫. 자동 차단이 아니다.

### 16.4 contradiction

```json
{
  "ok": false,
  "exit": 3,
  "contradictions": 1,
  "reviews": 0,
  "findings": [
    {
      "stem": "doc",
      "irIdentical": true,
      "irDiffCount": 0,
      "severity": "contradiction",
      "diverged": [
        {
          "observation": "pageCount",
          "hwp": {"kind": "value", "value": 6},
          "hwpx": {"kind": "value", "value": 7}
        }
      ]
    }
  ]
}
```

rhwp 가 "구조는 같다" 고 말한 뒤에도 쪽수가 다르다. 내부 모순이다.

### 16.5 도구 실패

```json
{
  "ok": true,
  "exit": 1,
  "toolFailed": true,
  "contradictions": 0,
  "findings": [],
  "toolErrors": [
    {
      "where": "find-bin",
      "kind": "os-error",
      "error": "OSError",
      "head": "..."
    }
  ]
}
```

바이너리를 못 찾았을 때 `ok=true` · exit 0 이라고 쓰면, 못 본 것을 본
척하는 것이다. `ok` 는 모순 0건이라는 뜻으로 남기고, exit 1 이 도구
실패를 가린다. findings 를 지어내지 않는다.

## 17. 한 쌍의 예외가 관문을 바꾸지 않는 예

같은 줄기, 같은 관측 한 줄.

| HWP 관측 | HWPX 관측 | 본문 해시 | IR | 결과 |
|---|---|---|---|---|
| value 6 | value 6 | (보지 않음) | (보지 않음) | 침묵 |
| value 6 | value 7 | 다름 | (보지 않음) | other-doc |
| value 6 | value 7 | 같음 | identical=false | review |
| value 6 | value 7 | 같음 | identical=true | contradiction |
| timeout | timeout (같은 페이로드) | (보지 않음) | (보지 않음) | 침묵 |
| timeout | value 6 | 둘 다 None | (보지 않음) | other-doc |
| timeout | value 6 | 같음 | 예외 | review |
| timeout | value 6 | 같음 | identical=true | contradiction |
| missing-bin | missing-bin | (보지 않음) | (보지 않음) | 침묵 |
| value 6 | permission | 같음 | identical=true | contradiction |

오류 관측은 값을 대신할 뿐, 오검출 관문의 순서를 바꾸지 않는다. 해시 부재를
`identical=true` 로 건너뛰지 않는다.

## 18. 짝짓기 표

`pick_twin_paths(hwps, hwpxs)` 의 결정 표. 입력을 뒤집어도 같다.

| hwps | hwpxs | 결과 | 왜 |
|---|---|---|---|
| `a.hwp` | `a.hwpx` | 그 쌍 | 유일 |
| `b/a.hwp`, `a.hwp` | `a.hwpx` | `a.hwp` + `a.hwpx` | 같은 디렉터리(루트) |
| `b/a.hwp` | `a.hwpx`, `b/a.hwpx` | `b/a.hwp` + `b/a.hwpx` | 같은 디렉터리 `b` |
| `z/a.hwp`, `a/a.hwp` | 양쪽 hwpx | `a/a.*` | 같은 디렉터리 중 사전순 |
| `deep/n/x.hwp`, `x.hwp` | `other/x.hwpx` | `x.hwp` + `other/x.hwpx` | 같은 폴더 짝 없음, 얕은 쪽 |
| `aa/z.hwp`, `sub/z.hwp` | `sub/z.hwpx` | `sub/z.*` | 같은 폴더 짝이 sub 에만 |
| `[]` | `a.hwpx` | `None` | 한쪽 없음. 지어내지 않음 |

이 표를 바꾸면 139쌍의 대표가 흔들린다. 시험이 표와 역순 입력을 같이
고정한다.

## 19. 관측 여섯 줄

기본 관측은 코드의 `OBSERVATIONS` 다. 시험이 이름 목록을 고정한다.

| 이름 | 명령 | 봉투 키 |
|---|---|---|
| `pageCount` | `info {f} --json` | `pageCount` |
| `tableCount` | `export-tables {f} --json` | `tableCount` |
| `paragraphCount` | `explain {f} --json` | `paragraphCount` |
| `fieldCount` | `fields {f} --json` | `fieldCount` |
| `footnoteCount` | `explain {f} --json` | `footnoteCount` |
| `endnoteCount` | `explain {f} --json` | `endnoteCount` |

`{f}` 는 그 형식의 상대 경로로 바뀐다. 관측을 더 얹으면 판정이 139×N 으로
는다. 관문을 건너뛰면 안 된다.

## 20. 치명 예외

다음 네 값은 관측으로 접지 않는다. 도구를 죽이는 것이 정직하다.

- `KeyboardInterrupt`
- `SystemExit`
- `MemoryError`
- `GeneratorExit`

`run_cli_safe` · `observe_with_run` · `compare_twins` · `write_report_safe` ·
`find_bin_safe` · `find_twins_safe` · `main` 이 모두 다시 올린다.

사용자가 Ctrl+C 를 눌렀는데 "쌍둥이 0쌍 · 결함 후보 0건" 이라고 쓰면,
못 끝낸 스윕을 끝낸 척하는 것이다.
