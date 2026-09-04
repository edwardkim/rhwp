---
kind: guide
status: active
canonical: gym/docs/score_runner.md
last_verified: 2026-09-02
---

# gym 채점기(score/runner) 예외 경로 규약

이 문서는 `gym/score.py` 진입점과 `gym/core/runner.py` 엔진의 **예외
경로 계약**, **pack 상태 삼원**, **입장 봉투**, **자리표시자 안전**,
**보고 카드**를 고정한다. 작업 기록은
[`mydocs/working/gym_score_runner.md`](../../mydocs/working/archives/gym_score_runner.md)
를 본다. 시험 계약은 `scripts/tests/test_gym_score.py`(성공 칸)와
`scripts/tests/test_gym_score_runner.py`(예외 칸)가 기계로 고정한다.

판별력 감사(`discriminate.py`)·트라젝토리 감사(`trajectory.py`)·퍼즈
변형(`fuzz_corpus.py`)·릴리스 게이트(`release_gate.py`)는 이 문서의
대상이 아니다. 채점기가 남긴 칸을 그 도구들이 소비할 수는 있어도, 이
규약이 그 도구의 판정 삼원을 바꾸지 않는다.

새 플래그는 없다. `--agent` `--submissions` `--bin` `--out` `--pack`
`--profile` 만 쓴다. 종료 코드도 예전과 같다. 0=만점, 3=그 외.

## 1. 왜 이 기둥이 필요한가

운동장 채점기는 정답을 골든으로 박제하지 않고, 채점 시점에 rhwp 로
다시 계산한다(#4653). 그 설계는 "실패도 데이터"를 전제로 한다. 그런데
예전이 실패를 데이터로 남기지 못하는 자리가 있었다.

- pack.json 하나가 깨지면 `score_all` 전체가 죽는다. 나머지 pack 의
  점수가 증발한다.
- 프로파일이 없으면 스택이 올라가고 입장 봉투가 없다.
- 과제 JSON 필수 키가 빠지면 `task["id"]` 에서 KeyError. 그 pack 의
  다른 과제도 같이 사라진다.
- `answer.json` 이 배열이면 연산자가 봉투처럼 읽고 엉뚱한 통과/실패를
  낸다.
- 경로형 `--bin` 이 없는데 `known_commands` 가 FileNotFoundError 를
  삼키지 못해 채점이 시작되지 않는다.
- pack 로드 실패를 0점이나 unavailable(명령 부재)로 부르면 거짓이다.

2026 벤치마크의 다른 위기는 false-pass 이지만, 채점기 자체의 위기는
**false-silence** 다. 탐색을 못 한 자리를 "만점 아님" 한 줄로 접으면
어느 pack 이 도구 실패인지 안 보인다. 리더보드 입장 게이트
(`admission.json`)는 "채점이 유효하게 완주했는가"를 묻는다. 완주하지
못한 자리를 숨기면 입장 판정도 거짓말이다.

그래서 예외는 삼키지 않고 kind 로 남긴다. 점수는 그대로 pack 별로
보존한다. 오류 pack 은 `status=error` 이고 `score=None` 이다. 0점도
아니고 unavailable 도 아니다.

## 2. 사용

```bash
python gym/score.py --agent <이름>
python gym/score.py --agent <이름> --profile editor
python gym/score.py --agent <이름> --pack security
python gym/score.py --agent <이름> --bin target/debug/rhwp --out gym/out
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--agent` | 필수 | 제출자 이름. 공백뿐이거나 `/` `\` 를 품으면 거부. |
| `--submissions` | `gym/submissions/<agent>` | 제출 루트. |
| `--bin` | `RHWP_BIN` 또는 `target/debug/rhwp` | rhwp 바이너리. 상대경로는 절대화. |
| `--out` | 제출 루트와 같음 | `scorecard.json` · `report.md` · `admission.json`. |
| `--pack` | 전 pack 탐색 | 반복 지정 가능. |
| `--profile` | 없음 | pack 묶음. 점수를 뭉치지 않는다. |

새 플래그는 없다. `--json` `--limit` `--task` `--timeout` 을 붙이지
않는다. 기계 봉투는 `--out` 아래 JSON 파일이고, stdout 은 사람용
한 줄 요약이다.

종료 코드:

| 코드 | 상수 | 의미 |
|---|---|---|
| 0 | `EXIT_PERFECT` | 채점된 pack 만점이면서 error pack 0 |
| 3 | `EXIT_IMPERFECT` | 그 외(실패 과제, error, 빈 채점, 도구 실패) |
| 2 | argparse | 필수 인자 없음. 예전 argparse 계약 |

도구 자리 오류를 위한 새 종료 코드를 만들지 않는다. 만점이 아니면 3
이다. 입장 봉투의 `verdict` 는 종료 코드와 별개다 — 낮은 점수도
allow 일 수 있다.

## 3. pack 상태 삼원 — 바꾸지 않는 칸

`PACK_STATUSES = (scored, unavailable, error)`.

| status | 언제 | score | 총점 가산 | 입장 packsScored |
|---|---|---|---|---|
| `scored` | 로드 성공, 요구 명령 충족 | 통과 과제 티어 합 | 예 | 예 |
| `unavailable` | 요구 명령이 바이너리에 없음 | `None` | 아니오 | 아니오 |
| `error` | 로드·식별자·JSON·권한 실패 | `None` | 아니오 | 아니오 |

규칙:

1. **부재는 실패가 아니다.** 오래된 바이너리에게 0점은 거짓말이다.
   그 자리는 `unavailable` 이다.
2. **도구 실패는 부재가 아니다.** pack.json 이 깨진 것을 "명령이 없다"고
   부르면 다음 사람이 바이너리를 탓한다. 그 자리는 `error` 이다.
3. **error 를 unavailable 로 세지 않는다.** `total.packsUnavailable` 은
   `status==unavailable` 개수다. `len(packs) - packsScored` 로 되돌리면
   error 가 명령 부재로 위장된다.
4. **error 는 0점이 아니다.** `score=None`. 합계는 scored pack 만.

`total` 고정 키:

| 키 | 의미 |
|---|---|
| `score` | scored pack 점수 합 |
| `max` | scored pack 만점 합 |
| `packsScored` | status=scored 개수 |
| `packsUnavailable` | status=unavailable 개수 |
| `packsErrored` | status=error 개수 |
| `exceptionCount` | 카드 수준 `exceptions` 길이 |

부가 키 `exceptions` · `exceptionCount` · `binPath` · `binMissing` ·
`trusted` 는 집계를 뒤집지 않는다. `trusted` 는 예외 0 이고 error pack
0 일 때만 참이다.

## 4. 예외 kind 카탈로그

`EXCEPTION_KINDS` 와 `EXCEPTION_KIND_HELP` 가 정본이다. 시험이 문서에
각 kind 가 백틱으로 적혔는지 대조한다.

| kind | 자리 | 점수에 미치는 영향 |
|---|---|---|
| `missing-bin` | 경로형 바이너리 부재, CreateProcess 실패 | 카드 `exceptions`. 채점은 이어갈 수 있음 |
| `missing-submit` | 과제 제출 폴더 없음 | 과제 실패. 예전 문구 `제출 폴더 없음` |
| `missing-pack` | pack.json / pack 폴더 없음 | pack `status=error` |
| `missing-profile` | profiles/<id>.json 없음 | 빈 카드 + exceptions |
| `missing-file` | 제출 파일·해시 대상 없음 | 체크 실패 |
| `missing-tasks-dir` | pack 에 tasks/ 없음 | pack `status=error` |
| `missing-input` | `{input}` 인데 과제에 input 없음 | 체크 실패 |
| `malformed-json` | JSON 파싱 실패(문맥 없음) | 해당 자리 실패 |
| `malformed-answer` | answer.json 깨짐·비객체 | 과제 실패. 문구 `answer.json 파싱 실패:` |
| `malformed-task` | 과제 비객체·필수 키 없음 | 과제 또는 pack error |
| `malformed-check` | 체크 비객체·op 없음 | 체크 실패 |
| `malformed-pack` | pack.json 비객체·title/axis 없음 | pack `status=error` |
| `malformed-profile` | 프로파일 비객체·packs 빈 목록 | 빈 카드 |
| `malformed-cmd` | cmd 가 문자열 목록이 아님 | 체크 실패 |
| `unsafe-id` | pack/profile/파일 자리에 `..` / 구분자 | 해당 자리 거부 |
| `permission` | 읽기·실행 권한 없음 | 해당 자리 실패. 문구 `권한 없음:` |
| `os-error` | 그 밖의 OSError | 해당 자리 실패 |
| `decode-error` | UTF-8 디코드 실패 | 해당 자리 실패 |
| `value-error` | 값 형태 불일치 | 해당 자리 실패 |
| `type-error` | 타입 불일치 | 해당 자리 실패 |
| `timeout` | 자식 프로세스 대기 한도 | 체크 실패 |
| `subprocess` | 자식 기동·통신 실패 | 체크 실패 |
| `unknown-op` | 레지스트리에 없는 op | 체크 실패. 문구 `미지 op:` |
| `bad-expect-exits` | expect_exits 가 비지 않은 정수 목록 아님 | 체크 실패. 예전 문구 유지 |
| `cli-exit` | 종료 코드가 허용 집합 밖 | 체크 실패. 문구 `exit N (허용 …)` |
| `envelope-parse` | stdout 이 JSON 객체가 아님 | 체크 실패. 문구 `봉투 파싱 실패:` |
| `path-eval` | KeyError/IndexError/TypeError | 체크 실패. 문구 `경로 평가 실패:` |
| `empty-checks` | 과제 checks 가 `[]` | 과제 실패. 통과 칸이 없음 |
| `empty-agent` | `--agent` 가 공백 | 채점 시작 전 거부 |
| `write-error` | scorecard/report/admission 기록 실패 | artifacts.errors |
| `unexpected` | 분류되지 않은 운영 예외 | 해당 자리 실패 |

`FileNotFoundError` 의 kind 는 문맥에 따른다. 바이너리 자리면
`missing-bin`, pack 자리면 `missing-pack`, 제출 폴더면
`missing-submit`, 그 밖이면 `missing-file`. 한 예외 타입을 한 kind 로
고정하면 없는 바이너리를 없는 제출로 부른다.

## 5. 치명 예외 — 삼키지 않는 자리

`FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)`.

운영 예외(`CATCHABLE_EXCEPTIONS`)만 접는다. `except Exception` 으로
BaseException 을 삼키지 않는다. 시험이 `is_fatal_exception` 과
`is_catchable_exception` 경계를 고정한다.

`eval_check` 가 잡는 칸:

1. 체크 뼈대(비객체, op 없음, 미지 op) — 예외가 아니라 사전 거부.
2. CLI 자리의 cmd/expect_exits/exit/봉투 — 예외가 아니라 분기.
3. `FileNotFoundError` — 예전 접두 `파일 없음:` 유지.
4. `PermissionError` — `권한 없음:`.
5. `ScoreRunnerError` — `kind` 를 그대로 옮김.
6. `KeyError` `IndexError` `TypeError` — 예전 접두 `경로 평가 실패:`.
7. 그 밖의 catchable — `Type: message` 와 kind.

`bool` 은 `int` 의 하위 타입이다. `expect_exits: [True]` 는 허용하지
않는다. `type(v) is not int` 가 그 계약을 지킨다.

## 6. 자리표시자와 식별자 안전

pack id · profile id 는 한 칸 이름이다. 허용은 영숫자와 `-` `_`.
거부: 빈 문자열, `.`, `..`, `/`, `\`, `:`, NUL.

`{file:name}` · `{sha256:name}` 은 상대경로를 받는다. `out/o1.hwp` 는
허용. `../secret`, 절대경로, 빈 칸, `.`, `..` 칸은 `unsafe-id`.

`{input}` 은 `task["input"]` 문자열이다. 키가 없거나 비면
`missing-input`.

`find_bin` 계약은 그대로다. `--bin` > `RHWP_BIN` > `target/debug/rhwp`.
상대경로는 cwd 와 저장소 루트에서 절대화한다. 맨이름 `rhwp` 는 PATH
조회라 `binMissing` 으로 단정하지 않는다. 경로형인데 디스크에 없을
때만 `binMissing=true` 와 `missing-bin` 한 줄을 남기고 채점은 이어간다
— 파일 전용 연산자(`file_exists` 등)는 바이너리 없이 돈다.

## 7. 과제 뼈대와 answer.json

채점에 필요한 최소 키는 `id` `tier` `title` `checks` 다. 스키마의
`TASK_REQUIRED` 전체(instructions·input·submit)는 audit 가 보고,
러너는 채점에 필요한 뼈대만 본다. 뼈대가 없으면 과제는
`malformed-task` 로 실패하고 pack 의 다음 과제로 간다.

`tier` 는 `type(v) is int` 여야 한다. `"1"` 과 `True` 는 거부.

`answer.json` 이 있으면 객체여야 한다. 배열·문자열·깨진 JSON 은
`malformed-answer`. 파싱 실패 문구는 예전처럼
`answer.json 파싱 실패:` 로 시작한다. 파일이 없으면 빈 객체로 본다
— answer 과제가 아닌 산출물 과제가 있기 때문이다.

artifact 텍스트에 쓰는 `text_file_envelope_eq`는 제출 파일 전체와 CLI JSON
봉투의 `path` 문자열을 정확히 비교한다. BOM·CRLF를 포함한 UTF-8 바이트가
모두 같아야 하며, 기대 크기와 다르면 파일 본문을 읽지 않고 실패한다. 기대
문자열이 8 MiB를 넘어도 거부한다. 결과 상세는 본문을 재출력하지 않고
SHA-256과 바이트 수만 남긴다.

`checks` 가 `[]` 이면 `empty-checks`. `bool([]) and all(...)` 가
거짓이라 예전도 실패였지만, 이유가 없었다. 이제 kind 가 남는다.

제출 폴더가 없으면 예전 문구 그대로 `제출 폴더 없음`. kind 는
`missing-submit`. 이 문구를 바꾸면 기존 리포트·베이스라인 대조가
깨진다.

## 8. 스코어카드와 입장 봉투

카드 `kind=gymScorecard`, `schemaVersion=2.0`. 이 두 칸을 올리면
리더보드·게이트가 읽지 못한다.

입장 봉투 `kind=gymAdmission`, `schemaVersion=1.0`.

| 칸 | 규칙 |
|---|---|
| `verdict` | `packsScored >= 1` 이면 `allow`, 아니면 `deny` |
| `packsScored` | scored pack 수 |
| `packsUnavailable` | unavailable pack 수 |
| `packsErrored` | error pack 수 (부가, 예전 리더가 몰라도 됨) |
| `score` / `max` | 총점 편의값 |
| `runner` | 실행 신원. 바이너리 부재면 빈 문자열 칸 |

allow 는 만점이 아니다. 0/32 도 pack 을 하나라도 채점했으면 allow.
error 만 있고 scored 가 0 이면 deny. 채점이 시작 전에 죽으면
`deny_card` 가 예외 한 줄짜리 빈 카드를 남기고 가능하면 세 파일을
쓴다.

기록 파일:

| 파일 | 내용 |
|---|---|
| `scorecard.json` | 카드 원문. UTF-8, BOM 없음, LF, indent=2 |
| `report.md` | 사람용 표. error pack 과 exceptions 를 숨기지 않음 |
| `admission.json` | 입장 봉투 |

한 파일이 실패해도 나머지를 시도한다. 실패 줄은 `write-error`.

## 9. 성공 칸 — 이 작업이 바꾸지 않는 것

아래는 `test_gym_score.py` 가 지키는 칸이다. 예외 보강이 이 칸을
옮기면 회귀다.

1. `expect_exits: [0, 3]` 에서 exit 3 + 봉투 `identical: false` 는
   비교 대상이지 폐기 대상이 아니다.
2. 허용 집합 밖 exit 는 거부되고 오류 문자열에 허용 값이 남는다.
3. 단일 `expect_exit` 는 하위 호환.
4. T12 는 실제 HWPX 와 IR 판정 exit 를 요구한다.
5. T07 은 첫 필드, T08 은 (0,0) 셀, T10 은 원본 무편집 복사를 거부.
6. `{sha256:file}` 는 채점 시점 해시로 풀린다.
7. 체크 명령은 그 pack 의 `requires.commands` 에 선언돼 있어야 한다.

라이브 오라클 원칙도 그대로다. 기대값은 채점 시점에 rhwp 가 다시
계산한다. 이 문서가 골든 파일을 추가하지 않는다.

## 10. 다른 기둥과의 경계

| 도구 | 축 | 이 문서와의 관계 |
|---|---|---|
| `runner.py` / `score.py` | 종점 채점 | 이 문서의 대상 |
| `discriminate.py` | 약한 오라클(음성 대조) | 러너의 `score_task` 를 부르되 파일을 수정하지 않음 |
| `trajectory.py` | 경로(마지막 스텝) | 러너의 `score_task` 를 부르되 파일을 수정하지 않음 |
| `fuzz_corpus.py` | 결정적 변형 | 채점기를 직접 고치지 않음 |
| `release_gate.py` | 수동 Gym 구·신 차등 조사 | 채점 결과를 소비할 수 있으나 제품 릴리스 권한은 없음 |
| `audit.py` | pack 정합 | 바이너리 없이 파일만. 러너 예외와 별개 |

이 작업은 automation / core-cli / casual-rides pack JSON 을 고치지
않는다. 과제 확장은 다른 이슈의 일이다.

## 11. 정직 조항

1. 없는 바이너리를 전 과제 0점으로 부르지 않는다.
2. pack 로드 실패를 명령 부재로 부르지 않는다.
3. 제출 폴더 없음을 미지 op 로 부르지 않는다.
4. 연극·회귀·차등 판정을 채점기가 대신하지 않는다.
5. 새 CLI 를 발명해 예외를 "숨김 플래그" 뒤로 치우지 않는다.
6. 치명 예외를 운영 예외 목록에 넣지 않는다.
7. `trusted=true` 를 예외가 있는 카드에 붙이지 않는다.
8. 만점 종료 코드 0 을 error pack 이 있는 카드에 주지 않는다.

## 12. 검증

```bash
python -m unittest scripts.tests.test_gym_score scripts.tests.test_gym_score_runner
python gym/tools/audit.py
```

`cargo fmt --all` 은 이 변경의 검증이 아니다. Python 과 문서만
고친다.

## 13. 구현 지도

코드가 문서의 표를 구현하는 위치. 함수 이름을 바꾸면 이 절과 시험을
같이 고친다.

| 함수 | 역할 | 실패 시 kind |
|---|---|---|
| `find_bin` | --bin / RHWP_BIN / target 기본값 | 없음. 없는 경로도 문자열로 돌려줌 |
| `bin_is_missing` | 경로형만 디스크 부재 단정 | 카드 `missing-bin` |
| `prepare_cli` | argv 검증 | `missing-bin` `malformed-cmd` |
| `run_cli` | 자식 실행 | FileNotFound 그대로, 그 밖 ScoreRunnerError |
| `decode_cli_stdout` | 바이트 → 텍스트 | 교체 디코드. 예외 없음 |
| `parse_envelope` | stdout → 객체 | 실패는 None (`envelope-parse`) |
| `resolve_placeholder` | `{input}` `{file:}` `{sha256:}` | `missing-input` `unsafe-id` `missing-file` |
| `resolve_args` | cmd 목록 전개 | `malformed-cmd` + 위 |
| `validate_expect_exits` | 정수 목록 | `bad-expect-exits` |
| `eval_check` | 체크 하나 | 카탈로그 전 칸 |
| `task_shape_error` | 과제 뼈대 | `malformed-task` |
| `read_answer_json` | answer.json 객체 | `malformed-answer` `decode-error` `permission` |
| `score_task` | 과제 하나 | `missing-submit` `empty-checks` + 위 |
| `load_pack` | pack.json + tasks | `unsafe-id` `missing-pack` `malformed-pack` `missing-tasks-dir` `malformed-task` |
| `discover_packs` | pack.json 있는 폴더 | 안전하지 않은 이름 건너뜀 |
| `load_profile` | profiles/<id>.json | `unsafe-id` `missing-profile` `malformed-profile` |
| `score_pack` | pack 하나 | `error` 또는 `unavailable` 또는 `scored` |
| `score_all` | 카드 | 프로파일/목록 실패는 빈 카드 |
| `attach_card_counts` | total 재계산 | 없음 |
| `admission_from_card` | 입장 봉투 | 깨진 카드는 deny |
| `render_report` | 사람용 표 | 깨진 카드도 문자열 |
| `exit_from_card` | 0 또는 3 | 깨진 카드는 3 |
| `normalize_agent` | --agent | `empty-agent` `unsafe-id` |
| `write_score_artifacts` | 세 파일 | `write-error` |

진입점 `main` 은 `build_parser` → `run_score` → 종료 코드. `run_score` 를
시험이 직접 부른다. argparse 를 우회해도 같은 채점 경로다.

## 14. 봉투 표본

채점이 한 pack 을 로드하지 못했을 때 카드 핵심만. 신원 해시는 생략.

```json
{
  "kind": "gymScorecard",
  "schemaVersion": "2.0",
  "profile": null,
  "total": {
    "score": 0,
    "max": 0,
    "packsScored": 0,
    "packsUnavailable": 0,
    "packsErrored": 1,
    "exceptionCount": 0
  },
  "packs": [
    {
      "id": "broken",
      "status": "error",
      "score": null,
      "kind": "missing-pack",
      "error": "pack.json 이 없다: broken"
    }
  ],
  "trusted": false
}
```

이 카드의 입장 봉투는 `verdict=deny`, `packsErrored=1`, `packsScored=0`.
종료 코드는 3. unavailable 칸은 0 이다 — 없는 pack 을 명령 부재로
부르지 않는다.

경로형 바이너리가 없을 때 카드 머리:

```json
{
  "binPath": "C:/no/rhwp",
  "binMissing": true,
  "exceptions": [
    {"kind": "missing-bin", "where": "bin", "message": "경로형 바이너리가 없다: C:/no/rhwp"}
  ]
}
```

`exceptions` 가 있어도 pack 채점은 이어진다. 파일 전용 체크는
바이너리 없이 통과할 수 있다. CLI 체크는 `파일 없음:` +
`missing-bin` 으로 실패한다.

## 15. 시험이 주입하는 자리

바이너리 없이 예외 칸을 고정하는 방법. `test_gym_score_runner.py` 가
이 표를 구현한다.

| 시험 급 | 주입 | 기대 |
|---|---|---|
| 카탈로그 | `EXCEPTION_KINDS` 순회 | 문서 백틱 · HELP 비지 않음 |
| 안전 id | `../x`, `a/b`, 절대경로 | `is_safe_id` 거짓, require 가 `unsafe-id` |
| 자리표시자 | `{input}` 없는 과제 | `missing-input` |
| 자리표시자 | `{file:../secret}` | `unsafe-id` |
| 자리표시자 | `{sha256:없는파일}` | `missing-file`, 접두 `파일 없음:` |
| eval | 비객체 체크 | `malformed-check` |
| eval | 미지 op | 예전 문구 + `unknown-op` |
| eval | cmd 문자열 | `malformed-cmd` |
| eval | expect_exits `"0"` | `bad-expect-exits` |
| eval | exit 2 | `cli-exit` |
| eval | 봉투 None | `envelope-parse` |
| eval | FileNotFoundError from run_cli | 접두 `파일 없음:` + `missing-bin` |
| eval | PermissionError | 접두 `권한 없음:` + `permission` |
| score_task | 제출 폴더 없음 | 문구 `제출 폴더 없음` |
| score_task | answer `{` | `malformed-answer` |
| score_task | answer `[1]` | `malformed-answer` |
| score_task | checks `[]` | `empty-checks` |
| load_pack | 없는 id | `missing-pack` |
| load_pack | title 빈 문자열 | `malformed-pack` |
| load_pack | tasks/ 없음 | `missing-tasks-dir` |
| load_profile | 없는 id | `missing-profile` |
| load_profile | packs `[]` | `malformed-profile` |
| score_pack | available=empty | `unavailable`, score None |
| score_all | 없는 pack + 있는 pack | packsErrored=1, packsUnavailable=0 |
| 입장 | packsScored 0 / 1 | deny / allow |
| 진입점 | 플래그 dest 집합 | 6개, 추가 없음 |
| 진입점 | agent 공백 | 종료 3 |
| 문서 동기 | 규약 파일 | 모든 kind 백틱 |

성공 칸(T07/T08/T10/T12/expect_exits [0,3])은 `test_gym_score.py` 가
그대로 지킨다. 예외 보강이 그 파일을 깨면 이 작업이 성공 칸을 옮긴
것이다.

## 16. 경로 탈출 거부 표

Windows 와 POSIX 를 같이 막는다. 채점기가 제출 폴더 밖으로 나가
저장소 파일이나 홈 디렉터리를 해시·인자로 넘기면 안 된다.

| 입력 | pack/profile id | `{file:}` / `{sha256:}` |
|---|---|---|
| `core-cli` | 허용 | 허용 (파일 이름) |
| `out/o1.hwp` | 거부 (`/`) | 허용 |
| `../x` | 거부 | 거부 |
| `a/../b` | 거부 | 거부 |
| `/abs` | 거부 | 거부 |
| `C:foo` | 거부 (`:`) | 거부 (드라이브) |
| `.` `..` | 거부 | 거부 |
| 빈 문자열 | 거부 | 거부 |
| `a_b-1` | 허용 | 허용 |

agent 이름은 pack id 와 같은 구분자 규칙을 쓴다. 제출 폴더
`gym/submissions/<agent>` 가 경로 탈출하면 다른 에이전트 제출을
덮을 수 있다.
