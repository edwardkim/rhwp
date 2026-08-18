---
kind: guide
status: active
canonical: gym/packs/table-editing/README.md
last_verified: 2026-08-18
---

# table-editing — 표 좌표 편집 pack

이 pack 은 rhwp 표 편집 능력의 **좌표 축**이다. 표를 통째로 훑거나
본문 어딘가에 문자열이 있는지로 채점하지 않는다. 판정은 언제나
`(표 인덱스, 행, 열)` 이다. 값이 맞더라도 자리가 틀리면 실패다.
자리가 맞더라도 값이 틀리면 실패다. 원본을 그대로 복사하면
`differs_from_input` 에서 실패다.

이 문서는 pack 내부 안내서다. `gym/README.md` · `gym/PARK.md` ·
`gym/profiles/` 는 이 확장에서 건드리지 않는다. 이슈 #5230 DoD 가
profiles/README 미수정을 요구하기 때문이다. pack 의 과제 수가 늘어도
운동장 지도의 존 배치는 그대로다 — 편집존의 표 어트랙션이 길어질 뿐이다.

## 왜 이 pack 인가

한국 공문·서식·통계표는 표로 산다. 에이전트가 표를 다룰 때 가장 흔한
실패는 "문서 어딘가에 원하는 글자가 생겼다"고 착각하는 것이다. 옆칸을
고치거나, 두 번째 표를 고치거나, 머리글만 본문에 붙여 넣어도 전역
검색은 통과한다. 그 오검출이 #4600 이었고, 그래서 편집 축은
`deep_contains` 를 금지한다.

이 pack 은 그 교훈을 과제로 고정한다.

- 조사 과제는 `export-tables --json` 의 경로를 `answer_eq` 로 읽는다.
- 편집 과제는 `edit set-cell` 로 칸을 고치고 `cell_text_eq` 로 그 칸만 본다.
- 추출 과제는 `table-to-csv` 로 표를 뽑고 `csv_cell_eq` 로 칸을 본다.
- 정답 숫자는 박제하지 않는다. 라이브 오라클이 채점 시점에 다시 센다.

TB01–TB08 은 기존 축이다. TB09–TB12 는 #5240 1차 확장이다.
TB13–TB40 은 같은 연산자·같은 표본·같은 CLI 로 좌표 지목을 더 촘촘히
늘린 2차 확장이다. 새 pack 도, 새 CLI 도, T07/fill-fields 복제도 없다.

## 하지 않는 것

이 pack 이 의도적으로 하지 않는 일이 있다. 경계가 과제 수보다 중요하다.

1. **T07 을 복제하지 않는다.** T07 은 `core-cli` 의 누름틀(fill-fields)
   과제다. 표 좌표와 누름틀 이름은 다른 축이다. 이 pack 에 T07.json 을
   두거나 fill-fields 를 과제 힌트로 넣지 않는다. TB07 은 BOM CSV 추출이지
   T07 이 아니다.
2. **fill-fields 를 끌어오지 않는다.** 누름틀 채움은 `rhwp-form-fill` 스킬과
   core-cli 의 몫이다. 표 칸을 누름틀처럼 다루면 좌표 축이 흐려진다.
3. **deep_contains 를 쓰지 않는다.** 편집 과제에서 전역 훑기는 스키마가
   막는다(`GLOBAL_SCAN_OPS`). 이 pack 의 신규 과제는 전부 `cell_text_eq` 다.
4. **새 pack 을 만들지 않는다.** `table-csv` 는 CSV 자산 왕복의 다른 pack 이다.
   이 확장은 `table-editing` 안에만 과제를 더한다.
5. **새 CLI 를 만들지 않는다.** `export-tables` · `edit set-cell` ·
   `table-to-csv` · `csv-to-table` 만 쓴다. `runner.*` 신원은 그대로 둔다.
6. **profiles / gym/README / PARK / checks.py 를 고치지 않는다.**
   pack 내부 README 와 과제·기준풀이·pack 전용 테스트만 추가한다.
7. **골든 숫자를 박제하지 않는다.** 행 수·열 수·셀 수는 `answer_eq` 경로로
   재계산한다. 편집 기대 문자열만 과제에 적는다 — 그것은 지시이지 오라클이 아니다.

## 요구 capability

`pack.json` 의 `requires.commands` 는 `edit` 와 `export-tables` 다.
TB03·TB04·TB11–TB40 이 `edit set-cell` 을 쓰고, 조사·채점 재조회는
`export-tables` 를 쓴다. 바이너리에 이 명령이 없으면 점수는 0 이 아니라
`unavailable` 이다. 부재를 실패로 위장하지 않는 것이 이 저장소의 결이다.

기준 실행 신원(`runner.rhwpVersion` · `rhwpCommit` · `capabilitiesSha256`)은
이 확장에서 바꾸지 않는다. 점수의 신원을 조용히 갈아끼우지 않기 위해서다.

## 표본

과제는 이미 pack 이 쓰던 표본만 재사용한다. 새 픽스처를 추가하지 않는다.

| 표본 | 쓰임 | 비고 |
|---|---|---|
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | 다중 표·셀 지목 | 중첩 셀 쪽나눔 회귀 표본. 첫 표 (0,0)(0,1)(1,0) 이 실측돼 있다. |
| `samples/table-001.hwp` | 단일 표 머리 칸 | 첫 셀 원문이 `구 분`. CSV 추출·머리 치환의 기준 표본. |
| `samples/143E433F503322BD33.hwp` | 실문서 표 수확 | 합성 픽스처가 아닌 현장 문서. 첫 표 (0,0) 을 지목한다. |

표본을 늘리고 싶으면 먼저 `export-tables --json` 으로 좌표가 존재함을
실측하고, 그 좌표만 과제에 적는다. 존재하지 않는 (행,열) 을 적으면
`cell_text_eq` 는 `None` 으로 실패한다 — 좌표 부재를 통과로 위장하지 않는다.

## 연산자 계약

판정 어휘는 `gym/core/checks.py` 한곳이다. 이 pack 은 연산자를 고르기만 한다.

| 연산자 | 이 pack 에서의 역할 | 쓰지 않는 이유 / 쓰는 이유 |
|---|---|---|
| `cell_text_eq` | 편집 과제의 본판정 | (table,row,col) 을 지목한다. TB13+ 전 과제의 공통 축. |
| `differs_from_input` | 무편집 복사 거부 | 원본을 그대로 내면 값이 우연히 같아도 편집이 아니다. |
| `answer_eq` | 조사 과제의 라이브 오라클 | 행·열·셀 수·표 개수를 채점 시점에 재계산한다. |
| `csv_cell_eq` | CSV 추출의 칸 대조 | 파일 안의 (row,col) 을 지목한다. 전역 검색이 아니다. |
| `utf8_bom` | BOM 추출 확인 | TB07 전용. 엑셀 한글 깨짐 방지. |
| `file_exists` | 산출물 실재 | 최소 바이트와 함께 빈 파일을 거른다. |
| `deep_contains` | **금지** | 전역 훑기. 편집 축에서 스키마가 거부한다. |
| `not_contains` | **금지** | 전역 부재 검사. 이 pack 의 축이 아니다. |

`cell_text_eq` 는 `export-tables --json` 의 `tables` 배열에서
`find_cell(tables, table, row, col)` 로 칸을 찾고, 정규화한 텍스트가
기대값과 같은지 본다. 칸이 없으면 `actual` 은 `None` 이고 `ok` 는
거짓이다. "없으면 통과"가 아니다.

편집 과제의 `cmd` 는 제출 산출물을 다시 읽는다.
`{file:artifact.hwp}` 자리표는 채점기가 제출 폴더의 그 파일로 치환한다.
원본 픽스처를 덮어쓰지 않는다. 기준 풀이도 `-o {sub:...}` 로 산출을 분리한다.

## 과제 목록

난도 티어는 1=입문, 2=초급, 3=중급, 4=고급, 5=보스다. 이 pack 의 보스는
세 칸 이상 동시 지목(TB31)과 가로·세로 쌍(TB18·TB19·TB25·TB26·TB39·TB40)
이다. 사다리 완주급 tier 5 는 expert-challenges 의 몫으로 남겨 둔다 —
표 좌표 pack 이 자동화 사다리를 삼키지 않게 하기 위해서다.

### 기존 과제 TB01–TB12

#### TB01 — 표 좌표 조사

- **티어**: 1
- **축**: 조사
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `answer_eq(tables[0].rows) · answer_eq(tables[0].cols)`
- **요지**: 첫 표의 행·열 수를 라이브 오라클로 읽는다.
- **기준 풀이**: `reference/TB01.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB01 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB01 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB02 — 표 CSV 추출

- **티어**: 2
- **축**: 추출
- **표본**: `samples/table-001.hwp`
- **지목 연산자**: `file_exists · differs_from_input · csv_cell_eq(0,0)==구 분`
- **요지**: 첫 표를 CSV 로 뽑아 첫 셀을 대조한다.
- **기준 풀이**: `reference/TB02.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB02 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB02 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB03 — 다중 셀 지목 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `cell_text_eq(0,0)==앞칸 · cell_text_eq(1,0)==뒷칸`
- **요지**: 두 좌표를 각각 지목한다. 하나만 고치면 실패다.
- **기준 풀이**: `reference/TB03.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB03 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB03 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB04 — 표 CSV 왕복

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **지목 연산자**: `cell_text_eq(0,0)==왕복검증 · differs_from_input`
- **요지**: CSV 왕복의 결과는 다시 셀 좌표로 판정한다.
- **기준 풀이**: `reference/TB04.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB04 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB04 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB05 — 전체 표 계수

- **티어**: 1
- **축**: 조사
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `answer_eq(tableCount)`
- **요지**: 표 개수를 라이브 오라클로 센다.
- **기준 풀이**: `reference/TB05.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB05 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB05 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB06 — 두 번째 표 지목

- **티어**: 2
- **축**: 조사
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `answer_eq(tables[1].cellCount)`
- **요지**: index 1 표를 셀 개수로 지목한다.
- **기준 풀이**: `reference/TB06.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB06 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB06 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB07 — BOM 붙은 CSV

- **티어**: 2
- **축**: 추출
- **표본**: `samples/table-001.hwp`
- **지목 연산자**: `utf8_bom · csv_cell_eq(0,0)==구 분`
- **요지**: 엑셀 한글 깨짐을 막는 BOM 추출이다. 이 과제는 표 추출이지 T07 이 아니다.
- **기준 풀이**: `reference/TB07.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB07 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB07 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB08 — 실문서 표 수확

- **티어**: 2
- **축**: 조사
- **표본**: `samples/143E433F503322BD33.hwp`
- **지목 연산자**: `answer_eq(tableCount) · answer_eq(tables[0].rows)`
- **요지**: 실문서에서 표 개수와 첫 표 행 수를 읽는다.
- **기준 풀이**: `reference/TB08.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB08 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB08 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB09 — 두 번째 표 좌표

- **티어**: 2
- **축**: 조사
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `answer_eq(tables[1].rows) · answer_eq(tables[1].cols)`
- **요지**: 두 번째 표의 행·열을 라이브 오라클로 읽는다.
- **기준 풀이**: `reference/TB09.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB09 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB09 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB10 — 첫 표 셀 개수

- **티어**: 1
- **축**: 조사
- **표본**: `samples/table-001.hwp`
- **지목 연산자**: `answer_eq(tables[0].cellCount)`
- **요지**: 첫 표 셀 개수를 박제하지 않고 재계산한다.
- **기준 풀이**: `reference/TB10.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB10 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB10 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB11 — 첫 표 (0,1) 셀 지목 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **지목 연산자**: `cell_text_eq(0,1)==옆칸 · differs_from_input`
- **요지**: 옆칸만 지목한다. deep_contains 를 쓰지 않는다.
- **기준 풀이**: `reference/TB11.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB11 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB11 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

#### TB12 — 표 머리 셀 치환

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **지목 연산자**: `cell_text_eq(0,0)==표머리 · differs_from_input`
- **요지**: 머리 셀만 지목한다. 전역 훑기를 쓰지 않는다.
- **기준 풀이**: `reference/TB12.json`
- **금지**: T07 복제 없음. `deep_contains` 없음. 정답 숫자 박제 없음.

TB12 는 이 pack 의 기존 계약이다. 이번 확장은 이 과제를 지우거나
연산자를 바꾸지 않는다. 과제 ID 는 전역 고유하므로 다른 pack 이
TB12 를 다시 쓰면 `audit.py` 가 충돌로 막는다.

### 확장 과제 TB13–TB40

TB13 이후는 모두 편집 과제다. 공통 계약은 다음 네 줄이다.

1. `submit.kind` 는 `artifact` 다. 산출물 이름은 과제마다 다르다.
2. 본판정은 하나 이상의 `cell_text_eq` 다. `table` 은 항상 0 이다.
3. 무편집 복사는 `differs_from_input` 이 거부한다.
4. 기준 풀이는 `edit set-cell` 의 연속 호출이다. 중간 산출은 `{sub:stepN.hwp}`.

#### TB13 — 첫 표 (0,0) 좌상 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `left_top.hwp`
- **지목 좌표**: (0,0)→좌상
- **지목 연산자**: `cell_text_eq(0,0,0)==좌상` · `differs_from_input`
- **요지**: 첫 표 원점 좌표만 지목한다. 다른 칸을 고치면 통과하지 못한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:left_top.hwp} --json`
- **기준 풀이**: `reference/TB13.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB13 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB14 — 첫 표 (1,0) 좌하 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `left_bottom.hwp`
- **지목 좌표**: (1,0)→좌하
- **지목 연산자**: `cell_text_eq(0,1,0)==좌하` · `differs_from_input`
- **요지**: 같은 열 다음 행을 지목한다. 행 인덱스 실수가 드러난다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:left_bottom.hwp} --json`
- **기준 풀이**: `reference/TB14.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB14 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB15 — 첫 표 (0,1) 우상 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `right_top.hwp`
- **지목 좌표**: (0,1)→우상
- **지목 연산자**: `cell_text_eq(0,0,1)==우상` · `differs_from_input`
- **요지**: 같은 행 다음 열을 지목한다. 열 인덱스 실수가 드러난다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:right_top.hwp} --json`
- **기준 풀이**: `reference/TB15.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB15 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB16 — table-001 머리칸 치환

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `head_cell.hwp`
- **지목 좌표**: (0,0)→머리칸
- **지목 연산자**: `cell_text_eq(0,0,0)==머리칸` · `differs_from_input`
- **요지**: 다른 표본의 (0,0) 을 지목한다. 표본 고정이 아니라 좌표 계약이다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:head_cell.hwp} --json`
- **기준 풀이**: `reference/TB16.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB16 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB17 — 실문서 첫 셀 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/143E433F503322BD33.hwp`
- **산출**: `real_head.hwp`
- **지목 좌표**: (0,0)→실문서머리
- **지목 연산자**: `cell_text_eq(0,0,0)==실문서머리` · `differs_from_input`
- **요지**: 실문서 표본에서도 같은 좌표 연산자가 성립하는지 본다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:real_head.hwp} --json`
- **기준 풀이**: `reference/TB17.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB17 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB18 — 가로 이웃 두 칸 교정

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `row_pair.hwp`
- **지목 좌표**: (0,0)→갑, (0,1)→을
- **지목 연산자**: `cell_text_eq(0,0,0)==갑 · cell_text_eq(0,0,1)==을` · `differs_from_input`
- **요지**: 한 행의 이웃 두 칸을 각각 지목한다. 하나만 고치면 실패한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:row_pair.hwp} --json`
- **기준 풀이**: `reference/TB18.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB18 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB19 — 세로 이웃 두 칸 교정

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `col_pair.hwp`
- **지목 좌표**: (0,0)→전, (1,0)→후
- **지목 연산자**: `cell_text_eq(0,0,0)==전 · cell_text_eq(0,1,0)==후` · `differs_from_input`
- **요지**: 한 열의 이웃 두 칸을 각각 지목한다. 행만 맞추고 열을 놓치면 실패한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:col_pair.hwp} --json`
- **기준 풀이**: `reference/TB19.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB19 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB20 — table-001 표제 치환

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `title_cell.hwp`
- **지목 좌표**: (0,0)→표제
- **지목 연산자**: `cell_text_eq(0,0,0)==표제` · `differs_from_input`
- **요지**: 머리칸과 다른 표제로 같은 좌표를 다시 지목한다. 값 계약이 좌표와 분리된다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:title_cell.hwp} --json`
- **기준 풀이**: `reference/TB20.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB20 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB21 — 첫 표 옆칸 재교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `side_fix.hwp`
- **지목 좌표**: (0,1)→옆교정
- **지목 연산자**: `cell_text_eq(0,0,1)==옆교정` · `differs_from_input`
- **요지**: TB11 과 같은 좌표를 다른 값으로 지목한다. 과제 ID 가 값과 묶이지 않는다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:side_fix.hwp} --json`
- **기준 풀이**: `reference/TB21.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB21 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB22 — 실문서 머리 재명명

- **티어**: 3
- **축**: 편집
- **표본**: `samples/143E433F503322BD33.hwp`
- **산출**: `real_title.hwp`
- **지목 좌표**: (0,0)→실표머리
- **지목 연산자**: `cell_text_eq(0,0,0)==실표머리` · `differs_from_input`
- **요지**: 실문서 (0,0) 을 다른 표지로 바꾼다. 표본이 달라도 연산자는 같다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:real_title.hwp} --json`
- **기준 풀이**: `reference/TB22.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB22 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB23 — 첫 표 (0,0) 표지 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `mark_origin.hwp`
- **지목 좌표**: (0,0)→표지
- **지목 연산자**: `cell_text_eq(0,0,0)==표지` · `differs_from_input`
- **요지**: 원점 칸에 짧은 표지를 심는다. 전역 검색이 아니라 좌표 대조다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:mark_origin.hwp} --json`
- **기준 풀이**: `reference/TB23.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB23 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB24 — table-001 항목명 교정

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `item_name.hwp`
- **지목 좌표**: (0,0)→항목명
- **지목 연산자**: `cell_text_eq(0,0,0)==항목명` · `differs_from_input`
- **요지**: 분류 머리 자리를 항목명으로 바꾼다. CSV 왕복이 아니라 set-cell 이다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:item_name.hwp} --json`
- **기준 풀이**: `reference/TB24.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB24 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB25 — 첫 표 좌상·우상 쌍

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `left_right.hwp`
- **지목 좌표**: (0,0)→좌, (0,1)→우
- **지목 연산자**: `cell_text_eq(0,0,0)==좌 · cell_text_eq(0,0,1)==우` · `differs_from_input`
- **요지**: 첫 행 두 칸을 짧은 표지로 동시에 지목한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:left_right.hwp} --json`
- **기준 풀이**: `reference/TB25.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB25 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB26 — 첫 표 좌상·좌하 쌍

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `up_down.hwp`
- **지목 좌표**: (0,0)→상, (1,0)→하
- **지목 연산자**: `cell_text_eq(0,0,0)==상 · cell_text_eq(0,1,0)==하` · `differs_from_input`
- **요지**: 첫 열 두 칸을 짧은 표지로 동시에 지목한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:up_down.hwp} --json`
- **기준 풀이**: `reference/TB26.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB26 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB27 — 실문서 (0,0) 표제칸

- **티어**: 3
- **축**: 편집
- **표본**: `samples/143E433F503322BD33.hwp`
- **산출**: `real_heading.hwp`
- **지목 좌표**: (0,0)→표제칸
- **지목 연산자**: `cell_text_eq(0,0,0)==표제칸` · `differs_from_input`
- **요지**: 실문서 원점 칸을 표제칸으로 명명한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:real_heading.hwp} --json`
- **기준 풀이**: `reference/TB27.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB27 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB28 — table-001 분류칸

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `class_cell.hwp`
- **지목 좌표**: (0,0)→분류칸
- **지목 연산자**: `cell_text_eq(0,0,0)==분류칸` · `differs_from_input`
- **요지**: 원본 '구 분' 자리를 분류칸으로 치환한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:class_cell.hwp} --json`
- **기준 풀이**: `reference/TB28.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB28 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB29 — 첫 표 (1,0) 본문칸

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `body_cell.hwp`
- **지목 좌표**: (1,0)→본문칸
- **지목 연산자**: `cell_text_eq(0,1,0)==본문칸` · `differs_from_input`
- **요지**: 둘째 행 첫 열을 본문칸으로 지목한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:body_cell.hwp} --json`
- **기준 풀이**: `reference/TB29.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB29 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB30 — 첫 표 (0,1) 보조칸

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `aux_cell.hwp`
- **지목 좌표**: (0,1)→보조칸
- **지목 연산자**: `cell_text_eq(0,0,1)==보조칸` · `differs_from_input`
- **요지**: 첫째 행 둘째 열을 보조칸으로 지목한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:aux_cell.hwp} --json`
- **기준 풀이**: `reference/TB30.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB30 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB31 — 세 칸 지목 교정

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `triple.hwp`
- **지목 좌표**: (0,0)→가, (0,1)→나, (1,0)→다
- **지목 연산자**: `cell_text_eq(0,0,0)==가 · cell_text_eq(0,0,1)==나 · cell_text_eq(0,1,0)==다` · `differs_from_input`
- **요지**: 세 좌표를 각각 대조한다. 한 칸만 고친 제출은 탈락한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:triple.hwp} --json`
- **기준 풀이**: `reference/TB31.json` — set-cell 3회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB31 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB32 — table-001 머리표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `head_mark.hwp`
- **지목 좌표**: (0,0)→머리표지
- **지목 연산자**: `cell_text_eq(0,0,0)==머리표지` · `differs_from_input`
- **요지**: table-001 원점에 머리표지를 심는다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:head_mark.hwp} --json`
- **기준 풀이**: `reference/TB32.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB32 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB33 — 실문서 분류머리

- **티어**: 3
- **축**: 편집
- **표본**: `samples/143E433F503322BD33.hwp`
- **산출**: `real_class.hwp`
- **지목 좌표**: (0,0)→분류머리
- **지목 연산자**: `cell_text_eq(0,0,0)==분류머리` · `differs_from_input`
- **요지**: 실문서 원점을 분류머리로 바꾼다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:real_class.hwp} --json`
- **기준 풀이**: `reference/TB33.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB33 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB34 — 첫 표 (0,0) 좌표표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `coord_mark.hwp`
- **지목 좌표**: (0,0)→좌표표지
- **지목 연산자**: `cell_text_eq(0,0,0)==좌표표지` · `differs_from_input`
- **요지**: 원점에 좌표표지를 심어 지목 연산자를 재확인한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:coord_mark.hwp} --json`
- **기준 풀이**: `reference/TB34.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB34 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB35 — 첫 표 (0,1) 옆표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `side_mark.hwp`
- **지목 좌표**: (0,1)→옆표지
- **지목 연산자**: `cell_text_eq(0,0,1)==옆표지` · `differs_from_input`
- **요지**: 옆칸에 옆표지를 심는다. TB11·TB21 과 좌표는 같고 값이 다르다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:side_mark.hwp} --json`
- **기준 풀이**: `reference/TB35.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB35 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB36 — 첫 표 (1,0) 아래표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `below_mark.hwp`
- **지목 좌표**: (1,0)→아래표지
- **지목 연산자**: `cell_text_eq(0,1,0)==아래표지` · `differs_from_input`
- **요지**: 아래칸에 아래표지를 심는다. TB14·TB29 와 좌표는 같고 값이 다르다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:below_mark.hwp} --json`
- **기준 풀이**: `reference/TB36.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB36 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB37 — table-001 구분표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/table-001.hwp`
- **산출**: `class_mark.hwp`
- **지목 좌표**: (0,0)→구분표지
- **지목 연산자**: `cell_text_eq(0,0,0)==구분표지` · `differs_from_input`
- **요지**: 원본 구분 자리를 구분표지로 치환한다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:class_mark.hwp} --json`
- **기준 풀이**: `reference/TB37.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB37 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB38 — 실문서 현장표지

- **티어**: 3
- **축**: 편집
- **표본**: `samples/143E433F503322BD33.hwp`
- **산출**: `field_mark.hwp`
- **지목 좌표**: (0,0)→현장표지
- **지목 연산자**: `cell_text_eq(0,0,0)==현장표지` · `differs_from_input`
- **요지**: 실문서 원점에 현장표지를 심는다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:field_mark.hwp} --json`
- **기준 풀이**: `reference/TB38.json` — set-cell 1회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB38 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB39 — 가로쌍 표지

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `h_pair.hwp`
- **지목 좌표**: (0,0)→가로1, (0,1)→가로2
- **지목 연산자**: `cell_text_eq(0,0,0)==가로1 · cell_text_eq(0,0,1)==가로2` · `differs_from_input`
- **요지**: 가로 두 칸에 번호 표지를 심는다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:h_pair.hwp} --json`
- **기준 풀이**: `reference/TB39.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB39 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

#### TB40 — 세로쌍 표지

- **티어**: 4
- **축**: 편집
- **표본**: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- **산출**: `v_pair.hwp`
- **지목 좌표**: (0,0)→세로1, (1,0)→세로2
- **지목 연산자**: `cell_text_eq(0,0,0)==세로1 · cell_text_eq(0,1,0)==세로2` · `differs_from_input`
- **요지**: 세로 두 칸에 번호 표지를 심는다.
- **힌트 명령**: `rhwp edit set-cell`
- **채점 재조회**: `rhwp export-tables {file:v_pair.hwp} --json`
- **기준 풀이**: `reference/TB40.json` — set-cell 2회
- **금지**: `deep_contains` · `not_contains` · `fill-fields` · T07
- **원본 보존**: `-o` 로 산출을 분리한다. 표본 파일을 덮어쓰지 않는다.

TB40 의 채점은 지목한 칸만 본다. 다른 칸을 함께 고쳐도
그 자체로는 감점이 아니지만, 지목 칸이 비거나 값이 다르면 즉시 실패다.
전역 검색으로 "문자열이 문서 어딘가에 있다"를 증명하는 제출은
이 과제에서 점수를 받을 수 없다. 그것이 좌표 축이다.

기준 풀이는 정답을 사람이 외우게 하지 않는다. `edit set-cell` 을
지시서와 같은 좌표·같은 문자열로 호출할 뿐이다. 채점기는 산출물을
다시 `export-tables` 로 열어 그 좌표의 `text` 를 대조한다.
라이브 오라클과 기준 풀이가 같은 CLI 를 쓰는 것이 이 pack 의 왕복이다.

## 기준 풀이 왕복

저장소에 들어온 과제는 풀 수 있음이 실측된 과제여야 한다. 기준 풀이
(`reference/*.json`)가 그 선언이다. 짝이 없거나 id 가 다르거나 고아
기준풀이면 `python gym/tools/audit.py` 가 거부한다.

```bash
python gym/tools/build_baseline.py --agent baseline --pack table-editing
python gym/score.py --agent baseline --pack table-editing
```

조사 과제의 기준 풀이는 `answer` 블록이다. 경로는 과제의 `answer_eq.path`
와 같다. 편집 과제의 기준 풀이는 `run` 블록이다. 명령은 `edit set-cell`
이고 산출은 `{sub:파일}`. 여러 칸이면 중간 파일을 체인한다.

이 왕복을 바이너리 없이 선언만 검사하는 층이 `audit.py` 다.
스키마·짝·전역 ID 고유는 파일만으로 판정한다. CI 가 그 층을 상시 돈다.

## 채점과 제출

- 조사: `submissions/<이름>/table-editing/<TB>/answer.json`
- 편집: `submissions/<이름>/table-editing/<TB>/<산출.hwp>`
- 추출: `submissions/<이름>/table-editing/<TB>/<산출.csv>`

산출물 `.hwp` 는 커밋하지 않는다. `.gitignore` 가 막고, 재실행하면
누구나 같은 산출을 다시 만들 수 있다. 그 재생산 가능성이 검증 문화다.

```bash
python gym/score.py --agent <이름> --pack table-editing
```

프로파일 `editor` 가 이 pack 을 고른다. 점수는 pack 별로 보존된다.
총점은 편의값이다. 표 좌표가 약한지는 이 pack 점수가 말한다.

## 감사와 테스트

이 pack 을 손본 뒤에는 다음만 돌린다. Rust 포맷 게이트는 JSON·문서
변경에 해당 없다. `cargo fmt --all` 을 이 확장의 통과 조건으로 넣지 않는다.

```bash
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs -v
python -m unittest scripts.tests.test_gym_table_editing_pack -v
```

`scripts/tests/test_gym_table_editing_pack.py` 는 이 pack 전용 가드다.

- TB13+ 모든 과제가 `cell_text_eq` 를 가진다.
- 어떤 과제도 `deep_contains` 를 쓰지 않는다.
- T07.json 이 이 pack 에 없다.
- fill-fields 문자열이 과제·기준풀이에 없다.
- 과제와 기준풀이가 1:1 이고 id 가 같다.
- 표본은 위 세 경로만 쓴다.
- pack README 와 working 문서가 존재한다.

전 pack 계약은 계속 `test_gym_packs` 가 본다. 이 전용 테스트는 그 위에
표 좌표 축의 금지 목록을 고정한다.

## 확장 규칙 — 다음에 과제를 더할 때

1. 과제 ID 는 `TBnn` 이고 전역 고유해야 한다. T07 을 가져오지 마라.
2. 편집 과제는 `cell_text_eq` + `differs_from_input` 이다.
3. 조사 과제는 `answer_eq` 이고 숫자는 박제하지 마라.
4. 표본은 실측된 좌표만. 새 표본이면 `export-tables` 로 먼저 재라.
5. 기준 풀이를 같은 이름으로 남겨라. 고아 reference 는 audit 가 막는다.
6. `runner.*` 를 함부로 바꾸지 마라. 신원은 기준 왕복을 다시 돌린 뒤에만.
7. profiles / gym/README / PARK / checks.py 를 이 pack 확장의 편의를 위해
   고치지 마라. 지도는 존을 말하고, 과제 목록은 이 README 가 말한다.
8. 새 CLI 나 새 pack 이 필요해 보이면 이슈를 따로 연다. 이 pack 의 범위가 아니다.

## 관련

- 이슈: [#5230](https://github.com/edwardkim/rhwp/issues/5230)
- PR: [#5240](https://github.com/edwardkim/rhwp/pull/5240)
- 작업 기록: [`mydocs/working/gym_table_editing.md`](../../../mydocs/working/gym_table_editing.md)
- 전용 테스트: [`scripts/tests/test_gym_table_editing_pack.py`](../../../scripts/tests/test_gym_table_editing_pack.py)
- 스키마: `gym/core/schema.py` — 편집 축의 `GLOBAL_SCAN_OPS` 금지
- 연산자: `gym/core/checks.py` — `cell_text_eq` 정의
- 감사: `gym/tools/audit.py`

## 부록 — 좌표 축을 한 문장으로

표 편집의 정답은 "문서에 그 글자가 있다"가 아니라
"그 표의 그 행 그 열이 그 글자이다"다.

## 부록 — 과제 한 줄 지시

아래는 `tasks/*.json` 의 `instructions` 를 사람이 훑기 쉽게 옮긴 것이다.
채점 계약의 정본은 여전히 과제 JSON 이다. 이 표는 안내다.

| ID | 한 줄 |
|---|---|
| TB01 | 첫 표 행·열 수를 answer.json 으로 제출 |
| TB02 | 첫 표를 table.csv 로 추출 |
| TB03 | (0,0)=앞칸, (1,0)=뒷칸 인 cells.hwp |
| TB04 | (0,0)=왕복검증 인 roundtrip.hwp |
| TB05 | 표 개수를 answer.json 으로 제출 |
| TB06 | 두 번째 표 셀 개수를 answer.json 으로 제출 |
| TB07 | BOM 을 붙인 bom.csv |
| TB08 | 실문서 표 개수와 첫 표 행 수 |
| TB09 | 두 번째 표 행·열 수 |
| TB10 | 첫 표 셀 개수 |
| TB11 | (0,1)=옆칸 인 cells.hwp |
| TB12 | (0,0)=표머리 인 headed.hwp |
| TB13 | (0,0)=좌상 인 left_top.hwp |
| TB14 | (1,0)=좌하 인 left_bottom.hwp |
| TB15 | (0,1)=우상 인 right_top.hwp |
| TB16 | (0,0)=머리칸 인 head_cell.hwp |
| TB17 | (0,0)=실문서머리 인 real_head.hwp |
| TB18 | (0,0)=갑, (0,1)=을 인 row_pair.hwp |
| TB19 | (0,0)=전, (1,0)=후 인 col_pair.hwp |
| TB20 | (0,0)=표제 인 title_cell.hwp |
| TB21 | (0,1)=옆교정 인 side_fix.hwp |
| TB22 | (0,0)=실표머리 인 real_title.hwp |
| TB23 | (0,0)=표지 인 mark_origin.hwp |
| TB24 | (0,0)=항목명 인 item_name.hwp |
| TB25 | (0,0)=좌, (0,1)=우 인 left_right.hwp |
| TB26 | (0,0)=상, (1,0)=하 인 up_down.hwp |
| TB27 | (0,0)=표제칸 인 real_heading.hwp |
| TB28 | (0,0)=분류칸 인 class_cell.hwp |
| TB29 | (1,0)=본문칸 인 body_cell.hwp |
| TB30 | (0,1)=보조칸 인 aux_cell.hwp |
| TB31 | (0,0)=가, (0,1)=나, (1,0)=다 인 triple.hwp |
| TB32 | (0,0)=머리표지 인 head_mark.hwp |
| TB33 | (0,0)=분류머리 인 real_class.hwp |
| TB34 | (0,0)=좌표표지 인 coord_mark.hwp |
| TB35 | (0,1)=옆표지 인 side_mark.hwp |
| TB36 | (1,0)=아래표지 인 below_mark.hwp |
| TB37 | (0,0)=구분표지 인 class_mark.hwp |
| TB38 | (0,0)=현장표지 인 field_mark.hwp |
| TB39 | (0,0)=가로1, (0,1)=가로2 인 h_pair.hwp |
| TB40 | (0,0)=세로1, (1,0)=세로2 인 v_pair.hwp |

## 부록 — 왜 값이 과제마다 다른가

같은 좌표를 여러 과제가 지목한다. (0,0) 만 봐도 TB03·TB12·TB13·TB16
등 여럿이다. 값이 다르기 때문에 한 기준 풀이를 그대로 복사해 다른
과제에 제출하는 전략이 통하지 않는다. 좌표 축은 "어디를 고쳤는가"와
"무엇으로 고쳤는가"를 동시에 묻는다. 값까지 과제 ID 에 묶어야
복붙 제출이 점수를 훔치지 못한다.

같은 이유로 산출 파일 이름도 과제마다 다르다. `cells.hwp` 를 여러
과제가 공유하면 제출 폴더를 통째로 복사하는 꼼수가 생긴다.
TB13 이후는 산출 이름을 과제에 고정한다.

## 부록 — CLI 호출 뼈대

조사:

```bash
rhwp export-tables samples/table-001.hwp --json
```

한 칸 편집:

```bash
rhwp edit set-cell INPUT.hwp --table 0 --row 0 --col 0 --text 표지 -o OUT.hwp --json
```

CSV 추출:

```bash
rhwp table-to-csv INPUT.hwp --table 0 -o table.csv --json
rhwp table-to-csv INPUT.hwp --table 0 --bom -o bom.csv --json
```

채점 재조회:

```bash
rhwp export-tables OUT.hwp --json
```

이 네 호출이면 이 pack 의 모든 과제를 풀 수 있다. 더 많은 명령을
끌어오지 않는다. 특히 `fill-fields` 는 여기 없다.
