---
kind: guide
status: active
canonical: gym/docs/pack_health.md
last_verified: 2026-09-02
---

# gym pack 건강 감사

정본 구현은 `gym/tools/pack_health.py` 다. 이 문서는 그 도구가 **무엇을 보고**,
**무엇을 보지 않으며**, **어떤 코드로 보고하는가**를 사람이 고를 수 있게
풀어 쓴 목록이다. 작업 기록·오탐 결정·시험 지도는
[`mydocs/working/gym_pack_health.md`](../../mydocs/working/gym_pack_health.md)
에 남긴다.

`audit.py` 가 스키마·기준풀이 짝·전역 과제 ID 를 보면, 이 도구는 그 **다음 층**
이다. 지시문이 비거나 한 줄이고, 같은 과제에서 `check.name` 이 겹치고, 힌트가
답을 그대로 적고, `submit.kind` 가 모르는 값이어도 `audit.py` 는 통과한다.
운동장 품질이 조용히 내려간다. pack 건강은 그 구멍을 같은 JSON 봉투로 닫는다.

바이너리·네트워크를 부르지 않는다. `packs/` 아래 JSON 만 읽는다. 실제 pack에서
문제가 나오면 검사기 오검출과 과제 결함을 먼저 분리한다. 규칙을 넓혀 실패를
만들지도 않고, 전역 exclude로 실제 결함을 숨기지도 않는다.

## 한 줄 결론

```bash
python gym/tools/pack_health.py            # 리포트, 이슈가 있어도 0
python gym/tools/pack_health.py --json     # 같은 리포트 JSON, 기본 0
python gym/tools/pack_health.py --strict   # 이슈가 있으면 1
python gym/tools/pack_health.py --pack id  # 한 pack 만
python gym/tools/pack_health.py --codes    # 이슈 코드 목록
```

기본은 **관측**이다. `--strict` 만 품질 관문이다. `packs/` 자체를 못 읽으면
`--strict` 없이도 종료 1 이다.

## audit.py 와 나눈 일

| 층 | 도구 | 보는 것 | 보지 않는 것 |
|---|---|---|---|
| 정합 | `audit.py` | kind/schema, 과제↔기준 짝, 고아 reference, 전역 ID 충돌, 등록 연산자 | 지시 길이, 힌트 유출, 이름 중복, 경로 위생 |
| 건강 | `pack_health.py` | 지시·힌트·이름·제출 형식·경로 위생·연산자 필수 필드·매니페스트 신원 | 바이너리 존재, 채점 실행, 기준풀이 왕복 성공 |

두 도구는 같은 파일을 읽어도 **판정 어휘가 다르다**. audit 이슈는 문자열
한 줄이고, pack 건강 이슈는 `{code, severity, task, where, message}` 다.
코드 표는 `--codes` 와 이 문서가 공유한다.

같은 계약을 두 번 쓰는 곳이 있다. pack.json 신원, 고아 reference, 미등록
연산자는 audit 도 본다. 건강 도구가 다시 보는 이유는 CI 에서 audit 을 돌리기
전에 **같은 봉투** 로 관측할 수 있게 하려는 것이다. 실패 메시지를 복사하지
않고 코드를 붙인다.

## 봉투

```json
{
  "kind": "gymPackHealth",
  "schemaVersion": "1.0",
  "ok": true,
  "packCount": 18,
  "taskCount": 112,
  "issueCount": 0,
  "errorCount": 0,
  "warningCount": 0,
  "codes": {},
  "packs": [
    {
      "id": "core-cli",
      "taskCount": 14,
      "issueCount": 0,
      "issues": []
    }
  ]
}
```

`ok` 는 `scanError` 가 없고 `issueCount == 0` 일 때만 참이다. 경고도 이슈로
센다. 경고만 빼고 보려면 `--exclude empty_hint` 처럼 코드를 지정한다.

이슈 한 줄:

| 키 | 뜻 |
|---|---|
| `code` | 아래 표의 기계 이름 |
| `severity` | `error` 또는 `warning` |
| `task` | 과제 id. pack 단위 이슈면 `null` |
| `where` | `tasks/A01.json#checks[0]` 같은 위치 |
| `message` | 사람이 읽는 한 줄 (한국어) |

## 종료 코드

| 상황 | 종료 |
|---|---|
| 리포트 성공 (이슈 있어도 기본) | 0 |
| `--strict` 이고 이슈 있음 | 1 |
| `packs/` 없음 (`scanError`) | 1 |
| `--min-instructions` / `--min-title` < 1 | 2 |
| `--codes` 목록만 | 0 |

자기시험(`unittest`)은 픽스처에 일부러 이슈를 심고 기본 종료 0 을 확인한다.
게이트는 `--strict` 다. 두 경로를 섞지 마라.

## 검사 층

### 1. 지시문

- 키 없음 / 빈 문자열 / 문자열이 아님
- 앞뒤 공백을 뺀 글자가 기본 20 미만 (`--min-instructions`)
- 힌트 마커 앞에 본문이 없음 (`힌트:` 로만 시작)
- 마커는 있는데 꼬리가 빔 (warning)
- 문장 끝 힌트 꼬리가 두 번 이상 (warning)
- `TODO` / `FIXME` / `XXX` / `TBD` / `lorem ipsum` / `여기를 채우`
- NUL 같은 제어 문자 (개행·탭은 허용)

본문 안 괄호 힌트 `(힌트: export-text)` 는 꼬리로 치지 않는다. T06 처럼
"스스로 찾아(힌트: 명령) … 힌트: 실제 CLI" 는 한 번의 꼬리다.

### 2. 과제 신원

- `id` 공란, 앞뒤·내부 공백, 파일명(`A01.json`) 과 불일치
- `title` 공란, 앞뒤 공백, `--min-title`(기본 2) 미만
- 같은 pack 안에서 `id` 중복
- `tier` 없음, 정수 아님, `bool` 위장, 1~5 밖
- `input` 없음, 빈 값, 배열, 앞뒤 공백, 절대 경로, 백슬래시, `..` 탈출

`title` 안의 중간 공백(`쪽수 세기`)은 허용한다. `id` 의 중간 공백은 허용하지
않는다. 리더보드가 ID 로 과제를 가르기 때문이다.

### 3. 검사(check)

- `checks` 없음·빈 배열·배열 아님·항목이 객체 아님
- `name` 없음·빈 값·앞뒤 공백·같은 과제 안 중복
- `op` 없음·빈 값·`REGISTRY` 에 없는 이름
- `answer_eq` / `len_answer_eq` 에 `answer` 없음
- `value_eq` 계열·`cell_text_eq`·`csv_cell_eq`·`xml_root_eq` 에 `value` 없음
- 파일 연산자에 `file` 없음, 해시 비교에 `files` 2개 미만
- `file`/`files` 가 절대 경로이거나 백슬래시
- CLI 연산자에 `cmd` 없음·빈 배열·문자열 항목 아님
- 파일 연산자에 `cmd` 가 있음 (스키마와 같은 계약)
- `cell_text_eq` 에 `table`/`row`/`col` 이 0 이상 정수가 아님 (`True` 거절)
- `csv_cell_eq` 에 `row`/`col` 없음
- 편집·보안 축에서 `deep_contains`/`not_contains` 를 `allowGlobalScan` 없이 사용

같은 `name` 이 **다른 과제** 에 있는 것은 허용한다. "쪽수 일치" 는 pack 안
관용 이름이다.

등록부 목록은 `gym/core/checks.py` 의 `REGISTRY` 를 우선하고, 그 모듈을 못
읽으면 도구 안의 내장 목록으로 후퇴한다. 후퇴 목록은 현재 트리와 같다.

### 4. 제출

- `submit` 없음, 객체 아님, `kind` 없음
- `kind` 가 `answer` / `artifact` / `pair` 가 아님
- `artifact` 인데 `files` 가 빔 (warning)
- `pair` 인데 `files` 가 2개 미만 (warning)
- `files` 가 배열이 아님, 항목이 빈 문자열·비문자열, 앞뒤 공백
- 절대 경로, 백슬래시, 같은 이름 중복

`answer` 에 `files` 가 없는 것은 허용한다. 채점기가 `answer.json` 을 기본으로
열기 때문이다. `files` 를 썼으면 위생은 본다.

### 5. 힌트 유출

`split_hint` 가 본문과 꼬리를 가른다. 꼬리가 있을 때만 본다.

- `답은 4`, `정답은 …`, `answer is …` 같은 직접 스포일러
- 구체 스칼라만 가진 작은 JSON (`{"pages": 4}`)
- 검사 `value`/`expected` 가 본문에 없는데 꼬리에 단독으로 등장

오탐으로 치지 않는 것:

- 자리표 `{ "<필드이름>": "홍길동" }`, `{"pages": "<수>"}`
- 본문이 이미 시킨 값을 CLI 예시에 반복 (`첫 칸을 '계획실행'으로` + 힌트 명령)
- 명령 접미의 형식 토큰 (`export-hwpx`, `conv.hwpx` 안의 `hwpx`)
- `0` / `1` / `-1` 과 `ok`/`true`/`json`/`hwp` 같은 흔한 짧은 토큰

### 6. 기준풀이

짝 파일이 **있을 때만** 본다. 없는 것은 audit 몫이다.

- `steps` 키 없음 / `null` / 배열 아님 / 빈 배열 / 빈 객체만
- `reference.id` 가 과제 `id` 와 다름
- `steps` 항목이 객체가 아님
- `run` 이 있는데 비었음
- `answer` 가 있는데 비었음
- `answer.*.cmd` 가 있는데 비었음
- 짝 과제가 없는 `reference/*.json` (고아)

### 7. pack.json 매니페스트

- 객체가 아님, `kind != gymPack`, `schemaVersion != 1.0`
- `id` 가 폴더 이름과 다름
- `title` / `axis` 공란 또는 앞뒤 공백
- `requires.commands` 없음·빔·비문자열
- `runner` 없음, `rhwpVersion` / `rhwpCommit` / `capabilitiesSha256` 공란
- `tasks/` 에 JSON 이 없음 (warning)

## 이슈 코드

기계 이름은 `--codes` 가 찍는 표와 같다. 새 코드를 넣으면 카탈로그 튜플과
단위시험 `test_catalog_covers_all_code_constants` 가 같이 늘어나야 한다.

### 지시

| code | severity | 뜻 |
|---|---|---|
| `empty_instructions` | error | 키가 없거나 비었다 |
| `short_instructions` | error | 최소 글자 미만 |
| `instructions_type` | error | 문자열 아님 |
| `instructions_hint_only` | error | 마커 앞에 본문 없음 |
| `empty_hint` | warning | 마커만 있고 꼬리 없음 |
| `duplicate_hint_marker` | warning | 문장 끝 꼬리가 둘 이상 |
| `instructions_todo` | error | TODO/FIXME 자리표 |
| `instructions_control_char` | error | 제어 문자 |

### 검사

| code | severity | 뜻 |
|---|---|---|
| `missing_check_name` | error | name 없음 |
| `empty_check_name` | error | name 빔 |
| `duplicate_check_name` | error | 같은 과제에서 이름 중복 |
| `check_name_whitespace` | error | name 앞뒤 공백 |
| `missing_check_op` | error | op 없음 |
| `empty_check_op` | error | op 빔 |
| `unknown_check_op` | error | 미등록 연산자 |
| `check_missing_answer` | error | answer_eq 계열에 answer 없음 |
| `check_missing_value` | error | 값 비교에 value 없음 |
| `check_missing_file` | error | 파일 연산자에 file 없음 |
| `check_missing_files` | error | 해시 비교에 files 없음 |
| `check_files_short` | error | files 가 2개 미만 |
| `check_file_empty` | error | file 항목 빔 |
| `check_file_absolute` | error | 절대 경로 |
| `check_file_backslash` | error | 백슬래시 경로 |
| `check_missing_cmd` | error | CLI 연산자에 cmd 없음 |
| `check_cmd_type` | error | cmd 가 배열 아님 |
| `check_cmd_empty` | error | cmd 빔 |
| `check_cmd_item_type` | error | cmd 항목이 문자열 아님 |
| `check_unexpected_cmd` | error | 파일 연산자에 cmd |
| `cell_missing_coord` | error | cell_text_eq 좌표 없음 |
| `csv_missing_coord` | error | csv_cell_eq 좌표 없음 |
| `global_scan_undeclared` | error | 편집 축 전역 훑기 |
| `checks_type` | error | checks 가 배열 아님 |
| `empty_checks` | error | checks 빔 |
| `check_type` | error | 항목이 객체 아님 |

### 신원·입력

| code | severity | 뜻 |
|---|---|---|
| `task_id_whitespace` | error | id 공백 |
| `task_title_whitespace` | error | title 앞뒤 공백 |
| `empty_task_id` | error | id 빔 |
| `empty_title` | error | title 빔 |
| `title_too_short` | error | title 최소 글자 미만 |
| `id_filename_mismatch` | error | 파일명 ≠ id |
| `duplicate_task_id` | error | pack 안 id 중복 |
| `missing_tier` | error | tier 없음 |
| `tier_type` | error | tier 가 정수 아님 |
| `tier_range` | error | tier 가 1~5 밖 |
| `missing_input` | error | input 없음 |
| `empty_input` | error | input 빔 |
| `input_type` | error | input 이 문자열 아님 |
| `input_whitespace` | error | input 앞뒤 공백 |
| `input_absolute` | error | 절대 경로 |
| `input_backslash` | error | 백슬래시 |
| `input_parent_traversal` | error | `..` 탈출 |

### 제출·힌트·기준풀이·매니페스트

| code | severity | 뜻 |
|---|---|---|
| `unknown_submit_kind` | error | kind 미지 |
| `missing_submit` | error | submit 없음 |
| `missing_submit_kind` | error | kind 없음 |
| `submit_type` | error | submit 이 객체 아님 |
| `artifact_without_files` | warning | artifact 에 files 없음 |
| `pair_without_files` | warning | pair 에 files 2개 미만 |
| `submit_files_type` | error | files 가 배열 아님 |
| `submit_file_empty` | error | 항목 빔 |
| `submit_file_type` | error | 항목이 문자열 아님 |
| `submit_file_whitespace` | error | 항목 앞뒤 공백 |
| `submit_file_absolute` | error | 절대 경로 |
| `submit_file_backslash` | error | 백슬래시 |
| `submit_file_duplicate` | error | 같은 이름 중복 |
| `hint_answer_dump` | error | 정답 JSON |
| `hint_spoiler` | error | "답은 N" |
| `hint_embeds_check_value` | error | 기대값 복붙 |
| `empty_reference_steps` | error | steps 빔 |
| `reference_steps_type` | error | steps 타입 |
| `reference_id_mismatch` | error | reference.id ≠ 과제 id |
| `reference_step_type` | error | step 이 객체 아님 |
| `reference_run_empty` | error | run 빔 |
| `reference_answer_empty` | error | answer 빔 |
| `reference_cmd_empty` | error | answer.cmd 빔 |
| `orphan_reference` | error | 짝 과제 없음 |
| `parse_error` | error | JSON 파싱 실패 |
| `missing_pack_json` | error | pack.json 없음 |
| `missing_tasks_dir` | error | tasks/ 없음 |
| `empty_pack` | warning | 과제 0건 |
| `pack_type` | error | pack.json 이 객체 아님 |
| `pack_kind` | error | kind ≠ gymPack |
| `pack_schema_version` | error | schemaVersion ≠ 1.0 |
| `pack_id_mismatch` | error | id ≠ 폴더 |
| `pack_empty_title` | error | title 빔 |
| `pack_empty_axis` | error | axis 빔 |
| `pack_title_whitespace` | error | title 앞뒤 공백 |
| `pack_axis_whitespace` | error | axis 앞뒤 공백 |
| `pack_missing_requires` | error | requires.commands 없음 |
| `pack_empty_commands` | error | commands 빔 |
| `pack_command_type` | error | commands 항목 타입 |
| `pack_missing_runner` | error | runner 없음 |
| `pack_missing_runner_field` | error | runner 필드 빔 |

## CLI 플래그

| 플래그 | 뜻 |
|---|---|
| `--json` | 봉투를 JSON 으로 |
| `--strict` | 이슈가 있으면 종료 1 |
| `--pack ID` | 한 pack. 여러 번 지정 가능 |
| `--root DIR` | `packs/` 를 담은 디렉터리 |
| `--min-instructions N` | 지시 최소 글자 (기본 20) |
| `--min-title N` | title 최소 글자 (기본 2) |
| `--codes` | 코드 표만 출력 |
| `--exclude CODE` | 집계에서 뺄 코드. 여러 번 가능 |

`--pack` 에 없는 id 를 주면 그 id 로 `missing_pack_json` 한 줄을 만든다.
없는 pack 을 조용히 건너뛰지 않는다.

## 오탐 정책

규칙을 넓힐 때는 **현재 `gym/packs` 전건이 통과하는지** 먼저 본다.
실패하는 규칙이 진짜 품질 구멍이면 pack 을 고친다. 본문 관용 표현이면 규칙을
좁힌다. 기존 과제를 실패로 뒤집으려고 문구를 다시 쓰지 않는다.

이미 좁힌 예:

- `(힌트: export-text)` 는 꼬리 마커가 아니다. T06 본문 안내를 중복으로
  세지 않기 위해서다.
- `export-hwpx` 안의 `hwpx` 는 기대값 복붙이 아니다.
- 자리표 키 `<필드이름>` 을 가진 JSON 은 정답 봉투가 아니다.
- `fields[0]` 같은 경로 인덱스는 JSON 배열 정답이 아니다.
- `--data '{...}'` 는 편집 명령의 입력이지 채점 결과 answer가 아니다.
- 본문에서 이미 요구한 값을 힌트 명령에 반복하는 것은 유출이 아니다.
- 판정 명령에 명시적으로 넣은 기대 인수를 같은 check가 에코하는 것은 출력
  정답을 숨겨 제공한 경우와 구별한다.
- “정답 숫자를 박제하지 마라” 같은 금지 문장은 “답은 N”과 구별한다.
- `tier=True` 는 정수 1 로 보지 않는다. `bool` 은 `int` 하위형이다.

2026-09-02 전수 재검토에서는 196개 진단을 192개 오검출과 4개 실제 지시 경계
결함으로 분류했다. `AU14`, `T50`, `TE46`, `TE54`는 채점에 필요한 값을 힌트가
아니라 본문에 명시하도록 고쳤다. check/reference는 바꾸지 않았다. 정당한 예외가
없어 예외 원장과 `--exclude` 항목은 0개다.

## 새 규칙을 넣는 법

1. `CODE_*` 상수를 추가하고 `ISSUE_CATALOG` 한 줄을 같은 이름으로 넣는다.
2. 스캔 함수는 픽스처만 보고, 실제 pack 경로를 하드코딩하지 않는다.
3. `scripts/tests/test_gym_pack_health.py` 에 통과·실패 픽스처를 한 쌍 넣는다.
4. `RealRepoHealthGateTests.test_current_tree_stays_clean` 이 여전히 이슈 0
   인지 확인한다. 실패하면 오탐인지 실제 구멍인지 먼저 가른다.
5. 이 표와 `--codes` 출력이 같은지 눈으로 본다.

`python -m unittest scripts/tests/test_gym_pack_health.py` 와
`python gym/tools/audit.py` 를 같이 돌린다. 이 도구는 Python 만 추가하므로
`cargo fmt --all` 은 필요 없다.

## 관련 문서

- [`gym/README.md`](../README.md) — 제출 형식 `answer` / `artifact` / `pair`
- [`gym/core/checks.py`](../core/checks.py) — 채점 연산자 `REGISTRY`
- [`gym/core/schema.py`](../core/schema.py) — pack/task 스키마
- [`gym/tools/audit.py`](../tools/audit.py) — 정합 감사
- [`mydocs/working/gym_pack_health.md`](../../mydocs/working/gym_pack_health.md) — 작업 기록
