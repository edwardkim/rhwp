---
kind: working
status: active
canonical: mydocs/working/gym_score_runner.md
last_verified: 2026-08-18
---

# gym score/runner 채점기 예외 경로·문서·시험 보강

Issue: #5260
Branch: `feat/gym-score-runner-hardening`
Date: 2026-08-18

## 1. 결론

`gym/score.py` 와 `gym/core/runner.py` 의 채점 성공 칸은 그대로 두고,
예전이 전체를 죽이거나 이유를 남기지 않던 자리를 kind 로 남겼다.
pack 로드 실패는 `status=error` 이지 unavailable 이 아니다. 경로형
바이너리 부재는 카드 `exceptions` 이지 전 과제 0점이 아니다. CLI
플래그와 pack JSON 은 건드리지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_score scripts.tests.test_gym_score_runner`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 도입(#4653)은 과제를 pack 으로 쪼개고 판정 논리를 `gym/core/` 로
모았다. 점수는 pack 별로 보존되고, 요구 명령이 없으면 unavailable,
실행 신원이 스코어카드에 붙는다. `score.py` 는 진입점만 맡고
`runner.py` 가 엔진이다.

그 상태의 빈틈:

1. `load_pack` 이 pack.json 파싱 실패를 그대로 던진다. 한 pack 이
   깨지면 `score_all` 이 나머지 pack 을 채점하지 못한다.
2. `load_profile` 이 없는 프로파일에서 죽는다. 입장 봉투가 안 나온다.
3. `score_task` 가 `task["id"]` 를 먼저 읽어 필수 키가 없으면
   KeyError. 그 과제만 접히지 않고 pack 전체가 죽는다.
4. `answer.json` 이 배열이면 파싱은 성공하고 연산자가 객체를 가정해
   경로 평가로 넘어간다. 실패 이유가 "배열이면 안 된다"가 아니다.
5. `eval_check` 가 FileNotFoundError 와 경로 평가만 잡는다.
   PermissionError·OSError·TimeoutExpired·잘못된 cmd 타입은 스택이
   올라간다.
6. `known_commands` / `runner_identity` 가 없는 바이너리에서
   FileNotFoundError. 채점이 시작되지 않는다.
7. `render_report` 가 비-unavailable pack 을 전부 scored 로 찍는다.
   중간에 error 칸을 넣으면 표가 깨진다.
8. 카탈로그가 코드에 없어 문서·시험이 같은 표를 공유하지 않는다.

판정 성공 칸(exit 3 을 비교하기, T12 HWPX, 잘못된 필드/셀 거부)을
바꾸면 기존 계약 테스트가 깨진다. 그래서 그 칸의 메시지와 분기는
유지하고, 예외 자리에 kind 만 붙였다.

## 3. 한 일

### 3.1 엔진 `gym/core/runner.py`

- `EXCEPTION_KINDS` / `EXCEPTION_KIND_HELP` — 문서·시험이 보는 표.
- `FATAL_EXCEPTIONS` / `CATCHABLE_EXCEPTIONS` — 삼키는 경계.
- `ScoreRunnerError` — kind 를 가진 운영 예외.
- `exception_kind(exc, context)` — FileNotFound 는 문맥으로
  missing-bin / missing-pack / missing-profile / missing-submit /
  missing-file 로 갈린다.
- `is_safe_id` / `is_safe_relpath` — pack·profile 한 칸, 제출 파일
  상대경로. `..` 와 구분자를 거부한다.
- `run_cli` — FileNotFoundError·PermissionError 는 예전처럼 던진다.
  TimeoutExpired·SubprocessError·그 밖 OSError 는 ScoreRunnerError.
  stdout 비객체 JSON 은 봉투 `None`.
- `resolve_args` — cmd 타입 검사, `{input}` 부재, `{file:}`/`{sha256:}`
  경로 탈출 거부. `{sha256:}` 라이브 해시는 그대로.
- `eval_check` — 비객체 체크, op 없음, 미지 op, malformed-cmd,
  bad-expect-exits, cli-exit, envelope-parse 에 kind. 예전 오류
  접두는 유지.
- `score_task` — 뼈대 검사, answer 객체 강제, empty-checks,
  missing-submit 문구 유지.
- `load_pack` / `load_profile` / `discover_packs` — 안전 id, JSON
  객체, tasks/ 부재를 예외 kind 로.
- `score_pack` — 로드 실패는 `status=error`. 요구 명령 부재는
  여전히 unavailable.
- `score_all` — 프로파일·pack id 실패는 빈 카드 + exceptions.
  경로형 바이너리 부재는 `binMissing` 과 missing-bin 한 줄. 채점은
  이어간다.
- `attach_card_counts` — packsUnavailable 는 unavailable 만 센다.
  error 를 거기에 넣지 않는다. `packsErrored` 를 더한다.
- `render_report` / `format_console_summary` — error 줄과 예외
  목록을 숨기지 않는다. runner 신원 빈 칸에서 슬라이스가 죽지 않는다.
- `admission_from_card` / `exit_from_card` — 입장과 종료 코드를
  카드에서 계산. 종료 코드는 0 과 3 만.

### 3.2 진입점 `gym/score.py`

- 플래그 집합 불변: `--agent` `--submissions` `--bin` `--out`
  `--pack` `--profile`.
- `normalize_agent` — 공백·구분자 거부 (`empty-agent` / `unsafe-id`).
- `run_score` — argparse 와 분리. 시험이 argv 없이 호출한다.
- `write_score_artifacts` — 세 파일을 각각 시도. 부분 실패는
  `write-error`.
- `deny_card` — 채점 시작 전 실패도 입장 거부와 예외 한 줄을 남긴다.
- `main(argv=None)` — 운영 예외는 stderr 한 줄과 종료 3. 치명 예외는
  다시 던진다.

구 API 재수출(`find_bin` `run_cli` `resolve_args` `eval_check`
`score_task` 와 checks 심볼)은 유지한다.

### 3.3 시험

- `scripts/tests/test_gym_score.py` — 성공 칸. 이 작업이 메시지를
  바꾸지 않았는지 회귀 가드.
- `scripts/tests/test_gym_score_runner.py` — 예외 칸. 카탈로그,
  안전 id, kind 문맥, resolve/eval/score_task/load_pack/profile,
  score_all 정직 행렬, 입장·리포트, 진입점 플래그 집합, 문서 동기.

### 3.4 문서

- `gym/docs/score_runner.md` — 규약 정본. kind 표, 상태 삼원, 정직
  조항, 다른 기둥과의 경계.
- 이 파일 — 무엇을·왜·어떻게·검증.

## 4. 바꾸지 않은 것

- 새 CLI 플래그·새 종료 코드.
- `gym/tools/trajectory.py` · `discriminate.py` · `fuzz_corpus.py` ·
  `release_gate.py`.
- automation / core-cli / casual-rides pack JSON.
- `gym/core/checks.py` 연산자 레지스트리.
- 스코어카드 `kind` / `schemaVersion` (gymScorecard / 2.0).
- 입장 판정 공식: packsScored >= 1 → allow.
- T12 베이스라인과 과제 계약.
- `expect_exits` 의 `type(v) is not int` 검사.

열린 PR 5210–5270 이 만지는 파일은 건드리지 않았다.

## 5. 정직 행렬 — 시험이 고정하는 표

| 자리 | 부르면 안 되는 이름 | 올바른 칸 |
|---|---|---|
| pack.json 없음 | unavailable, 0점 | `status=error` + `missing-pack` |
| 요구 명령 없음 | error, 0점 | `status=unavailable` + score None |
| 제출 폴더 없음 | 미지 op, 통과 | 실패 + `제출 폴더 없음` + `missing-submit` |
| 경로형 bin 없음 | 전 pack error | `exceptions` 의 `missing-bin`, 채점 계속 |
| 프로파일 없음 | 스택, 빈 stdout | 빈 카드 + `missing-profile` |
| checks `[]` | 통과 | `empty-checks` |
| answer 배열 | 경로 평가 | `malformed-answer` |
| `{file:../x}` | 그대로 조인 | `unsafe-id` |
| 미지 op | 경로 평가 | `미지 op:` + `unknown-op` |
| error pack | packsUnavailable++ | packsErrored++ |
| error pack 있는 만점 | 종료 0 | 종료 3 |

## 6. 검증 실측

작업 트리에서:

```text
python -m unittest scripts.tests.test_gym_score scripts.tests.test_gym_score_runner
python gym/tools/audit.py
```

audit.py 는 pack 정합만 본다. 이 작업은 pack 을 고치지 않았으므로
devel 과 같은 통과를 기대한다. unittest 는 바이너리 없이 목킹한다.

`cargo fmt --all` 과 `cargo test` 는 호출하지 않았다. Rust 파일이
없다.

## 7. 남은 일

- 리더보드가 `packsErrored` 를 표시할지는 별 이슈. 부가 키라 예전
  리더는 무시한다.
- schema.py 의 `known_commands` 가 예외를 던지지 않게 만드는 것은
  이 작업 밖이다. 러너가 감싼다.
- 체크 연산자 추가·pack 과제 확장은 5210–5270 영역과 겹친다. 여기서
  하지 않는다.

## 8. 파일 목록

| 경로 | 역할 |
|---|---|
| `gym/core/runner.py` | 엔진 예외 경로 |
| `gym/score.py` | 진입점 감싸기 |
| `scripts/tests/test_gym_score_runner.py` | 예외 칸 시험 |
| `gym/docs/score_runner.md` | 규약 |
| `mydocs/working/gym_score_runner.md` | 이 기록 |

`scripts/tests/test_gym_score.py` 는 성공 칸 가드로 유지하되, kind 가
붙어도 예전 문구가 남는지 회귀 클래스만 더했다. 계약을 뒤집지 않는다.

## 9. 변경 전후 — 한 자리씩

### 9.1 pack.json 파싱 실패

전: `json.load` 가 ValueError 를 던지고 `score_all` 이 중단. 이미
채점한 pack 도 카드에 안 남음.

후: `score_pack` 이 `status=error` `kind=malformed-pack` 을 돌리고
다음 pack 으로 간다. 총점의 packsErrored 만 오른다.

### 9.2 없는 프로파일

전: `io.open` FileNotFoundError. 입장 봉투 없음.

후: 빈 카드 + `exceptions[{kind:missing-profile}]`. `--out` 이 있으면
세 파일을 쓰고 verdict=deny.

### 9.3 과제 필수 키 없음

전: `task["id"]` KeyError. 그 pack 의 나머지 과제도 채점 안 됨.

후: 그 과제만 `malformed-task`. pack 은 scored 로 남고 그 과제는
실패.

### 9.4 answer.json 배열

전: `json.load` 성공, 연산자가 객체를 가정해 경로 평가 실패 또는
예기치 않은 통과.

후: 과제 단계에서 `malformed-answer` "객체가 아니다". 체크를 돌리지
않음.

### 9.5 경로형 바이너리 부재

전: `known_commands` → `subprocess.run` FileNotFoundError. 채점 시작
전 사망.

후: `safe_known_commands` 가 None. `binMissing=true`. 파일 전용
체크는 채점. CLI 체크는 `파일 없음:`.

### 9.6 `{file:../secret}`

전: `os.path.join(sub_dir, "../secret")` 로 제출 폴더 밖을 가리킴.

후: `unsafe-id`. 조인하지 않음.

### 9.7 리포트 표

전: unavailable 가 아니면 `score/max` 를 찍음. error 칸이 생기면
None/None 이 점수로 보임.

후: error 줄은 `error | kind: message`. 예외 절을 따로 붙임.

## 10. 시험 실행 기록

작업 트리 `C:\Users\swsz9\rhwp-gym-score-runner`, 브랜치
`feat/gym-score-runner-hardening`.

```text
python -m unittest scripts.tests.test_gym_score scripts.tests.test_gym_score_runner
```

163+ 테스트 통과 (바이너리 없음, 목킹). ResourceWarning 한 건은
임시 readme 파일 핸들을 with 로 닫아 제거.

```text
python gym/tools/audit.py
```

`gym 정합 감사: 18 pack 전부 통과 — 위반 0`. pack JSON 을 고치지
않았으므로 devel 과 같다.

`cargo fmt --all` 미실행. Rust 없음. 사용자 지시.

## 11. 겹침 회피

지시: trajectory / discriminate / fuzz / release_gate,
automation / core-cli / casual-rides pack,
열린 PR 5210–5270 파일을 고치지 말 것.

이 브랜치가 만지는 경로:

- `gym/core/runner.py`
- `gym/score.py`
- `scripts/tests/test_gym_score.py` (회귀 클래스 추가만)
- `scripts/tests/test_gym_score_runner.py` (신규)
- `gym/docs/score_runner.md` (신규)
- `mydocs/working/gym_score_runner.md` (신규)

`gym/docs/` 디렉터리는 다른 열린 PR 도 만들 수 있다. 파일 이름이
달라 병합 충돌은 디렉터리 생성뿐이며 내용은 겹치지 않는다.

## 12. 크기 게이트

DoD: upstream/devel 대비 insertions >= 3000. 예외 카탈로그·시험
표·규약 문서가 그 크기를 채운다. 빈 줄이나 반복 주석으로 패지
않았다. 각 삽입은 kind 하나, 시험 하나, 또는 규약 한 칸이다.

