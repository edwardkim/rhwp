---
kind: investigation
status: active
canonical: gym/packs/text-editing/README.md
last_verified: 2026-08-18
---

# text-editing pack 확장 작업 노트

이 문서는 PR #5242 (`feat/gym-text-editing-expand`) 를 키운 작업 기록이다.
규범 문서는 [gym/packs/text-editing/README.md](../../../gym/packs/text-editing/README.md)
다. 예외·가장자리는
[gym_text_editing_exceptions.md](gym_text_editing_exceptions.md) 에 모은다.

## 무엇을 했는가

text-editing pack은 본문 편집 축인데 과제가 얇았다. 기존 PR은 TE11–TE14
네 건만 더했다(약 309 insertions). 같은 계약으로 TE15–TE90 을 더하고,
여정 README·팩 테스트·예외 노트를 붙였다.

건드리지 않은 것:

- 새 CLI, 새 pack, T07 복제, `deep_contains`
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- 다른 pack 의 과제 ID
- `cargo fmt --all` (JSON·문서·테스트만 바꿨다)
- `pack.json` 의 `runner` 신원과 `requires.commands`

사용한 명령은 이미 선언된 `digest` · `edit` · `explain` ·
`export-structure` · `info` · `search` 뿐이다.

## 왜 이 두께인가

본문 편집 축은 "한 명령 × 한 표본" 으로 닫히지 않는다. 같은
`replace-text` 라도

- 전건인가, `--occurrence k`(0 기준) 인가
- 읽는 필드가 `matchCount` 인가 `matches[0].paragraph` 인가
- 재검색 바늘이 치환 대상인가, 무관 문구인가
- 표본이 중첩셀인가, 실문서인가, hwp3 인가

가 다른 계약이다. `insert-text` 는 좌표 (section, para, offset) 와
재검색 필드(`matchCount` / `paragraph` / `page` / `offset`) 가 갈라진다.
과제를 합치면 에이전트가 힌트 한 줄을 외워 모든 왕복을 통과한다.

occurrence 와 all 을 한 과제에 섞지 않는다. 잔여 옛 문구 `value_ge 1` 이
전건을 거르고, 전건 과제의 `value_eq 0` 이 한 건 치환을 거른다. 두
검사가 서로를 보완한다.

## 과제 계보

### 기존 (devel, TE01–TE10)

| ID | 명령 | 요지 |
|---|---|---|
| TE01 | replace-text 전건 | 규제 → 점검, 잔여 0 |
| TE02 | replace-text + info | 쪽수 라이브 |
| TE03 | sanitize | 제출용 정리 |
| TE04 | search --ignore-case | ⅰ 건수 |
| TE05 | digest | paraCount |
| TE06 | explain | paragraphCount |
| TE07 | export-structure | nodeCount (중첩셀) |
| TE08 | replace-text --dry-run | replacedCount |
| TE09 | search | 보험료 건수 |
| TE10 | replace-text 전건 | 보험료 → 납입금 |

### 첫 확장 (TE11–TE14)

| ID | 명령 | 요지 |
|---|---|---|
| TE11 | replace-text --occurrence 0 | 첫 규제만, 잔여 ≥ 1 |
| TE12 | replace-text + search | 무관 ⅰ 재검색 |
| TE13 | insert-text (0,0,0) | 짐표지TE13 1건 |
| TE14 | replace-text + search | 점검 첫 문단 |

### 후속 확장 (TE15–TE90)

| 구간 | 여정 | 요지 |
|---|---|---|
| TE15–TE21, TE56–TE58 | J2 occurrence | 규제·의·국어·보험료의 k번째 |
| TE22–TE24, TE51–TE55, TE59–TE60 | J1 전건 + 재검색 | 다른 표본·필드 |
| TE25–TE26, TE49, TE83–TE84, TE90 | J5 dry-run | replacedCount / insertedChars / 좌표 |
| TE27–TE48, TE50 | J3 insert-text | 표본별 (0,0,0) + 좌표 되읽기 |
| TE61–TE76, TE85–TE89 | J6 조사 | search/digest/explain/info/structure |
| TE77–TE82 | J7 sanitize·문단 | 정리와 insert-paragraph |

TE07 과 TE69/TE70/TE88 은 모두 `export-structure` 의 `nodeCount` 를
읽는다. 표본이 다르다. core-cli T07(`fill-fields`) 과는 명령도 축도
다르다.

## 지목 연산자

쓰는 것:

- `value_eq` / `value_ge` — 잔여 0, 새 문구 ≥ 1, 표지 1건, format
- `answer_eq` — 라이브 오라클 (건수·쪽·문단·오프셋·형식)
- `differs_from_input` — 무편집 복사 거부
- `file_exists` — 산출 존재 (sanitize, 일부 치환)

쓰지 않는 것:

- `deep_contains` / `not_contains` — 편집 축 전역 훑기 금지
- `cell_text_eq` — 표 pack 의 좌표
- `json_value_eq` 로 골든 해시 박제

## 표본

기존 표본을 재사용한다. 새 픽스처를 넣지 않았다.

- `samples/basic/issue2007_nested_cell_pagination_42065.hwp` — 규제, ⅰ
- `samples/143E433F503322BD33.hwp` — 보험료
- `samples/para-001.hwp` — 좌표 삽입의 기준 표본
- `samples/hwp3-sample.hwp` — 조사 '의' 다수 (occurrence 계약)
- `samples/2022년 국립국어원 업무계획.hwp` — 국어
- `samples/table-001.hwp` — 표
- `samples/76076_regulatory_analysis.hwp` — 규제 (다른 문서)
- `samples/hwpx_sample2.hwpx` — HWPX 삽입·sanitize
- 시험지·각주·가로·그림·서식 등 머리 삽입용 단순 표본

치환 쌍은 검색어가 치환어에 포함되지 않게 골랐다. `표` → `도표` 는
넣지 않았다.

## 검증

- `python gym/tools/audit.py` — 전 pack 정합
- `python -m unittest scripts.tests.test_gym_packs -v`
- `python -m unittest scripts.tests.test_gym_text_editing_pack -v`
- `cargo fmt --all` 생략 (Rust 변경 없음)

로컬에서 새 과제를 `build_baseline` 으로 라이브 채점하지는 않았다.
기준 풀이는 기존 TE 과제와 같은 라이브 재계산 형식이다.

## 남긴 한계

- TE11 계열의 잔여 `value_ge 1` 은 두 건 이상 치환해도 잔여가 있으면
  통과한다. 전건만 확실히 거절한다.
- insert-text (0,0,0) 은 그 좌표가 존재하는 표본을 골랐다. 범위 밖이면
  기준풀이가 실패한다.
- TE58 은 보험료가 2건 이상이어야 occurrence 0 + 잔여가 성립한다.
- HWP3 표본의 `format` 표기는 라이브로 읽는다. `hwp5` 로 추측하지
  않는다.
