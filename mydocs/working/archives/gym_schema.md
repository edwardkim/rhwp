---
kind: working
status: active
canonical: gym/docs/schema.md
last_verified: 2026-08-18
---

# gym 코어 스키마 검증 고도화 — 작업 기록 (#5279)

정본 규약은 [`gym/docs/schema.md`](../../../gym/docs/schema.md) 다. 이 문서는
왜 그 칸이 생겼는지, 어떤 예외를 실패로 고정했는지, 단위시험이 무엇을
재현하는지를 남긴다. pack JSON 과 `checks.REGISTRY` 는 여기서 바꾸지 않는다.

## 한 줄 결론

`gym/core/schema.py` 의 pack/task/profile 검증이 객체 아님·공란·bool tier·
미등록 연산자·없는 pack 참조에서 죽거나 한 줄로 뭉개지던 자리를 kind 로
남긴다. 새 CLI 는 없다. 열린 PR 파일은 그대로다.

## 배경

gym 4부(#4653)가 선언 검증을 `schema.py` 한곳으로 모았다. `audit.py` 는
그 함수를 전 pack 에 돌리고, `test_gym_packs` 는 저장소 나무가 깨끗한지
본다. 그 성공 칸은 이미 단단하다.

약한 칸은 실패 경로였다.

1. **필수 키가 빠지면** `필수 키 없음` 은 남겼지만, 같은 함수가 `checks` 를
   순회하다 타입이 틀리면 예외로 죽었다. 감사기 한 pack 이 그 예외로
   멈춘다.
2. **tier 가 범위 밖이면** 메시지는 있었지만 `True` 가 1 로 통과했다.
3. **미등록 연산자** 는 거절했지만 kind 가 없어 시험이 한글 조각에만
   매였다. 등록부를 읽기만 해야 하는데, 실수로 키를 더하기 쉬운 자리였다.
4. **프로파일이 없는 pack 을 가리키면** `없는 pack 참조` 는 있었지만
   `packs` 가 문자열이면 글자를 순회했다.

#5279 는 그 네 자리를 명시했다. 스키마 모듈만 고친다. 채점기·감사기·
연산자 등록부·tutorial 은 이웃 PR 이 이미 붙잡고 있다.

## 하지 않은 것 (경계)

이 작업은 아래를 의도적으로 건드리지 않았다.

- `gym/core/checks.py` 의 `REGISTRY` / `GLOBAL_SCAN_OPS` / `needs_cli`.
  PR 5210 이 지목 연산자를 더하는 중이다.
- `gym/core/runner.py`, `gym/score.py`. PR 5278 이 채점기 예외 경로를
  고치는 중이다.
- `gym/tools/audit.py`, `gym/certify.py`, `gym/report.py`.
- `gym/tutorial/**`, `gym/PARK.md`, `gym/INVITE.md`.
- 열린 PR 5210–5278 이 추가·수정한 모든 파일.
- 저장소 pack JSON, 프로파일 JSON, 기준 풀이.

성공 칸의 메시지는 그대로다. `test_gym_packs` 가 기대한 조각
(`필수 키 없음`, `tier 는 1~5`, `미등록 연산자`, `없는 pack 참조`,
`checks 가 비었다`, `packs 가 비었다`)을 바꾸지 않았다.

## 설계

### 문자열 목록은 그대로, kind 는 선택

`audit.py` 는 `issues: list[str]` 에 줄을 쌓는다. 그 계약을 깨면 감사기를
고쳐야 하고, 감사기는 금지 목록에 있다.

그래서 `_fail(errors, where, message, kind=..., field=..., got=...)` 는
`errors` 가 `append_issue` 를 가진 때만 구조화한다. 평범한 list 는
예전처럼 `"<where>: <message>"` 만 받는다.

`IssueList` 는 list 의 하위형이다. `append` 계약이 살아 있고, 추가로
`structured` / `has_kind` / `of_kind` / `as_dicts` 를 준다.

### 객체가 아니면 즉시 접는다

`validate_pack` / `validate_task` / `validate_profile` 의 첫 분기는
`is_mapping`. 목록·문자열·`None` 은 `not-a-mapping` 한 칸이고 return
한다. 그 다음 `.get` 을 부르지 않는다.

`validate_task` 의 `pack` 이 객체가 아니면 빈 객체로 본다. 과제 하나를
검증하는데 pack 축만 못 읽는 자리와, 과제 자체가 깨진 자리를 가른다.

`checks` 항목·`requires`·`runner`·`submit` 도 같은 결이다. 항목 하나가
문자열이어도 다음 항목을 본다.

### bool tier

```python
isinstance(True, int)  # True
```

예전 검사 `isinstance(task.get("tier"), int) and 1 <= tier <= 5` 는
`True` 를 1 로, `False` 를 범위 밖으로 본다. `False` 는 우연히 거절되고
`True` 는 입문이 된다. JSON `true` 가 그대로 들어온다.

`is_valid_tier` 는 `isinstance(value, bool)` 을 먼저 거절한다. 메시지
조각은 예전 `MSG_TIER` 그대로다. 시험이 `True`/`False`/`"1"`/`1.0`/`None`
을 각각 남긴다.

### REGISTRY 는 읽기만

`registered_ops()` / `is_registered_op` / `op_needs_cli` /
`is_global_scan_op` 는 `from . import checks` 뒤에 등록부를 읽는다.
`schema.REGISTRY` 를 만들지 않는다. 시험
`RegistryIsolationTests` 가 `hasattr(schema, "REGISTRY")` 가 거짓인지,
`registered_ops()` 가 `frozenset(checks.REGISTRY)` 와 같은지, 미등록
연산자를 본 뒤에 등록부 키가 그대로인지 본다.

`CHECK_FIELD_HINTS` 는 devel 키만 나열한다. 기본 검증은 힌트를 강제하지
않는다. 5210 이 키를 더하면 힌트 표는 후속이 따라가면 된다. 이 브랜치가
등록부에 키를 더해 맞추지 않는다.

### 프로파일 없는 pack

예전: `for pid in profile.get("packs", []): if pid not in pack_ids`.
`packs` 가 `"core-cli"` 문자열이면 `c`,`o`,`r`,`e`,`-` … 각각이
`없는 pack 참조` 가 된다. 한 칸이어야 할 자리가 일곱 칸이 된다.

이제 목록이 아니면 `not-a-list` 한 칸이다. 목록인데 대상이 없으면
`profile-missing-pack` 이고 메시지는 `없는 pack 참조: <id>` 그대로다.

`pack_ids is None` 이면 존재 검사를 건너뛴다. 모양만 보는 호출자를 위해
남긴 구멍이다. `validate_gym_tree` 는 발견한 pack id 집합을 넘긴다.

## 네 자리 재현

이슈 본문이 적은 네 자리다. 시험 클래스 이름과 같다.

### 필수 키 없음 — `TaskMissingKeyTests`

```python
body = clone_minimal_task()
del body["input"]
issues = collect_task(body, pack)
assert issues.has_kind("missing-key")
assert "input" in issues.fields_of("missing-key")
assert any("필수 키 없음: input" in line for line in issues)
```

빈 객체는 일곱 키를 모두 남긴다. 키가 있는데 값이 `""` 이면 `missing-key`
가 아니라 `empty-field` 다. 이 가름이 예전에 없었다.

### 나쁜 tier — `TaskTierTests`

```python
for tier in (0, 6, True, False, "1", 1.0, None):
    assert collect_task(clone_minimal_task(tier=tier), pack).has_kind("bad-tier")
for tier in (1, 2, 3, 4, 5):
    assert list(collect_task(clone_minimal_task(tier=tier), pack)) == []
```

0 과 6 은 `test_gym_packs.TierRangeTests` 가 이미 본다. 이 파일은 그
메시지를 유지한 채 bool·문자·실수를 더한다.

### 미등록 연산자 — `TaskUnknownOpTests`

```python
issues = collect_task(clone_minimal_task(
    checks=[{"name": "x", "op": "not_an_op"}]), pack)
assert any("미등록 연산자: not_an_op" in line for line in issues)
assert "not_an_op" not in checks.REGISTRY
```

`op` 가 빠지면 `미등록 연산자: None` 이다. 빈 문자열도 같다. 등록부에
없는 이름을 넣어도 등록부는 그대로다.

### 프로파일 없는 pack — `ProfileMissingPackTests`

```python
issues = collect_profile(clone_minimal_profile(packs=["no-such-pack"]),
                         {"demo-pack"})
assert issues.has_kind("profile-missing-pack")
assert any("없는 pack 참조: no-such-pack" in line for line in issues)
```

`packs=["demo-pack"]` 은 통과한다. `packs=[]` 는 `packs 가 비었다`.
`packs="demo-pack"` 은 `not-a-list`.

## 추가로 막은 자리

네 자리만 막으면 같은 부류의 구멍이 남는다. 같은 결로 옆 칸을 메웠다.

- pack `requires` 가 목록이면 `malformed-requires`. 예전엔
  `list.get` 으로 죽었다.
- pack `runner` 가 문자열이면 `malformed-runner`.
- runner commit/sha 가 hex 가 아니면 `bad-runner-identity`.
  `test_gym_packs` 가 이미 길이를 본다. 스키마가 그 길이와 문자를 같이
  본다.
- `submit.kind` 가 `zip` 이면 `unknown-submit-kind`.
- `cmd` 가 문자열이면 `malformed-cmd`. 예전엔 `cmd[0]` 이 `"i"` 가 되어
  `CLI 에 없는 명령: i` 가 나왔다.
- 편집 축 + `deep_contains` 는 예전 메시지를 유지한 채
  `global-scan-forbidden` kind 를 붙인다.
- 한 과제 안 `check.name` 중복은 `duplicate-check-name`.
- id 의 `../` 는 `unsafe-id`.
- 깨진 JSON 파일은 `validate_gym_tree` 가 `malformed-object` 로 남기고
  다음 파일을 본다.

## capabilities 경로

`runner.py` 가 `known_commands(bin_path)` 와
`runner_identity(bin_path, ROOT)` 를 부른다. 바깥 반환 모양을 바꾸면
채점기가 깨지고, 채점기는 금지 목록에 있다.

그래서:

- `capabilities_digest` 는 계속 `(digest, raw)` 를 돌려준다. 빈
  `bin_path` 만 `ValueError` 로 거절한다.
- `known_commands` 는 계속 `set | None` 이다. 잡은 예외만
  `TypeError`·`UnicodeError`·`AttributeError` 로 넓혔다. 명령 항목이
  문자열이어서 `c["name"]` 이 죽던 자리가 None 이 된다.
- 빈 집합과 None 을 섞지 않는다. 파싱에 실패하면 None 이다. 명령을
  읽었는데 이름이 없으면 빈 집합이다. 후자는 정상 JSON `{"commands":[]}`
  의 결과로, 예전 집합 내포도 그렇게 동작한다.
- `try_known_commands` 는 바이너리 부재를 None 으로 접는 **새** 함수다.
  `known_commands` 를 바꾸지 않으려고 옆에 두었다.
- `parse_*` 도움말은 예외를 던지지 않는다. 시험이 깨진 UTF-8, 배열
  루트, 이름 없는 항목을 직접 넣는다.

`runner_identity` 의 버전 파싱이 죽지 않게 `TypeError` 등을 더 잡았다.
git 부재는 예전처럼 빈 커밋이다.

## 저장소 나무

강화가 기존 pack 을 깨면 이 PR 은 등재될 수 없다. 그래서 시험이

- 모든 `pack.json`
- 모든 `tasks/*.json`
- 모든 `profiles/*.json`
- `validate_gym_tree(gym/)`

을 다시 돌린다. 2026-08-18 devel 나무는 위반 0 이다. `audit.py` 도
같은 성공 칸을 본다. 감사기는 기준 풀이 짝을 추가로 보고, 스키마 나무는
선언만 본다.

힌트 표(`CHECK_FIELD_HINTS`)가 devel `REGISTRY` 키와 같은지도 본다.
키가 빠지면 시험을 고친다. 등록부에 키를 더해 맞추지 않는다.

## 시험 지도

`scripts/tests/test_gym_schema.py` 의 클래스와 이슈 본문의 대응.

| 클래스 | 자리 |
|---|---|
| `CatalogTests` | kind 카탈로그, 상수, 메시지 조각 |
| `HelperTests` | tier/id/hex/clone |
| `IssueListTests` | 평범한 list 와 IssueList |
| `PackMissingAndTypeTests` | pack 키·타입·runner |
| `TaskMissingKeyTests` | 필수 키 없음 |
| `TaskTierTests` | 나쁜 tier |
| `TaskUnknownOpTests` | 미등록 연산자 |
| `TaskCheckShapeTests` | checks 뼈대 |
| `TaskSubmitTests` | submit.kind |
| `TaskCmdTests` | cmd 유무·형태 |
| `TaskGlobalScanTests` | 편집 축 전역 훑기 |
| `ProfileMissingPackTests` | 없는 pack 참조 |
| `ExistingTreeTests` | 저장소 나무 성공 칸 |
| `CapabilitiesTests` | digest·known_commands |
| `RunnerIdentityTests` | runner_identity·git_head |
| `TreeWalkTests` | 임시 나무 전수 |
| `RegistryIsolationTests` | REGISTRY 를 건드리지 않음 |
| `MessageCompatibilityTests` | 예전 메시지 조각 |

`test_gym_packs.py` 는 고치지 않았다. 성공 칸의 소유권은 그 파일에
남겨 둔다. 이 파일은 예외 칸만 더한다.

## 검증

로컬에서 돌리는 것:

```bash
python -m unittest scripts.tests.test_gym_schema scripts.tests.test_gym_packs
python gym/tools/audit.py
```

Rust 표면은 바꾸지 않았다. `cargo fmt --all -- --check` 는 PR 전 하드
게이트로 한 번 돌린다. clippy/test 는 해당 없다.

`audit.py` 가 통과해야 한다. 스키마가 저장소 나무에 새 칸을 내면 감사기
exit 1 이다. 그 경우 스키마를 느슨하게 되돌리지 않고, 나무가 이미
어긋난 것인지를 먼저 본다. 2026-08-18 devel 나무는 깨끗했다.

## 크기와 범위

이슈 DoD 는 additions >= 3000 이다. 이 브랜치의 삽입은

- `gym/core/schema.py` 고도화 (카탈로그·IssueList·타입 가드·나무 전수)
- `scripts/tests/test_gym_schema.py` 예외 칸
- `gym/docs/schema.md` 정본 규약
- `mydocs/working/gym_schema.md` 이 기록

네 파일에만 모인다. 금지 파일에 공백을 넣어 숫자를 채우지 않는다.

## 후속

- PR 5210 이 등록부에 키를 더하면 `CHECK_FIELD_HINTS` 를 같은 키로
  따라가면 된다. 기본 검증은 계속 힌트를 강제하지 않는 편이 안전하다.
- PR 5278 이 채점기 예외 kind 를 남기면, 스키마 kind 와 이름을 맞출지
  한 번 보면 좋다. 지금은 두 카탈로그가 다른 층이다(선언 vs 실행).
- 프로파일 `schemaVersion` 을 필수로 올릴지는 후속. 지금은 키가 있을
  때만 1.0 을 강제한다.
- `validate_gym_tree` 를 감사기에 붙이는 것도 후속. 이 이슈는 감사기를
  고치지 않는다.

## 모르는 키는 거절하지 않는다

pack 의 `description`, 과제의 `notes`·자체 `axis`, 프로파일의 설명
문자열은 스키마가 보지 않는다. 후속이 필드를 더해도 이 모듈을 고칠
필요가 없게 하려는 결이다. 필수 키만 강제하고, 나머지는 채점기나
문서가 소비한다.

hex 는 대소문자를 가리지 않는다. pack 선언의 commit/sha 가 대문자로
들어와도 통과한다. `is_commit_hex` / `is_sha256_hex` 가 그 자리를 본다.

`validate_gym_tree(..., known_commands={"info"})` 는 임시 나무에서
명령 표면을 넘기는 시험 경로다. 저장소 전수는 `None` 으로 돌려
바이너리 없이 선언만 본다.

## 관련

- 이슈: [#5279](https://github.com/edwardkim/rhwp/issues/5279)
- 스키마 원점: [#4653](https://github.com/edwardkim/rhwp/issues/4653)
- 전역 훑기 금지: [#4600](https://github.com/edwardkim/rhwp/issues/4600)
- 티어 1~5: [#4664](https://github.com/edwardkim/rhwp/issues/4664)
- 지목 연산자 PR: #5210 (파일 미수정)
- 채점기 예외 PR: #5278 (파일 미수정)
