---
kind: guide
status: active
canonical: gym/docs/schema.md
last_verified: 2026-08-18
---

# gym pack·task·profile 스키마 규약

이 문서는 `gym/core/schema.py` 의 **검증 계약**을 고정한다. 작업 기록은
[`mydocs/working/gym_schema.md`](../../mydocs/working/archives/gym_schema.md) 를 본다.
기계 시험은 `scripts/tests/test_gym_schema.py`(예외 칸)와
`scripts/tests/test_gym_packs.py`(저장소 나무 성공 칸)가 지킨다.

채점 엔진(`runner.py`)·연산자 등록부(`checks.py`)·감사기(`audit.py`)·점수
진입점(`score.py`)은 이 문서의 대상이 아니다. 스키마는 그 도구들이 소비하는
**선언의 모양**만 본다. 새 CLI 플래그는 없다. `REGISTRY` 키를 더하거나 빼지
않는다.

## 1. 왜 이 기둥이 필요한가

운동장은 pack 이 늘수록 "선언만 있고 돌지 않는 과제"의 위험이 커진다.
`validate_pack` / `validate_task` / `validate_profile` 이 그 선언을 막는다.
그런데 예전의 세 함수는 성공한 나무만 잘 보았다.

- 과제 JSON 이 목록이면 `task.get` 에서 AttributeError. 감사기 한 pack 이 죽는다.
- `checks` 가 객체면 키 문자열을 순회하다 `str.get` 으로 죽는다.
- `tier: true` 는 `bool` 이 `int` 의 하위형이라 입문 과제로 통과한다.
- `submit.kind` 가 `zip` 이어도 아무도 안 막는다.
- 프로파일 `packs` 가 문자열이면 글자 하나하나를 pack id 로 본다.
- 미등록 연산자는 거절했지만, 그 거절이 kind 로 남지 않아 시험이 메시지
  조각에만 매였다.

#5279 는 그 자리를 **죽이지 않고 칸으로 남긴다.** 점수는 바꾸지 않는다.
채점 성공 경로는 그대로다. 저장소에 이미 있는 pack 은 통과해야 한다.

부재를 실패로 위장하지 않는 결은 여기에도 같다. 스키마 위반은 0점이 아니라
**등재 거부**다. 바이너리가 없어서 명령을 못 읽는 자리(`known_commands is
None`)는 명령 존재 검사를 건너뛴다. 그 자리는 채점기가 unavailable 로 본다.

## 2. 검증 API

예전 시그니처를 지킨다. `audit.py` 와 `test_gym_packs` 가 이 세 함수를
직접 부른다.

```text
validate_pack(manifest, pack_dir, errors)
validate_task(task, pack, known_commands, errors)
validate_profile(profile, pack_ids, errors)
```

`errors` 는 문자열 목록이다. 한 줄의 모양은 예전과 같다.

```text
<where>: <message>
```

예:

```text
table-editing: requires.commands 가 비었다 — 요구 capability 선언은 필수
table-editing/TB01: 필수 키 없음: input
table-editing/TB01: tier 는 1~5 정수 (1=입문 … 5=보스)
table-editing/TB01: 미등록 연산자: ghost_op
profiles/starter: 없는 pack 참조: no-such-pack
```

`errors` 자리에 `IssueList` 를 넘기면 같은 문자열과 함께 `kind` 가 남는다.
평범한 `list` 를 넘기면 문자열만 쌓인다. 두 경로의 텍스트는 같아야 한다.

선택 API:

| 함수 | 역할 |
|---|---|
| `collect_pack` / `collect_task` / `collect_profile` | `IssueList` 를 만들어 돌려준다 |
| `validate_gym_tree(gym_root)` | `packs/`·`profiles/` 전수. audit.py 를 바꾸지 않는 읽기 경로 |
| `load_json_mapping(path)` | 깨진 JSON·배열 루트·부재를 칸으로 |
| `registered_ops()` | `checks.REGISTRY` 키의 읽기 전용 집합 |
| `is_valid_tier` / `is_safe_id` / `is_known_submit_kind` | 단위 판정 |
| `lint_check_fields(check)` | 연산자 권장 필드. 기본 검증은 강제하지 않는다 |

새 진입점 스크립트는 없다. `python gym/core/schema.py` 를 실행하지 않는다.

## 3. 상수 — 바꾸면 나무가 흔들린다

| 이름 | 값 | 의미 |
|---|---|---|
| `PACK_KIND` | `gymPack` | pack.json `kind` |
| `PROFILE_KIND` | `gymProfile` | profiles/*.json `kind` |
| `SCHEMA_VERSION` | `1.0` | 선언 버전. 2.0 으로 몰래 올리지 않는다 |
| `TIER_MIN` / `TIER_MAX` | 1 / 5 | 입문…보스 |
| `TIER_NAMES` | 1=입문 … 5=보스 | 놀이공원 키 제한 |
| `SUBMIT_KINDS` | answer, artifact, pair | README 의 세 칸 |
| `EDITING_AXES` | 편집, 보안 | 이 접두로 시작하면 전역 훑기 금지 |
| `TASK_REQUIRED` | id, tier, title, input, instructions, submit, checks | 과제 필수 키 |
| `PACK_REQUIRED` | schemaVersion, kind, id, title, axis, requires, runner | pack 필수 키 |
| `PROFILE_REQUIRED` | schemaVersion, kind, id, title, packs | 프로파일 필수 키 |
| `RUNNER_KEYS` | rhwpVersion, rhwpCommit, capabilitiesSha256 | 기준 실행 신원 |

`REGISTRY` 와 `GLOBAL_SCAN_OPS` 와 `needs_cli` 는 `checks.py` 가 소유한다.
스키마는 `from . import checks` 로 읽기만 한다. 키를 추가·삭제·이름 바꾸기
하지 않는다. 열린 PR 5210 이 그 등록부를 키우고 있다.

## 4. pack 검증

대상: `packs/<id>/pack.json`. `where` 는 폴더 이름이다.

통과 조건:

1. 루트가 객체다. 목록·문자열·`null` 이면 `not-a-mapping` 이고 나머지 검사는
   하지 않는다(죽어 본 자리를 한 칸으로 접는다).
2. `kind == gymPack`. 아니면 `bad-kind`. 메시지: `kind 가 gymPack 가 아니다`.
3. `schemaVersion == 1.0`. 아니면 `bad-schema-version`.
4. `id` 가 폴더 이름과 같다. 아니면 `pack-id-mismatch`.
5. `id` 가 안전하다(`SAFE_ID_RE`, 경로 구분자 없음). 아니면 `unsafe-id`.
6. `title`·`axis` 가 비지 않은 문자열. 공란은 `empty-field`, 숫자·목록은
   `bad-type`.
7. `requires` 가 객체이고 `commands` 가 비지 않은 문자열 목록. 비면 예전
   메시지 그대로 `requires.commands 가 비었다 — 요구 capability 선언은 필수`.
   항목이 `""` 이면 `empty-commands`.
8. `runner` 가 객체이고 세 키가 비지 않은 문자열. `rhwpCommit` 은 40자리
   hex, `capabilitiesSha256` 은 64자리 hex. 길이·문자가 틀리면
   `bad-runner-identity`.

`requires.commands` 가 비었다는 것은 "이 pack 을 채점할 capability 를
선언하지 않았다"는 뜻이다. 명령이 바이너리에 없는 자리와 다르다. 후자는
채점기의 `unavailable` 이다.

## 5. task 검증

대상: `packs/<id>/tasks/<ID>.json`. `where` 는 `<pack.id>/<task.id>` 다.

### 5.1 필수 키

`TASK_REQUIRED` 의 각 키가 없으면 `missing-key` 이고 메시지는
`필수 키 없음: <key>` 다. 이 조각은 이슈 본문이 명시한 첫 자리이며
`test_gym_packs` 이전부터 있었다.

키가 있는데 값이 공란이면 `empty-field` 다. 키가 없는 것과 한 줄로 뭉개지
않는다.

`id` 는 비지 않은 안전 id 다. `../T01`, `T 01`, 빈 문자열은 거절한다.

`title`·`input`·`instructions` 는 문자열이어야 한다. 숫자·목록은 `bad-type`.

### 5.2 tier

`is_valid_tier` 만 통과한다.

| 값 | 결과 | 이유 |
|---|---|---|
| 1, 2, 3, 4, 5 | 통과 | 입문…보스 |
| 0, 6, -1, 99 | `bad-tier` | 범위 밖 |
| `true`, `false` | `bad-tier` | bool 은 int 의 하위형 |
| `"1"`, `1.0`, `null` | `bad-tier` | 타입이 아님 |
| 키 없음 | `missing-key` + `bad-tier` | 예전 코드도 두 줄을 남겼다 |

메시지는 예전과 같다: `tier 는 1~5 정수 (1=입문 … 5=보스)`.

bool 을 받는 것은 입문 과제가 실수로 생기는 구멍이다. JSON `true` 가
Python `True` 가 되고, `isinstance(True, int)` 가 참이라 예전 검사가
속았다.

### 5.3 submit

`submit` 은 객체다. `kind` 는 `answer` / `artifact` / `pair` 만.
그 외는 `unknown-submit-kind`.

`files` 는 없어도 된다(`T01` 의 answer 가 그렇다). 있으면 비지 않은 문자열
목록이어야 한다. 빈 목록·문자열 하나·공란 항목은 `malformed-submit` 또는
`empty-field`.

원본 픽스처를 덮어쓰는 경로는 스키마가 아니라 채점·과제 지시가 막는다.
스키마는 선언의 모양만 본다.

### 5.4 checks

| 자리 | kind | 예전 메시지 |
|---|---|---|
| 키 없음 또는 `[]` | `empty-checks` | `checks 가 비었다` |
| 객체·문자열·숫자 | `not-a-list` | (예전엔 AttributeError) |
| 항목이 객체 아님 | `malformed-check` | (예전엔 AttributeError) |
| `name` 공란 | `malformed-check` | (예전에 스키마는 안 봄, 시험만 봄) |
| `name` 중복 | `duplicate-check-name` | (신규) |
| `op` 미등록 | `unknown-op` | `미등록 연산자: <op>` |
| 편집 축 + 전역 훑기, 사유 없음 | `global-scan-forbidden` | `전역 훑기 연산자` 포함 |
| `needs_cli` 인데 `cmd` 없음 | `missing-cmd` | `<op> 는 cmd 가 필요하다` |
| 파일 연산자인데 `cmd` 있음 | `unexpected-cmd` | `<op> 는 CLI 를 부르지 않는데 cmd 가 있다` |
| `cmd` 가 문자열 목록 아님 | `malformed-cmd` | (예전엔 `cmd[0]` 이 글자 하나) |
| `known_commands` 가 집합이고 `cmd[0]` 부재 | `unknown-command` | `CLI 에 없는 명령: <name>` |

`known_commands is None` 이면 명령 존재 검사를 건너뛴다. 바이너리 없이
스키마·연산자 계약만 보는 경로(`audit.py`, `test_gym_packs`)가 이 값이다.

`op` 가 미등록이면 그 항목의 cmd 검사는 하지 않는다. 없는 연산자에게
cmd 를 요구하는 것은 거짓말이다.

### 5.5 전역 훑기

`EDITING_AXES = ("편집", "보안")`. 과제 `axis` 가 있으면 그것을, 없으면
pack `axis` 를 본다. `startswith` 이라 `편집 (표 좌표 지정)` 도 편집이다.

`GLOBAL_SCAN_OPS` 는 `checks.py` 의 `{deep_contains, not_contains}` 다.
스키마가 이 집합을 늘리지 않는다. 편집·보안 축에서 쓰려면
`allowGlobalScan` 에 사유 문자열을 남긴다(#4600).

## 6. profile 검증

대상: `gym/profiles/<id>.json`. `where` 는 `profiles/<id>` 다.

1. 루트가 객체. 아니면 `not-a-mapping`.
2. `kind == gymProfile`. 아니면 `bad-kind`. 메시지: `kind 가 gymProfile 가 아니다`.
3. `schemaVersion` 이 있으면 `1.0` 이어야 한다. 키가 없는 기존 파일은
   이 칸을 내지 않는다(저장소 프로파일은 모두 가지고 있다).
4. `id` 가 있으면 안전해야 한다.
5. `packs` 가 비면 `empty-packs`. 메시지: `packs 가 비었다`.
6. `packs` 가 목록이 아니면 `not-a-list`. 문자열을 글자로 쪼개지 않는다.
7. 항목이 공란이면 `empty-field`, 경로면 `unsafe-id`, 중복이면
   `duplicate-pack`.
8. `pack_ids` 가 주어졌고 항목이 그 집합에 없으면 `profile-missing-pack`.
   메시지: `없는 pack 참조: <id>`. 이슈 본문의 네 번째 자리다.

`pack_ids is None` 이면 존재 검사를 건너뛴다. pack 나무를 아직 모를 때
모양만 보는 경로다.

프로파일은 pack 을 **고르는** 도구이지 점수를 뭉치는 도구가 아니다.
스키마도 그 결을 지킨다. packs 목록의 합산 규칙을 만들지 않는다.

## 7. 이슈 kind 카탈로그

`SCHEMA_ISSUE_KINDS` 와 `SCHEMA_ISSUE_HELP` 가 같은 표다. 시험
`CatalogTests` 가 길이·중복·도움말을 고정한다. 모르는 kind 는
`unexpected` 도움말로 접힌다.

| kind | 한 줄 |
|---|---|
| `missing-key` | 필수 키가 객체에 없다 |
| `empty-field` | 키는 있는데 값이 공란이거나 falsy 다 |
| `bad-type` | 값의 파이썬 타입이 계약과 다르다 |
| `bad-kind` | kind 가 gymPack / gymProfile 이 아니다 |
| `bad-schema-version` | schemaVersion 이 1.0 이 아니다 |
| `bad-tier` | tier 가 1~5 정수가 아니다 (bool 포함) |
| `bad-id` | id 가 비었거나 허용 문자가 아니다 |
| `pack-id-mismatch` | pack.id 가 폴더 이름과 다르다 |
| `unknown-op` | checks[].op 가 REGISTRY 에 없다 |
| `unknown-submit-kind` | submit.kind 가 answer/artifact/pair 가 아니다 |
| `empty-checks` | checks 가 비어 통과할 칸이 없다 |
| `malformed-check` | checks 항목이 객체가 아니거나 이름/op 가 없다 |
| `malformed-cmd` | cmd 가 비지 않은 문자열 목록이 아니다 |
| `malformed-submit` | submit 이 객체가 아니거나 files 가 깨졌다 |
| `malformed-requires` | requires 가 객체가 아니거나 commands 가 목록이 아니다 |
| `malformed-runner` | runner 가 객체가 아니다 |
| `malformed-object` | JSON 파싱 실패 또는 루트가 객체가 아니다 |
| `missing-cmd` | needs_cli 연산자인데 cmd 가 없다 |
| `unexpected-cmd` | 파일 연산자인데 cmd 가 있다 |
| `unknown-command` | cmd[0] 이 알려진 CLI 명령이 아니다 |
| `global-scan-forbidden` | 편집·보안 축에서 전역 훑기 연산자를 썼다 |
| `profile-missing-pack` | 프로파일이 없는 pack id 를 가리킨다 |
| `empty-packs` | 프로파일 packs 가 비었다 |
| `duplicate-pack` | 프로파일 packs 에 같은 id 가 두 번 있다 |
| `unsafe-id` | id 에 경로 구분자나 .. 가 있다 |
| `bad-runner-identity` | runner 신원의 길이·hex 가 틀렸다 |
| `not-a-mapping` | 객체여야 할 자리가 객체가 아니다 |
| `not-a-list` | 목록이어야 할 자리가 목록이 아니다 |
| `duplicate-check-name` | 한 과제 안에서 check.name 이 겹친다 |
| `empty-commands` | requires.commands 가 비었거나 항목이 공란이다 |
| `unexpected` | 분류되지 않은 스키마 위반 |

kind 를 늘릴 때는 도움말과 시험을 같이 늘린다. 도움말 없는 kind 는
카탈로그 시험이 막는다.

## 8. 안전 id

`is_safe_id` 가 허용하는 것:

- 첫 글자 `[A-Za-z0-9]`
- 나머지 `[A-Za-z0-9._-]*`
- 앞뒤 공백 없음
- `/` `\` `:` `..` `.` 없음

그래서 `table-editing`, `core-cli`, `TB01`, `T10`, `render-tree`,
`studio-e2e`, `self-description` 은 통과하고 `../x`, `T 01`, `한글`,
`-leading` 은 거절한다.

pack id 와 폴더 이름이 같다는 검사와 겹친다. 폴더가 `../x` 이면 둘 다
불이 난다. 경로 구분자로 pack 을 고르면 채점기가 디렉터리를 벗어난다.
스키마가 그 입구를 막는다.

## 9. capabilities 와 실행 신원

이 세 함수의 **바깥 계약**은 예전과 같다. `runner.py` 가 그대로 부른다.

```text
capabilities_digest(bin_path) -> (sha256, raw)
known_commands(bin_path) -> set[str] | None
runner_identity(bin_path, repo_root) -> {rhwpVersion, rhwpCommit, capabilitiesSha256}
```

hardening:

- `bin_path` 가 비면 `ValueError`. 빈 문자열로 `subprocess.run` 을 부르지
  않는다.
- stdout 이 `None` 이면 빈 바이트로 해시를 남긴다.
- `known_commands` 는 `TypeError`·`UnicodeError`·`AttributeError` 도 None
  으로 접는다. 예전에는 `ValueError`·`KeyError` 만 잡아 목록이 깨지면
  채점 전체가 죽었다.
- 빈 집합과 None 은 다르다. 빈 집합은 "명령을 읽었고 하나도 없다".
- `try_known_commands` 는 바이너리 부재(`OSError`)를 None 으로 접는 선택
  경로다. `known_commands` 자체는 예전처럼 예외를 던질 수 있다.
- `parse_capabilities_payload` / `parse_command_names` /
  `parse_capabilities_version` 은 예외를 밖으로 던지지 않는다.

`runner.rhwpCommit` 의 40자리 hex 와 `capabilitiesSha256` 의 64자리 hex 는
pack 선언을 검증할 때 본다. 실행 시점 `runner_identity` 가 빈 커밋을 돌려줄
수는 있다(git 이 없는 자리). 그 빈 값은 pack 에 넣으면 `empty-field` 다.

## 10. 나무 전수

`validate_gym_tree(gym_root)` 는 `gym/packs/*/pack.json` 과
`gym/packs/*/tasks/*.json` 과 `gym/profiles/*.json` 을 읽는다. JSON 이
깨지면 `malformed-object`, 루트가 배열이면 `not-a-mapping`, 파일이 없으면
그 pack 만 건너뛴다. 한 파일이 죽어도 나머지를 본다.

이 함수는 `audit.py` 를 대체하지 않는다. 감사기는 기준 풀이 짝·고아
reference·전역 과제 ID 충돌을 추가로 본다. 스키마 나무는 선언의 모양만
본다. 두 도구를 한 파일로 합치지 않는다 — 감사기는 열린 이웃 PR 의
대상이 될 수 있고, 이슈는 그 파일을 고치지 말라고 했다.

## 11. 연산자 필드 힌트

`CHECK_FIELD_HINTS` 는 devel 에 있는 `REGISTRY` 키만 나열한다. 새 키를
여기서 만들어 등록하지 않는다. `lint_check_fields(check)` 가 빠진 필드를
돌려준다.

기본 `validate_task` 는 이 힌트를 **강제하지 않는다.** 연산자의 필수
인자는 채점 시점의 연산자 구현이 실패로 남긴다. 스키마가 힌트를 강제로
올리면, 열린 PR 5210 이 등록부에 키를 더하는 순간 힌트 표와 어긋난다.

힌트 표에 있는 키가 `REGISTRY` 에 없는 것은 시험이 막는다. `REGISTRY` 에
있는 키가 힌트 표에 없는 것도 막는다 — devel 표면이 흔들리면 시험을
고친다. 등록부 자체는 고치지 않는다.

## 12. 하지 않는 것

- 새 CLI 플래그, 새 연산자, 새 pack, 새 프로파일.
- `checks.REGISTRY` 변경. `GLOBAL_SCAN_OPS` 변경. `needs_cli` 뒤집기.
- `audit.py`, `certify.py`, `report.py`, `score.py`, `runner.py` 수정.
- tutorial 문서 수정.
- 열린 PR 5210–5278 이 만진 파일 수정.
- 성공한 과제의 점수를 바꾸기.
- 부재(명령 없음)를 스키마 위반으로 바꾸기.
- `schemaVersion` 을 2.0 으로 올리기.

## 13. 사용 예

저장소 나무가 깨끗한지(감사기와 같은 성공 칸):

```bash
python -m unittest scripts.tests.test_gym_packs scripts.tests.test_gym_schema
python gym/tools/audit.py
```

임시 나무가 네 자리를 남기는지:

```python
from gym.core import schema

task = {"id": "X", "tier": 0, "checks": [{"name": "c", "op": "ghost"}]}
print(schema.collect_task(task, {"id": "p"}).kinds())
# missing-key, bad-tier, unknown-op, ...

profile = {"kind": "gymProfile", "id": "z", "title": "z", "packs": ["ghost"]}
print(schema.collect_profile(profile, {"core-cli"}).kinds())
# profile-missing-pack
```

`IssueList.kinds()` / `has_kind` / `of_kind` / `as_dicts` 가 시험이 쓰는
손잡이다. 문자열 목록만 받는 예전 호출자는 손대지 않아도 된다.

## 14. 최소 뼈대

시험과 문서가 같은 상수를 쓴다. 저장소 pack 을 바꾸지 않는다.

```json
{
  "schemaVersion": "1.0",
  "kind": "gymPack",
  "id": "demo-pack",
  "title": "데모",
  "axis": "시험",
  "requires": {"commands": ["info"]},
  "runner": {
    "rhwpVersion": "0.0.0",
    "rhwpCommit": "<40 hex>",
    "capabilitiesSha256": "<64 hex>"
  }
}
```

```json
{
  "id": "D01",
  "tier": 1,
  "title": "데모 과제",
  "input": "samples/x.hwp",
  "instructions": "제출하라",
  "submit": {"kind": "answer"},
  "checks": [{"name": "존재", "op": "file_exists", "file": "answer.json"}]
}
```

```json
{
  "schemaVersion": "1.0",
  "kind": "gymProfile",
  "id": "demo",
  "title": "데모 코스",
  "packs": ["demo-pack"]
}
```

`schema.MINIMAL_PACK` / `MINIMAL_TASK` / `MINIMAL_PROFILE` 과 같다.
`clone_minimal_*` 는 깊은 복사다. 시험이 한 뼈대를 더럽혀도 다음 칸이
깨끗하다.

## 15. 메시지 조각 — 바꾸면 소비자가 깨진다

아래 조각은 `audit.py` 출력과 `test_gym_packs` 와 이 모듈의
`MSG_*` 상수가 공유한다. 뜻을 바꾸려면 시험을 먼저 고친다.

| 조각 | 자리 |
|---|---|
| `kind 가 gymPack 가 아니다` | pack kind |
| `schemaVersion 이 1.0 이 아니다` | pack / profile 버전 |
| `pack id(...) 가 폴더 이름과 다르다` | pack id |
| `<key> 가 비었다` | title / axis / runner.* |
| `requires.commands 가 비었다 — 요구 capability 선언은 필수` | requires |
| `runner.<key> 가 비었다 — 기준 실행 신원 선언은 필수` | runner |
| `필수 키 없음: <key>` | task |
| `tier 는 1~5 정수 (1=입문 … 5=보스)` | tier |
| `checks 가 비었다` | checks |
| `미등록 연산자: <op>` | op |
| `편집 과제에 전역 훑기 연산자(<op>)` | 전역 훑기 |
| `<op> 는 cmd 가 필요하다` | cmd 부재 |
| `CLI 에 없는 명령: <name>` | 명령 표면 |
| `<op> 는 CLI 를 부르지 않는데 cmd 가 있다` | 파일 연산자 + cmd |
| `kind 가 gymProfile 가 아니다` | profile kind |
| `packs 가 비었다` | profile packs |
| `없는 pack 참조: <id>` | profile 대상 |

새 칸은 새 문장을 쓴다. 예전 문장을 재사용하지 않는다. 한 문장이 두
kind 를 겸하면 시험이 갈라지지 못한다.

## 16. IssueList 손잡이

`IssueList` 는 `list` 의 하위형이다. `audit.py` 처럼 평범한 목록을 넘기면
이 손잡이는 쓰이지 않는다. 시험과 도구가 kind 로 가르고 싶을 때만 쓴다.

| 손잡이 | 반환 | 쓰임 |
|---|---|---|
| `append_issue(kind, where, message, field=, got=)` | `SchemaIssue` | `_fail` 가 부른다 |
| `kinds()` | kind 문자열 목록 (등장 순) | 한 호출의 칸 종류 |
| `has_kind(kind)` | bool | 네 자리 재현 시험 |
| `of_kind(kind)` | `SchemaIssue` 목록 | 같은 kind 의 필드 |
| `fields_of(kind)` | field 목록 | `missing-key` 의 빠진 키 |
| `as_dicts()` | JSON 으로 남길 수 있는 목록 | 기계 봉투 |

`SchemaIssue.as_text()` 는 평범한 list 에 쌓이는 줄과 같다.
`as_dict()` 는 `field`·`got` 가 있을 때만 그 키를 넣는다. `got` 는 80
글자로 접힌다. 객체는 타입 이름만 남긴다.

모르는 kind 를 생성자에 넘기면 `unexpected` 로 접힌다. 카탈로그에 없는
이름을 시험이 발명하지 못하게 한다.

## 17. 관련 문서

- 운동장 입구: [`gym/README.md`](../README.md)
- 테마파크 지도: [`gym/PARK.md`](../PARK.md) — 이 PR 은 고치지 않는다
- 채점 연산자 목록: 열린 PR 5210 의 `gym/docs/checks.md` (이 브랜치에 없음)
- 채점기 예외 경로: 열린 PR 5278 의 `gym/docs/score_runner.md` (이 브랜치에 없음)
- 작업 기록: [`mydocs/working/gym_schema.md`](../../mydocs/working/archives/gym_schema.md)
