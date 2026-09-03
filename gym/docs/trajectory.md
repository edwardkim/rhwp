---
kind: guide
status: active
canonical: gym/docs/trajectory.md
last_verified: 2026-09-02
---

# gym 트라젝토리 필요성 감사 규약

이 문서는 `gym/tools/trajectory.py` 의 **마지막 스텝 load-bearing 판정**,
**수집 전용 tail**, **예외 경로 계약**, **보고 봉투**를 고정한다. 작업
기록은
[`mydocs/working/gym_trajectory.md`](../../mydocs/working/gym_trajectory.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_trajectory.py` 가 기계로
고정한다.

판별력 감사(`discriminate.py`)는 같은 원리를 **종점**으로 돌린다. 이
문서가 다루는 것은 경로 — 다단계 기준풀이의 마지막 외부 의미 스텝이
채점에 필요한가. 두 도구의 판정 삼원을 섞지 말라. 이쪽의 결과는
`load-bearing` / `theater` 이고, 저쪽의 결과는 약한 오라클(종점
anti-false-pass)이다.

CLI 플래그는 `--bin` 과 `--json` 뿐이다. 새 플래그·새 pack 을 이 도구에
붙이지 않는다.

## 1. 왜 이 기둥이 필요한가

운동장 채점기는 종점 오라클이다. 다단계 과제가 "N 스텝을 하라"고
광고해도, 채점이 마지막 스텝의 산출을 실제로 요구하지 않으면 그 과제는
연극이다. 에이전트는 N-1 스텝만 하고도 만점을 받는다.

2026 에이전트 평가의 합의: 종점만 보면 안 된다. 프론티어는 트라젝토리를
채점하지만 대부분 LLM-judge 아니면 골든 경로로 한다. 둘 다 취약하다.
이 감사기는 골든도 judge 도 없이, 기준풀이에서 마지막 외부 의미 스텝만
빼고 같은 조립기·같은 채점기로 다시 돌린다.

- 부분 트라젝토리가 **통과** → 마지막 스텝이 채점에 무의미. 연극.
- 부분 트라젝토리가 **실패**(빌드 실패 포함) → 마지막 스텝이
  load-bearing. 정상.

이것이 판별력 감사(#4808, 종점: "산출이 입력과 다른가")를 **경로**로 민
것이다. 모든 선언된 스텝이 결과를 바꿔야 한다.

수동 Gym 전건 벤치마크는 이 감사를 실행한다. 연극이 하나라도 있으면 그
벤치마크 결과를 무효화한다. 제품 릴리스의 허용·차단 조건은 아니다.

## 2. 사용

```bash
python gym/tools/trajectory.py --bin target/debug/rhwp
python gym/tools/trajectory.py --bin target/debug/rhwp --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--bin` | 필수 | rhwp 바이너리. 상대경로는 러너가 절대화한다. |
| `--json` | 꺼짐 | 사람용 문장 대신 JSON 봉투. |

프로브 명령은 없다. 부분 트라젝토리 조립은 `build_baseline.build_task`,
채점은 `runner.score_task` 다. 시험은 둘을 목킹해 바이너리 없이 핵심
경로를 고정한다.

작업 디렉터리는 `gym/submissions/_trajectory_audit` 이다. 시작과 끝에
지운다. 제출물 폴더를 남기지 않는다.

## 3. 마지막 스텝 load-bearing — 바꾸지 않는 칸

`audit_one` 의 판정 네 칸은 예외 보강 전과 같다.

| 자리 | 결과 | 이유 |
|---|---|---|
| `score_task` 가 `{"pass": true}` | 연극 (`loadBearing=false`) | 마지막 스텝을 빼도 통과 |
| `score_task` 가 `{"pass": false}` | load-bearing | 마지막 스텝이 필수 |
| `score_task` 가 `pass` 키 없음 | load-bearing | 실패로 접는다 |
| `build_task` 가 `RuntimeError` | load-bearing | 부분 트라젝토리가 유효 제출을 못 만듦 |

`FileNotFoundError` 만 네 칸 밖이다. 없는 바이너리를 load-bearing 으로
부르면 전 과제가 정상으로 위장된다. 그 자리는 `missing-bin` 이다.

단스텝 과제는 감사 대상이 아니다. `taskCount` 에 넣지 않는다. 예외도
아니다.

## 4. 수집 전용 tail

trailing `answer` · `keyring_from` 은 제출을 모으는 내부 단계다. 마지막
실제 에이전트 동작을 보려면 이 tail 을 남기고, 그 앞의 외부 의미 스텝을
빼야 한다.

`COLLECTION_STEP_KEYS = {"answer", "keyring_from"}`.

`last_meaningful_step_index(steps)` 는 뒤에서부터 와 수집 키가 **없는**
매핑을 고른다. 그 인덱스를 `truncate_steps` 가 한 칸만 뺀다. 원본
기준풀이 dict 는 바꾸지 않는다.

| steps | 고르는 인덱스 | 부분 트라젝토리 |
|---|---|---|
| `[run, run]` | 1 | `[run]` |
| `[run, run, answer]` | 1 | `[run, answer]` |
| `[run, keyring_from, answer]` | 0 | `[keyring_from, answer]` |
| `[run, answer, run]` | 2 | `[run, answer]` |
| `[answer, keyring_from]` | 없음 | 감사하지 않음. `collection-only-tail` |
| `[answer]` | 없음 | 단스텝. 예외 아님 |
| `[]` | 없음 | `empty-steps` |

수집 키가 하나라도 있으면 그 스텝은 수집이다. `{"answer":..., "run":...}`
는 tail 로 남긴다. 키가 없는 빈 매핑 `{}` 은 의미 스텝이다.

비매핑 칸(`None`, 문자열, 목록)은 의미 스텝이 아니다. 건너뛴다.

## 5. 기준풀이 분류

`classify_steps` / `classify_reference` 가 한 열을 다섯 라벨로 접는다.

| 라벨 | 언제 | audit 가 하는 일 |
|---|---|---|
| `multi` | 길이 ≥2 이고 의미 스텝이 있다 | 부분 트라젝토리 채점 |
| `single-step` | 길이 1 (수집 전용이어도) | `skipped` 에만 남김 |
| `empty-steps` | `steps` 없음 · `null` · `[]` | `exceptions` |
| `collection-only-tail` | 길이 ≥2 이고 의미 스텝이 없다 | `exceptions` |
| `malformed-reference` | `steps` 가 목록이 아님 · 기준풀이가 객체 아님 | `exceptions` |

T01 처럼 `answer` 한 줄인 과제를 `collection-only-tail` 로 부르지 않는다.
단스텝은 트라젝토리가 아니다. 길이 2 이상일 때만 수집 전용 tail 이
예외다.

`multi_step_tasks` 는 예전처럼 **길이 ≥2** 인 기준풀이만 배출한다.
수집 전용 tail 도 여기에 들어간다. 분류와 예외 기록은 `audit` /
`scan_gym` 의 몫이다.

## 6. 예외 경로 — 침묵하지 않는 자리

탐색이 과제를 건너뛰는 자리는 연극 판정이 아니다. 예전에는 아래 네
자리가 `continue` 로 사라졌다. 그러면 "연극 0"이 탐색을 못 한 자리까지
덮는다.

| kind | 자리 | 연극으로 부르나 | load-bearing 으로 부르나 |
|---|---|---|---|
| `missing-reference` | 과제 JSON 은 있는데 짝 기준풀이 파일이 없다 | 아니오 | 아니오 |
| `empty-steps` | 기준풀이에 `steps` 가 없거나 빈 목록 | 아니오 | 아니오 |
| `collection-only-tail` | 2개 이상인데 전부 수집 키 | 아니오 | 아니오 |
| `missing-bin` | 조립·채점이 `FileNotFoundError` | 아니오 | 아니오 |

부가 예외:

| kind | 자리 |
|---|---|
| `malformed-json` | 과제 또는 기준풀이 JSON 파싱 실패 |
| `malformed-task` | 과제 JSON 이 객체가 아님 |
| `malformed-reference` | 기준풀이가 객체가 아니거나 `steps` 가 목록이 아님 |
| `permission` | 읽기 권한 없음 |
| `os-error` | 그 외 OS 오류 |
| `decode-error` | 유니코드 오류 |
| `value-error` / `type-error` | 값·형식 오류 |
| `unexpected` | 카탈로그 밖. `RuntimeError` 조립 실패는 여기로 가지 않고 load-bearing |

한 과제의 JSON 이 깨져도 다음 과제로 간다. 한 pack 의 예외가 전수
탐색을 멈추게 하지 않는다.

치명 예외(`KeyboardInterrupt` · `SystemExit` · `MemoryError` ·
`GeneratorExit`)는 삼키지 않는다. 사용자가 끊었는데 연극 0건이라고 쓰면
거짓말이다.

## 7. 바이너리 부재 — 전 과제를 정상으로 위장하지 않는다

`runner.find_bin` 은 파일이 없어도 경로 문자열을 돌려준다. 조립기가 그
경로로 자식 프로세스를 띄우면 `FileNotFoundError` 가 난다. 예전 코드는
모든 예외를 load-bearing 으로 접었다. 없는 바이너리로 26과제를 돌리면
"26 전부 load-bearing — 연극 0" 이 된다.

지금 계약:

1. `FileNotFoundError` 는 `missing-bin` 이다. `results` 에 넣지 않는다.
2. 한 과제에서 `missing-bin` 이 나면 나머지 다단계 과제는 조립하지
   않는다. 같은 거짓말을 반복하지 않는다.
3. 이미 끝난 탐색 예외(기준풀이 부재 등)는 그대로 남긴다.
4. `ok` 는 여전히 연극 0건과만 같다. `exit` 는 1, `trusted` 는 거짓,
   `missingBin` 은 참.

시험이 넘기는 더미 `"bin"` / `"rhwp"` 는 경로 구분자가 없다.
`bin_looks_present` 는 이 이름을 파일이 없다고 부르지 않는다. 핵심
경로(분류·연극·load-bearing)는 바이너리 없이 돈다.

`--bin` 이 `dir/rhwp` 또는 `*.exe` 처럼 경로로 보이는데 파일이 없으면
`main` 이 조립 루프에 들어가기 전에 `missing-bin` 봉투를 낸다.

## 8. 탐색 순서 — 결정적

`scan_gym` 은 pack 이름 사전순, 그다음 과제 파일 이름 사전순이다.
`os.listdir` 의 원시 순서에 의존하지 않는다.

| 입력 | 결과 |
|---|---|
| `packs/b-pack/tasks/B.json`, `packs/a-pack/tasks/A.json` | a-pack/A 가 앞 |
| `tasks/notes.txt` | 무시. `*.json` 만 |
| `packs/empty/` (tasks 없음) | pack 자체를 건너뜀 |
| `packs/` 가 없음 | 레코드 0. 도구 오류 없음 |
| `os.listdir(packs)` 가 `OSError` | 레코드 0. `toolErrors` 한 줄 |

`tasks/` 는 있는데 `reference/` 가 없으면 그 pack 의 모든 과제가
`missing-reference` 다.

## 9. JSON 봉투

`kind=gymTrajectoryNecessity`, `schemaVersion=1.0`. 키 집합은 시험이
`REPORT_KEYS` 로 고정한다.

| 키 | 형 | 의미 |
|---|---|---|
| `kind` | str | 항상 `gymTrajectoryNecessity` |
| `schemaVersion` | str | 항상 `1.0` |
| `ok` | bool | `theater == []` 과만 참 |
| `taskCount` | int | 감사한 다단계 과제 수 (`results` 길이) |
| `loadBearing` | int | `results` 중 `loadBearing=true` 건수 |
| `theater` | list[str] | 연극 문구. 기존 문장 유지 |

부가 키:

| 키 | 의미 |
|---|---|
| `results` | `{pack, task, loadBearing, steps, removedStep}` |
| `exceptions` | `{kind, pack, task, path, head}` |
| `exceptionCount` | `exceptions` 길이 |
| `skipped` | 단스텝 `{pack, task, reason, steps}` |
| `skipCount` | `skipped` 길이 |
| `trusted` | 예외 0 이고 도구 실패 아님 |
| `toolFailed` | 도구 자리 오류 또는 `missingBin` |
| `toolErrors` | 탐색 OS 오류 |
| `exit` | 0 또는 1 |
| `missingBin` | 바이너리 부재를 봤는가 |
| `binPath` | 사용한 경로. 판정을 뒤집지 않는다 |

`validate_report` 가 정직 계약을 검사한다.

- `ok` 는 연극 0건과만 같다.
- `taskCount` 는 `results` 길이와 같다.
- `loadBearing` 은 `results` 의 참 집계와 같다.
- 연극 건수는 `loadBearing=false` 결과 수와 같다.
- 예외 행의 `kind` 는 카탈로그 안이다.
- `missingBin` 이면 `exit` 는 1.
- 연극이 있으면 `exit` 는 1.
- `trusted` 가 참이면 예외·`missingBin`·`toolFailed` 가 없어야 한다.

`ok` 와 `exit` 를 섞지 말라. 기준풀이 부재는 pack 정합(`audit.py`)의
본업이다. 이 도구는 그 자리를 예외로 **보여 주고**, 연극 유무는 그대로
`ok` 에만 담는다. 바이너리 부재는 `ok` 를 뒤집지 않고 `exit=1` 로
가린다. 게이트가 "연극 0"을 도구 실패와 혼동하지 않게 하려는 것이다.

## 10. 연극 문구

기존 계약 문장을 유지한다. 게이트 로그·시험이 이 문자열을 본다.

```
{pack}/{task} (마지막 실제 스텝 {removedStep}을 빼도 통과 — {N}→{N-1})
```

`removedStep` 은 뺀 스텝의 키를 정렬해 `/` 로 이은 것이다. `{"run":...}`
→ `run`. `{"b":1,"a":2}` → `a/b`.

## 11. 종료 코드와 사람용 본문

| 상황 | ok | exit | 본문 |
|---|---|---|---|
| 다단계 N건 전부 load-bearing, 예외 0 | 참 | 0 | `N 다단계 과제 전부 마지막 스텝이 load-bearing — 연극 0` |
| 연극 K건 | 거짓 | 1 | `연극(무의미한 마지막 스텝) K건` + 각 줄 |
| 연극 0, missing-bin | 참 | 1 | `연극 0 · 도구 실패` |
| 연극 0, 기준풀이 부재 등 예외만 | 참 | 0 | 성공 문장 + `예외 경로 N건` |
| 보고 봉투가 dict 아님 | — | — | `보고 봉투가 아니다` |

예외가 있으면 본문 아래에 `예외 경로 N건:` 과
`kind: pack/task — head` 를 붙인다.

## 12. 예외 kind 카탈로그 (기계 표)

시험 `EXCEPTION_KINDS` 와 같은 순서의 앞 네 칸이 필수 경로다.

```
missing-reference
empty-steps
collection-only-tail
missing-bin
malformed-json
malformed-task
malformed-reference
permission
timeout
os-error
decode-error
value-error
type-error
unexpected
```

`exception_kind(exc, context)` 매핑:

| 예외 | context=`audit` | context=`load` |
|---|---|---|
| `FileNotFoundError` | `missing-bin` | `missing-bin` |
| `json.JSONDecodeError` | `value-error` | `malformed-json` |
| `PermissionError` | `permission` | `permission` |
| `TimeoutError` | `timeout` | `timeout` |
| `UnicodeError` | `decode-error` | `decode-error` |
| `TypeError` / `AttributeError` | `type-error` | `type-error` |
| `ValueError` / `KeyError` / `IndexError` | `value-error` | `value-error` |
| `OSError` | `os-error` | `os-error` |
| `RuntimeError` | `unexpected` (행으로 안 남김, load-bearing) | `unexpected` |
| `None` | `unexpected` | `unexpected` |

카탈로그 밖 `kind` 를 `exception_row` 에 넣으면 `unexpected` 로 접는다.

## 13. 정직 행렬

`ok` / `exit` / `trusted` 를 한 표로 고정한다. 시험
`GeneratedHonestyMatrixTests` 가 같은 일곱 줄을 본다.

| 연극 | missing-bin | 그 외 예외 | ok | exit | trusted |
|---|---|---|---|---|---|
| 없음 | 없음 | 없음 | 참 | 0 | 참 |
| 있음 | 없음 | 없음 | 거짓 | 1 | 참 |
| 없음 | 있음 | 없음 | 참 | 1 | 거짓 |
| 있음 | 있음 | 없음 | 거짓 | 1 | 거짓 |
| 없음 | 없음 | 있음 | 참 | 0 | 거짓 |
| 있음 | 없음 | 있음 | 거짓 | 1 | 거짓 |
| 없음 | 있음 | 있음 | 참 | 1 | 거짓 |

`trusted=true` 이면서 예외가 있는 봉투는 `validate_report` 가 거절한다.

## 14. 분류 표본

시험 `CLASSIFY_CASES` / `GeneratedClassifyTableTests` 와 같은 표다.

| steps | 라벨 |
|---|---|
| `None` | `empty-steps` |
| `[]` | `empty-steps` |
| `"nope"` | `malformed-reference` |
| `{"run":["a"]}` | `malformed-reference` |
| `[{"run":["a"]}]` | `single-step` |
| `[{"answer":{}}]` | `single-step` |
| `[{"keyring_from":"x"}]` | `single-step` |
| `[{"run":["a"]},{"run":["b"]}]` | `multi` |
| `[{"run":["a"]},{"answer":{}}]` | `multi` |
| `[{"run":["a"]},{"keyring_from":"x"}]` | `multi` |
| `[{"answer":{}},{"keyring_from":"x"}]` | `collection-only-tail` |
| `[{"answer":{}},{"answer":{}},{"keyring_from":"x"}]` | `collection-only-tail` |
| `[{},{"answer":{}}]` | `multi` |
| `[{"run":["a"]},{"run":["b"]},{"run":["c"]}` | `multi` |

## 15. 함수 지도

핵심 경로는 순수 함수다. 시험이 바이너리 없이 각 칸을 고정한다.

| 함수 | 순수 | 역할 |
|---|---|---|
| `is_collection_step` | 예 | 수집 키 교집합 |
| `last_meaningful_step_index` | 예 | 뒤에서 첫 의미 스텝 |
| `truncate_steps` / `truncate_reference` | 예 | 한 칸만 뺀 사본 |
| `classify_steps` / `classify_reference` | 예 | 다섯 라벨 |
| `exception_kind` / `exception_row` | 예 | 예외 접기 |
| `verdict_from_score` | 예 | pass → 연극 여부 |
| `verdict_from_build_error` | 예 | fatal / missing-bin / load-bearing |
| `make_theater_line` | 예 | 기존 문구 |
| `report_ok` / `report_exit` / `report_trusted` | 예 | 정직 행렬 |
| `validate_report` | 예 | 봉투 계약 |
| `scan_task_pair` / `scan_gym` | 파일 | 결정적 탐색 |
| `audit_one` / `audit` | 조립 주입 | 한 과제 / 전수 |
| `render_text_report` | 예 | 사람용 본문 |
| `main` | CLI | `--bin` `--json` 만 |

`safe_load_json` · `safe_listdir` · `safe_isdir` · `safe_isfile` 은
치명 예외만 다시 올린다. 나머지는 `(값, 예외)` 로 접는다.

## 16. 시험 지도

`scripts/tests/test_gym_trajectory.py` 가 고정하는 묶음.

| 클래스 | 고정하는 것 |
|---|---|
| `TrajectoryTests` | 기존 5칸. 연극·load-bearing·빌드실패·단스텝·answer tail |
| `CollectionStepTests` | 수집 키, 빈 매핑, 비매핑 |
| `LastMeaningfulStepTests` | 인덱스 선택 |
| `TruncateTests` | 원본 불변, 범위 밖 |
| `ClassifyStepsTests` | 분류 표 |
| `ExceptionKindTests` | kind 매핑, 치명 예외 |
| `VerdictTests` | pass / 문구 |
| `ReportContractTests` | 봉투 정직 |
| `ScanDiscoveryTests` | 네 예외 경로 + JSON 파손 |
| `MissingBinTests` | 부재를 load-bearing 으로 안 부름 |
| `MixedGymTests` | 한 gym 에 예외·연극·단스텝 혼재 |
| `RenderTextTests` | 사람용 본문 |
| `AuditOneTests` | 한 과제, 치명 예외 전파 |
| `SafeIoTests` | JSON/디렉터리 접기 |
| `MainCliTests` | `--json` 부재 경로, 텍스트 성공 |
| `Generated*Tests` | 분류 표·카탈로그·정직 행렬 |
| `LoadBearingLogicKeptTests` | 예외 보강 뒤에도 네 칸 유지 |

새 CLI 플래그 시험은 없다. 플래그를 늘리지 않았기 때문이다.

## 17. 이 도구가 하지 않는 것

- 새 pack · 새 과제 · 새 기준풀이를 만들지 않는다.
- `discriminate.py` · `fuzz_corpus.py` · `release_gate.py` ·
  `release_diff.py` 를 고치지 않는다.
- `gym/core/checks.py` · coverage · leaderboard 등 열린 PR 파일을
  고치지 않는다.
- automation / core-cli / casual-rides pack JSON 을 고치지 않는다.
- LLM-judge 나 골든 경로를 도입하지 않는다.
- 단스텝 과제를 연극으로 부르지 않는다.
- 기준풀이 부재를 연극으로 부르지 않는다.
- 없는 바이너리를 load-bearing 으로 부르지 않는다.

## 18. Gym Benchmark Validation과의 관계

`gym-release-gate.yml`의 역사적 파일 경로는 유지하지만 표시 이름과 역할은
`Gym Benchmark Validation`이다. 수동 전건 실행에서 이 스크립트의 원문 JSON과
종료 코드를 증적으로 남긴다.

- 연극 → `exit=1`. 해당 Gym 결과 무효.
- missing-bin → `exit=1`. "연극 0" 위장 방지.
- 기준풀이 부재만 → `exit=0`. pack 정합은 `audit.py`가 별도로 판단한다.

이 종료 코드는 제품 PR/devel/main CI, 태그, 릴리스·게시를 차단하지 않는다.

## 19. 작업 디렉터리와 결정성

- 조립 뿌리는 `work_root/<pack_id>` 다. `score_task` 에 넘기는
  `sub_root` 도 그 경로다. 기존과 같다.
- 레코드 순서는 pack · 파일 이름 사전순이다.
- 연극 문구·예외 행·JSON 키는 호스트 시각·난수를 넣지 않는다.
- `head` 는 `ERROR_HEAD_LIMIT`(160) 에서 자른다. 경로 전체가 판정을
  바꾸지 않는다.

## 20. 관련 문서

- 작업 기록: `mydocs/working/gym_trajectory.md`
- 운동장 개요: `gym/README.md` (트라젝토리 절)
- pack 정합 감사: `gym/tools/audit.py` (기준풀이 짝·ID 고유)
- 종점 판별력: `gym/tools/discriminate.py` (이 가지에서 수정하지 않음)
- 이슈: [#5254](https://github.com/edwardkim/rhwp/issues/5254)
- 원 도입: [#4810](https://github.com/edwardkim/rhwp/issues/4810) /
  PR #4811

## 21. 표본 시나리오

아래 일곱 줄은 단위 시험이 각각 한 번씩 조립한다. 라이브 바이너리는
쓰지 않는다.

### 21.1 연극

기준풀이 `[run, run]`. 목 채점이 `pass=true`. 보고는
`ok=false`, theater 한 줄, `exit=1`. 예외 0.

### 21.2 load-bearing 채점 실패

같은 기준풀이. 목 채점이 `pass=false`. `ok=true`, `loadBearing=1`,
`exit=0`.

### 21.3 load-bearing 조립 실패

`build_task` 가 `RuntimeError`. 채점 함수는 호출되지 않아도 된다.
`ok=true`, 예외 0, `loadBearing=1`.

### 21.4 단스텝 무시

`[run]` 한 칸. 채점이 통과해도 `taskCount=0`. 예외 0. T01 의
`[answer]` 도 같다.

### 21.5 answer tail 유지

`[run, answer]`. 조립기에 넘어가는 steps 는 `[answer]` 뿐이다.
`removedStep=run`.

### 21.6 네 예외

한 gym 에 기준풀이 없는 과제, `steps=[]`, `[answer, keyring_from]`,
단스텝, 다단계 연극을 같이 심는다. 예외 kind 세 개와 skip 1, theater
1 이 동시에 난다. 단스텝은 예외 목록에 없다.

### 21.7 missing-bin

`build_task` 가 `FileNotFoundError`. `loadBearing=0`, `missingBin=true`,
`ok=true`, `exit=1`. 다음 다단계 과제는 조립하지 않는다.

## 22. 기존 5칸 회귀

원 시험 클래스 `TrajectoryTests` 의 다섯 메서드 이름과 주장은 유지한다.

| 메서드 | 주장 |
|---|---|
| `test_flags_theater_when_truncated_passes` | theater 문구 정확 일치 |
| `test_load_bearing_when_truncated_fails` | ok, theater=[], loadBearing=1 |
| `test_build_error_means_load_bearing` | RuntimeError → ok, theater=[] |
| `test_single_step_tasks_are_ignored` | taskCount=0, ok |
| `test_removes_last_meaningful_step_but_keeps_answer_collection` | 조립 steps == `[answer]` |

이 다섯이 깨지면 예외 보강이 load-bearing 로직을 건드린 것이다. 예외
시험을 고치는 것으로 이 다섯을 느슨하게 만들지 말라.

## 23. 구현 파일 위치

```
gym/tools/trajectory.py          # 도구
scripts/tests/test_gym_trajectory.py
gym/docs/trajectory.md           # 이 규약
mydocs/working/gym_trajectory.md # 작업 기록
```

다른 도구·pack·워크플로 YAML 은 이 규약의 범위 밖이다.

## 24. 예외 행 필드

`exception_row` 가 내는 최소 키.

| 키 | 형 | 비고 |
|---|---|---|
| `kind` | str | 카탈로그 안. 밖이면 `unexpected` |
| `pack` | str | 없으면 빈 문자열 |
| `task` | str | 파일 줄기 또는 과제 id |
| `path` | str | 과제/기준풀이 경로. 판정에 안 씀 |
| `head` | str | 메시지 머리. 160자 |

`exception_from_exc` 는 여기에 `error`(예외 클래스 이름)를 더한다.
`None` 예외는 `error=NoneType`, `kind=unexpected`.

## 25. skip 행 필드

| 키 | 의미 |
|---|---|
| `pack` | pack id |
| `task` | 과제 id |
| `reason` | 지금은 `single-step` 만 |
| `steps` | 기준풀이 스텝 수 (보통 1) |

skip 은 예외가 아니다. `trusted` 를 뒤집지 않는다. 단스텝 과제가 많은
운동장에서 `trusted` 가 항상 거짓이 되는 일을 막는다.

## 26. 결과 행 필드

기존 키를 유지한다.

| 키 | 의미 |
|---|---|
| `pack` | pack id |
| `task` | 과제 id |
| `loadBearing` | bool |
| `steps` | 원본 스텝 수 (부분 트라젝토리 길이가 아님) |
| `removedStep` | 뺀 칸의 키 라벨 |

`steps` 가 부분 길이가 아님에 주의한다. `[run, run, answer]` 에서 run
을 빼도 `steps=3` 이다. 문구의 `3→2` 와 맞춘다.

## 27. 치명 예외 목록

`FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)`.

이 네 가지는 `safe_*` · `audit_one` · `resolve_bin_safe` 가 다시 올린다.
`audit` 는 `audit_one` 이 돌려준 `fatal` 을 그대로 raise 한다. 시험
`test_fatal_is_propagated_from_audit` 가 `KeyboardInterrupt` 를 본다.

`BaseException` 전부를 잡으면 종료와 인터럽트가 "연극 0"으로 접힌다.

## 28. 관련 이슈 좌표

| 이슈 | 역할 |
|---|---|
| #4808 | 종점 판별력. 이 도구의 형제 |
| #4810 / PR #4811 | 트라젝토리 감사 원 도입 |
| #5254 | 이 문서의 예외·문서·시험 보강 |

#5254 는 새 pack 을 요구하지 않는다. 연극이 다시 보이면 해당 pack
가지에서 기준풀이·체크를 고친다. 이 도구 가지에서 pack 을 고치면
열린 pack PR 과 충돌한다.
