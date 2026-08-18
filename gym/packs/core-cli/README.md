---
kind: guide
status: active
canonical: gym/packs/core-cli/README.md
last_verified: 2026-08-18
---

# core-cli — 코어 CLI 온램프 (조사·추출·편집·검증)

## 왜 이 pack 인가

새 에이전트가 rhwp 운동장에 들어오면 먼저 문서를 열고 숫자를 읽고 표를 세고
검색하고 서식을 조사해야 한다. 입문존(`casual-rides`)이 "한 숫자" 라면
이 pack 은 그 다음 층이다. T01–T14 만으로는 표본이 한쪽으로 몰리고,
검색 질의가 '국어' 하나이며, 서식 채움은 T07 한 칸(`fields[0]==홍길동`)에
기대게 된다.

T15+ 는 **같은 명령 표면**에서 표본·경로·축을 갈라 온램프를 두껍게 한다.
새 CLI 는 없다. 새 연산자는 없다. 새 표본 파일은 만들지 않는다.
T07 의 `fields[0].value == 홍길동` 판정은 복제하지 않는다.

권위 출처: `gym/README.md`, `gym/PARK.md`, 스킬 `rhwp-cli` · `rhwp-doc-triage`
· `rhwp-form-fill` · `rhwp-security-sweep`.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `export-tables` · `extract-data` ·
   `fields` · `gate` · `harness` · `harness-status` · `info` · `inspect` ·
   `ir-diff` · `replay` · `search` 이다. 새 명령을 요구하지 않는다.
2. **기존 연산자만.** `answer_eq` · `len_answer_eq` · `value_eq` ·
   `cell_text_eq` · `same_hash` · `differs_from_input` · `len_ge`.
   전역 훑기(`deep_contains` · `not_contains`)는 쓰지 않는다.
3. **기존 표본만.** `samples/` 에 이미 있는 파일만 가리킨다.
4. **라이브 오라클.** 쪽수·매치 수·필드 수·추출 건수는 과제에 박제하지 않는다.
   채점기가 같은 명령을 다시 돌린다.
5. **T07 을 복제하지 않는다.** 첫 필드에 홍길동을 쓰지 않는다. T32 는
   `fields[0].name` 을 읽을 뿐 값을 쓰지 않는다. T24·T46 은 검색·스윕만 한다.
6. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` / `{sub:}` 뿐이다.
7. **runner 신원을 복사한다.** `pack.json` 의 `rhwpVersion` · `rhwpCommit` ·
   `capabilitiesSha256` 은 devel 값을 유지한다.
8. **새 pack / 새 CLI 없음.**

## 명령 표면 (pack.json requires)

| 명령 | 이 pack 에서 하는 일 | 읽는 봉투 |
|------|----------------------|-----------|
| `info` | 형식·쪽수·문단 수 | `format` · `pageCount` · `paraCount` |
| `search` | 본문 매치 | `matches` 길이 · `matchCount` |
| `fields` | 누름틀 대장 (읽기) | `fields` 길이 · `fields[n].name` · `fieldType` |
| `extract-data` | 날짜·금액·수량 | `items` 길이 |
| `export-tables` | 표 수확 · 셀 좌표 | `tables` 길이 · 셀 text |
| `inspect` | 은닉·유니코드·주입 | `hiddenCharCount` · `findingCount` · `injectionSignals` |
| `ir-diff` | 변환 자기검증 | `identical` |
| `replay` | 계획 재현 | `input` · `reproduced` |
| `gate` / `harness` | T13·T14 사다리 | `verdict` · `capsules` |

`edit` · `run` · `export-hwpx` 는 기준풀이 생성에만 쓰이고, 채점은 위 명령의
봉투로 닫는다. 새 명령을 requires 에 넣지 않았다.

## 함정

- **T07 금지.** `fields[0].value == 홍길동` 을 맞추려 하지 마라. 조회 과제는
  이름·타입만 읽는다.
- **쪽수 ≠ 문단 수 ≠ 표 수 ≠ 매치 수.** 같은 파일이라도 키가 다르다.
- **'국어' 와 '국립' 은 다른 질의다.** T02 답을 T22 에 옮기지 마라.
- **inspect 축을 섞지 마라.** hidden-text · unicode · injection 은 다른 봉투다.
- **변환 과제는 IR 동등을 라이브로 읽는다.** identical 을 true 로 박제하지 마라.
- **결정론 과제는 원본 복사를 거부한다.** `differs_from_input` + replay 재현.
- **좌표를 지목하라.** 셀 편집은 `cell_text_eq` 다. 전역 훑기는 쓰지 않는다.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급. 보스(5) 사다리 완주는 XC 의 일이다.

### T01–T14 — 개장 코어 (devel)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| T01 | 1 | 문서 신상 쪽수 | issue2007 중첩표 |
| T02 | 1 | '국어' 검색 | 국어원 업무계획 |
| T03 | 2 | 누름틀 개수 | field-01 |
| T04 | 2 | 데이터 추출 | 수출입 확정치 |
| T05 | 2 | 표 개수 | issue2007 |
| T06 | 2 | 문구 치환 | issue2007 |
| T07 | 3 | 서식 채움 (홍길동, 복제 금지) | field-01 |
| T08 | 3 | 첫 셀 짐검증 | issue2007 |
| T09 | 3 | 두 단계 계획 | issue2007 |
| T10 | 3 | 결정론 실증 | issue2007 |
| T11 | 2 | 주입 스윕 | 143E433F… |
| T12 | 2 | HWPX 변환 자기검증 | field-01 |
| T13 | 3 | 하네스 루프 | issue2007 |
| T14 | 3 | 관문 통과 | issue2007 |

### T15+ — 신상 · 검색 · 필드 · 추출 · 스윕 · 변환 · 좌표

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| T15 | 1 | 표 표본 쪽수 | `samples/table-001.hwp` |
| T16 | 1 | 국어원 문서 형식 | `samples/2022년 국립국어원 업무계획.hwp` |
| T17 | 1 | 표 표본 문단 수 | `samples/table-001.hwp` |
| T18 | 1 | 서식 표본 쪽수 | `samples/form-01.hwp` |
| T19 | 1 | 누름틀 표본 형식 | `samples/field-01.hwp` |
| T20 | 1 | 한 쪽 시험지 쪽수 | `samples/exam-kor-1p.hwp` |
| T21 | 1 | 표 표본에서 '표' 검색 | `samples/table-001.hwp` |
| T22 | 2 | 국어원 문서에서 '국립' 검색 | `samples/2022년 국립국어원 업무계획.hwp` |
| T23 | 2 | 서식 본문에서 '서식' 검색 | `samples/form-01.hwp` |
| T24 | 2 | 누름틀 표본에서 '회사' 검색 | `samples/field-01.hwp` |
| T25 | 2 | 홍보 문서에서 '홍보' 검색 | `samples/20250130-hongbo.hwp` |
| T26 | 2 | 다중 표 문서에서 '표' 검색 | `samples/multi-table-001.hwp` |
| T27 | 2 | 문단 표본에서 '한글' 검색 | `samples/para-001.hwp` |
| T28 | 2 | 중첩표 표본에서 '규제' 검색 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| T29 | 2 | form-01 누름틀 개수 | `samples/form-01.hwp` |
| T30 | 2 | form-02 누름틀 개수 | `samples/form-02.hwp` |
| T31 | 2 | 메모 서식 누름틀 개수 | `samples/field-01-memo.hwp` |
| T32 | 2 | 첫 칸 이름만 읽기 | `samples/field-01.hwp` |
| T33 | 2 | 둘째 칸 이름만 읽기 | `samples/field-01.hwp` |
| T34 | 2 | 첫 칸 타입만 읽기 | `samples/field-01.hwp` |
| T35 | 2 | 해외직구 안내 데이터 추출 | `samples/156457624_210622 7월부터 해외직구 구매대행업체 등록제 시행.hwp` |
| T36 | 2 | 국어원 계획 데이터 추출 | `samples/2022년 국립국어원 업무계획.hwp` |
| T37 | 2 | 표 표본 표 개수 | `samples/table-001.hwp` |
| T38 | 2 | 다중 표 표본 표 개수 | `samples/multi-table-001.hwp` |
| T39 | 2 | 서식 문서 표 개수 | `samples/form-01.hwp` |
| T40 | 2 | 홍보 문서 데이터 추출 | `samples/20250130-hongbo.hwp` |
| T41 | 2 | 조판 은닉 글자 수 | `samples/issue1892_hwp3_tab_roundtrip.hwp` |
| T42 | 2 | 유니코드 기만 소견 수 | `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp` |
| T43 | 2 | 표 표본 주입 신호 | `samples/table-001.hwp` |
| T44 | 2 | 표 표본 은닉 스윕 | `samples/table-001.hwp` |
| T45 | 2 | 표 표본 유니코드 스윕 | `samples/table-001.hwp` |
| T46 | 2 | 누름틀 표본 주입 스윕 | `samples/field-01.hwp` |
| T47 | 2 | 서식 표본 HWPX 변환 자기검증 | `samples/form-01.hwp` |
| T48 | 2 | 표 표본 HWPX 변환 자기검증 | `samples/table-001.hwp` |
| T49 | 2 | 표 표본 첫 셀 온램프 표기 | `samples/table-001.hwp` |
| T50 | 2 | 다중 표 첫 셀 표기 | `samples/multi-table-001.hwp` |
| T51 | 2 | 내부표 첫 셀 표기 | `samples/inner-table-01.hwp` |
| T52 | 3 | 표 표본 두 칸 원자 계획 | `samples/table-001.hwp` |
| T53 | 3 | 표 표본 결정론 실증 | `samples/table-001.hwp` |
| T54 | 2 | 한 쪽 시험지 데이터 추출 | `samples/exam-kor-1p.hwp` |

## 재현

기준풀이는 `reference/T15.json` … `reference/T54.json` 이다. 자리표는
`{input}` · `{sub:}` 이다. `{file:}` 는 채점 쪽 경로다.

```
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs
python -m unittest scripts.tests.test_gym_core_casual_packs
```

바이너리 없이 스키마·짝·고유 ID 는 audit 가 본다. 라이브 오라클 수치는
채점 시점 CLI 가 다시 계산한다.
