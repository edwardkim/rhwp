---
kind: investigation
status: active
canonical: gym/docs/audit.md
last_verified: 2026-08-18
---

# gym 정합 감사 — 예외 경로·문서·시험 고도화

Issue: #5277
Branch: `feat/gym-audit-hardening`
Date: 2026-08-18

## 1. 결론

`gym/tools/audit.py` 의 전 pack 정합 감사를 예외 경로까지 닫았다.
빠진 `pack.json`, 고아 기준풀이, 전역·pack 안 과제 ID 충돌, 나쁜
스키마를 코드로 접고, 없는 packs 루트·깨진 JSON·객체가 아닌 JSON
에서도 도구가 죽지 않게 했다.

원 계약은 유지한다.

- 보고 `kind=gymAudit`, `schemaVersion=1.0`
- `packs[].issues` 는 한글 문자열 목록
- `taskIdCollisions` 는 `{id: [pack, …]}`
- `issueCount = pack 이슈 줄 + 전역 충돌 수`
- CLI 는 `--json` 만
- `schema.validate_pack` / `validate_task` 를 다시 구현하지 않고
  감싼다
- 치명 예외는 삼키지 않는다

새 CLI/pack 없음. `certify.py` · `report.py` · `score.py` ·
`runner.py` · `build_baseline.py` · tutorial · expert-challenges ·
PR 5210–5276 이 연 파일은 건드리지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_audit`
- `python gym/tools/audit.py`
- `cargo fmt --all -- --check` (하드 게이트, PR 전)

## 2. 배경

원 도구(#4803)는 전 pack 을 훑어 네 가지를 본다. 스키마, 과제↔기준
짝, 고아 기준풀이, 전역 ID 충돌. CI 가 `test_gym_audit.py` 다섯
자리로 고정했다.

대비 `upstream/devel` 의 구현은 약 155줄, 시험은 5건이었다.

그 상태의 빈틈:

1. `os.listdir(packs_dir)` 가 없는 루트에서 예외를 그대로 올린다.
   시험 픽스처가 아닌 자리에서 도구가 죽으면 CI 가 붉어져 정합
   위반과 하네스 결함을 구분할 수 없다.
2. `pack.json` / 과제 / 기준풀이가 배열이면 `obj.get` 이
   `AttributeError` 로 감사기를 죽인다. "정합 위반"이 아니라
   "도구 크래시"다.
3. `UnicodeDecodeError` 는 `ValueError` 하위라 파싱 실패로 접히지만,
   문맥(매니페스트·과제·기준풀이)이 한 문구로 섞인다.
4. 같은 pack 의 두 파일이 같은 `id` 를 쓰면 전역 충돌 맵에
   `["p1", "p1"]` 로 들어간다. 집계가 pack 을 가른다는 가정과
   어긋난다.
5. `X01.json` 이 `id=Y99` 를 들고 있어도 통과한다. 파일 이름으로
   짝짓는 쪽과 id 로 집계하는 쪽이 갈린다.
6. 카탈로그가 코드에만 있어 문서·시험이 같은 표를 공유하지 않는다.
7. JSON 봉투에 구조화 코드가 없다. `ok=false` 가 "고아"인지
   "스키마"인지 "루트 부재"인지 기계가 못 가른다.
8. 빈 pack(과제 0)은 원 구현에서 통과한다. 해결 가능성 선언이
   없는데 정합이라고 부른다.

이슈 #5277 의 DoD 는 이 빈틈을 닫는 것이다. additions >= 3000.
unittest + audit.py. 새 CLI/pack 없음. PR 전
`cargo fmt --all -- --check`.

다른 고도화 PR(differential / discriminate / fuzz_corpus /
release_gate)의 예외 접기 결을 따른다. 감사기의 정체성은 유지했다.

- 전 pack 을 한 번에 본다. `--pack` 을 열지 않는다.
- 바이너리를 부르지 않는다. `known_commands=None`.
- 레거시 한글 줄을 깨지 않는다.
- 스키마 규칙을 여기 복제하지 않는다.

## 3. 한 일

### 3.1 도구

`gym/tools/audit.py`

- `REPORT_KIND` / `SCHEMA_VERSION` / `ISSUE_CODES` / `ISSUE_FAMILY`
  / `ISSUE_TEXT` / `REPORT_KEYS` / `EXIT_*` / `FATAL_EXCEPTIONS`.
- `is_fatal_exception` · `truncate_head` · `exception_kind` ·
  `exception_record`.
- `make_issue` · `pack_issue_line` · `append_issue` — 한글 줄과
  구조화 줄을 동시에 남긴다.
- `list_dir_safe` · `load_json_safe` · `load_object` — 예외를
  올리지 않는다.
- `pair_names` · `detect_in_pack_duplicates` ·
  `detect_global_collisions` — 순수.
- `run_validate_pack` / `run_validate_task` — schema 예외를
  메시지로 접는다. 치명 예외는 다시 올린다.
- `classify_schema_message` — 한글 스키마 문장 → `schemaTag`.
- `audit_one_pack` — pack 하나. 매니페스트가 없으면 과제 스캔을
  하지 않는다.
- `audit` — 루트 부재는 `toolFailed` + exit 2. 빈 packs 는 원
  계약대로 위반 0.
- `empty_report` · `validate_report` · `format_human_report` ·
  `format_json_report` · `resolve_exit`.
- `parse_args` / `main` — `--json` 만.
- `_load` 는 원 날것 로더로 남긴다.

### 3.2 시험

`scripts/tests/test_gym_audit.py`

원 다섯 자리를 그대로 둔다.

- `test_real_repo_all_packs_conform`
- `test_missing_reference_is_flagged`
- `test_orphan_reference_is_flagged`
- `test_task_id_collision_across_packs_is_flagged`
- `test_clean_fixture_passes`

뒤에 카탈로그·예외 kind·순수 헬퍼·빠진 pack.json·짝짓기·중복
ID·나쁜 스키마(매니페스트·tier·미등록 연산자·전역 훑기·cmd
유무)·파싱/비객체/비UTF-8·루트 부재·보고 봉투·CLI·치명 예외·
네 자리 동시 픽스처를 더한다.

### 3.3 문서

- `gym/docs/audit.md` — 규약 정본. 카탈로그·봉투·예외·하지 않는
  것·네 자리 픽스처.
- `mydocs/working/gym_audit.md` — 이 기록.

## 4. 검사 순서 (구현이 지키는 것)

```
audit(packs_root)
  ├─ packs/ 없음 → missing-packs-root, exit 2
  ├─ packs/ 파일 → packs-not-dir, exit 2
  ├─ listdir 실패 → unlistable-packs, exit 2
  ├─ 폴더 0 → ok, packCount 0, exit 0
  └─ 각 pack 폴더
       ├─ pack.json 없음 → missing-pack-json, 다음 pack
       ├─ 파싱/비객체/읽기 실패 → 해당 코드, 다음 pack
       ├─ validate_pack → bad-schema*
       ├─ tasks/reference 나열
       ├─ 각 tasks/*.json
       │    ├─ 파싱/비객체 → task-*, 짝 없으면 missing-reference
       │    ├─ validate_task → bad-schema*
       │    ├─ id 공백 → task-empty-id
       │    ├─ id ≠ stem → task-filename-id-mismatch
       │    ├─ reference/name 없음 → missing-reference
       │    └─ ref.id ≠ task.id → reference-id-mismatch
       ├─ reference 에만 있는 이름 → orphan-reference
       ├─ pack 안 같은 id 여러 파일 → task-id-duplicate-in-pack
       └─ 나열 성공 · 과제 0 → empty-pack
  └─ 서로 다른 pack 의 같은 id → taskIdCollisions + task-id-collision
```

## 5. 예외 접기 표

문맥이 코드를 가른다. 같은 `FileNotFoundError` 라도 자리가 다르다.

| 예외 | packs-root | pack-json | task | reference | listdir-tasks |
|---|---|---|---|---|---|
| FileNotFoundError | missing-packs-root | missing-pack-json | (기본 루트) | (기본 루트) | missing-tasks-dir |
| NotADirectoryError | packs-not-dir | — | — | — | tasks-not-dir |
| PermissionError | unlistable-packs | pack-json-unreadable | task-unreadable | reference-unreadable | unlistable-tasks |
| JSONDecodeError | — | pack-json-parse | task-parse | reference-parse | — |
| UnicodeError | — | pack-json-unreadable | task-unreadable | reference-unreadable | — |
| TypeError | unexpected | pack-json-not-object | task-not-object | reference-not-object | — |

`load_object` 가 배열·숫자를 받으면 예외 없이 `*-not-object` 를
만든다. `task.get` 크래시 경로를 이 검사가 없앤다.

## 6. 보고 계약

원 키가 빠지면 레거시 시험과 CI 가 깨진다. 추가 키는 옵트인이다.

유지:

```
kind, schemaVersion, ok, packCount, packs, taskIdCollisions, issueCount
```

추가:

```
taskCount, referenceCount, okPacks, emptyPacks, issues,
issueCountsByCode, issueCountsByFamily, toolErrors,
missingPacksRoot, toolFailed, exit
```

`ok` 공식:

```
ok = (issueCount == 0) and (not toolFailed)
```

`exit` 공식:

```
toolFailed or missingPacksRoot → 2
ok → 0
그 외 → 1
```

전역 충돌의 `owners` 는 처음 본 pack 순, 중복 제거. 같은 pack 이
두 파일에 같은 id 를 써도 `taskIdCollisions` 에는 넣지 않는다.

## 7. 레거시 한글 줄

바꾸면 안 되는 부분 문자열.

| 코드 | 부분 문자열 |
|---|---|
| missing-pack-json | `pack.json 이 없다` |
| pack-json-parse | `pack.json 파싱 실패` |
| missing-reference | `기준풀이` |
| orphan-reference | `고아` |
| reference-id-mismatch | `의 id(` … `가 과제 id(` |
| task-parse | `tasks/{name} 파싱 실패` |
| reference-parse | `reference/{name} 파싱 실패` |

사람 요약의 통과/위반 머리도 유지한다.

```
gym 정합 감사: {n} pack 전부 통과 — 위반 0
gym 정합 감사: 위반 {n}건
  [{pack}] {issue}
  [전역] 과제 ID '{id}' 충돌: {owners}
```

도구 실패 머리만 새로 생겼다.

```
gym 정합 감사: 도구 실패 — {detail}
```

## 8. 건드리지 않은 것

- `gym/certify.py`
- `gym/report.py`
- `gym/score.py`
- `gym/core/runner.py`
- `gym/tools/build_baseline.py`
- `gym/tutorial/**`
- `gym/packs/expert-challenges/**`
- `gym/core/schema.py` · `gym/core/checks.py`
- PR 5210–5276 이 연 파일 (다른 gym 도구·pack 확장·docs)
- 새 CLI 플래그
- 새 pack · 새 과제
- 워크플로 YAML (`ci.yml` 의 unittest 목록은 이미
  `test_gym_audit.py` 를 포함한다)

## 9. 시험 지도

| 클래스 | 고정하는 것 |
|---|---|
| `AuditTests` | 원 다섯 자리 |
| `CatalogContractTests` | 코드·가족·문구·CLI·봉투 키 |
| `ExceptionKindTests` | 문맥별 접기 · 치명 표시 |
| `PureHelperTests` | 이름·짝짓기·충돌·schemaTag · validate_report |
| `MissingPackJsonTests` | 폴더만 있는 pack, 매니페스트가 디렉터리 |
| `OrphanAndPairingTests` | 고아 · 빠진 기준 · id 불일치 · 파일명 짝 |
| `DuplicateIdTests` | 전역 충돌 · pack 안 중복 · 순서 |
| `BadSchemaTests` | kind/version/id/title/requires/runner/tier/op/cmd |
| `ParseAndShapeTests` | 깨진 JSON · 배열 · 비UTF-8 · 비상승 |
| `RootExceptionTests` | 루트 부재 · packs 파일 · 빈 루트 · tasks 파일 |
| `ReportShapeTests` | 봉투 · 사람/JSON 요약 · 실제 저장소 정합 |
| `CliTests` | main exit 0/1/2 · 모르는 플래그 |
| `FatalAndFoldTests` | KeyboardInterrupt/SystemExit/MemoryError/GeneratorExit |
| `MixedPackTests` | 한 루트에 네 자리 혼재 |
| `DoDCoverageTests` | 이슈가 명한 네 자리 동시 |
| `RealRepoHonestyTests` | 실제 gym/packs 가 비어 있지 않음 |

## 10. 실제 저장소에서 기대한 값

devel 기준(이 브랜치가 갈라진 시점):

- pack 18개 전후 (casual-rides … text-editing, showcase 포함)
- 과제 수 = 기준풀이 수
- 전역 충돌 0
- 빈 pack 0
- `python gym/tools/audit.py` exit 0

숫자가 조금 달라도 `packCount >= 10` · `ok is True` 면 계약은
유지된다. pack 을 이 PR 에서 늘리지 않으므로 숫자는 devel 과
같다.

## 11. 설계 선택

### 11.1 왜 `--pack` 을 안 열었나

전역 ID 충돌은 전 pack 을 봐야 한다. 한 pack 만 통과했다고 전
저장소가 정합이라고 쓰면 거짓말이다. 이슈도 "새 CLI 없음"을
명시했다.

### 11.2 왜 스키마를 복제하지 않았나

`schema.py` 는 PR 5210 이 `checks.py` 를 건드리는 자리와 붙어
있다. 규칙을 여기 복사하면 연산자가 늘 때 감사기가 거짓
음성을 낸다. 감싸서 메시지를 접는 쪽이 정직하다.

### 11.3 왜 같은 pack 중복을 전역 충돌에서 뺐나

리더보드가 pack + id 로 행을 가르면 같은 pack 의 두 파일은
전역 충돌이 아니라 pack 내부 결함이다. `["p1", "p1"]` 을
전역 맵에 넣으면 집계 쪽이 "두 pack"으로 오해한다. pack 안
중복은 `task-id-duplicate-in-pack` 으로만 남긴다.

### 11.4 왜 빈 packs 루트는 통과인가

원 계약이 `packCount=0, issueCount=0, ok=true` 였다. 시험이
임시 디렉터리에 `packs/` 만 만들고 끝나는 경로를 도구 실패로
바꾸면 의미가 다르다. 실제 저장소는 `packCount >= 10` 으로
따로 막는다.

### 11.5 왜 empty-pack 을 추가했나

과제 0 인 pack 은 해결 가능성을 선언할 수 없다. 원 구현은
이를 통과시켰다. 나열에 성공한 뒤에만 이 코드를 남긴다.
나열 실패를 빈 pack 으로 부르지 않는다.

### 11.6 왜 파일명으로 짝을 짓나

원 계약이 `tasks/X.json ↔ reference/X.json` 이다. id 로
바꾸면 `Y99.json` + `id=Y99` 인 파일을 고아/미선언으로
오인한다. 파일명 불일치는 별도 코드다.

## 12. 위험과 비위험

위험하지 않은 것:

- 실제 저장소 pack 이 이미 정합이면 동작 변화 없음 (exit 0).
- 레거시 시험 다섯 자리는 같은 주장을 한다.
- 바이너리·네트워크를 부르지 않는다.

주의할 것:

- 빈 pack 은 이제 위반이다. devel 의 실제 pack 은 과제가
  있으므로 영향 없다. 누군가 과제 없이 폴더만 올리면 CI 가
  막는다. 그것이 이 이슈의 점이다.
- pack 안 중복 ID 는 이제 명시적으로 막힌다. 예전에는 전역
  맵에 `["p1","p1"]` 로만 보였다.
- 파일명≠id 는 이제 위반이다. devel 의 실제 과제는 파일명과
  id 가 같다.

## 13. 검증 명령

작업 트리 `C:\Users\swsz9\rhwp-gym-audit` 에서:

```bash
python -m unittest scripts.tests.test_gym_audit
python gym/tools/audit.py
python gym/tools/audit.py --json
cargo fmt --all -- --check
```

unittest 가 red 면 PR 을 열지 않는다. audit.py 가 실제 gym 에서
exit 0 이 아니면 PR 을 열지 않는다. fmt 하드 게이트를 건너뛰지
않는다. 이 변경은 Python·문서만이라 rustfmt 대상 `.rs` 는 없다.

## 14. 크기 게이트

이슈 DoD: additions >= 3000 vs `upstream/devel`.

의도한 증가 자리:

| 파일 | 역할 |
|---|---|
| `gym/tools/audit.py` | 카탈로그·예외 접기·구조화 보고 |
| `scripts/tests/test_gym_audit.py` | 원 계약 + 예외·스키마·CLI |
| `gym/docs/audit.md` | 규약 정본 |
| `mydocs/working/gym_audit.md` | 이 기록 |

다른 파일을 부풀려 숫자를 채우지 않는다.

## 15. 후속 (이 PR 밖)

- pack 건강 감사(`pack_health.py`)와의 중복 메시지 정리. 그
  도구는 지시·힌트 축이고 이 도구는 정합 축이다. 합치지 않는
  편이 맞다.
- profile 이 가리키는 pack id 존재 여부. `schema.validate_profile`
  이 이미 있다. 감사기에 넣으면 CLI 없이 할 수 있으나 이슈
  범위(스키마·기준 짝·ID 충돌) 밖이다.
- `reference` 의 `steps` 스키마. 채점기가 읽는 자리라 여기
  복제하지 않았다.

## 16. 커밋 범위 체크리스트

- [x] isolation worktree (`rhwp-gym-audit`, `rhwp-desk*` 아님)
- [x] 브랜치 `feat/gym-audit-hardening` ← `upstream/devel`
- [x] `gym/tools/audit.py` 고도화
- [x] `scripts/tests/test_gym_audit.py` 확대
- [x] `gym/docs/audit.md` 추가
- [x] `mydocs/working/gym_audit.md` 추가
- [x] 새 CLI/pack 없음
- [x] 금지 파일 미수정
- [ ] unittest 통과
- [ ] `python gym/tools/audit.py` 통과
- [ ] additions >= 3000
- [ ] `cargo fmt --all -- --check`
- [ ] 한국어 PR, `closes #5277`, `--body-file`

## 17. 관련

- 원 감사: #4803
- 이 고도화: #5277
- 스키마: `gym/core/schema.py` (#4653)
- 비슷한 결의 고도화: #5256 fuzz_corpus, #5259 release_gate,
  differential / discriminate 순수 시험
