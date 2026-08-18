---
kind: guide
status: active
canonical: gym/docs/audit.md
last_verified: 2026-08-18
---

# gym 정합 감사 규약

이 문서는 `gym/tools/audit.py` 의 **전 pack 정합 계약**, **위반 코드
카탈로그**, **예외 경로**, **보고 봉투**를 고정한다. 작업 기록은
[`mydocs/working/gym_audit.md`](../../mydocs/working/gym_audit.md) 를
본다. 시험 계약은 `scripts/tests/test_gym_audit.py` 가 기계로 고정한다.

개별 검증(`schema.validate_pack` / `validate_task`)은 pack 하나·과제
하나만 본다. 이 도구는 **전 저장소에 걸친 정합**을 본다. 과제↔기준
짝, 과제 ID 전역 고유, 고아 기준풀이, 빠진 `pack.json`, 스키마 위반.
바이너리 없이 순수 파일 검사라 CI 에서 상시 돈다.

이 도구는 새 CLI 를 열지 않는다. 새 pack 을 만들지 않는다. `--json`
이외의 플래그를 추가하지 않는다. 한 pack 만 봐서 전역 ID 충돌이
없다고 말하면 거짓말이다.

## 1. 왜 이 기둥이 필요한가

gym 이 자라고 기여자가 늘수록, 새 pack 이 조용히 규약을 어길 수
있다. 그 구멍은 채점기가 아니라 정합 감사기가 막는다.

| 기둥 | 도구 | 질문 |
|---|---|---|
| 종점 무결성 | `discriminate.py` | 일 안 한 제출이 만점을 받나? |
| 경로 무결성 | `trajectory.py` | 마지막 스텝을 빼도 통과하나? |
| 전 저장소 정합 | `audit.py` (#4803 / #5277) | 빠진 기준풀이·ID 충돌·스키마 위반이 있나? |

채점기는 과제가 선언한 검사를 돌린다. 과제가 기준풀이 없이
들어와도 채점기는 "이 과제는 풀 수 있다"고 전제한다. 리더보드는
과제 ID 로 행을 가른다. 두 pack 이 같은 ID 를 쓰면 집계가 섞인다.
`schema.validate_task` 는 그 파일만 보고, 옆 pack 의 같은 ID 는
모른다.

그래서 감사기는 전 pack 을 한 번에 본다. CI 의
`Validate gym scorer contracts` 가 `test_gym_audit.py` 를 돌리고,
그 테스트가 실제 `gym/packs` 와 픽스처 네 자리(빠진 pack.json ·
고아 기준풀이 · 중복 ID · 나쁜 스키마)를 함께 고정한다.

## 2. 사용

```bash
python gym/tools/audit.py
python gym/tools/audit.py --json
```

| 인자 | 기본 | 의미 |
|---|---|---|
| `--json` | 꺼짐 | 사람 요약 대신 `gymAudit` 봉투를 stdout 에 쓴다. |

새 플래그는 없다. `--pack` / `--task` / `--out` / `--strict` /
`--root` 는 없다. 라이브러리 `audit(packs_root)` 는 시험이 임시
디렉터리를 넘길 수 있게 루트를 받는다. CLI 는 항상 `gym/` 을
본다.

종료 코드:

| 코드 | 상수 | 의미 |
|---|---|---|
| 0 | `EXIT_OK` | 위반 0 · 도구 실패 없음 |
| 1 | `EXIT_VIOLATION` | pack 정합 위반 또는 전역 ID 충돌 |
| 2 | `EXIT_TOOL` | packs 루트 부재· packs 가 디렉터리가 아님 · 나열 실패 |

`ok` 는 `issueCount == 0` **그리고** `toolFailed` 가 거짓일 때만
true 다. 루트를 못 읽었는데 정합 0건이라고 쓰면 거짓말이다.

사람 요약:

```
gym 정합 감사: 18 pack 전부 통과 — 위반 0
```

또는

```
gym 정합 감사: 위반 3건
  [ghost] pack.json 이 없다
  [p1] 고아 기준풀이 reference/X01.json — 짝 과제(tasks/X01.json)가 없다
  [전역] 과제 ID 'DUP' 충돌: p1, p2
```

도구 실패:

```
gym 정합 감사: 도구 실패 — packs 루트가 없다 — 정합 0건으로 위장하지 않는다
```

## 3. 검사하는 것

감사기가 한 pack 에 대해 보는 순서다. 앞 단계에서 매니페스트를
못 읽으면 그 pack 의 과제 스캔은 하지 않는다. 없는 매니페스트를
스키마 위반으로 위장하지 않는다.

1. **packs 루트** — `packs_root/packs` 가 있어야 한다. 없으면
   `missing-packs-root`, exit 2.
2. **pack 폴더** — 디렉터리만 본다. `packs/notes.txt` 같은 파일은
   무시한다. README 도 pack 이 아니다.
3. **pack.json** — 있어야 하고, UTF-8 JSON 객체여야 한다. 없으면
   `missing-pack-json`. 파싱 실패는 `pack-json-parse`. 배열·숫자는
   `pack-json-not-object`.
4. **스키마** — `schema.validate_pack(manifest, pack_dir, errors)`.
   메시지는 `bad-schema` 로 접고, `schemaTag` 로 세부(kind /
   schemaVersion / pack-id / title / axis / requires / runner)를
   남긴다.
5. **tasks / reference** — 디렉터리면 나열한다. 파일이면
   `tasks-not-dir` / `reference-not-dir`. 나열 실패는
   `unlistable-tasks` / `unlistable-reference`. 없는 디렉터리는
   원 계약대로 빈 목록이다.
6. **과제 파일** — 소문자 `.json` 만. `.JSON` · `.txt` ·
   `.hidden.json` 은 과제가 아니다. 객체여야 하고,
   `schema.validate_task(task, manifest, None, errors)` 로 구조를
   본다. `known_commands=None` — 명령 존재 검사는 러너 몫이다.
7. **파일명 ↔ id** — `X01.json` 의 `id` 는 `X01` 이어야 한다.
   빈 id 는 `task-empty-id`. 다른 id 는
   `task-filename-id-mismatch`.
8. **짝 기준풀이** — 같은 이름의 `reference/X01.json` 이 있어야
   한다. 없으면 `missing-reference`. 있으면 객체를 읽고
   `ref.id == task.id` 인지 본다. 다르면
   `reference-id-mismatch`.
9. **고아 기준풀이** — `reference` 에만 있는 `.json` 은
   `orphan-reference`. 짝짓기는 **파일 이름**이지 id 가 아니다.
10. **pack 안 중복** — 같은 pack 의 두 파일이 같은 `id` 를 쓰면
    `task-id-duplicate-in-pack`. 전역 충돌로 위장하지 않는다.
11. **빈 pack** — 나열에 성공했는데 과제 `.json` 이 0 이면
    `empty-pack`. 나열을 못 한 자리에는 빈 pack 이라고 쓰지
    않는다.
12. **전역 ID** — 서로 다른 pack 이 같은 `id` 를 쓰면
    `taskIdCollisions` 와 `task-id-collision`. 같은 pack 의 이중
    등록은 여기 넣지 않는다.

## 4. 위반 코드 카탈로그

`ISSUE_CODES` 는 아래 표와 같다. 코드를 추가하면 이 표와
`ISSUE_FAMILY` · `ISSUE_TEXT` · 시험을 같이 고친다. 지금 카탈로그의
모든 코드는 차단이다. 경고 등급을 숨기지 않는다.

### 4.1 루트 (family=`root`)

| 코드 | 언제 | 종료 |
|---|---|---|
| `missing-packs-root` | `packs/` 가 없다 | 2 |
| `packs-not-dir` | `packs` 가 파일이다 | 2 |
| `unlistable-packs` | `listdir` 권한·OS 오류 | 2 |
| `empty-packs-root` | 카탈로그에만 있다. 빈 `packs/` 는 원 계약대로 위반 0 | 0 |

빈 `packs/` 디렉터리는 위반이 아니다. 시험 픽스처가 폴더만 만들고
pack 을 아직 안 넣은 상태와, 저장소에 pack 이 한 개도 없는 상태를
도구 실패로 부르지 않는다. 실제 저장소 시험은 `packCount >= 10` 을
따로 강제한다.

### 4.2 매니페스트 (family=`manifest`)

| 코드 | 한글 줄 (packs[].issues) |
|---|---|
| `missing-pack-json` | `pack.json 이 없다` |
| `pack-json-parse` | `pack.json 파싱 실패: …` |
| `pack-json-not-object` | `pack.json 이 객체가 아니다` |
| `pack-json-unreadable` | `pack.json 을 읽을 수 없다: …` |

원 계약 문구 `pack.json 이 없다` 와 `pack.json 파싱 실패` 는
바꾸지 않는다. `test_gym_audit.py` 의 레거시 시험이 이 부분
문자열을 본다.

### 4.3 스키마 (family=`schema`)

| 코드 | 의미 |
|---|---|
| `bad-schema` | `schema.validate_pack` 또는 `validate_task` 가 메시지를 남겼거나, 그 호출이 비치명 예외로 죽었다 |

`schemaTag` 는 메시지를 다시 분류한 세부다. 코드는 항상
`bad-schema` 다.

| schemaTag | 메시지 단서 |
|---|---|
| `kind` | `kind` + `아니다` |
| `schemaVersion` | `schemaVersion` |
| `pack-id` | `폴더 이름` 또는 `pack id` |
| `title` | `title` + `비었` |
| `axis` | `axis` + `비었` |
| `requires` | `requires.commands` |
| `runner` | `runner.` |
| `task-required` | `필수 키 없음` |
| `tier` | `tier` |
| `checks-empty` | `checks 가 비었` |
| `unknown-op` | `미등록 연산자` |
| `global-scan` | `전역 훑기` |
| `missing-cmd` | `cmd 가 필요` |
| `unexpected-cmd` | `cmd 가 있다` |
| `other` | 위에 안 걸림 |

스키마 함수가 `TypeError` / `RuntimeError` 로 죽어도 감사기는
죽지 않는다. 그 문장은 `schema.validate_pack 예외: …` 로 접혀
같은 `bad-schema` 가 된다. `KeyboardInterrupt` · `SystemExit` ·
`MemoryError` · `GeneratorExit` 는 다시 올린다.

### 4.4 배치 (family=`layout`)

| 코드 | 언제 |
|---|---|
| `missing-tasks-dir` | 나열 문맥에서 tasks 가 없음 (예외 접기) |
| `tasks-not-dir` | `tasks` 가 파일이다 |
| `unlistable-tasks` | tasks 를 나열할 수 없다 |
| `missing-reference-dir` | 나열 문맥에서 reference 가 없음 |
| `reference-not-dir` | `reference` 가 파일이다 |
| `unlistable-reference` | reference 를 나열할 수 없다 |
| `empty-pack` | 나열 성공 · 과제 `.json` 0 |

없는 `tasks/` 는 원 계약대로 빈 목록이다. 그 위에 고아 기준풀이가
있으면 `orphan-reference` 와 `empty-pack` 이 같이 난다. 나열을 못
했는데 빈 pack 이라고 쓰면 거짓말이다.

### 4.5 과제 파일 (family=`task`)

| 코드 | 한글 줄 |
|---|---|
| `task-parse` | `tasks/{name} 파싱 실패: …` |
| `task-not-object` | `tasks/{name} 이 객체가 아니다` |
| `task-unreadable` | `tasks/{name} 을 읽을 수 없다: …` |

파싱에 실패해도 파일 이름은 짝짓기에 남는다. 같은 이름의
기준풀이가 없으면 `missing-reference` 를 추가로 남긴다. 기준풀이가
있으면 고아로 부르지 않는다 — 과제가 * committ 되어 있고 내용만
깨진 것이다.

### 4.6 짝짓기 (family=`pairing`)

| 코드 | 한글 줄 |
|---|---|
| `missing-reference` | `과제 {name} 에 짝 기준풀이(reference/{name})가 없다 — 해결 가능성 미선언` |
| `reference-parse` | `reference/{name} 파싱 실패: …` |
| `reference-not-object` | `reference/{name} 이 객체가 아니다` |
| `reference-unreadable` | `reference/{name} 을 읽을 수 없다: …` |
| `reference-id-mismatch` | `reference/{name} 의 id({rid}) 가 과제 id({tid}) 와 다르다` |
| `orphan-reference` | `고아 기준풀이 reference/{name} — 짝 과제(tasks/{name})가 없다` |

짝짓기 키는 **파일 이름**이다. `Y99.json` 이 `id=Y99` 를 들고
있으면 파일명 불일치는 identity 가족이고, 짝은 이미 맞다.

### 4.7 신원 (family=`identity`)

| 코드 | 한글 줄 |
|---|---|
| `task-empty-id` | `과제 {name} 의 id 가 비었다` |
| `task-filename-id-mismatch` | `과제 {name} 의 id({tid}) 가 파일 이름과 다르다` |
| `task-id-duplicate-in-pack` | `pack 안 과제 ID '{tid}' 가 여러 파일에 있다: a.json, b.json` |
| `task-id-collision` | 전역. `packs[].issues` 가 아니라 `taskIdCollisions` |

전역 충돌은 `issueCount` 에 충돌 키 수만큼 더한다. 원 계약:
`issueCount = sum(len(p.issues)) + len(taskIdCollisions)`.

### 4.8 도구 (family=`tool`)

| 코드 | 언제 |
|---|---|
| `unexpected` | 카탈로그 밖 예외, 모르는 코드 |

## 5. 보고 봉투

`kind=gymAudit`, `schemaVersion=1.0`. 버전을 올리지 않는다. 키를
빼면 `validate_report` 가 막는다. 원 계약 키는 그대로다.

| 키 | 형 | 원 계약 | 의미 |
|---|---|---|---|
| `kind` | str | 예 | 항상 `gymAudit` |
| `schemaVersion` | str | 예 | 항상 `1.0` |
| `ok` | bool | 예 | 위반 0 이고 도구 실패 아님 |
| `packCount` | int | 예 | 본 pack 폴더 수 |
| `packs` | list | 예 | 이슈가 있는 pack 만 `{id, issues}` |
| `taskIdCollisions` | object | 예 | `{id: [pack, …]}` · 서로 다른 pack 만 |
| `issueCount` | int | 예 | pack 이슈 줄 + 전역 충돌 수 |
| `taskCount` | int | 추가 | 과제 `.json` 수 |
| `referenceCount` | int | 추가 | 기준풀이 `.json` 수 |
| `okPacks` | list | 추가 | 이슈 없는 pack id |
| `emptyPacks` | list | 추가 | 과제 0 인 pack id |
| `issues` | list | 추가 | 구조화 위반 |
| `issueCountsByCode` | object | 추가 | 코드별 건수 |
| `issueCountsByFamily` | object | 추가 | 가족별 건수 |
| `toolErrors` | list | 추가 | 도구 자리 오류 |
| `missingPacksRoot` | bool | 추가 | packs 루트 부재 |
| `toolFailed` | bool | 추가 | 도구가 전수 검사를 못 함 |
| `exit` | int | 추가 | 0 / 1 / 2 |

구조화 이슈 한 줄:

```json
{
  "code": "orphan-reference",
  "pack": "p1",
  "path": "p1/reference/X01.json",
  "message": "고아 기준풀이 reference/X01.json — 짝 과제(tasks/X01.json)가 없다",
  "family": "pairing"
}
```

필수 키는 `code` · `pack` · `path` · `message` · `family` 다.
선택 키: `schemaTag`, `taskId`, `owners`.

`packs[].issues` 는 **문자열 목록**이다. 구조화 객체로 바꾸지
않는다. 레거시 시험이 `"기준풀이" in i`, `"고아" in i` 를 보기
때문이다.

## 6. 예외 경로

도구가 한 파일의 파싱 실패로 죽지 않는다. 치명 예외만 다시 올린다.

`FATAL_EXCEPTIONS = (KeyboardInterrupt, SystemExit, MemoryError, GeneratorExit)`

접는 자리:

| 자리 | 접는 것 | 접지 않는 것 |
|---|---|---|
| packs 루트 | 부재 · 파일 · 나열 실패 | 치명 예외 |
| pack.json | 파싱 · 디코드 · 권한 · 비객체 | 치명 예외 |
| tasks/reference 나열 | 권한 · OS 오류 · 파일 | 치명 예외 |
| 과제/기준풀이 JSON | 파싱 · 디코드 · 비객체 | 치명 예외 |
| `schema.validate_*` | TypeError · RuntimeError · ValueError | 치명 예외 |
| pack 루프 | 그 pack 만 `unexpected` | 치명 예외 |

`exception_kind(exc, context)` 는 같은 `FileNotFoundError` 라도
문맥에 따라 코드가 갈린다.

| context | FileNotFoundError |
|---|---|
| `packs-root` | `missing-packs-root` |
| `pack-json` | `missing-pack-json` |
| `listdir-tasks` | `missing-tasks-dir` |
| `listdir-reference` | `missing-reference-dir` |

`JSONDecodeError` 는 `pack-json-parse` / `task-parse` /
`reference-parse`. `PermissionError` 는 unlistable / unreadable.
`NotADirectoryError` 는 `*-not-dir`.

`load_object` 는 JSON 이 객체인지까지 본다. 배열·숫자·문자열은
파싱 성공이 아니라 `*-not-object` 다. `task.get("id")` 가
`AttributeError` 로 감사기를 죽이던 구멍을 이 검사가 막는다.

## 7. 순수 함수

시험이 바이너리 없이 고정하는 함수다. 파일 시스템이 필요 없는
것은 순수다.

| 함수 | 순수 | 역할 |
|---|---|---|
| `is_fatal_exception` | 예 | 치명 여부 |
| `truncate_head` | 예 | 오류 머리 |
| `exception_kind` | 예 | 예외 → 코드 |
| `exception_record` | 예 | 오류 한 줄 |
| `issue_family` / `issue_codes` / `catalog_ids` | 예 | 카탈로그 |
| `is_known_code` / `is_blocking_code` | 예 | 코드 소속 |
| `format_issue_message` / `make_issue` | 예 | 구조화 이슈 |
| `posix_rel` / `is_json_name` / `stem_of` | 예 | 이름 |
| `pair_names` | 예 | 파일명 짝짓기 |
| `detect_in_pack_duplicates` | 예 | pack 안 중복 |
| `detect_global_collisions` | 예 | 전역 충돌 (서로 다른 pack) |
| `classify_schema_message` | 예 | schemaTag |
| `pack_issue_line` | 예 | 레거시 한글 줄 |
| `empty_report` / `validate_report` | 예 | 봉투 계약 |
| `format_human_report` / `format_json_report` | 예 | 출력 |
| `resolve_exit` | 예 | 종료 코드 |
| `list_dir_safe` / `load_json_safe` / `load_object` | 아니오 | I/O, 예외 접기 |
| `run_validate_pack` / `run_validate_task` | 아니오 | schema 호출 |
| `audit_one_pack` / `audit` | 아니오 | 전수 검사 |
| `parse_args` / `main` | 아니오 | CLI |

`_load` 는 원 계약의 날것 로더다. 예외를 그대로 올린다. 감사
본문은 `load_object` 를 쓴다.

## 8. 하지 않는 것

- 새 CLI 플래그를 열지 않는다.
- 새 pack · 새 과제를 만들지 않는다.
- `certify.py` · `report.py` · `score.py` · `runner.py` ·
  `build_baseline.py` 를 고치지 않는다.
- `schema.py` 의 규칙을 여기서 다시 구현하지 않는다. 감싸서
  메시지를 접을 뿐이다.
- 명령 존재 검사를 하지 않는다. `known_commands=None`.
- 한 pack 만 골라 통과했다고 전 저장소 정합을 선언하지 않는다.
- 도구 실패를 위반 0 으로 위장하지 않는다.
- 치명 예외를 삼키지 않는다.
- `.JSON` 대문자 확장자를 과제로 치지 않는다. 원 계약은 소문자
  `.json` 이다.
- README · assets · 숨은 파일을 위반으로 부르지 않는다.

## 9. 픽스처로 고정한 네 자리

이슈 #5277 가 명시한 자리. 시험이 각각 red 로 막는다.

### 9.1 빠진 pack.json

```
packs/ghost/          # 폴더만 있다
```

결과: `ok=false`, `exit=1`, 코드 `missing-pack-json`, 한글
`pack.json 이 없다`. 과제 스캔은 하지 않는다. 없는 매니페스트를
`bad-schema` 로 부르지 않는다.

### 9.2 고아 기준풀이

```
packs/p1/pack.json
packs/p1/reference/X01.json
# tasks/X01.json 없음
```

결과: `orphan-reference` + `empty-pack`. 한글에 `고아` 가 있다.

### 9.3 중복 과제 ID

```
packs/p1/tasks/DUP.json   id=DUP
packs/p2/tasks/DUP.json   id=DUP
```

결과: `taskIdCollisions.DUP == ["p1", "p2"]`, 코드
`task-id-collision`. 사람 요약에 `[전역]`.

같은 pack 의 `A01.json` 과 `A01b.json` 이 둘 다 `id=A01` 이면
`task-id-duplicate-in-pack` 이지 전역 충돌이 아니다.

### 9.4 나쁜 스키마

매니페스트 `kind` 가 `gymPack` 이 아니거나, `schemaVersion` 이
`1.0` 이 아니거나, `id` 가 폴더 이름과 다르거나, `title`/`axis` 가
비었거나, `requires.commands` 가 비었거나, `runner.*` 가 비었거나,
과제에 필수 키가 없거나, `tier` 가 1~5 가 아니거나, `checks` 가
비었거나, 미등록 연산자이거나, 편집 축에 전역 훑기 연산자를
쓰거나, CLI 연산자에 `cmd` 가 없거나, 파일 연산자에 `cmd` 가
있으면 `bad-schema`.

## 10. 실제 저장소 계약

`audit(gym/)` 는 다음을 만족해야 한다. CI 가 본다.

- `ok is True`
- `packCount >= 10`
- `taskCount == referenceCount`
- `taskIdCollisions == {}`
- `emptyPacks == []`
- `toolFailed is False`
- `missingPacksRoot is False`
- `validate_report(report) == []`

새 pack 을 넣는 기여자는 이 감사기를 통과해야 한다. 기준풀이 없는
과제, 다른 pack 과 겹치는 ID, `pack.json` 없는 폴더, 스키마를
어긴 매니페스트는 이 관문에서 막힌다.

## 11. 구현 위치

| 경로 | 역할 |
|---|---|
| `gym/tools/audit.py` | 감사기 |
| `gym/core/schema.py` | pack/task 스키마 (이 도구가 감싼다, 고치지 않는다) |
| `scripts/tests/test_gym_audit.py` | 계약 시험 |
| `gym/docs/audit.md` | 이 문서 |
| `mydocs/working/gym_audit.md` | 작업 기록 |

CI: `.github/workflows/ci.yml` 의 `Validate gym scorer contracts`
가 `python3 -m unittest scripts/tests/test_gym_audit.py` 를 돌린다.

## 12. 변경 규칙

1. 원 계약 키(`ok`, `packs`, `taskIdCollisions`, `issueCount`,
   `packCount`, `kind`, `schemaVersion`)를 빼거나 형을 바꾸지
   않는다.
2. `packs[].issues` 는 문자열 목록으로 남긴다.
3. 레거시 한글 부분 문자열(`pack.json 이 없다`, `기준풀이`,
   `고아`, `파싱 실패`)을 바꾸지 않는다.
4. CLI 는 `--json` 만.
5. 코드를 추가하면 카탈로그 표·가족·문구·시험을 한 커밋에서
   맞춘다.
6. 치명 예외를 `except Exception` 으로 삼키지 않는다.
7. 새 pack 을 이 도구 PR 에 섞지 않는다.
