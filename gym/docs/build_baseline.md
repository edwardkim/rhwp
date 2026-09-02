---
kind: guide
status: active
canonical: gym/docs/build_baseline.md
last_verified: 2026-09-03
---

# gym 기준 풀이 조립기 규약

이 문서는 `gym/tools/build_baseline.py` 의 **자리표 치환**, **경로 안전**,
**부재 산출**, **실패 보고**, **기준 풀이 스텝**, **왕복 요약**을 고정한다.
작업 기록은
[`mydocs/working/gym_build_baseline.md`](../../mydocs/working/gym_build_baseline.md)
를 본다. 시험 계약은
`scripts/tests/test_gym_build_baseline.py` 와
`scripts/tests/test_gym_packs.py` 의 `BaselineResolveTests` 가 기계로
고정한다.

트라젝토리 감사(`trajectory.py`)는 이 조립기를 재사용한다. 부분
트라젝토리도 같은 `build_task` · 같은 `resolve` 를 탄다. 조립기의
공개 서명(`resolve` · `build_task` · `verify_built_task`)을 바꾸면
감사 계약이 깨진다. 이 문서는 그 서명을 유지한 채 예외 자리를
연다.

CLI 플래그는 `--agent` · `--pack` · `--bin` · `--json`뿐이다. 새 플래그·새
pack 을 이 도구에 붙이지 않는다.

## 1. 왜 이 기둥이 필요한가

과제를 손으로 늘리면 "돌아가지 않는 과제" 가 섞인다. pack 이 늘어나는
순간 그 위험은 pack 수만큼 커진다. 그래서 각 pack 은
`reference/<과제ID>.json` 에 **기준 풀이**를 두고, 이 스크립트가 그것을
실행해 제출물을 만든 뒤 곧바로 채점한다.

신규 과제는 이 왕복을 통과해야만 등재된다. 저장소에 들어간 모든 과제는
**풀 수 있음이 실측된 과제**다.

기준 풀이는 정답 노출이므로 `reference/` 로 분리한다. 과제를 푸는
에이전트는 이 폴더를 보지 않는 것이 규칙이다. 보더라도 측정되는 것은
"스스로 경로를 찾는 능력"이 아니게 될 뿐, 채점은 정직하게 돌아간다.

왕복이 없으면 세 가지가 동시에 생긴다.

1. 선언만 있고 돌지 않는 과제.
2. 자리표를 한 개만 바꿔 다세대 계획서가 다음 입력을 잃는 과제.
3. 생성은 됐는데 산출 파일이 없거나 채점이 실패한 과제를 성공으로
   집계하는 보고.

이 도구는 그 세 자리를 닫는다.

## 2. 사용

```bash
python gym/tools/build_baseline.py --agent claude-fable-5
python gym/tools/build_baseline.py --agent claude-fable-5 --pack core-cli
python gym/tools/build_baseline.py --agent claude-fable-5 --pack text-editing --bin target/debug/rhwp
python gym/tools/build_baseline.py --agent maintainer-run --bin target/debug/rhwp --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--agent` | `claude-fable-5` | 제출 폴더 이름. `gym/submissions/<agent>/` |
| `--pack` | 전 pack | 반복 가능. 지정한 pack 만 조립한다. |
| `--bin` | 러너 탐색 | rhwp 바이너리. `runner.find_bin` 이 상대경로를 절대화한다. |
| `--json` | 꺼짐 | 최종 `gymBaselineVerification` 봉투를 stdout에 쓴다. 진행·실패 줄은 stderr다. |

새 플래그는 없다. `--task` / `--limit` / `--out` / `--dry-run` 은 없다.
이 도구의 점은 **왕복을 실제로 돌리는 것**이다.
한 과제만 골라 성공했다고 말하면 나머지 pack 의 구멍이 남는다. 한
과제만 보고 싶을 때는 `--pack` 으로 pack 을 줄인다. 과제 ID 필터는
넣지 않는다.

종료 코드:

| 코드 | 의미 |
|---|---|
| 0 | `failed == 0` |
| 1 | 조립 오류·부재 산출·채점 실패가 한 건이라도 있다 |

기준 풀이 파일이 없는 과제는 `skipped` 다. 실패가 아니다. 왕복
요약의 "기준 풀이 없음" 칸이 그 수다.

`--json` 전수 수용에서는 `skipped`도 완료가 아니다. 봉투의 `ok`는 `taskCount>0`,
`built==taskCount`, `failed==0`, `skipped==0`, 모든 `results[].ok=true`일 때만 참이다.
집계 키는 `built`, `failed`, `skipped`, `missingArtifact`, `failedScore`, `buildError`다.
종료 코드는 봉투의 `exit`와 같다. 자동화는 stdout JSON만 파싱하고 stderr 진행 줄을
합치지 않는다.

작업 디렉터리는 `gym/submissions/<agent>/<pack>/<task>/` 이다. 한
과제를 다시 조립하면 그 폴더를 지우고 시작한다.

전수 수동 실행의 격리 worktree·증적·정리 절차는
[`Gym 벤치마크 수동 운영 매뉴얼`](../../mydocs/manual/gym_benchmark_operations.md)을 따른다.

## 3. 자리표 — 바꾸지 않는 칸

`resolve(token, task, sub_dir)` 의 공개 서명은 그대로다. 토큰 분류는
`classify_token` 이 여덟 칸으로 접는다.

| 자리 | 토큰 예 | 하는 일 |
|---|---|---|
| `exact-input` | `{input}` | 과제 `input` 을 그대로 돌려준다. 경로 구분자는 원본. |
| `exact-sub` | `{sub:edited.hwp}` | 제출 폴더 아래 경로. 부모 폴더를 만든다. 백슬래시는 이스케이프하지 않는다. |
| `embedded-sub` | `{"o":"{sub:a}","p":"{sub:b}"}` | 문자열 안의 `{sub:}` 를 **전부** 바꾼다. 계획서 JSON 을 위해 백슬래시를 두 번 쓴다. |
| `embedded-input` | `src={input}` | 문자열 안의 `{input}` 을 `/` 구분 경로로 바꾼다. |
| `mixed` | `{"in":"{input}","out":"{sub:o}"}` | `{sub:}` 를 모두 바꾼 뒤 `{input}` 도 바꾼다. |
| `literal` | `--json` | 그대로 둔다. |
| `unclosed-sub` | `{sub:o.hwp` | `RuntimeError`. 전 왕복을 죽이지 않고 그 과제만 실패. |
| `not-str` | `None` | 그대로 돌려준다. |

다세대 계획서는 input·output 을 모두 `{sub:}` 로 가리킨다. 첫 하나만
바꾸면 나머지가 리터럴로 남아 엉뚱한 이름의 파일이 생기고 다음 세대가
입력을 잃는다. 이것이 #4664 의 계약이다. #5273 은 그 계약을 **세 개
이상**과 **`{input}` 혼합**까지 연다.

`extract_sub_names` 는 등장 순서를 유지하고 중복도 남긴다.
`unique_sub_names` 는 등장 순서를 유지하되 한 번만 센다.
`count_sub_placeholders` 는 중복을 포함해 센다.

`has_unresolved_placeholder` 는 치환 뒤에 `{sub:` 또는 `{input}` 이
남아 있으면 참이다. 시험이 치환 결과를 이 함수로 검사한다.

## 4. 경로 안전

`{sub:이름}` 의 이름은 제출 폴더 안의 상대경로만 허용한다.
`normalize_rel` / `unsafe_rel_reason` 이 거절 이유를 카탈로그 단어로
남긴다.

| 이유 | 예 | 허용? |
|---|---|---|
| (없음) | `edited.hwp` · `capsules/work.json` | 예 |
| `empty` | `""` · `"   "` · `"."` | 아니오 |
| `not-str` | `None` · `3` | 아니오 |
| `absolute` | `/tmp/x` | 아니오 |
| `drive` | `C:/tmp/x` | 아니오 |
| `unc` | `//server/share` | 아니오 |
| `home` | `~/secret` | 아니오 |
| `parent` | `../escape.hwp` | 아니오 |

`.` 구간과 빈 구간은 접는다. `./edited.hwp` 는 `edited.hwp` 다.
`a//b` 는 `a/b` 다.

불안전 이름은 `RuntimeError("불안전 제출 경로 (parent): ...")` 다.
`ValueError` 로 전 왕복을 죽이지 않는다. `main` 의 잡기 목록은
`CATCHABLE_EXCEPTIONS` 이고 `ValueError` 도 포함한다. 닫히지 않은
자리표는 구현이 `RuntimeError` 로 올린다.

중첩 제출 경로(`capsules/…`)는 미리 만든다. 기준 풀이가 폴더 생성까지
신경 쓰게 하면 풀이가 절차 잡음으로 지저분해진다. `join_sub_path(...,
mkdir=True)` 가 부모를 만든다.

`escape_json_path` 는 계획서 JSON 문자열 안에 넣을 때 백슬래시를 두 번
쓴다. 토큰 전체가 `{sub:이름}` 인 자리(`exact-sub`)는 이스케이프하지
않는다. 이 차이는 기존 기준 풀이·Windows 경로와 호환하기 위한 것이다.

## 5. 기준 풀이 스텝

`STEP_KINDS` 는 다섯 값이다. 시험이 이 튜플을 고정한다.

| 키 | 하는 일 | 자리표 |
|---|---|---|
| `run` | rhwp 를 실행한다. `allowExits` 로 판정성 종료 코드를 허용한다. | 인자 각 칸 |
| `copy` | `from` 을 `to` 로 복사한다. 상대 `from` 은 저장소 루트. 절대 `from` 은 그대로. | `from` · `to` |
| `write_json` | 부속 JSON 을 쓴다. 본문의 `{input}` · `{sub:}` 를 치환한다. | `to` · `body` |
| `keyring_from` | 발급한 키의 공개키로 키링을 조립한다. | `key` · `out` |
| `answer` | 봉투에서 값을 길어 `answer.json` 에 합친다. `const` 는 라이브 호출 없이 쓴다. `len` 은 배열 길이가 답. | `cmd` |

`allowExits` 는 스텝 키로 세지 않는다. `step_keys` 는 이 키를 빼고
정렬한다. `step_kind` 는 `STEP_KINDS` 순서로 첫 알려진 키를 고른다.

`classify_reference` 의 네 라벨:

| 라벨 | 언제 |
|---|---|
| `ok` | `steps` 가 비지 않은 목록이고 모든 스텝이 알려진 키를 가진다 |
| `empty-steps` | `steps` 가 `[]` |
| `malformed-reference` | 기준 풀이가 객체가 아니거나 `steps` 가 목록이 아니다 |
| `unknown-step` | 목록 안에 알려진 키가 없는 칸이 있다 |

`validate_reference` 는 사람용 오류 문자열 목록을 돌려준다. 조립
루프는 이 목록을 강제하지 않는다. 알 수 없는 스텝은 `build_task` 가
`RuntimeError` 로 올린다. 검증 함수는 시험·문서가 같은 표를 보게
하려고 있다.

`collect_sub_names(reference)` 는 전 스텝을 걸어 `{sub:}` 이름을 등장
순·중복 제거로 모은다. 이 목록을 `submit.files` 대신 요구 산출로
승격하지 않는다. 중간 산출(키, 임시 hwp)까지 제출 계약으로 오인하기
때문이다.

## 6. 부재 산출

생성 성공만으로 통과 처리하지 않는다. 과제가 `submit.files` 를
선언했으면 그 파일이 제출 폴더에 있어야 한다.

`expected_artifacts(task)` 는 `submit.files` 만 본다. 상대경로로 쓸 수
있는 것만, 선언 순, 중복 제거. 불안전 칸은 버린다.

`missing_artifacts(sub_dir, expected)` 는 expected 순서로, 파일이 아닌
이름을 남긴다. 폴더만 있고 파일이 없으면 부재다.

`missing_artifact_message(pack_id, task, sub_dir)` 는 한 줄이다.

```
pack-b/T02: 부재 산출: edited.hwp
```

여러 파일이면 쉼표로 잇는다.

```
pack-b/T02: 부재 산출: edited.hwp, plan.json
```

`inspect_built_task` 는 부재를 **채점 전에** 본다. 파일이 없으면
`runner.score_task` 를 부르지 않는다. 없는 산출을 채점해 "제출 폴더
없음" 이나 `file_exists` 실패로 덮지 않는다. 자리는
`kind=missing-artifact` 다.

`submit.files` 가 비었거나 answer 과제면 산출 검사를 건너뛴다.
`answer.json` 은 이 목록에 넣지 않는다. `answer` 스텝이 답을 모으면
조립기가 그 파일을 스스로 쓴다.

## 7. 실패 보고

`verify_built_task(bin_path, pack_id, task, sub_root)` 의 공개 서명은
그대로다. 채점은 같은 pack 경로에서 한다.

```
runner.score_task(task, os.path.join(sub_root, pack_id), bin_path)
```

리터럴 `"/"` 로 붙이지 않는다. Windows 에서 백슬래시 결합과 어긋나
크로스플랫폼으로 깨진다(#4689).

`fold_score_result` 가 채점 봉투를 네 칸으로 접는다.

| 자리 | 결과 | 한 줄 |
|---|---|---|
| `pass` 가 참 | 통과. 반환 `None` | (없음) |
| `error` 필드 | 실패 | `pack/task: {error}` |
| `checks` 중 `ok` 가 아닌 칸 | 실패 | `pack/task: 이름: 이유; ...` |
| 비-dict · `pass` 키 없음 · 검사 목록 없음 | 실패 | `pack/task: 채점 결과가 dict 가 아니다` 또는 `채점 실패` |

기존 계약 한 줄은 유지한다.

```
pack-b/T02: 제출 폴더 없음
```

검사 실패는 이름을 남긴다. 이름 없으면 `op`, 그것도 없으면 `검사`.
이유 없으면 `판정 불일치`.

```
core-cli/T09: 1단계 반영: 없음; 2단계 반영: 없음
```

비-dict 채점 결과를 `result.get("pass")` 로 읽지 않는다. 그 자리는
`AttributeError` 로 전 왕복을 죽일 수 있다. `score_is_pass` 가 비-dict
를 실패로 접는다.

`kind=failed-score` 는 채점이 거부한 자리이지, 조립이 죽은 자리가
아니다. 조립 예외는 `kind=build-error` 다.

## 8. 왕복 루프

`process_one_task` 가 한 과제를 조립·검증하고 집계를 갱신한다.

| 자리 | `kind` | 집계 |
|---|---|---|
| 조립 예외 | `build-error` | `failed` + `buildError` |
| 부재 산출 | `missing-artifact` | `failed` + `missingArtifact` |
| 채점 실패 | `failed-score` | `failed` + `failedScore` |
| 통과 | `ok` | `built` |
| 기준 풀이 파일 없음 | (루프에서 continue) | `skipped` |

사람용 한 줄은 예전 형식이다.

```
기준 풀이 왕복: 성공 12 · 실패 3 · 기준 풀이 없음 1
```

실패가 있으면 부가 줄을 붙인다.

```
  내역: 부재 산출 2 · 채점 실패 1 · 조립 오류 0
```

`failed` 가 0 이면 종료 코드 0, 아니면 1. `skipped` 는 종료 코드를
뒤집지 않는다.

`process_pack` 은 기준 풀이 폴더가 없으면 pack 전체를 건너뛰고
`[pack] 기준 풀이 없음 — 건너뜀` 을 찍는다. 이 자리는 `skipped` 에
넣지 않는다. 과제 단위 건너뜀과 pack 단위 부재를 섞지 않는다.

기준 풀이 JSON 파싱이 죽으면 그 과제만 `build-error` 다. 다음 과제로
간다. 한 파일이 전 왕복을 멈추지 않는다.

## 9. 예외 — 삼키지 않는 자리, 죽이는 자리

`CATCHABLE_EXCEPTIONS`:

- `RuntimeError`
- `OSError`
- `KeyError`
- `IndexError`
- `TypeError`
- `ValueError`
- `json.JSONDecodeError`

`FATAL_EXCEPTIONS` 는 다시 올린다.

- `KeyboardInterrupt`
- `SystemExit`
- `MemoryError`
- `GeneratorExit`

닫히지 않은 `{sub:` 는 예전에는 `str.split("}", 1)` 이 `ValueError` 를
올렸다. `main` 이 `ValueError` 를 잡지 않아 전 왕복이 죽었다. 지금은
`RuntimeError` 로 올리고, 잡기 목록에도 `ValueError` 를 넣는다. 한
과제의 자리표 오류가 나머지 pack 을 가리지 않는다.

`FileNotFoundError` 는 `OSError` 의 하위라 조립 오류로 접힌다. 없는
바이너리를 "채점 실패" 로 부르지 않는다. 그 자리는 조립이 시작되지
못한 자리다.

## 10. 공개 함수 — 시험이 고정하는 이름

아래 이름은 문서·시험·도구가 같이 본다. 바꾸려면 시험을 먼저 바꾼다.

자리표:

- `classify_token`
- `extract_sub_names` · `unique_sub_names` · `extract_placeholders`
- `count_sub_placeholders` · `count_input_placeholders`
- `has_unclosed_sub` · `has_unresolved_placeholder`
- `resolve` · `resolve_args` · `resolve_write_json_body`

경로:

- `normalize_rel` · `unsafe_rel_reason` · `is_safe_sub_name`
- `join_sub_path` · `escape_json_path`

스텝:

- `step_kind` · `classify_step` · `classify_reference`
- `validate_step` · `validate_reference`
- `collect_sub_names`

산출:

- `submit_files` · `expected_artifacts`
- `missing_artifacts` · `artifact_status`
- `missing_artifact_message` · `inspect_built_task`

채점:

- `score_is_pass` · `fold_score_result` · `failed_check_lines`
- `score_failure_message` · `verify_built_task`

집계:

- `empty_counts` · `bump_count`
- `format_summary` · `format_summary_detail` · `summary_exit`

조립:

- `build_task` · `process_one_task` · `process_pack` · `main`

CLI:

- `parse_args` · `cli_flag_names`
- `CLI_FLAGS == ("--agent", "--pack", "--bin")`

## 11. 하지 않는 것

- 새 CLI 플래그를 붙이지 않는다.
- 새 pack · 새 과제를 만들지 않는다.
- `score.py` · `runner.py` 를 고치지 않는다. 채점 계약은 러너의
  몫이다. 이 도구는 봉투를 접을 뿐이다.
- `expert-challenges` · `studio-e2e` · `render-tree` · `tutorial` ·
  `PARK.md` · `release_gate.py` 를 열지 않는다.
- `trajectory.py` 가 의존하는 `build_task` · `resolve` 서명을 바꾸지
  않는다.
- `{sub:}` 이름을 `submit.files` 없이 요구 산출로 승격하지 않는다.
- 기준 풀이 없는 과제를 실패로 부르지 않는다. 그 자리는 `skipped` 다.

## 12. 기존 세 계약 (#4664 · #4689)

`BaselineResolveTests` 가 이미 고정한 세 줄은 그대로다.

1. 한 문자열의 여러 `{sub:}` 를 모두 바꾼다. 치환 뒤에 `{sub:` 가
   남으면 실패.
2. `verify_built_task` 는 `score_task(task, join(sub_root, pack), bin)`
   을 한 번 부른다.
3. `{"pass": false, "error": "제출 폴더 없음"}` 은
   `pack-b/T02: 제출 폴더 없음` 이다.

#5273 이 더하는 세 줄:

4. `{sub:}` 가 세 개여도 전부 바꾼다.
5. `submit.files` 가 선언한 파일이 없으면 채점 전에
   `pack/task: 부재 산출: 이름` 을 남긴다. `score_task` 는 부르지
   않는다.
6. 채점 실패는 검사 이름을 남긴다.
   `core-cli/T09: 1단계 반영: 없음`.

## 13. write_json 본문

`write_json.body` 는 JSON 객체다. 조립기는 `json.dumps` 한 뒤
`{input}` 을 `/` 구분 입력 경로로 바꾸고, `{sub:}` 가 있으면 제출
경로로 바꾼 다음 `json.loads` 로 되돌린다.

자리표가 없는 본문은 라운드트립이다. 키 순서는 `json.dumps` 기본이다.
본문 의미가 바뀌지 않으면 된다.

`to` 는 `resolve` 를 탄다. `{sub:plan.json}` 이면 제출 폴더에
`plan.json` 을 쓴다.

## 14. copy · keyring_from

`copy.from` 이 상대 경로이거나 `{input}` 이면 저장소 루트 아래에
붙인다. 이미 절대 경로면 그대로 쓴다. 시험이 임시 파일을 절대 경로로
심을 수 있게 하려는 자리다.

`copy.to` 는 `{sub:이름}` 이다. 부모 폴더를 만든다.

`keyring_from` 은 `key` JSON 의 `publicKey` 를 읽어 키링을 쓴다.
`schemaVersion` 은 `"1.0"`, `kind` 는 `"keyring"`, `revoked` 는
`null` 이다. `keyId` 는 기준 풀이가 준 문자열이다.

## 15. answer.json

`answer` 스텝이 하나라도 값을 모으면 `answer.json` 을 제출 폴더에
쓴다. 값이 없으면 파일을 만들지 않는다.

`const` 키는 라이브 호출 없이 그 값을 쓴다. `cmd` 키는 `run_step` 으로
봉투를 받고 `dig(env, path)` 로 값을 긴다. `len` 이 참이면 배열
길이가 답이다.

답안 봉투 파싱이 실패하면
`{task}: 답안 봉투 파싱 실패` 다. 빈 객체를 답으로 넣지 않는다.

## 16. 보고 문구 카탈로그

시험이 아래 문구의 머리를 고정한다. 번역하거나 꾸미지 않는다.

| 자리 | 문구 |
|---|---|
| 왕복 요약 | `기준 풀이 왕복: 성공 N · 실패 M · 기준 풀이 없음 K` |
| 실패 내역 | `  내역: 부재 산출 A · 채점 실패 B · 조립 오류 C` |
| 부재 산출 | `{pack}/{task}: 부재 산출: {names}` |
| 채점 error | `{pack}/{task}: {error}` |
| 채점 검사 | `{pack}/{task}: {name}: {error}; ...` |
| 채점 비-dict | `{pack}/{task}: 채점 결과가 dict 가 아니다` |
| 채점 공란 | `{pack}/{task}: 채점 실패` |
| 닫히지 않은 자리표 | `닫히지 않은 {sub:} 자리표: ...` |
| 불안전 경로 | `불안전 제출 경로 ({reason}): ...` |
| 알 수 없는 스텝 | `{task}: 알 수 없는 기준 풀이 단계 [...]` |
| 답안 파싱 | `{task}: 답안 봉투 파싱 실패` |
| pack 부재 | `[{pack}] 기준 풀이 없음 — 건너뜀` |
| 과제 실패 줄 | `  X {message}` |

## 17. 다른 기둥과의 경계

| 도구 | 축 | 이 조립기와의 관계 |
|---|---|---|
| `score.py` | 종점 채점 | 부르지 않는다. `runner.score_task` 만 쓴다. |
| `runner.py` | pack 로드·채점 | import 만. 수정하지 않는다. |
| `trajectory.py` | 경로(마지막 스텝) | `build_task` 를 재사용한다. |
| `discriminate.py` | 약한 오라클 | 음성 대조는 그쪽. 이 도구는 기준 풀이 양 대조. |
| `audit.py` | pack 정합 | 기준 풀이 짝이 있는지만 본다. 왕복은 이쪽. |
| `release_gate.py` | 릴리스 차단 | 열지 않는다. |

판별력 감사는 "일 안 한 제출을 거부하는가" 를 묻는다. 이 조립기는
"기준 풀이가 실제로 제출을 만들고 그 제출이 채점을 통과하는가" 를
묻는다. 두 질문을 한 도구에 넣지 않는다.

## 18. 검증

```bash
python -m unittest scripts.tests.test_gym_build_baseline scripts.tests.test_gym_packs.BaselineResolveTests
python gym/tools/audit.py
```

바이너리 없이 자리표·부재 산출·실패 보고가 고정된다. 라이브 왕복
(`--bin target/debug/rhwp`)은 기존처럼 rhwp 가 필요하다. 가드는 순수
경로만 탄다.

## 19. 실제 기준 풀이가 이 규약을 쓰는 자리

아래는 devel 에 있는 기준 풀이다. 조립기가 자리표를 어떻게 읽는지
고정하려고 인용한다. pack JSON 을 이 PR 이 바꾸지 않는다.

### 19.1 TE01 — exact-input + exact-sub

```
edit replace-text {input} --find 규제 --replace 점검 -o {sub:edited.hwp} --json
```

`{input}` 은 과제 입력 경로 그대로다. `{sub:edited.hwp}` 는 제출
폴더의 `edited.hwp` 다. 토큰이 각각 자리표 전체이므로 이스케이프하지
않는다.

### 19.2 T09 — 한 문자열의 `{sub:}` 하나

`--plan-json` 인자 하나가 JSON 문자열이다. 그 안에
`{sub:plan_out.hwp}` 가 박혀 있다. `classify_token` 은
`embedded-sub` 다. 백슬래시 경로는 두 번 써서 JSON 이 깨지지 않는다.

### 19.3 T10 — 같은 계획서가 두 세대를 가리킨다

두 `run` 스텝이 각각 `{sub:o1.hwp}` · `{sub:o2.hwp}` 를 쓴다. 한
토큰 안의 여러 `{sub:}` 는 T10 본문에는 없지만, 같은 패턴을 한
문자열에 넣으면 `{"input":"{sub:o1.hwp}","output":"{sub:o2.hwp}"}`
가 된다. #4664 가 그 자리를 열었고 #5273 이 세 개 이상을 시험으로
고정한다.

### 19.4 T14 — 중첩 제출 + 중간 산출

`{sub:key.json}` · `{sub:work.capsule.json}` · `{sub:_o.hwp}` ·
`{sub:anchor.ndjson}` · `{sub:keyring.json}` 이 한 기준 풀이에
같이 있다. `submit.files` 는 `work.capsule.json` · `keyring.json` ·
`anchor.ndjson` 뿐이다. `_o.hwp` 와 `key.json` 은 중간 산출이다.
부재 산출 검사는 제출 계약만 본다.

`{sub:capsules/work.json}` 처럼 폴더가 있으면 `join_sub_path` 가
`capsules/` 를 만든다.

## 20. 실패를 접는 순서

한 과제에 대해 `process_one_task` 가 보는 순서다.

1. `build_task` — 스텝을 앞에서 뒤로 적용. 여기서 죽으면
   `build-error`. 산출 검사는 하지 않는다.
2. `inspect_built_task`
   1. `expected_artifacts` 가 비지 않고 `missing_artifacts` 가
      있으면 `missing-artifact`. 채점하지 않는다.
   2. `verify_built_task` → `score_task`.
   3. `fold_score_result`. 통과가 아니면 `failed-score`.
3. 집계를 갱신하고 `  X {message}` 를 찍는다.

이 순서를 바꾸면 부재와 채점 실패가 다시 섞인다. 시험
`test_missing_artifact_short_circuits_score` 가 2.1 을 고정한다.

## 21. Windows 경로

- `exact-sub` 의 반환은 `os.path.join` 이라 구분자가 `\` 일 수 있다.
- `embedded-sub` 는 `escape_json_path` 로 `\` 를 `\\` 로 쓴다.
  계획서 JSON 문자열이 `\` 한 번이면 파서가 이스케이프로 먹는다.
- `embedded-input` 은 `task.input` 의 `\` 를 `/` 로 바꾼다. rhwp 가
  입력을 `/` 로도 받기 때문이다.
- 채점 경로는 `os.path.join(sub_root, pack_id)` 다. 리터럴 `"/"` 로
  잇지 않는다(#4689).
- `list_submission_files` 의 상대경로는 `/` 로 정규화한다. 시험이
  `"n/a.txt"` 를 기대한.

## 22. 용어

| 말 | 뜻 |
|---|---|
| 자리표 | `{input}` 또는 `{sub:이름}` |
| 기준 풀이 | `reference/<과제ID>.json` |
| 왕복 | 조립 → 산출 확인 → 채점 |
| 부재 산출 | 선언된 제출 파일이 없음 |
| 실패 보고 | 채점이 거부한 한 줄 |
| 중간 산출 | `{sub:}` 이지만 `submit.files` 가 아닌 파일 |
| 제출 계약 | `task.submit.files` |
| 공개 서명 | `resolve` · `build_task` · `verify_built_task` · `run_step` · `main` |
