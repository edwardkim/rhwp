---
kind: guide
status: active
canonical: gym/packs/text-editing/README.md
last_verified: 2026-08-18
---

# text-editing — 본문 편집 여정과 실패 모드

이 pack은 **탐색 → 치환·삽입 → 재검증** 을 채점한다. 고친 자리를 지목해
판정하고, 고치지 않은 것까지 바뀌지 않았는지 본다. 새 CLI는 없다. 기존
`edit` · `search` · `info` · `explain` · `digest` · `export-structure` 와
`samples/` 만 쓴다.

채점은 라이브 오라클이다. 일치 건수·쪽수·문단 번호를 JSON에 박제하지 않는다.
`answer_eq` 가 채점 시점에 rhwp를 다시 돌려 기댓값을 계산한다.
`value_eq` / `value_ge` 는 **잔여 0** 이나 **새 문구 ≥ 1** 처럼 계약 자체인
값만 고정한다.

편집 과제에 `deep_contains` 는 없다. 전역 훑기는 좌표를 지목하지 못한다.
`fill-fields`(core-cli **T07**) 복제도 없다. 이 pack의 축은 누름틀이 아니라
본문 문자열이다.

## 왜 이 pack 인가

에이전트가 문서를 "고쳤다"고 말하는 순간, 보통 다섯 가지 중 하나가 빠진다.

1. **전건과 k번째를 섞는다.** `replace-text` 기본은 전건이다.
   `--occurrence N` 은 **0 기준 N번째 한 건**만 바꾼다. 전건 산출물을
   occurrence 과제에 내면 옛 문구 잔여 검사에서 거절된다.
2. **없는 자리에 덮어쓴다.** `replace-text` 는 있는 문자열을 바꾼다.
   없는 자리에 글자를 넣는 축은 `insert-text --section --para --offset`
   이다. 좌표는 search 주소와 같이 **전부 0 기준**이다.
3. **고치지 않은 문구를 다시 세지 않는다.** 치환이 성공해도 무관 문구가
   사라지면 과잉 편집이다. 재검색이 그 구멍을 막는다.
4. **종료 코드만 보고 판정한다.** `replacedCount` 가 0 이면 산출 파일을
   만들지 않는다. 검색어를 틀리면 조용히 원본이 남는다.
5. **쪽·문단 축을 섞는다.** `search` 의 `matches[].page` 는 0 기준이다.
   `info` 의 `pageCount` 는 개수다. `extract-pages` 의 `--from` 은 1 기준인데
   이 pack은 그 명령을 쓰지 않는다.

이 pack의 과제는 위 구멍을 여정으로 나눈다.

## 좌표 계약 — occurrence 와 insert-text

### occurrence 대 all

| 호출 | 의미 | 잔여 옛 문구 | 대표 과제 |
|---|---|---|---|
| `edit replace-text --find A --replace B` | 전건 | 0 이어야 한다 | TE01, TE10, TE22–TE24 |
| `edit replace-text --occurrence 0` | 0 기준 첫 번째 한 건 | ≥ 1 이어야 한다 | TE11, TE17, TE20 |
| `edit replace-text --occurrence 1` | 0 기준 둘째 한 건 | ≥ 1 | TE15, TE18 |
| `edit replace-text --occurrence k` | 0 기준 k번째 한 건 | ≥ 1 | TE16, TE19, TE21, TE57 |
| `edit replace-text --dry-run` | 파일을 쓰지 않음 | 원본 불변 | TE08, TE25, TE26, TE90 |

`--occurrence` 는 **0 기준**이다. 한컴 서식의 체크박스(□ 여러 개 중 k번째만
☑)가 이 계약의 원형이다(`replace_occurrence_contract`). 문서 안내 한 줄이
"1 기준 k번째"로 보이더라도, 이 pack과 코어 테스트는 0 기준을 따른다.
1을 넣으면 **둘째**가 바뀐다.

전건 치환을 occurrence 과제에 제출하면 새 문구는 생기지만 옛 문구가
0이 된다. TE11/TE15–TE21/TE56–TE58 은 잔여 `value_ge 1` 로 그걸 거른다.
건수를 6으로 박제하지 않는다. 두 건 이상 치환해도 잔여가 남으면 통과할
수 있다 — 그게 이 검사의 한계이고, 전건(잔여 0)만은 확실히 거절한다.

### insert-text 좌표

```
rhwp edit insert-text 입력.hwp --section S --para P --offset O --text "표지" -o 산출.hwp --json
```

| 플래그 | 기준 | 범위 밖 |
|---|---|---|
| `--section` | 0 기준 구역 | exit 2, 원본 불변 |
| `--para` | 0 기준 문단 | exit 2 |
| `--offset` | 0 기준 문자. 문단 길이와 같으면 끝에 붙인다 | 길이를 넘으면 자르지 않고 exit 2 |

생략하면 세 값 모두 0 이다. TE13 과 TE27–TE50 의 기본 좌표가 (0,0,0) 인
이유다. 표지가 문서에 없던 문자열이어야 `matchCount == 1` 이 성립한다.
이미 있는 단어를 넣으면 1건 검사가 실패한다.

`search` 의 `matches[0].paragraph` / `.page` / `.offset` 은 같은 0 기준
주소다. 삽입 뒤 재검색으로 좌표를 되읽는다(TE45, TE48, TE50).

`--dry-run` 은 파일을 쓰지 않고 `insertedChars` · `section` · `paragraph` ·
`offset` 만 보고한다(TE49, TE83, TE84).

## 여정 지도

과제는 명령 가족이 아니라 **사람이 하는 일**로 묶는다. 한 여정이 여러
과제를 가질 수 있다. 표본이 바뀌면 같은 명령도 다른 계약이 된다.

### J1. 전건 치환 왕복 (`replace-text` without occurrence)

있는 문구를 모두 바꾸고, 옛 문구 0 · 새 문구 ≥ 1 · 원본과 다름을 본다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| TE01 | 규제 → 점검 | issue2007 중첩셀 | matchCount 0 / ≥ 1 |
| TE02 | 같은 치환 + 쪽수 | 같은 표본 | pageCount 라이브 |
| TE10 | 보험료 → 납입금 | 143E 실문서 | matchCount |
| TE22 | 국어 → 언어 | 국립국어원 업무계획 | matchCount |
| TE23 | 의 → ◎ | hwp3-sample | 다수 전건 |
| TE24 | 규제 → 점검 | 76076 규제영향 | 다른 표본 같은 단어 |
| TE51 | 규제 → 심사 + 무관 ⅰ | issue2007 | untouched 라이브 |
| TE52 | 보험료 → 보험금 + 첫 문단 | 143E | matches[0].paragraph |
| TE53 | 국어 → 언어 + 첫 쪽 | 국어원 | matches[0].page |
| TE54 | 규제 → 심사 + 형식 | issue2007 | format=hwp5 |
| TE55 | 규제 → 심사 + 쪽수 | issue2007 | pageCount 라이브 |
| TE59 | 규제 → 심사 + 첫 쪽 | issue2007 | matches[0].page |
| TE60 | 보험료 → 보험금 + 존재 | 143E | file_exists |

**실패 모드**

- 치환어가 검색어를 포함하면 옛 문구 0 검사가 깨진다. `표` → `도표` 가
  그 함정이다. 이 pack은 겹치지 않는 쌍만 쓴다.
- `replacedCount` 0 이면 `-o` 를 줘도 파일이 없다. 없는 단어를 찾으면
  원본 복사로 위장하기 쉽다. `differs_from_input` 이 그걸 거절한다.
- 전건 과제를 occurrence 한 건으로 제출하면 옛 문구가 남아 실패한다.

### J2. k번째만 치환 (`--occurrence`)

한 자리만 고치고 나머지는 남겨야 하는 계약이다.

| ID | 하는 일 | 표본 | occurrence |
|---|---|---|---|
| TE11 | 규제 첫 번째만 점검 | issue2007 | 0 |
| TE15 | 규제 둘째만 심사 | issue2007 | 1 |
| TE16 | 규제 셋째만 감독 | issue2007 | 2 |
| TE17 | 의 첫 번째만 ◎ | hwp3-sample | 0 |
| TE18 | 의 둘째만 | hwp3-sample | 1 |
| TE19 | 의 여섯 번째만 | hwp3-sample | 5 |
| TE20 | 국어 첫 번째만 언어 | 국어원 | 0 |
| TE21 | 국어 셋째만 | 국어원 | 2 |
| TE56 | 첫 규제만 + 점검 문단 | issue2007 | 0 |
| TE57 | 의 네 번째만 | hwp3-sample | 3 |
| TE58 | 보험료 첫 번째만 | 143E | 0 |

**실패 모드**

- `--occurrence 1` 을 "첫 번째"로 읽는다. 0 기준이라 둘째다.
- 전건 산출물을 낸다. 잔여 `value_ge 1` 이 거절한다.
- 단어가 문서에 k+1 건 미만이면 기준풀이 자체가 실패한다. TE58 은
  보험료가 1건뿐이면 전제가 깨진다고 힌트에 적어 두었다.
- 건수 N을 JSON에 박제하지 마라. 표본이 바뀌면 N이 바뀐다.

### J3. 문단 좌표 삽입 (`insert-text`)

없는 표지를 지정 좌표에 넣고, 재검색으로 1건을 확인한다.

| ID | 하는 일 | 표본 | 좌표 |
|---|---|---|---|
| TE13 | 짐표지TE13 | para-001 | (0,0,0) |
| TE27 | 짐표지TE27 | table-001 | (0,0,0) |
| TE28 | 짐표지TE28 | form-01 | (0,0,0) |
| TE29 | 짐표지TE29 | exam_kor | (0,0,0) |
| TE30 | 짐표지TE30 | footnote-01 | (0,0,0) |
| TE31 | 짐표지TE31 | landscape-001 | (0,0,0) |
| TE32 | 짐표지TE32 | pic2 | (0,0,0) |
| TE33 | 짐표지TE33 | aift | (0,0,0) |
| TE34 | 짐표지TE34 | biz_plan | (0,0,0) |
| TE35 | 짐표지TE35 | field-01 | (0,0,0) |
| TE36–TE39 | 시험지 1–4쪽 | exam-kor-Np | (0,0,0) |
| TE40 | 짐표지TE40 | endnote-01 | (0,0,0) |
| TE41 | 짐표지TE41 | 143E | (0,0,0) |
| TE42 | 짐표지TE42 | issue2007 | (0,0,0) |
| TE43 | 짐표지TE43 | form-02 | (0,0,0) |
| TE44 | 짐표지TE44 | field-01-memo | (0,0,0) |
| TE45 | 삽입 + 첫 문단 | para-001 | (0,0,0) + paragraph |
| TE46 | 삽입 + format | para-001 | (0,0,0) |
| TE47 | HWPX 삽입 | hwpx_sample2 | (0,0,0), 산출 .hwpx |
| TE48 | 삽입 + 첫 쪽 | para-001 | matches[0].page |
| TE50 | 삽입 + 오프셋 | 76076 | matches[0].offset |

**실패 모드**

- 구역·문단·오프셋이 범위를 벗어나면 exit 2. 기준풀이가 실패한 것이다.
  좌표를 추측하지 말고, 필요하면 `export-structure` / `search` 로 길이를
  확인하라.
- `--offset` 을 1 기준으로 주면 한 글자 밀린다. 첫 글자 앞은 0 이다.
- 산출 확장자를 입력과 다르게 준다. TE47 은 `.hwpx` 를 요구한다.
- 표지를 이미 있는 단어로 잡는다. `matchCount == 1` 이 깨진다.
- `fill-fields` 나 `set-cell` 로 같은 효과를 낸다. 이 pack은 그 명령을
  채점하지 않는다.

### J4. 치환·삽입 후 재검색

고친 뒤 **다른 필드**를 다시 읽는다. 라이브 오라클이다.

| ID | 재검색 대상 | 필드 |
|---|---|---|
| TE12 | 무관 문구 ⅰ | matchCount |
| TE14 | 점검 첫 문단 | matches[0].paragraph |
| TE45 / TE48 / TE50 | 삽입 표지 | paragraph / page / offset |
| TE51 | 무관 ⅰ | matchCount |
| TE52 / TE56 | 새 문구 첫 문단 | paragraph |
| TE53 / TE59 | 새 문구 첫 쪽 | page |

**실패 모드**

- 치환이 재검색 바늘 자체를 바꾸면 채점이 그 결과를 따른다. TE12 의
  `ⅰ` 가 그 예다. 값을 박제하지 않았기 때문이다.
- `matches[0]` 은 0건이면 경로가 없다. 새 문구 `value_ge 1` 을 먼저 둔다.

### J5. 선확인 (`--dry-run`)

파일을 만들지 않고 봉투만 읽는다.

| ID | 명령 | 필드 |
|---|---|---|
| TE08 | replace-text 전건 | replacedCount |
| TE25 | replace-text occurrence 1 | replacedCount |
| TE26 | 보험료 전건 | replacedCount |
| TE49 | insert-text (0,0,0) | insertedChars |
| TE83 | insert-text dry-run | paragraph |
| TE84 | insert-text dry-run | offset |
| TE90 | 의 occurrence 0 | replacedCount |

**실패 모드**

- dry-run 산출물을 제출한다. 이 여정은 `answer.json` 만 받는다.
- `replacedCount` 와 `matchCount` 를 섞는다. 전자는 edit 봉투, 후자는
  search 봉투다.

### J6. 조사 (`search` · `digest` · `explain` · `info` · `export-structure`)

편집 없이 봉투를 읽는다. `axis` 는 `조사` 다. TE07 은 중첩셀
`nodeCount` 다. TE69/TE70/TE88 은 **다른 표본**의 같은 필드다.
core-cli **T07**(`fill-fields`) 복제가 아니다.

| ID | 명령 | 필드 | 표본 |
|---|---|---|---|
| TE04 | search --ignore-case | matchCount | issue2007 ⅰ |
| TE05 | digest | paraCount | issue2007 |
| TE06 | explain | paragraphCount | issue2007 |
| TE07 | export-structure | nodeCount | issue2007 |
| TE09 | search | matchCount | 143E 보험료 |
| TE61 | search | matchCount | table-001 표 |
| TE62 | search | matchCount | 국어원 국어 |
| TE63 | search | matchCount | 유니코드 표본 |
| TE64 | search --ignore-case | matchCount | REGULATORY |
| TE65 | digest | paraCount | para-001 |
| TE66 | explain | paragraphCount | table-001 |
| TE67 / TE74 / TE76 | info | pageCount | 실문서 / 가로 / 2쪽 |
| TE68 / TE86 / TE87 | info | format | 실문서 / 중첩셀 / hwp3 |
| TE69 / TE70 / TE88 | export-structure | nodeCount | table / para / hwp3 |
| TE71–TE73 / TE89 | search | matches[0].* | 보험료 / 규제 / 국어 / 표 |
| TE75 | explain | paragraphCount | footnote-01 |
| TE85 | digest | paraCount | issue2007 |

`digest.paraCount` 와 `explain.paragraphCount` 는 다른 필드다. TE05 와
TE06 이 그 혼동을 가른다.

### J7. 제출용 정리 · 문단 끼우기

| ID | 명령 | 표본 | 지목 |
|---|---|---|---|
| TE03 | sanitize | issue2007 | format + differs |
| TE77 | sanitize | 143E | format hwp5 |
| TE78 | sanitize | para-001 | format hwp5 |
| TE79 | sanitize | hwpx_sample2 | format hwpx |
| TE80–TE82 | insert-paragraph | para / table / exam | paragraphCount 라이브 |

sanitize 는 메타데이터를 지운다. 누름틀을 채우지 않는다. 산출물이
여전히 열리는지 `info.format` 으로 본다.

## 공통 실패 모드

### F1. occurrence 0 과 1

첫 번째를 고치려면 `--occurrence 0` 이다. `1` 은 둘째다. TE11 과 TE15 가
그 쌍이다.

### F2. 전건 산출물을 occurrence 과제에 냄

잔여 옛 문구 `value_ge 1` 이 거절한다. 새 문구만 보면 속는다.

### F3. insert-text 좌표 범위

문단 길이보다 큰 offset, 없는 구역·문단은 exit 2. 조용히 잘리지 않는다.

### F4. 쪽 축 혼동

`search` 의 page 는 0 기준. `info.pageCount` 는 개수. 이 pack은
`extract-pages`(1 기준)를 쓰지 않는다. 다른 pack의 쪽 습관을 가져오지
마라.

### F5. replacedCount 0 = 무산출

없는 단어를 치환하면 출력 파일이 없다. `file_exists` 와
`differs_from_input` 이 위장 복사를 거절한다.

### F6. deep_contains 와 T07

편집 축에서 `deep_contains` 는 스키마가 막는다(#4600). `fill-fields` 는
core-cli T07 의 축이다. 이 pack 과제에 누름틀 채움을 넣지 마라.

### F7. 검색어가 옵션으로 먹힘

`search 파일 --json -- 규제` 처럼 `--` 뒤에 바늘을 둔다. `--` 없이
`-` 로 시작하는 바늘은 플래그로 오해된다.

### F8. 치환어 ⊂ 검색어

`표` → `도표` 는 옛 문구 0 이 불가능하다. 겹치지 않는 쌍만 쓴다.

### F9. dry-run 파일을 제출

선확인 과제는 `answer.json` 만 받는다. `-o` 로 파일을 만들어도 채점하지
않는다.

### F10. 힌트 한 줄로 모든 표본을 통과

같은 `replace-text` 라도 표본·occurrence·재검색 필드가 다르다. 과제를
합치면 에이전트가 힌트를 외워 모든 왕복을 통과한다. 그래서 여정을
나눴다.

## 쓰지 않는 것

- 새 CLI, 새 pack, 새 연산자
- `deep_contains` / `not_contains`
- `edit fill-fields` (T07), `set-cell`, `insert-text-in-cell`
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- `pack.json` 의 `runner` 신원 변경
- 골든 바이트, 박제된 matchCount / pageCount

## 기존 TE01–TE14

devel 의 TE01–TE10 과 첫 확장 TE11–TE14 는 그대로 둔다. 과제 ID 는
전역 고유하다. 이 문서의 TE15–TE90 이 그 위에 여정을 얹는다.

작업 계보는
[mydocs/working/gym_text_editing.md](../../../mydocs/working/gym_text_editing.md),
예외·가장자리는
[mydocs/working/gym_text_editing_exceptions.md](../../../mydocs/working/gym_text_editing_exceptions.md)
다.
