---
kind: working
status: active
canonical: mydocs/working/gym_build_baseline.md
last_verified: 2026-08-18
---

# gym 기준 풀이 조립기 — 자리표·부재 산출·실패 보고 고도화

Issue: #5273
Branch: `feat/gym-build-baseline-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/build_baseline.py` 의 공개 서명(`resolve` · `build_task` ·
`verify_built_task`)은 그대로 두고, 한 문자열의 여러 `{sub:}` ·
닫히지 않은 자리표 · 불안전 제출 경로 · 부재 산출 · 실패 보고를 순수
함수로 분리했다. 새 CLI 플래그와 새 pack 은 없다.
`score.py` · `runner.py` · `trajectory.py` · `expert-challenges` ·
`studio-e2e` · `render-tree` · `tutorial` · `PARK.md` ·
`release_gate.py` 는 열지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_build_baseline`
- `python -m unittest scripts.tests.test_gym_packs.BaselineResolveTests`
- `python gym/tools/audit.py`
- `cargo fmt --all -- --check`

## 2. 배경

원 도입(#4653)은 pack 마다 `reference/<과제ID>.json` 을 두고 기준
풀이를 실행해 제출물을 만든 뒤 채점하는 왕복기를 넣었다. #4664 는
한 문자열의 여러 `{sub:}` 를 모두 바꾸게 했다. 다세대 계획서가
input·output 을 모두 `{sub:}` 로 가리키는데 첫 하나만 바꾸면 다음
세대가 입력을 잃기 때문이다. #4689 는 채점 경로를 `os.path.join` 으로
붙여 Windows 에서도 같은 pack 폴더를 보게 했다.

그 상태의 빈틈:

1. `{sub:}` 가 세 개 이상인 계획서 JSON 을 시험이 고정하지 않았다.
   두 개만 바꾸는 구현이 다시 들어와도 기존 세 테스트는 통과한다.
2. `{input}` 이 문자열 안에 박힌 토큰은 그대로 남았다. 토큰 전체가
   `{input}` 일 때만 바꿨다. 계획서 JSON 이 입력을 자리표로 두면
   리터럴 `{input}` 파일이 생긴다.
3. 닫히지 않은 `{sub:` 는 `str.split("}", 1)` 이 `ValueError` 를
   올렸다. `main` 이 `ValueError` 를 잡지 않아 전 왕복이 죽었다.
4. `{sub:../escape}` 를 거부하지 않았다. 기준 풀이 하나가 제출 폴더
   밖으로 쓸 수 있었다.
5. 생성 성공 뒤에 `submit.files` 가 있는지 보지 않았다. 산출이 없어도
   채점 봉투의 `error` 나 `file_exists` 실패로만 남았다. "부재
   산출"과 "채점 실패"가 한 줄로 섞였다.
6. 채점 결과가 비-dict 이거나 `pass` 키가 없으면
   `result.get("pass")` 가 `AttributeError` 를 올리거나 공란을
   "채점 실패"로 뭉갰다. 검사 이름을 남기는 자리는 있었지만 비-dict
   자리는 시험이 없었다.
7. 카탈로그가 코드 주석에만 있어 문서·시험이 같은 표를 공유하지
   않았다.

이슈 #5273 의 DoD 는 `additions >= 3000`, 자리표 치환·부재 산출·실패
보고를 시험으로 고정, `unittest` + `audit.py`, PR 전
`cargo fmt --all -- --check` 다. 새 CLI/pack 금지, 열린 PR
(5210–5272) 파일 미수정.

판정 세 칸(여러 `{sub:}` 치환, pack 경로 채점, `pack/task: 이유`
한 줄)을 다섯 칸으로 늘리면 기존 `BaselineResolveTests` 가 깨진다.
그래서 그 세 칸은 유지하고, 부재 산출은 채점 **앞**에 새 자리로
열었다. 채점 한 줄 형식은 그대로다.

## 3. 한 일

### 3.1 도구

`gym/tools/build_baseline.py`

- `TOKEN_KINDS` — `exact-input` · `exact-sub` · `embedded-sub` ·
  `embedded-input` · `mixed` · `literal` · `unclosed-sub` ·
  `not-str`. 문서·시험이 같은 표를 본다.
- `classify_token` / `extract_sub_names` / `unique_sub_names` /
  `extract_placeholders` / `count_sub_placeholders` /
  `has_unclosed_sub` / `has_unresolved_placeholder`.
- `resolve` — 공개 서명 유지. 세 개 이상의 `{sub:}` 를 전부 바꾸고,
  혼합 토큰의 `{input}` 도 바꾼다. 닫히지 않은 `{sub:` · 불안전
  이름은 `RuntimeError`.
- `normalize_rel` / `unsafe_rel_reason` / `join_sub_path` —
  절대·드라이브·UNC·홈·부모 경로는 쓰지 않는다. 중첩 제출 경로는
  부모를 만든다.
- `STEP_KINDS` / `step_kind` / `classify_reference` /
  `validate_reference` / `collect_sub_names`.
- `submit_files` / `expected_artifacts` / `missing_artifacts` /
  `missing_artifact_message` — `submit.files` 만 요구 산출이다.
  `{sub:}` 이름을 승격하지 않는다.
- `inspect_built_task` — 부재면 `score_task` 를 부르지 않는다.
- `fold_score_result` / `score_failure_message` /
  `verify_built_task` — 비-dict·키 없음은 통과가 아니다. 기존
  `pack-b/T02: 제출 폴더 없음` 한 줄은 유지.
- `process_one_task` / `empty_counts` / `format_summary` /
  `format_summary_detail` — 실패를 부재·채점·조립으로 나눈다. 사람용
  한 줄은 예전 형식.
- `CATCHABLE_EXCEPTIONS` 에 `ValueError` · `JSONDecodeError` 를
  넣었다. `FATAL_EXCEPTIONS` 는 다시 올린다.
- `parse_args` / `CLI_FLAGS` — `--agent` · `--pack` · `--bin` 만.
- `write_json` 본문의 `{sub:}` 도 치환한다. `copy.from` 이 절대
  경로면 ROOT 를 붙이지 않는다. `answer` 의 `const` 는 라이브 호출
  없이 쓴다.

### 3.2 시험

`scripts/tests/test_gym_build_baseline.py` — 순수 함수·목킹 채점.
바이너리 없음.

`scripts/tests/test_gym_packs.py` `BaselineResolveTests` 에 세 칸을
더했다.

- `test_three_sub_placeholders_all_resolve`
- `test_missing_artifact_is_reported_before_score`
- `test_failed_score_lists_check_names`

기존 세 칸은 그대로 통과해야 한다.

### 3.3 문서

- `gym/docs/build_baseline.md` — 규약 정본.
- `mydocs/working/gym_build_baseline.md` — 이 기록.

## 4. 자리표 표

| 입력 | 분류 | 출력 요지 |
|---|---|---|
| `{input}` | exact-input | `task.input` 원본 구분자 |
| `{sub:o.hwp}` | exact-sub | `join(sub, o.hwp)`, mkdir, 이스케이프 없음 |
| `{"a":"{sub:a}","b":"{sub:b}"}` | embedded-sub | 둘 다 경로, `\\` 이스케이프 |
| `{"a":"{sub:a}","b":"{sub:b}","c":"{sub:c}"}` | embedded-sub | 셋 다 경로 |
| `{"in":"{input}","out":"{sub:o}"}` | mixed | input + 모든 sub |
| `src={input}` | embedded-input | `src=` + `/` 구분자 |
| `--json` | literal | 그대로 |
| `{sub:o.hwp` | unclosed-sub | `RuntimeError` |
| `{sub:../x}` | (안전 거부) | `RuntimeError (parent)` |
| `None` | not-str | `None` |

`extract_sub_names("{sub:a}-{sub:b}-{sub:a}")` 는
`["a", "b", "a"]`. `unique_sub_names` 는 `["a", "b"]`.

## 5. 부재 산출 표

| `submit.files` | 제출 폴더 | `kind` | `score_task` |
|---|---|---|---|
| 없음 / answer | (무관) | 채점으로 | 부른다 |
| `["edited.hwp"]` | 파일 있음 | 채점으로 | 부른다 |
| `["edited.hwp"]` | 파일 없음 | `missing-artifact` | 부르지 않는다 |
| `["a", "b"]` | `a` 만 | `missing-artifact` (`b`) | 부르지 않는다 |
| `["../x"]` | (무관) | 요구 목록에서 탈락 | 요구가 비면 채점 |
| `["o.hwp", "o.hwp"]` | 없음 | 부재 한 줄 (`o.hwp`) | 부르지 않는다 |

메시지:

```
pack-b/T02: 부재 산출: edited.hwp
pack-b/T02: 부재 산출: edited.hwp, plan.json
```

중간 산출(`{sub:key.json}`)은 이 목록에 넣지 않는다. 키가 제출
계약이 아닌데 부재로 부르면 서명 과제가 전부 실패한다.

## 6. 실패 보고 표

| 채점 봉투 | 한 줄 |
|---|---|
| `{"pass": true}` | `None` |
| `{"pass": false, "error": "제출 폴더 없음"}` | `pack-b/T02: 제출 폴더 없음` |
| 검사 두 칸 실패 | `core-cli/T09: 1단계 반영: 0; 2단계 반영: 없음` |
| `{}` | `pack/task: 채점 실패` |
| `None` / `"x"` | `pack/task: 채점 결과가 dict 가 아니다` |

`verify_built_task` 는 예전처럼 채점만 본다. 부재 산출은
`inspect_built_task` 가 앞단에서 접는다. 기존 테스트가
`verify_built_task` 를 직접 부르므로 그 함수에 산출 검사를 넣지
않았다. 넣으면 "폴더만 있고 파일 없는" 픽스처가 채점 mock 에 닿지
않아 기존 세 번째 테스트와 새 검사가 섞인다.

## 7. 집계

`COUNT_KEYS`:

- `built`
- `failed`
- `skipped`
- `missingArtifact`
- `failedScore`
- `buildError`

`failed == missingArtifact + failedScore + buildError` 가
`process_one_task` 경로의 불변이다. pack 단위 "기준 풀이 없음" 은
이 합에 넣지 않는다.

사람용 한 줄은 예전 세 칸만 보여 기존 스크립트·로그가 깨지지 않는다.
실패가 있을 때만 내역 줄을 붙인다.

종료 코드는 `failed == 0` 이면 0, 아니면 1. `skipped` 는 영향을
주지 않는다.

## 8. CLI

```
--agent   기본 claude-fable-5
--pack    action=append, 기본 전 pack
--bin     기본 러너 탐색
```

`cli_flag_names()` 가 이 튜플을 돌려준다. 시험이
`("--agent", "--pack", "--bin")` 과 같다고 고정한다. `--json` 을
붙이지 않았다. 이 도구의 출력은 사람용 왕복 로그다. JSON 봉투는
감사기(trajectory·discriminate)의 몫이다.

## 9. 기존 테스트를 깨지 않는 법

`test_gym_packs.BaselineResolveTests` 세 칸:

1. `resolve` 가
   `'{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'` 에서
   `{sub:` 를 남기지 않고 `o1.hwp` · `o2.hwp` 를 포함한다.
2. `verify_built_task` 가
   `score_task(task, join("/tmp/sub", "pack-a"), "/tmp/rhwp")` 를
   한 번 부른다.
3. `{"pass": False, "error": "제출 폴더 없음"}` 이
   `"pack-b/T02: 제출 폴더 없음"` 이다.

구현이 `score_is_pass` 로 `pass` 를 읽고, `error` 가 있으면 그
문자열을 그대로 붙인다. 접두 `부재 산출:` 을 `verify_built_task` 에
넣지 않는다.

`trajectory.py` 는 `build_task(bin, pack, task, reference, sub_root)`
를 부른다. 스텝 적용을 `apply_step` 으로 나눴을 뿐 서명과 반환
(제출 폴더 경로)은 같다. 부분 트라젝토리가 산출을 다 만들지 못하는
것은 그쪽 판정(load-bearing)의 재료이므로, `build_task` 안에서 부재
산출을 예외로 올리지 않는다.

## 10. 열지 않은 파일

지시가 금지한 경로:

- `gym/score.py`
- `gym/core/runner.py`
- `gym/packs/expert-challenges/**`
- `gym/packs/studio-e2e/**`
- `gym/packs/render-tree/**`
- `gym/tutorial/**`
- `gym/PARK.md`
- `gym/tools/release_gate.py`
- 열린 PR 5210–5272 가 이미 고친 파일
  (`discriminate.py`, `trajectory.py`, `fuzz_corpus.py`,
  `coverage.py`, `robustness.py`, `pack_health.py`,
  `competitive_bench.py`, `from_e2e.mjs`, `leaderboard.py`,
  `release_diff.py`, `agent_session.py`, `oracle_probe.py`,
  `gym/docs/*.md` 중 이 도구가 아닌 것, 각 pack 확장 문서)

이 PR 이 만지는 파일:

- `gym/tools/build_baseline.py`
- `scripts/tests/test_gym_build_baseline.py` (신규)
- `scripts/tests/test_gym_packs.py` (`BaselineResolveTests` 만 확장)
- `gym/docs/build_baseline.md` (신규)
- `mydocs/working/gym_build_baseline.md` (신규)

`test_gym_packs.py` 는 5210–5272 의 파일 목록에 없었다. 기존 세
계약을 같은 클래스에 남겨 회귀를 한 파일에서 보게 하려고 확장했다.

## 11. 검증 실측

작업 트리: `C:\Users\swsz9\rhwp-gym-build-baseline`
브랜치: `feat/gym-build-baseline-hardening`
베이스: `upstream/devel`

실측 (2026-08-18, 작업 트리):

```
python -m unittest scripts.tests.test_gym_build_baseline -q
  Ran 128 tests, OK

python -m unittest scripts.tests.test_gym_packs -q
  Ran 18 tests, OK

python gym/tools/audit.py
  gym 정합 감사: 18 pack 전부 통과 — 위반 0

cargo fmt --all -- --check
  exit 0 (crates·tests/generated 스텁은 gitignore, PR 에 넣지 않음)

git diff --cached --shortstat
  5 files changed, 3198+ insertions, 105 deletions
```

`audit.py` 는 pack 정합만 본다. 이 PR 은 pack JSON 을 건드리지
않아 18 pack 이 통과했다.

`cargo fmt --all -- --check` 는 HARD GATE 다. 이 브랜치는 `.rs` 를
바꾸지 않는다. 스파스 워크트리라 `crates/` 를 펼치고,
gitignore 된 `tests/generated/regression_suite_*.rs` 빈 스텁만
로컬에 두어 cargo metadata 가 파일을 찾게 했다. 스텁은 커밋하지
않는다.

## 12. 설계 선택

### 12.1 부재를 채점 앞에 둔 이유

채점기가 `file_exists` 를 돌리면 부재는 결국 실패한다. 그런데
그 실패는 "1단계 반영: 없음" 처럼 검사 이름으로 남는다. 조립기가
파일을 안 만든 것과 기준 풀이가 틀린 검사를 남긴 것이 한 줄로
섞인다. 왕복의 점은 "기준 풀이가 제출을 만들었는가" 와 "그 제출이
맞는가" 를 가르는 것이다. 그래서 부재는 채점 전에 `부재 산출:`
머리로 남긴다.

### 12.2 `{sub:}` 를 요구 산출로 승격하지 않은 이유

T14 는 `{sub:key.json}` 을 만들어 키링을 조립한다. `key.json` 은
제출 계약이 아니다. `submit.files` 는
`work.capsule.json` · `keyring.json` · `anchor.ndjson` 이다. 기준
풀이의 `{sub:}` 전부를 요구하면 중간 파일이 없는 순간을 실패로
부른다. 중간 파일은 다음 스텝이 소비하면 충분하다.

### 12.3 `verify_built_task` 에 산출 검사를 넣지 않은 이유

기존 테스트가 이 함수를 채점 전용으로 부른다. 서명을 유지하면서
행동을 바꾸면 #4689 경로 계약과 #4664 실패 한 줄 계약이 한 함수에
섞인다. 새 자리는 `inspect_built_task` 다. `main` /
`process_one_task` 는 이쪽을 탄다.

### 12.4 `--json` 을 붙이지 않은 이유

지시가 "새 CLI/pack 없음" 이다. JSON 봉투는 감사기의 계약이다.
조립기는 사람용 왕복 로그와 종료 코드만 낸다. 집계 dict 는
함수로 열려 있어 시험이 바이너리 없이 본다.

### 12.5 `ValueError` 를 잡기 목록에 넣은 이유

닫히지 않은 자리표를 `RuntimeError` 로 올려도, 다른 자리
(`json.loads` 본문, `int` 변환)가 `ValueError` 를 올릴 수 있다.
한 과제의 값 오류가 전 pack 을 멈추면 왕복의 점이 사라진다.
치명 예외만 다시 올린다.

## 13. 회귀 시나리오

아래 시나리오는 시험 이름과 1:1 이다. 구현을 되돌리면 이 이름들이
실패한다.

1. 한 문자열의 `{sub:}` 두 개 — `test_multiple_sub_placeholders_all_resolve`
2. 한 문자열의 `{sub:}` 세 개 — `test_three_sub_placeholders_in_plan_json`
3. `{input}` + `{sub:}` 혼합 — `test_mixed_resolves_input_and_all_subs`
4. 닫히지 않은 `{sub:` — `test_unclosed_raises_runtime_error`
5. `{sub:../x}` — `test_unsafe_embedded_sub_raises`
6. 부재 산출이 채점을 건너뜀 — `test_missing_artifact_short_circuits_score`
7. 산출이 있으면 채점 실패를 보고 — `test_present_artifact_then_failed_score`
8. 기존 한 줄 — `test_failed_built_submission_reports_the_task`
9. 검사 이름 — `test_failed_checks_are_joined`
10. 비-dict 채점 — `test_non_dict_score_is_not_a_pass`
11. CLI 세 플래그 — `test_no_new_flags`
12. `write_json` 본문 자리표 — `test_replaces_input_and_sub_in_body`
13. `const` 답 — `test_answer_const_writes_answer_json`
14. 키링 — `test_keyring_from_reads_public_key`
15. 요약 한 줄 — `test_format_summary_legacy_line`

`BaselineResolveTests` 의 원래 세 칸 + 새 세 칸은
`test_gym_packs.py` 가 따로 고정한다. 두 파일이 같은 계약을 두 번
보는 것은 의도다. pack 가드는 조립기 가드를 import 하지 않을 수
있고, 조립기 가드는 pack 가드를 돌리지 않을 수 있다.

## 14. 공개 서명 동결

다음 시그니처는 이 PR 이 바꾸지 않는다.

```
resolve(token, task, sub_dir)
run_step(bin_path, args, task, sub_dir, allow_exits)
build_task(bin_path, pack_id, task, reference, sub_root) -> sub_dir
verify_built_task(bin_path, pack_id, task, sub_root) -> str | None
main() -> int
```

추가된 함수는 위 다섯을 분해한 것이다. 호출자가 다섯만 알아도
왕복은 돈다. 시험은 분해된 이름을 직접 부른다.

## 15. 이후

- 라이브 왕복(`--bin target/debug/rhwp`)은 이 PR 의 가드가 아니다.
  바이너리·샘플이 있는 CI 잡이 기존처럼 돈다.
- `trajectory.py` 가 부분 트라젝토리를 조립할 때 불안전 `{sub:}` 가
  있으면 이제 `RuntimeError` 다. 그 자리는 원래 load-bearing 으로
  접히므로 연극 집계를 뒤집지 않는다. 기준 풀이에 `..` 가 있으면
  그쪽 감사가 예외를 볼 것이다. devel 기준 풀이를 훑었을 때
  `..` 자리표는 없었다.
- JSON 보고 봉투가 필요해지면 별 이슈로 `--json` 을 연다. 이 PR 의
  금지 목록에 걸린다.

## 16. 커밋 단위

한 커밋으로 도구·시험·문서·작업 기록을 같이 올린다. 자리표만
커밋하고 부재 산출을 다음에 올리면 왕복의 두 자리가 갈라진다.
이슈 한 건의 DoD 가 세 자리를 같이 요구한다.

## 17. 함수 책임 한 줄

자리표:

- `classify_token` — 토큰을 여덟 칸으로 접는다.
- `iter_sub_placeholders` — `{sub:이름}` 의 (이름, 시작, 끝).
- `extract_sub_names` — 등장 순, 중복 유지.
- `unique_sub_names` — 등장 순, 중복 제거.
- `extract_placeholders` — input/sub/unclosed 행 목록.
- `has_unclosed_sub` — 닫히지 않은 `{sub:` 가 있는가.
- `remaining_placeholders` — 치환 뒤에 남은 머리.
- `resolve` — 공개 치환. exact/embedded/mixed 를 가른다.
- `replace_embedded_subs` — 문자열 안의 `{sub:}` 전부.
- `replace_embedded_inputs` — 문자열 안의 `{input}` 전부.
- `resolve_args` — run 인자 목록.
- `resolve_write_json_body` — dumps → 치환 → loads.

경로:

- `normalize_rel` — `/` 정규화. 불안전하면 None.
- `unsafe_rel_reason` — empty/not-str/absolute/drive/unc/home/parent.
- `join_sub_path` — 제출 폴더 아래. mkdir 선택.
- `escape_json_path` — `\` → `\\`.
- `require_safe_sub_name` — 거절이면 RuntimeError.

산출:

- `submit_files` — 선언 순, 안전 상대경로만.
- `expected_artifacts` — submit.files 만.
- `missing_artifacts` — expected 중 파일 아닌 것.
- `artifact_status` — expected/present/missing/ok.
- `missing_artifact_message` — `pack/task: 부재 산출: ...`.
- `inspect_built_task` — 부재면 채점하지 않는다.

채점:

- `score_is_pass` — dict 이고 pass 가 참.
- `normalize_score` — 최소 봉투.
- `failed_check_lines` — `이름: 이유` 목록.
- `fold_score_result` — ok/kind/reason/failedChecks.
- `score_failure_message` — verify 한 줄.
- `verify_built_task` — pack 경로 채점.

집계:

- `empty_counts` — COUNT_KEYS 0.
- `bump_count` — 키를 더한다.
- `format_summary` — 예전 한 줄.
- `format_summary_detail` — 부재·채점·조립.
- `summary_exit` — failed==0 → 0.

조립:

- `apply_step` — run/copy/write_json/keyring_from/answer.
- `build_task` — 폴더를 지우고 스텝을 적용.
- `process_one_task` — 조립+검증+집계.
- `process_pack` — pack 의 과제 루프.
- `main` — CLI.

## 18. 시험 클래스 지도

`test_gym_build_baseline.py`:

- `TokenClassifyTests` — 여덟 칸.
- `SubExtractTests` — 이름·개수·남은 자리표.
- `PathSafetyTests` — 부모/절대/드라이브/UNC/홈.
- `ResolveTests` — 두 개·세 개·혼합·닫힘·불안전.
- `WriteJsonBodyTests` — 본문 자리표.
- `StepClassifyTests` — 다섯 스텝·기준 풀이 라벨.
- `ArtifactTests` — submit.files·부재·목록.
- `ScoreFoldTests` — pass/error/checks/비-dict.
- `VerifyBuiltTaskTests` — 경로·한 줄·검사 이름.
- `InspectBuiltTaskTests` — 부재가 채점을 건너뜀.
- `BuildTaskPureStepsTests` — copy/write_json/const/keyring.
- `ProcessOneTaskTests` — 집계 칸.
- `SummaryTests` — 한 줄·종료 코드.
- `CliContractTests` — 플래그 세 개.
- `MultiPlaceholderPlanTests` — 네 개·중첩·중복.
- `MissingArtifactEdgeTests` — 디렉터리·중복·부분 부재.
- `FailedScoreEdgeTests` — 빈 검사·이름 없음·pass 불일치.
- `ProcessPackSkipTests` — pack 부재·파일 부재·JSON 파손.
- `NormalizeScoreTests` / `main` 종료 코드.

`test_gym_packs.BaselineResolveTests`:

- 기존 세 칸 + 세 개 `{sub:}` + 부재 산출 + 검사 이름.

## 19. 열어 본 열린 PR (만지지 않음)

5210 checks · 5211 agent_session · 5212 coverage · 5213 form-journeys
· 5214 oracle_probe · 5221 robustness · 5222 OM/LR/CD · 5225 security
· 5226 selfdesc · 5227 pack_health · 5228 differential · 5232
serialization · 5238 work-receipt · 5239 competitive_bench · 5240
table-editing · 5241 from_e2e · 5242 text-editing · 5244 leaderboard
· 5248 release_diff · 5266 automation · 5267 core/casual · 5268
discriminate · 5269 trajectory · 5270 fuzz_corpus · 5272 release_gate.

이 목록의 파일 경로와 `build_baseline.py` /
`test_gym_build_baseline.py` / `gym/docs/build_baseline.md` /
`mydocs/working/gym_build_baseline.md` 는 겹치지 않는다.
`test_gym_packs.py` 는 목록에 없었다.
