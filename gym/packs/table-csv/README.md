---
kind: guide
status: active
canonical: gym/packs/table-csv/README.md
last_verified: 2026-08-18
---

# table-csv — 표 CSV 왕복 (되쓰기)

이 pack 은 표를 CSV 로 뽑아 고친 뒤 **같은 자리**에 되넣는 편집
축이다. `table-to-csv` ↔ `csv-to-table` 왕복이 본판이고,
`export-tables` 는 채점 재조회에만 쓴다. 새 CLI 는 없다. 표본은
이미 저장소 `samples/` 에 있는 파일만 쓴다.

채점은 라이브 오라클과 좌표 지목을 섞는다. 행 수·열 수·변경 칸 수는
`answer_eq` 가 채점 시점에 다시 센다. 되쓴 칸은 `cell_text_eq` 가
`(표, 행, 열)` 로 지목한다. 추출 CSV 는 `csv_cell_eq` 가 같은
좌표로 본다. 원본을 그대로 복사하면 `differs_from_input` 에서
실패다.

이 문서는 pack 내부 안내서다. `gym/README.md` · `gym/PARK.md` ·
`gym/profiles/` 는 이 확장에서 건드리지 않는다. `table-editing` 은
`edit set-cell` 좌표 축이고, 이 pack 은 **CSV 자산 왕복** 축이다.
둘을 합치지 않는다.

## 왜 이 pack 인가

한국 공문·통계표는 스프레드시트에서 고친 뒤 다시 한글로 들어간다.
에이전트가 이 왕복을 할 때 가장 흔한 실패는 여섯 가지다.

1. **치수를 바꾼다.** `csv-to-table` 은 표 크기를 바꾸지 않는다.
   행·열이 다르면 한 칸도 쓰지 않고 exit 2. 조용히 자르지 않는다.
2. **BOM 을 잊는다.** 엑셀이 한글을 깨뜨린다. `--bom` 은 파일 앞에
   UTF-8 BOM 을 붙인다. JSON 봉투의 `csv` 문자열에는 붙지 않는다.
3. **dry-run 없이 쓴다.** 잘못된 CSV 를 바로 적용하면 원본이 아닌
   산출이 더러워진다. `--dry-run` 은 파일을 쓰지 않고 `changed[]` 만
   보고한다.
4. **좌표를 전역 검색으로 확인한다.** 옆칸을 고쳐도 문자열이 있으면
   통과한다. 이 pack 은 `deep_contains` 를 쓰지 않는다.
5. **형식 쌍을 같다고 가정한다.** `.hwp` 재정표와 `.hwpx` 재정표는
   따로 채점한다.
6. **이미 같은 값을 다시 썼다고 착각한다.** 동일 격자는
   `changedCount == 0`. TC19 가 그 계약이다.

이 pack 의 과제는 위 구멍을 여정으로 나눈다.

## 하지 않는 것

1. **T07 을 복제하지 않는다.** 누름틀 채움은 core-cli 다.
2. **fill-fields 를 끌어오지 않는다.** 표 칸은 누름틀이 아니다.
3. **edit set-cell 을 본판정으로 쓰지 않는다.** 그건 `table-editing`.
   이 pack 의 기준 풀이는 `table-to-csv` / `csv-to-table` 만 부른다.
   채점 재조회만 `export-tables` 다.
4. **deep_contains 를 쓰지 않는다.** 편집 축이라 스키마가 막는다.
5. **새 CLI 를 만들지 않는다.**
6. **profiles / gym/README / PARK / checks.py 를 고치지 않는다.**
7. **치수를 늘리는 CSV 를 자산으로 두지 않는다.** 잘못된 치수는
   exit 2 이고, 이 pack 은 성공 경로와 선확인만 채점한다.

## 요구 capability

`pack.json` 의 `requires.commands` 는 `table-to-csv`, `csv-to-table`,
`export-tables` 다. 없으면 `unavailable`. 기준 실행 신원은 이
확장에서 바꾸지 않는다.

## 명령 계약

### table-to-csv

```
rhwp table-to-csv <파일> [--table <번호>] [-o <경로>] [--bom] [--json]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| `--table 0` | 본문 최상위 표, 0 기준 | TC02, TC04–TC06, TC10–TC15, TC17, TC18, TC22, TC23 |
| `--table 1` | 둘째 표 | TC25 |
| (생략) | 표 전부. `tableCount` | TC21 |
| `--bom` | 파일 앞에 UTF-8 BOM. 봉투 `csv` 에는 없음 | TC05, TC06, TC23 |
| `-o` | `--table` 과 함께면 파일, 없으면 폴더 | 추출 과제 |
| `--json` | `tables[].rowCount` / `colCount` / `tableCount` | 치수 과제 |

대상은 **본문 최상위 표뿐**이다. 중첩 표는 v1 범위 밖.
병합으로 덮인 칸은 빈 문자열로 채워 직사각 격자를 만든다. 앵커만
이어 붙이면 열이 밀린다.

### csv-to-table

```
rhwp csv-to-table <파일> --csv <경로.csv> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]
```

| 플래그 | 의미 | 대표 과제 |
|---|---|---|
| `--csv` | UTF-8 CSV. 치수 불일치면 exit 2 | 전 되쓰기 |
| `--table` | 0 기준, 필수 | 전 되쓰기 |
| `--dry-run` | 파일을 쓰지 않고 `changedCount` | TC01 채점, TC07, TC08, TC19, TC20 채점 |
| `--verify` | 저장 직후 IR 자기검증. 차이 시 exit 3 | TC24 |
| `-o` | 산출. 기본은 `<입력>_csv.<확장자>` | TC01, TC03, TC09, TC16, TC20, TC24 |

값이 실제로 달라지는 앵커 칸만 다시 쓴다. 무변경 칸은 서식을
보존한다. `edit set-cell` 과 달리 글자색을 검정으로 덮지 않는다.

성공 봉투: `changedCount`, `changed[{row,col,oldText,newText}]`,
`invalid:[]`, `dryRun`, `output?`, `verify?`.

선검증 실패 봉투: `changedCount: 0`, `invalid[{reason,row?,col?}]`,
exit 2.

### export-tables

채점 재조회 전용. `cell_text_eq` 가 `tables[t](r,c)` 를 지목한다.
에이전트 힌트에 `export-tables` 로 확인하라고 적을 수는 있으나,
기준 풀이는 되쓰기 뒤 채점기가 부를 뿐 풀이 단계로 넣지 않는다
(TC03·TC09·TC16·TC24).

## 표본과 자산

| 표본 | 쓰임 | 비고 |
|---|---|---|
| `samples/143E433F503322BD33.hwp` | TC01, TC08, TC14, TC18, TC20, TC23 | 실문서 재정표. |
| `samples/hwpx/143E433F503322BD33.hwpx` | TC12 | 같은 표 HWPX. |
| `samples/hwpx/basic-table-01.hwpx` | TC02, TC03, TC05, TC07, TC09, TC10, TC16, TC19, TC24 | 3×4 격자. (0,0)=1, (1,2)=7. |
| `samples/table-001.hwp` | TC04, TC06, TC11 | 첫 칸 `구 분`. |
| `samples/multi-table-001.hwp` | TC13, TC21, TC25 | 표가 여럿. `--table` 지목. |
| `samples/table-004.hwp` | TC15 | 다른 표 표본. |
| `samples/hwpx/table-text.hwpx` | TC17 | HWPX 표. |
| `samples/hwp_table_test.hwp` | TC22 | 또 다른 표 표본. |

자산은 `gym/packs/table-csv/assets/` 에 손수 둔다. 치수는 원본과
같다.

| 자산 | 하는 일 | 과제 |
|---|---|---|
| `TC01-edit.csv` | 2010년 적립금 328→999 | TC01, TC08 dry-run |
| `TC03-edit.csv` | (0,0) 1→100 | TC03, TC07 dry-run |
| `TC09-edit.csv` | (1,2) 7→77 | TC09 |
| `TC16-edit.csv` | (0,1) 2→200 | TC16 |
| `TC19-identity.csv` | 원본과 동일 1,2,3,4… | TC19 |
| `TC20-edit.csv` | 2010년 수입 50→55 | TC20 |
| `TC24-edit.csv` | TC03 과 같은 (0,0)=100 | TC24 `--verify` |

`basic-table-01` 원본 격자:

```
1,2,3,4
5,6,7,8
9,10,11,12
```

재정표 머리: `구분,적립금,수입,지출,비고`. 2010년 행은
`328,50,11`. TC01 은 적립금만 999, TC20 은 수입만 55.

## 여정 지도

### J1. 추출 격자 (`table-to-csv` + `csv_cell_eq`)

CSV 를 제출하고 좌표 칸을 대조한다. 무편집 복사는 거부한다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| TC02 | 격자 (0,0)=1, (1,2)=7 | basic-table-01 | csv_cell_eq |
| TC04 | (0,0)=구 분 | table-001 | csv_cell_eq |
| TC14 | (0,0)=구분 | 재정표 HWP | csv_cell_eq |

실패 모드:

- 문서 전체를 복사해 `table.csv` 로 이름을 바꾼다. `differs_from_input`.
- 표 1을 뽑는다. 칸 값이 다르다.
- 머리글만 본문에 붙여 넣는다. 좌표가 틀리다.

### J2. BOM (`--bom` + `utf8_bom`)

엑셀 한글 깨짐 방지. 파일 선두 3바이트가 EF BB BF.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| TC05 | BOM + (0,0)=1 | basic-table-01 | utf8_bom + csv_cell_eq |
| TC06 | BOM + (0,0)=구 분 | table-001 | 같음 |
| TC23 | BOM + (0,0)=구분 | 재정표 HWP | 같음 |

실패 모드:

- `--bom` 없이 UTF-8 만 낸다. `utf8_bom` 실패.
- BOM 을 첫 셀 값에 포함해 `﻿1` 로 읽는다. `csv_cell_eq` 는
  `utf-8-sig` 로 연다. 에이전트가 손으로 세면 BOM 을 값으로 볼 수
  있다. 채점기는 그렇게 보지 않는다.
- JSON 봉투의 `csv` 문자열에 BOM 이 있기를 기대한다. 코어는 안 붙인다.

### J3. 치수 계약 (`rowCount` / `colCount` / `tableCount`)

되쓰기의 전제. 숫자를 박제하지 않는다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| TC10 | 행·열 | basic-table-01 | tables[0].rowCount/colCount |
| TC11 | 행·열 | table-001 | 같음 |
| TC12 | 행 | 재정표 HWPX | rowCount |
| TC13 | 표 0 행 | multi-table-001 | --table 0 |
| TC15 | 행 | table-004 | rowCount |
| TC17 | 행 | table-text.hwpx | rowCount |
| TC18 | 행·열 | 재정표 HWP | 둘 다 |
| TC21 | 표 개수 | multi-table-001 | tableCount (`--table` 없음) |
| TC22 | 행 | hwp_table_test | rowCount |
| TC25 | 표 1 행 | multi-table-001 | --table 1 |

실패 모드:

- TC21 에 `--table 0` 을 붙인다. `tableCount` 가 1이 된다.
- TC13 과 TC25 를 같은 숫자로 제출한다. 표 0 과 표 1 의 행 수가
  같으리라는 보장은 없다.
- `export-tables` 의 `tables[0].rows` 를 가져온다. 같은 값일 수
  있으나 오라클은 `table-to-csv` 다.

### J4. 선확인 (`--dry-run`)

파일을 쓰지 않는다.

| ID | 하는 일 | 자산 | 지목 |
|---|---|---|---|
| TC07 | TC03-edit 적용 시 변경 칸 수 | TC03-edit.csv | answer_eq changedCount |
| TC08 | TC01-edit 적용 시 변경 칸 수 | TC01-edit.csv | 같음 |
| TC19 | 동일 격자 → 0 | TC19-identity.csv | value_eq 0 + json_value_eq 0 |

실패 모드:

- dry-run 없이 `-o` 로 써 버린다. 과제는 `answer.json` 만 받는다.
- 동일 격자에 1을 낸다. TC19 는 계약 0.
- `changed[]` 배열 길이를 손으로 세다 칸을 빠뜨린다. 오라클은
  `changedCount` 다.

### J5. 되쓰기 (`csv-to-table` + 좌표 재조회)

한 칸만 바꾸고 옆칸은 남긴다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| TC01 | 적립금 328→999 | 재정표 HWP | dry-run 재적용 changedCount=0 |
| TC03 | (0,0) 1→100, (1,2)=7 유지 | basic-table-01 | cell_text_eq |
| TC09 | (1,2) 7→77, (0,0)=1 유지 | 같음 | cell_text_eq |
| TC16 | (0,1) 2→200, 1과 7 유지 | 같음 | cell_text_eq 3점 |
| TC20 | 2010 수입 50→55 | 재정표 HWP | dry-run 재적용 0 |
| TC24 | TC03 + `--verify` | basic-table-01 | cell_text_eq |

실패 모드:

- 표 전체를 100으로 채운다. 옆칸 검사에서 거절.
- `edit set-cell` 로 한 칸만 고친다. 채점은 통과할 수 있으나 이
  pack 의 기준 풀이·HINT 는 CSV 왕복이다. 측정 축을 속이는 풀이다.
- 치수가 다른 CSV 를 만든다. exit 2, 산출 없음, `file_exists` 실패.
- `--verify` 없이 TC24 를 낸다. 값이 맞으면 채점은 통과한다.
  기준 풀이는 `--verify` 를 켠다. 에이전트 힌트에도 있다.

## 과제 목록 (TC01–TC25)

### TC01 — 표 값 되쓰기 — 재정표 적립금

기존. 실문서. 산출 `out.hwp`. 채점은 같은 CSV 를 다시 dry-run 해서
`changedCount==0` 인지 본다. 이미 999 이어야 한다.

### TC02 — 표 CSV 추출 — 격자 값 대조

기존. basic-table-01. (0,0)=1, (1,2)=7. 이 pack 의 격자 기준점.

### TC03 — 표 값 되쓰기 — 첫 칸만 100

기존. (0,0)=100, (1,2)=7. TC07·TC24 가 이 자산을 재사용한다.

### TC04 — table-001 첫 칸

`구 분` (가운데 공백). `구분` 으로 쓰면 실패다. TB07 과 같은
문자열이다.

### TC05 / TC06 / TC23 — BOM 추출

세 표본. 파일 선두 BOM + 첫 칸 값. BOM 만 있고 칸이 틀리면 실패,
칸만 맞고 BOM 이 없어도 실패.

### TC07 / TC08 — dry-run 변경 칸 수

라이브 `changedCount`. 숫자를 과제에 박제하지 않는다. TC03-edit 는
한 칸만 바꾸므로 보통 1에 가깝지만, 원본이 바뀌면 오라클이 따라간다.

### TC09 — (1,2) 만 77

TC03 의 반대 칸. 첫 칸을 건드리면 (0,0)=1 검사에서 거절.

### TC10 / TC11 / TC18 — 행·열 쌍

치수 계약의 본문. 되쓰기 과제가 전제로 삼는 숫자다.

### TC12 — 재정표 HWPX 행 수

TC01 의 형식 쌍. 행 수가 같으면 같고, 달라도 각각이 정답이다.

### TC13 / TC25 — 표 선택

같은 문서, `--table 0` 과 `--table 1`. 봉투는 선택된 표가
`tables[0]` 이다. `tables[1]` 경로를 읽으면 안 된다.

### TC14 — 재정표 머리 칸

추출. (0,0)=`구분`. TC04 의 `구 분` 과 다르다.

### TC15 / TC17 / TC22 — 다른 표 표본 행 수

표본을 넓힌다. 칸 값을 모르면 치수만 묻는다. 라이브 오라클.

### TC16 — (0,1) 만 200

세 칸을 지목한다. 고친 칸 하나, 남긴 칸 둘. 과잉 편집을 거른다.

### TC19 — 동일 CSV 변경 0

계약 값 0. `value_eq` 가 라이브 dry-run 을 보고, `json_value_eq` 가
제출 값도 0인지 본다. 둘 중 하나만 맞아도 부족하다.

### TC20 — 재정표 수입만 55

TC01 과 같은 표, 다른 칸. 적립금을 999 로 바꾸면 dry-run 재적용이
0이 아니다.

### TC21 — tableCount

`--table` 없이. 표가 여럿인 문서의 개수.

### TC24 — --verify 되쓰기

TC03 과 같은 값, 자기검증 켜짐. 값이 맞으면 IR 도 맞아야 한다.
exit 3 이면 기준 풀이가 실패한다.

## 채점 연산자

| 연산자 | CLI | 쓰임 |
|---|---|---|
| `file_exists` | 아니오 | 산출 CSV/문서 |
| `differs_from_input` | 아니오 | 무편집 복사 거부 |
| `csv_cell_eq` | 아니오 | 제출 CSV 좌표 |
| `utf8_bom` | 아니오 | BOM |
| `cell_text_eq` | `export-tables` | 되쓴 칸 |
| `value_eq` | `csv-to-table --dry-run` | changedCount 계약 |
| `answer_eq` | `table-to-csv` / dry-run | 라이브 치수·변경 수 |
| `json_value_eq` | 아니오 | TC19 제출 값 |

금지: `deep_contains`, `not_contains`, `fill-fields`.

편집 축이므로 `GLOBAL_SCAN_OPS` 는 스키마가 막는다.

## 명령 레시피

```bash
# 격자 추출
rhwp table-to-csv samples/hwpx/basic-table-01.hwpx --table 0 -o table.csv --json

# BOM
rhwp table-to-csv samples/table-001.hwp --table 0 --bom -o bom.csv --json

# 치수
rhwp table-to-csv samples/hwpx/basic-table-01.hwpx --table 0 --json \
  | jq '.tables[0] | {rowCount, colCount}'

# 표 개수
rhwp table-to-csv samples/multi-table-001.hwp --json | jq '.tableCount'

# 둘째 표
rhwp table-to-csv samples/multi-table-001.hwp --table 1 --json \
  | jq '.tables[0].rowCount'

# 선확인
rhwp csv-to-table samples/hwpx/basic-table-01.hwpx \
  --table 0 --csv gym/packs/table-csv/assets/TC03-edit.csv \
  --dry-run --json | jq '.changedCount'

# 되쓰기
rhwp csv-to-table samples/hwpx/basic-table-01.hwpx \
  --table 0 --csv gym/packs/table-csv/assets/TC03-edit.csv \
  -o out.hwpx --json

# 자기검증 켜고 되쓰기
rhwp csv-to-table samples/hwpx/basic-table-01.hwpx \
  --table 0 --csv gym/packs/table-csv/assets/TC24-edit.csv \
  -o out.hwpx --verify --json

# 재조회
rhwp export-tables out.hwpx --json \
  | jq '.tables[0].cells[] | select(.row==0 and .col==0) | .text'
```

채점 왕복:

```bash
python gym/tools/build_baseline.py --agent baseline --pack table-csv --bin target/debug/rhwp
python gym/score.py --agent baseline --pack table-csv --bin target/debug/rhwp
```

## 실패 모드 상세

### 치수 불일치

CSV 가 3×4 인데 4×4 로 저장하면 exit 2. 한 칸도 안 쓰인다.
`file_exists` 가 산출을 못 찾는다. 자산을 손으로 늘리지 마라.

### 병합 덮인 칸

앵커가 아닌 칸에 값을 넣으면 `coveredCellNotEmpty`. 이 pack 의
자산은 직사각 격자이고, basic-table-01 은 병합이 없다. 재정표는
병합이 있을 수 있다. 자산은 이미 통과한 TC01 격자를 변형한 것이다.

### 옆칸 오염

TC03/TC09/TC16 은 고친 칸과 남긴 칸을 같이 본다. 전역 치환으로
숫자를 일괄 변경하면 남긴 칸 검사에서 거절된다.

### HWP / HWPX 산출 확장자

TC01·TC20 은 `.hwp`, TC03·TC09·TC16·TC24 는 `.hwpx`. 입력과 같은
형식으로 저장해야 한다. `csv-to-table` 은 `-o` 확장자를 따른다.

### dry-run 인데 파일을 씀

TC07·TC08·TC19 는 `answer.json` 만 받는다. `out.hwpx` 를 만들어도
채점이 보지 않는다. 원본을 `--in-place` 로 건드리면 저장소를
더럽힌다. 힌트에 `--dry-run` 이 있는 이유다.

### BOM 과 첫 셀

엑셀에서 다시 저장하면 BOM 이 사라지거나 UTF-16 이 된다.
`utf8_bom` 은 3바이트만 본다. `csv_cell_eq` 는 `utf-8-sig` 로
열어 BOM 을 값에서 뺀다. 두 검사가 하는 일이 다르다.

### table-editing 과 축 혼동

`edit set-cell --table 0 --row 0 --col 0 --text 100` 은 값이
맞을 수 있다. 이 pack 은 CSV 왕복을 측정한다. 힌트와 기준 풀이가
그 축이다. `table-editing` 의 TB 과제와 ID 도 다르다.

## 커버리지와의 관계

```
[table-csv] csv-to-table, export-tables, table-to-csv
```

세 명령은 TC01–TC03 으로 이미 노출돼 있었다. 이번 확장은
`--bom` · `--dry-run` · `--verify` · `--table 1` · HWPX 쌍 ·
다른 표 표본을 격자화한다. 커버리지 퍼센트의 분모는 그대로다.

남는 빈 곳:

- 치수 불일치 exit 2 를 과제로 고정하기. 지금은 성공 경로만.
- 중첩 표 (v1 범위 밖).
- `table-to-csv` 폴더 출력(표 여러 개 → `table0.csv` …).
- 병합 덮인 칸 거부.

그 빈 곳은 의도적으로 남긴다. 실패 종료를 과제로 만들면 러너의
`expect_exit` 계약이 필요해지고, 이 확장은 그걸 열지 않는다.

## 스키마 불변식

`scripts/tests/test_gym_table_csv_pack.py` 가 CI 에서 다시 본다.

- 과제 id 는 `TCnn`, 기준 풀이와 1:1.
- 연산자 화이트리스트 위의 표.
- 명령 화이트리스트: `table-to-csv`, `csv-to-table`, `export-tables`.
- 자산 경로가 실재한다.
- `fill-fields` · `deep_contains` · 새 CLI 이름 부재.
- `runner` 신원 고정.
- 편집 산출물은 `differs_from_input` 을 가진다(answer 과제 제외).

## 재현

```bash
python gym/tools/build_baseline.py --agent baseline --pack table-csv --bin target/debug/rhwp
python gym/score.py               --agent baseline --pack table-csv --bin target/debug/rhwp
```

TC01–TC03 은 기존 축이다. TC04–TC25 는 같은 연산자·같은 CLI 로
BOM·dry-run·치수·형식 쌍·다른 칸을 늘린 확장이다. 새 pack 도,
새 CLI 도, T07 복제도 없다.

## 관련

- `table-editing` — `edit set-cell` 좌표. CSV 자산이 없다.
- `extraction` — 읽기만. 표를 되쓰지 않는다.
- `batch-ops` — 누름틀 메일머지. 표 칸이 아니다.
- `mydocs/manual/cli_commands.md` § table-to-csv / csv-to-table.
- `mydocs/working/gym_coverage_and_extract.md`.
