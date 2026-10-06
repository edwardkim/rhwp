---
kind: working
status: active
canonical: gym/docs/pack_health.md
last_verified: 2026-08-18
---

# gym pack 건강 감사 — 작업 기록 (#5215)

정본 목록은 [`gym/docs/pack_health.md`](../../../gym/docs/pack_health.md) 다. 이
문서는 왜 그 층이 생겼는지, 1차와 2차가 무엇을 갈랐는지, 어떤 오탐을 거절했는지,
단위시험이 무엇을 재현하는지를 남긴다. pack 과제 JSON 은 여기서 바꾸지 않는다.

## 한 줄 결론

`audit.py` 는 스키마·기준풀이 짝·전역 ID 만 본다. 지시문이 비거나 힌트가 답을
적어도 통과한다. #5215 는 `gym/tools/pack_health.py` 를 추가해 그 다음 층을
같은 저장소 안에서 관측한다. 기본 종료는 0 이고 `--strict` 만 게이트다.
기존 pack 은 고치지 않는다. 새 규칙은 픽스처로 고정한다.

## 배경

gym 4부(#4653)가 pack 을 `packs/<id>/{pack.json,tasks,reference}` 로 쪼갰다.
`audit.py` 는 "등재될 자격"(짝이 있나, id 가 겹치나, 연산자가 등록됐나)을
강제한다. 그런데 등재 자격과 풀이 품질은 다르다.

- 지시문이 `"세어라"` 네 글자여도 스키마는 통과한다.
- `checks` 두 줄의 `name` 이 같아도 채점기는 인덱스로 돈다. 리포트는 사람을
  속인다.
- 힌트에 `{"pages": 4}` 를 박아도 audit 은 파일을 읽지 않는다.
- `submit.kind: "folder"` 는 채점기가 제출을 증발시키지만 등재는 된다.

운동장이 자랄수록 이 구멍은 pack 수만큼 늘어난다. pack 건강은 그 구멍을
바이너리 없이 막는다.

## 1차와 2차

1차는 지시·이름·힌트·제출 kind 만 봤다. 현재 트리 18 pack · 112 과제가 이슈 0
인 것을 확인한 뒤, 같은 봉투에 2차 계약을 올렸다. 2차가 보는 것:

| 층 | 이유 | 현재 트리가 이미 지키는가 |
|---|---|---|
| pack.json 신원 | audit 과 같은 최소 선언을 코드로 | 예 |
| tier 1~5, bool 거절 | 스키마와 같음. `True` 를 1 로 보지 않음 | 예 |
| input 경로 위생 | 절대 경로·백슬래시·`..` 는 재현 불가 | 예 (전부 상대 POSIX) |
| 연산자 필수 필드 | `answer_eq` 에 `answer` 없음은 채점 불능 | 예 |
| CLI `cmd` / 파일 `file` | 스키마 `needs_cli` 와 같음 | 예 |
| 편집 축 전역 훑기 | #4600 재발 | 예 (현재 트리에 전역 훑기 없음) |
| 제출 파일 경로 | 채점기가 상대 경로로 연다 | 예 |
| 힌트 골격 | 본문 없는 꼬리, TODO 자리표 | 예 |
| reference.id / 빈 run | audit 문자열을 코드로 | 예 |
| 고아 reference | audit 과 같음 | 예 |

"이미 지키는 계약만 승격한다" 가 2차의 원칙이다. 규칙을 넓혀 현재 트리가
실패하면, 먼저 오탐인지 실제 구멍인지 가른다. 오탐이면 규칙을 좁히고 pack
문구를 다시 쓰지 않는다.

## 오탐으로 거절한 것

### T06 괄호 힌트

`core-cli/T06` 본문은 이렇게 적혀 있다.

```text
… 문구 하나를 스스로 찾아(힌트: export-text) 그 문구를 '검증완료' 로 …
힌트: rhwp edit replace-text.
```

문장 안 `(힌트: export-text)` 는 "어느 명령으로 찾을지" 안내이지 꼬리 힌트가
아니다. 첫 구현은 `str.find("힌트:")` 로 마커를 세어 이 과제를
`duplicate_hint_marker` 경고로 만들었다. 경고도 `issueCount` 에 들어가
`ok=false` 가 된다.

처리는 pack 을 고치는 것이 아니라 `split_hint` / `count_hint_markers` 가
괄호 안 마커를 꼬리로 치지 않게 한 것이다. 문장 끝(`.` / 줄바꿈 / 맨 앞)만
꼬리다. 픽스처
`test_parenthetical_hint_is_not_duplicate_tail` 가 T06 문형을 재현한다.

### 자리표 JSON 과 CLI 접미

fill-fields 예시는 `{"<필드이름>": "홍길동"}` 이다. 키가 자리표면 정답 봉투가
아니다. `export-hwpx` / `conv.hwpx` 안의 `hwpx` 는 형식 토큰이지
`value_eq: "hwpx"` 의 복붙이 아니다. 1차에서 이미 거절한 오탐이고 2차도
유지한다.

### 0/1 과 흔한 토큰

쪽수·건수 힌트에 `1` 이 흔하다. `value` 가 0/1/-1 이면 복붙으로 치지 않는다.
`json`, `hwp`, `true` 같은 짧은 토큰도 같다.

### bool 좌표

`cell_text_eq` 의 `table=True` 는 Python 에서 `int` 하위형이라 `>= 0` 검사를
통과한다. 좌표로 쓰면 1번 표가 아니라 "참" 이 된다. `is_nonneg_int` 가
`bool` 을 거절한다.

## 일부러 보지 않는 것

- **기준풀이 부재.** audit 가 이미 짝을 강제한다. 건강 도구가 없으면 침묵한다.
  자기시험에서 기준 없이 도구만 돌릴 수 있게 하려는 것이다.
- **전역 과제 ID 충돌.** audit 의 `taskIdCollisions` 몫이다. 건강 도구는 pack
  안 중복만 본다.
- **바이너리 명령 존재.** `cmd[0]` 이 그 바이너리에 있는지는 러너 몫이다.
- **채점 성공.** 건강은 파일이 채점 **가능한 모양** 인지만 본다. 왕복은
  `build_baseline.py` + `score.py` 다.
- **지시문이 과제를 풀 수 있을 만큼 구체적인가.** 너무 주관적이다. 길이·자리표·
  스포일러만 본다.
- **pack 과제를 자동으로 고침.** 도구는 리포트만 낸다.

## 종료 코드를 가른 이유

현재 트리가 항상 깨끗하다고 가정할 수 없다. 다른 브랜치의 pack 이 섞이거나,
작성 중인 픽스처를 같은 트리에서 돌릴 수 있다. 그래서:

- 기본: 관측. 이슈가 있어도 0. CI 자기시험이 도구 **자체** 를 깨지 않는다.
- `--strict`: 품질 관문. 머지 전 게이트나 로컬 훅이 쓴다.
- `scanError`: packs/ 자체가 없으면 관측도 실패다. 기본도 1.

`unittest` 는 픽스처에 이슈를 심고 `exit_status(..., strict=False) == 0` 과
`strict=True == 1` 을 같이 고정한다.

## 시험 지도

파일은 `scripts/tests/test_gym_pack_health.py` 다. 실제 `gym/packs` 를 고쳐
실패를 만들지 않는다. 임시 디렉터리에 pack 하나를 심고 코드를 확인한다.

| 클래스 | 고정하는 계약 |
|---|---|
| `EnvelopeTests` | kind/schema, 기본 종료 0 |
| `InstructionTests` | 빈/짧은/타입, `--min-instructions` |
| `CheckNameTests` | 이름 없음·빈 값·중복, 과제 사이 동명 허용 |
| `IdentityTests` | id/title 공백, 파일명 불일치 |
| `ReferenceStepTests` | 부재는 침묵, 빈 steps, 빈 객체 |
| `SubmitKindTests` | 미지 kind, 세 허용 값, artifact 경고 |
| `HintHealthTests` | 스포일러·JSON 덤프·자리표·본문 반복 |
| `StructureTests` | pack.json 없음, 파싱 실패, pack 필터 |
| `RenderAndCliTests` | 사람 리포트, `--json`, `--strict` |
| `RealRepoTests` | 현재 트리 스캔이 예외 없이 봉투를 냄 |
| `UtilityTests` | 길이·공백·자리표·bare token |
| `ManifestHealthTests` | kind/schema/id/title/axis/requires/runner |
| `InputAndTierTests` | tier 범위, input 경로 위생, min-title |
| `CheckContractTests` | op·cmd·file·좌표·전역 훑기 |
| `SubmitPathTests` | files 타입·공백·절대 경로·중복 |
| `InstructionQualityTests` | 힌트만, 빈 꼬리, 중복 꼬리, TODO, 제어 문자, 괄호 힌트 |
| `ReferenceDetailTests` | id 불일치, 빈 run/answer/cmd, 고아, 빈 pack |
| `CatalogAndCliExtraTests` | `--codes`, `--exclude`, 카탈로그 완전성 |
| `RealRepoHealthGateTests` | 현재 트리 이슈 0 |

`test_current_tree_stays_clean` 이 2차의 안전장치다. 새 규칙이 실제 pack 을
실패로 뒤집으면 이 시험이 먼저 붉어진다. 그때 pack 을 고치기 전에 오탐부터
의심한다.

## 구현 메모

- 도구는 `gym/core` 를 **선택적으로** 읽는다. `REGISTRY` 를 가져오면 미지
  `op` 판정이 코어와 같다. 가져오지 못하면 내장 `FALLBACK_*` 로 후퇴한다.
  단위시험 `test_known_ops_include_registry` 가 현재 트리에서 등록부가
  보이는지 확인한다.
- `ISSUE_CATALOG` 는 `--codes` 와 문서 표의 단일 출처다.
  `test_catalog_covers_all_code_constants` 가 `CODE_*` 와 카탈로그를 묶는다.
- `scan_pack` 은 manifest → tasks → orphan reference 순이다. pack.json 이
  없으면 과제를 읽지 않는다.
- 경고도 `issues[]` 에 들어간다. `ok` 는 경고가 있어도 거짓이다. 경고만
  무시하려면 `--exclude`.

## 현재 트리 실측

이 기록을 남기는 시점의 로컬 스캔:

```text
python gym/tools/pack_health.py
# gym pack 건강: 18 pack · 112 과제 — 이슈 0

python gym/tools/audit.py
# gym 정합 감사: 18 pack 전부 통과 — 위반 0
```

숫자(18/112)는 브랜치가 자라는 대로 변한다. 시험은 `>= 10 pack`, `>= 40 과제`,
이슈 0 만 고정한다.

## 관련 이슈·문서

- #5215 — pack 건강 감사 도구
- #4653 — gym 4부 pack 구조
- #4600 — 편집 과제 전역 훑기 금지
- [`gym/docs/pack_health.md`](../../../gym/docs/pack_health.md) — 코드 표 정본
- [`gym/tools/audit.py`](../../../gym/tools/audit.py) — 정합 층
- [`gym/core/schema.py`](../../../gym/core/schema.py) — 스키마
