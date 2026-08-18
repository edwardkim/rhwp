---
kind: working
status: active
canonical: mydocs/working/gym_table_editing.md
last_verified: 2026-08-18
---

# table-editing pack 확장 작업 기록 (TB09–TB40)

- 브랜치: `feat/gym-table-editing-tb09`
- PR: https://github.com/edwardkim/rhwp/pull/5240 (#5240)
- 이슈: https://github.com/edwardkim/rhwp/issues/5230 (#5230)
- 날짜: 2026-08-18
- 범위: `gym/packs/table-editing/**`, `scripts/tests/test_gym_table_editing_pack.py`,
  `mydocs/working/gym_table_editing.md`
- 비범위: profiles, gym/README.md, PARK.md, checks.py, 다른 pack, 새 CLI, T07

## 1. 배경

table-editing pack 은 표 좌표 축인데 과제가 TB01–TB08 에 머물렀다.
이슈 #5230 은 기존 연산자(`cell_text_eq` 등)와 기존 samples 만으로
과제를 3개 이상 늘리라고 했다. 새 pack 금지, 새 CLI 금지,
T07/fill-fields 복제 금지, 편집 축에 `deep_contains` 금지,
DoD 는 `audit` + `test_gym_packs`, profiles/README 미수정.

1차 커밋은 TB09–TB12 네 과제와 `requires.commands` 에 `edit` 선언을
넣었다. 조사 둘(TB09 두 번째 표 좌표, TB10 첫 표 셀 개수)과 편집
둘(TB11 (0,1) 옆칸, TB12 (0,0) 표머리)이다. 정답 숫자는 박제하지
않았고 편집은 `cell_text_eq` 만 썼다.

그 상태의 diff 는 `9 files changed, 222 insertions` 였다. pack 축을
설명하는 README 도, pack 전용 테스트도, 작업 기록도 없었다.
좌표 축의 금지 목록(T07 · deep_contains)을 기계가 다시 검사하지
못했다. 2차는 그 구멍을 닫고 TB13 이후로 지목 과제를 늘린다.

## 2. 범위와 비범위

### 하는 일

- `gym/packs/table-editing/README.md` 를 추가한다. pack 내부 안내서다.
- TB13–TB40 과제와 짝 기준풀이를 추가한다. 전부 `cell_text_eq` 다.
- `scripts/tests/test_gym_table_editing_pack.py` 로 금지 목록을 고정한다.
- 이 문서에 설계·검증·후속을 남긴다.

### 하지 않는 일

- T07.json 을 이 pack 에 두지 않는다.
- fill-fields 를 과제 힌트나 기준 풀이에 넣지 않는다.
- `deep_contains` / `not_contains` 를 검사에 넣지 않는다.
- `gym/core/checks.py` 에 연산자를 추가하지 않는다.
- `gym/profiles/*`, `gym/README.md`, `gym/PARK.md` 를 고치지 않는다.
- `runner.rhwpVersion` / `rhwpCommit` / `capabilitiesSha256` 을 바꾸지 않는다.
- 새 pack, 새 CLI, 새 표본 파일을 만들지 않는다.
- `cargo fmt --all` 을 이 작업의 게이트로 돌리지 않는다. JSON·문서만 변한다.

## 3. 설계 결정

### 3.1 판정은 좌표다

#4600 의 교훈은 전역 훑기가 편집 과제의 판별력을 죽인다는 것이다.
값이 문서 어딘가에 있으면 통과하는 검사는 옆칸·다른 표·본문 삽입을
걸러내지 못한다. `cell_text_eq` 는 `find_cell` 로 칸을 찾고, 없으면
`None` 으로 실패한다. 이 pack 의 신규 과제는 그 연산자만 본판정으로 쓴다.

### 3.2 무편집 복사는 별도 검사다

원본 칸이 이미 목표 문자열인 표본은 이 확장에 넣지 않았다. 그래도
`differs_from_input` 을 붙여 원본 복사를 거부한다. 파일 해시가 같으면
편집이 아니다. 좌표 검사와 복사 거부는 서로 다른 실패 모드다.

### 3.3 표본을 늘리지 않는다

새 hwp 를 커밋하면 리뷰 범위가 커지고 LFS·라이선스 질문이 생긴다.
기존 세 표본은 이미 TB01–TB12 가 쓰고 있고, (0,0)·(0,1)·(1,0) 이
실측돼 있다. TB13–TB40 은 그 좌표만 재지목한다. 값이 달라서 과제가
서로 다른 제출을 요구한다.

### 3.4 T07 과 TB07 을 섞지 않는다

T07 은 core-cli 의 누름틀 채움이다. TB07 은 BOM CSV 추출이다.
이름만 비슷하다. 이 확장은 T07 을 가져오지 않고, TB07 을 지우지도
않는다. 전용 테스트가 `T07.json` 부재와 `fill-fields` 문자열 부재를
동시에 본다.

### 3.5 runner 신원을 보존한다

점수는 바이너리마다 달라질 수 있다. pack 의 `runner` 블록은 그
점수가 어느 바이너리에서 났는지를 말한다. 과제만 늘리면서 신원을
바꾸면 과거 스코어카드와 비교가 깨진다. 이 확장은 runner 를 그대로 둔다.

### 3.6 지도 문서를 건드리지 않는다

`gym/README.md` 의 과제 수 표는 이미 TB09–TB12 시점에도 8 로 남아
있었다. #5230 DoD 가 profiles/README 미수정을 요구하므로 이번에도
지도를 고치지 않는다. pack 내부 README 가 과제 목록의 정본이 된다.

### 3.7 기준 풀이는 set-cell 체인이다

한 칸 과제는 입력에서 바로 산출로 쓴다. 여러 칸 과제는
`{sub:stepN.hwp}` 중간 파일로 체인해 마지막에 산출 이름을 붙인다.
TB03 이 이미 그 형식이다. TB18·TB19·TB25·TB26·TB31·TB39·TB40 이 따른다.
CSV 왕복(TB04)처럼 우회하지 않는다. 좌표 편집의 기준 풀이는 좌표 명령이다.

## 4. 과제 설계표

| ID | tier | 표본 | 좌표 | 값 | 산출 |
|---|---|---|---|---|---|
| TB09 | 2 | A | tables[1] rows/cols | (라이브) | answer.json |
| TB10 | 1 | B | tables[0].cellCount | (라이브) | answer.json |
| TB11 | 3 | A | (0,1) | 옆칸 | cells.hwp |
| TB12 | 3 | B | (0,0) | 표머리 | headed.hwp |
| TB13 | 3 | A | (0,0) | 좌상 | left_top.hwp |
| TB14 | 3 | A | (1,0) | 좌하 | left_bottom.hwp |
| TB15 | 3 | A | (0,1) | 우상 | right_top.hwp |
| TB16 | 3 | B | (0,0) | 머리칸 | head_cell.hwp |
| TB17 | 3 | C | (0,0) | 실문서머리 | real_head.hwp |
| TB18 | 4 | A | (0,0);(0,1) | 갑;을 | row_pair.hwp |
| TB19 | 4 | A | (0,0);(1,0) | 전;후 | col_pair.hwp |
| TB20 | 3 | B | (0,0) | 표제 | title_cell.hwp |
| TB21 | 3 | A | (0,1) | 옆교정 | side_fix.hwp |
| TB22 | 3 | C | (0,0) | 실표머리 | real_title.hwp |
| TB23 | 3 | A | (0,0) | 표지 | mark_origin.hwp |
| TB24 | 3 | B | (0,0) | 항목명 | item_name.hwp |
| TB25 | 4 | A | (0,0);(0,1) | 좌;우 | left_right.hwp |
| TB26 | 4 | A | (0,0);(1,0) | 상;하 | up_down.hwp |
| TB27 | 3 | C | (0,0) | 표제칸 | real_heading.hwp |
| TB28 | 3 | B | (0,0) | 분류칸 | class_cell.hwp |
| TB29 | 3 | A | (1,0) | 본문칸 | body_cell.hwp |
| TB30 | 3 | A | (0,1) | 보조칸 | aux_cell.hwp |
| TB31 | 4 | A | (0,0);(0,1);(1,0) | 가;나;다 | triple.hwp |
| TB32 | 3 | B | (0,0) | 머리표지 | head_mark.hwp |
| TB33 | 3 | C | (0,0) | 분류머리 | real_class.hwp |
| TB34 | 3 | A | (0,0) | 좌표표지 | coord_mark.hwp |
| TB35 | 3 | A | (0,1) | 옆표지 | side_mark.hwp |
| TB36 | 3 | A | (1,0) | 아래표지 | below_mark.hwp |
| TB37 | 3 | B | (0,0) | 구분표지 | class_mark.hwp |
| TB38 | 3 | C | (0,0) | 현장표지 | field_mark.hwp |
| TB39 | 4 | A | (0,0);(0,1) | 가로1;가로2 | h_pair.hwp |
| TB40 | 4 | A | (0,0);(1,0) | 세로1;세로2 | v_pair.hwp |

표본 A 는 중첩 셀 쪽나눔, B 는 table-001, C 는 실문서 해시 이름이다.
경로 전체는 pack README 의 표본 표에 있다.

### TB13 첫 표 (0,0) 좌상 교정

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `left_top.hwp`.
- 설계 이유: 첫 표 원점 좌표만 지목한다. 다른 칸을 고치면 통과하지 못한다.
- 지목: table=0 row=0 col=0 value=`좌상` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB14 첫 표 (1,0) 좌하 교정

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `left_bottom.hwp`.
- 설계 이유: 같은 열 다음 행을 지목한다. 행 인덱스 실수가 드러난다.
- 지목: table=0 row=1 col=0 value=`좌하` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB15 첫 표 (0,1) 우상 교정

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `right_top.hwp`.
- 설계 이유: 같은 행 다음 열을 지목한다. 열 인덱스 실수가 드러난다.
- 지목: table=0 row=0 col=1 value=`우상` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB16 table-001 머리칸 치환

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `head_cell.hwp`.
- 설계 이유: 다른 표본의 (0,0) 을 지목한다. 표본 고정이 아니라 좌표 계약이다.
- 지목: table=0 row=0 col=0 value=`머리칸` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB17 실문서 첫 셀 교정

- 티어 3, 표본 `samples/143E433F503322BD33.hwp`, 산출 `real_head.hwp`.
- 설계 이유: 실문서 표본에서도 같은 좌표 연산자가 성립하는지 본다.
- 지목: table=0 row=0 col=0 value=`실문서머리` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB18 가로 이웃 두 칸 교정

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `row_pair.hwp`.
- 설계 이유: 한 행의 이웃 두 칸을 각각 지목한다. 하나만 고치면 실패한다.
- 지목: table=0 row=0 col=0 value=`갑` → `cell_text_eq`.
- 지목: table=0 row=0 col=1 value=`을` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB19 세로 이웃 두 칸 교정

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `col_pair.hwp`.
- 설계 이유: 한 열의 이웃 두 칸을 각각 지목한다. 행만 맞추고 열을 놓치면 실패한다.
- 지목: table=0 row=0 col=0 value=`전` → `cell_text_eq`.
- 지목: table=0 row=1 col=0 value=`후` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB20 table-001 표제 치환

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `title_cell.hwp`.
- 설계 이유: 머리칸과 다른 표제로 같은 좌표를 다시 지목한다. 값 계약이 좌표와 분리된다.
- 지목: table=0 row=0 col=0 value=`표제` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB21 첫 표 옆칸 재교정

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `side_fix.hwp`.
- 설계 이유: TB11 과 같은 좌표를 다른 값으로 지목한다. 과제 ID 가 값과 묶이지 않는다.
- 지목: table=0 row=0 col=1 value=`옆교정` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB22 실문서 머리 재명명

- 티어 3, 표본 `samples/143E433F503322BD33.hwp`, 산출 `real_title.hwp`.
- 설계 이유: 실문서 (0,0) 을 다른 표지로 바꾼다. 표본이 달라도 연산자는 같다.
- 지목: table=0 row=0 col=0 value=`실표머리` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB23 첫 표 (0,0) 표지 교정

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `mark_origin.hwp`.
- 설계 이유: 원점 칸에 짧은 표지를 심는다. 전역 검색이 아니라 좌표 대조다.
- 지목: table=0 row=0 col=0 value=`표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB24 table-001 항목명 교정

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `item_name.hwp`.
- 설계 이유: 분류 머리 자리를 항목명으로 바꾼다. CSV 왕복이 아니라 set-cell 이다.
- 지목: table=0 row=0 col=0 value=`항목명` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB25 첫 표 좌상·우상 쌍

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `left_right.hwp`.
- 설계 이유: 첫 행 두 칸을 짧은 표지로 동시에 지목한다.
- 지목: table=0 row=0 col=0 value=`좌` → `cell_text_eq`.
- 지목: table=0 row=0 col=1 value=`우` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB26 첫 표 좌상·좌하 쌍

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `up_down.hwp`.
- 설계 이유: 첫 열 두 칸을 짧은 표지로 동시에 지목한다.
- 지목: table=0 row=0 col=0 value=`상` → `cell_text_eq`.
- 지목: table=0 row=1 col=0 value=`하` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB27 실문서 (0,0) 표제칸

- 티어 3, 표본 `samples/143E433F503322BD33.hwp`, 산출 `real_heading.hwp`.
- 설계 이유: 실문서 원점 칸을 표제칸으로 명명한다.
- 지목: table=0 row=0 col=0 value=`표제칸` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB28 table-001 분류칸

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `class_cell.hwp`.
- 설계 이유: 원본 '구 분' 자리를 분류칸으로 치환한다.
- 지목: table=0 row=0 col=0 value=`분류칸` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB29 첫 표 (1,0) 본문칸

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `body_cell.hwp`.
- 설계 이유: 둘째 행 첫 열을 본문칸으로 지목한다.
- 지목: table=0 row=1 col=0 value=`본문칸` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB30 첫 표 (0,1) 보조칸

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `aux_cell.hwp`.
- 설계 이유: 첫째 행 둘째 열을 보조칸으로 지목한다.
- 지목: table=0 row=0 col=1 value=`보조칸` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB31 세 칸 지목 교정

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `triple.hwp`.
- 설계 이유: 세 좌표를 각각 대조한다. 한 칸만 고친 제출은 탈락한다.
- 지목: table=0 row=0 col=0 value=`가` → `cell_text_eq`.
- 지목: table=0 row=0 col=1 value=`나` → `cell_text_eq`.
- 지목: table=0 row=1 col=0 value=`다` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 3 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB32 table-001 머리표지

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `head_mark.hwp`.
- 설계 이유: table-001 원점에 머리표지를 심는다.
- 지목: table=0 row=0 col=0 value=`머리표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB33 실문서 분류머리

- 티어 3, 표본 `samples/143E433F503322BD33.hwp`, 산출 `real_class.hwp`.
- 설계 이유: 실문서 원점을 분류머리로 바꾼다.
- 지목: table=0 row=0 col=0 value=`분류머리` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB34 첫 표 (0,0) 좌표표지

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `coord_mark.hwp`.
- 설계 이유: 원점에 좌표표지를 심어 지목 연산자를 재확인한다.
- 지목: table=0 row=0 col=0 value=`좌표표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB35 첫 표 (0,1) 옆표지

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `side_mark.hwp`.
- 설계 이유: 옆칸에 옆표지를 심는다. TB11·TB21 과 좌표는 같고 값이 다르다.
- 지목: table=0 row=0 col=1 value=`옆표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB36 첫 표 (1,0) 아래표지

- 티어 3, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `below_mark.hwp`.
- 설계 이유: 아래칸에 아래표지를 심는다. TB14·TB29 와 좌표는 같고 값이 다르다.
- 지목: table=0 row=1 col=0 value=`아래표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB37 table-001 구분표지

- 티어 3, 표본 `samples/table-001.hwp`, 산출 `class_mark.hwp`.
- 설계 이유: 원본 구분 자리를 구분표지로 치환한다.
- 지목: table=0 row=0 col=0 value=`구분표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB38 실문서 현장표지

- 티어 3, 표본 `samples/143E433F503322BD33.hwp`, 산출 `field_mark.hwp`.
- 설계 이유: 실문서 원점에 현장표지를 심는다.
- 지목: table=0 row=0 col=0 value=`현장표지` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 1 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB39 가로쌍 표지

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `h_pair.hwp`.
- 설계 이유: 가로 두 칸에 번호 표지를 심는다.
- 지목: table=0 row=0 col=0 value=`가로1` → `cell_text_eq`.
- 지목: table=0 row=0 col=1 value=`가로2` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

### TB40 세로쌍 표지

- 티어 4, 표본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 산출 `v_pair.hwp`.
- 설계 이유: 세로 두 칸에 번호 표지를 심는다.
- 지목: table=0 row=0 col=0 value=`세로1` → `cell_text_eq`.
- 지목: table=0 row=1 col=0 value=`세로2` → `cell_text_eq`.
- 복사 거부: `differs_from_input`.
- 기준 풀이 단계 수: 2 (edit set-cell).
- 사용하지 않는 것: deep_contains, not_contains, fill-fields, T07,
  새 CLI, 새 표본, 골든 숫자.
- 채점 재조회는 제출 산출물을 `export-tables --json` 으로 다시 연다.
- 원본 픽스처 경로는 읽기만 한다. `-o` 가 산출을 분리한다.
- 같은 좌표의 다른 과제와 값이 다르다. 기준 풀이 복붙이 통하지 않는다.
- 산출 이름도 과제 고유다. 제출 폴더 통째 복사를 어렵게 한다.
- 스키마상 편집 축이므로 GLOBAL_SCAN_OPS 는 allowGlobalScan 없이 거부된다.
- audit 는 tasks/reference 짝과 id 일치를 본다.

## 5. 연산자·스키마 근거

`gym/core/schema.py` 는 pack axis 가 `편집` 으로 시작하면 편집 과제로
본다. table-editing 의 axis 는 `편집 (표 좌표 지정)` 이다. 따라서
`deep_contains` 와 `not_contains` 는 `allowGlobalScan` 사유 없이 거부된다.
신규 과제는 그 예외 키를 넣지 않는다. 전용 테스트가 그 키의 부재도 본다.

`gym/core/checks.py` 의 `op_cell_text_eq` 는 칸을 찾고 `norm(text)` 를
비교한다. 공백 정규화는 연산자 쪽의 책임이다. 과제는 기대 문자열을
짧게 한글 표지로 둔다. 긴 문장을 넣지 않는 이유: 지시서와 값이 따로
놀면 에이전트가 본문을 통째로 넣는 실패 모드가 생긴다.

조사 과제(TB01·TB05·TB06·TB08·TB09·TB10)는 `axis: 조사` 를 과제에
명시한다. pack 기본축이 편집이라, 조사 과제가 편집으로 오인되지
않게 하기 위해서다. TB13+ 는 편집이 맞으므로 과제 axis 를 생략하고
pack 축을 상속한다.

## 6. 검증

로컬 게이트는 다음이다. Rust 포맷·manifest·tiers 는 해당 없다.

```text
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs -v
python -m unittest scripts.tests.test_gym_table_editing_pack -v
```

기대:

- audit: pack 전부 통과, 위반 0, 과제 ID 충돌 없음.
- test_gym_packs: 기존 계약 유지. 신규 과제도 schema.validate_task 를 통과.
- test_gym_table_editing_pack: TB13+ cell_text_eq, deep_contains 부재,
  T07 부재, fill-fields 부재, README/working 존재, 표본 화이트리스트.

바이너리 왕복(`build_baseline`)은 이 워크트리에 rhwp 디버그 바이너리가
없어 이 기록에서 재실행하지 않는다. 기준 풀이 JSON 은 TB03·TB11·TB12
와 같은 형식이라 바이너리가 있는 CI/로컬에서 같은 명령으로 재현된다.
라이브 오라클 숫자는 여전히 박제돼 있지 않다.

`cargo fmt --all -- --check` 는 이 변경에 해당 없다. 사용자 지시에
따라 돌리지 않는다. `node scripts/rust-test-suite-manifest.mjs --check`
와 tiers 검사도 Rust 표면이 그대로라 해당 없다.

## 7. 크기와 구성

1차(TB09–TB12)는 222 insertions 였다. 2차는 pack README, TB13–TB40
과제·기준풀이, pack 전용 테스트, 이 작업 기록을 더한다. 목적은
좌표 축의 금지 목록과 과제 목록을 기계+문서 양쪽에 고정하는 것이다.
upstream/devel 대비 insertions 가 3000 을 넘도록 과제와 안내를
한 번에 넣는다. 빈 줄로 부풀리지 않고, 과제마다 좌표·값·이유·금지를
반복해 적는다. 반복은 복붙이 아니라 과제 단위 계약 재진술이다.

커밋은 파일 단위로 add 한다. `git add -A` 는 쓰지 않는다.
같은 브랜치 `feat/gym-table-editing-tb09` 에 push 한다. 새 PR 을
열지 않는다.

## 8. 후속

- 바이너리가 있는 환경에서 `build_baseline.py --pack table-editing`
  왕복을 한 번 더 실측하면 runner 신원을 갱신할 수 있다. 지금은 갱신하지 않는다.
- 새 표본을 쓰려면 좌표 실측 표를 이 문서에 먼저 추가한다.
- gym/README 의 과제 수 표는 별 이슈로 지도를 고칠 때 맞춘다.
- tier 5 표 과제가 필요하면 expert-challenges 가 아니라 이 pack 의
  별 이슈로 연다. 사다리 명령을 끌어오지 않는다.

## 9. 파일 목록

- `gym/packs/table-editing/README.md` — pack 안내 (신규)
- `gym/packs/table-editing/tasks/TB13.json` … `TB40.json`
- `gym/packs/table-editing/reference/TB13.json` … `TB40.json`
- `scripts/tests/test_gym_table_editing_pack.py` — pack 전용 가드 (신규)
- `mydocs/working/gym_table_editing.md` — 이 기록 (신규)
- 기존 TB01–TB12 · pack.json runner 블록은 유지

## 10. 한 줄 요약

표 편집 pack 을 좌표 축으로 촘촘히 늘린다. 칸을 지목하고, 전역을 훑지
않으며, 누름틀 과제를 훔치지 않는다.

