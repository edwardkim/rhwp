# 00 — 서식 채움 판단 트리

이 장은 에이전트가 **어느 명령을 먼저 칠지**만 고른다. 사다리는 강제 순회가
아니다. 질문이 이미 답이면 멈춘다.

gym 경로가 아니다. 새 CLI 도, 새 fill 구현도 없다. 아래 상자는
`mydocs/manual/cli_commands.md` 와 레시피 01·05 가 이미 고정한 명령이다.

```
fields <서식> --json
  │
  ├─ exit 1 ── 파일 없음·파싱 실패
  │              원본 불변. 중단 (F01)
  │
  ├─ fieldCount == 0
  │              누름틀 서식이 아니다.
  │              export-tables 로 빈 칸이 있으면 rhwp-table-exchange (F02)
  │              빈 칸도 없으면 "이 스킬의 대상이 아니다" 고지
  │
  ├─ textSecurity.status 가 있고 "clean" 이 아님
  │              값을 넣기 전에 rhwp-security-sweep / 레시피 04 (F03)
  │
  └─ fieldCount >= 1
       │
       ├─ 질문 = "뭘 채워야 해 / 칸 목록"
       │        names · guide · memo · 반복 횟수만 보고 정지 (F04)
       │
       ├─ 단건 값
       │    │
       │    ├─ 같은 이름이 2회 이상
       │    │     fields 목록 순서가 곧 0 기준 순번
       │    │     --data 키를 이름[N] 으로 (F05)
       │    │     순번 없는 키는 첫 매치 + ambiguous
       │    │
       │    ├─ --dry-run --json   (파일을 쓰지 않음)
       │    │     notFound 잔류 → 오타, name 을 그대로 복사 (F06)
       │    │     ambiguous 잔류 → 이름[N] 재지목 (F05)
       │    │
       │    ├─ edit fill-fields --data -o --verify --json
       │    │     통과: identical && notFound==[] && ambiguous==[] (F07)
       │    │     identical false → exit 3, 산출은 남음 (F12)
       │    │
       │    └─ 제출 요청
       │          [선택] insert-image (직인)
       │          edit sanitize -o
       │          두 번째 sanitize 의 removedCount 는 0 (F08)
       │
       └─ 명단 N행
            │
            ├─ --data 가 .jsonl 또는 .csv 인가
            │     헤더 = fields[].name (또는 이름[N])
            │     UTF-8. CP949 는 exit 1
            │     행 0개 → exit 2 (F09)
            │
            ├─ batch fill --form --data --out-dir --dry-run --json
            │     stdin 파일 목록을 넣지 않는다 (P03)
            │     --out-dir 는 dry-run 에도 필수
            │
            └─ batch fill --verify [--name-field]
                  행별 NDJSON 으로 게이트 (F10)
                  name-field 컬럼의 notFound 는 오탐 (F11)
```

## 축을 고르는 한 줄

| 관찰 | 축 |
| --- | --- |
| `fields` fieldCount ≥ 1 | 이 스킬 (`fill-fields` / `batch fill`) |
| fieldCount 0 + 표 빈 칸 | `edit set-cell` (rhwp-table-exchange) |
| 둘 다 아니고 문구만 바꿈 | `edit replace-text` (rhwp-safe-edit) |
| 문서가 뭔지만 | rhwp-doc-triage (읽기) |

실물 서식은 축이 섞인다. 머리 표는 누름틀, 본문 표는 맨 셀인 식이다.
축별로 나눠 처리하고 마지막에 한 번에 검증한다. 이 스킬은 누름틀 축만
끝까지 책임진다.

## 명령 상자 (발명 금지)

살아 있는 동사는 이 일곱이다.

1. `fields`
2. `edit fill-fields`
3. `batch fill`
4. `--dry-run` (위 쓰기 명령의 플래그)
5. `--verify` (위 쓰기 명령의 플래그)
6. `edit insert-image`
7. `edit sanitize`

없는 것: 메일머지 전용 하위명령, 일괄 채움 별칭, N번째 전용 동사,
세션 메일머지 도구, gym pack runner. 오타 난 하위명령은 exit 2.

코어 재사용:

- 조회 = 기존 `collect_all_fields()`
- 채움 = 기존 `set_field_value_by_name`
- 메일머지 = 행마다 그 fill-fields 경로를 다시 부름
- 레이아웃·문단·표 구조를 이 스킬이 바꾸지 않는다

## 원본 불변

`-o` 로 산출을 분리한다. 실패(exit 1)와 `--dry-run` 은 출력 파일을 만들지
않는다. 원본을 `--in-place` 로 덮는 습관은 이 스킬에서 금지.

`batch fill` 의 `--out-dir` 도 같다. 서식 파일은 행마다 다시 열릴 뿐
덮어쓰지 않는다.

## 에이전트가 하지 말 것

- 필드 이름을 한글 동의어로 바꿔 넣기 (성명 ≠ 이름)
- 순번을 1부터 세기
- `filledCount` 만 보고 제출
- stderr 요약 줄만 보고 batch 성공 판정
- 머리말 누름틀이 안 보인다고 재귀 코드를 추가
- gym/ 아래에 과제를 만들기
