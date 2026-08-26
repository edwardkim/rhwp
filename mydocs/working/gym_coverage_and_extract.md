---
kind: working-note
status: active
canonical: mydocs/working/gym_coverage_and_extract.md
last_verified: 2026-08-18
related:
  - gym/tools/coverage.py
  - gym/packs/extraction/README.md
  - gym/packs/table-csv/README.md
  - gym/packs/batch-ops/README.md
  - scripts/tests/test_gym_coverage.py
---

# gym 커버리지 분모 정직성과 추출·표CSV·배치 확장

이 문서는 PR #5212 (`feat/gym-coverage-and-extract`) 의 작업 기록이다.
무엇을 재고, 무엇을 과제화했고, 무엇을 일부러 남겼는지를 한곳에 둔다.
팩 README 가 과제 안내서라면, 여기는 **분모와 빈 곳의 정직성** 기록이다.

새 CLI 는 없다. 새 pack 도 없다. `gym/README.md` · `PARK.md` ·
`profiles/` 는 건드리지 않았다. `coverage.py` 의 기존 키 의미는
그대로다. 격자(`packs`)와 미사용 연산자(`unusedOperators`)는 같은
봉투에 덧붙인 것이다.

## 1. 왜 분모가 문제인가

커버리지를 "명령 몇 %가 과제에 나왔는가"로 재면, 분모에 무엇을
넣느냐가 곧 정치가 된다. 진단 명령을 넣으면 숫자가 낮아지고, 빈
곳이 아닌 것이 빈 곳처럼 보인다. 그 오인이 이미 한 번 사고를
냈다. `fill-fields` 를 이미 커버한 core-cli T07 위에 중복 pack
(#4781)이 만들어졌다 자진 철회됐다. 만들기 전에 **진짜 빈 곳**만
재는 장치가 없었던 것이다.

`gym/tools/coverage.py` 는 그 재발을 막는다. 분모는 capabilities 의
`category` 가 `batch` · `edit` · `export` · `query` 인 명령만이다.
`diagnostic`(hwp5-* · dump-* 개발 probe) · `internal` · `serve`
(인프라)는 제외한다. 제외된 이름은 `excludedNonAgent` 로 보이되
`uncoveredByCategory` 에는 안 넣는다. 빈 곳 목록을 오염시키지
않기 위해서다.

한 명령은 과제 또는 기준풀이의 첫 토큰에 나타나면 '노출'이다.

- `checks[].cmd[0]`
- `steps[].run[0]`
- `steps[].answer.*.cmd[0]`

명령 합계만으로는 pack 축이 안 보인다. 같은 스캔이 pack×명령
격자를 낸다. 과제가 없는 pack 도 빈 행으로 남긴다. 누락과 빈
곳을 구분하기 위해서다.

`gym.core.checks.REGISTRY` 에 등록됐지만 어떤 과제의 `checks[].op`
에도 안 나온 이름은 `unusedOperators` 다. 기준풀이에는 checks 가
없으므로, 기준풀이만 있는 연산자는 사용으로 치지 않는다. 판정
어휘가 과제에 노출됐는지를 재는 것이다.

## 2. 정직한 분모의 불변식

불변식은 `scripts/tests/test_gym_coverage.py` 가 바이너리 없이
고정한다.

1. **에이전트-대면만 분모.** `query`/`edit`/`export`/`batch` 만
   `agentFacingTotal` 에 들어간다.
2. **진단은 빈 곳이 아니다.** `hwp5-inventory` 가 used 에 없어도
   `uncoveredByCategory` 에 안 나온다. `excludedNonAgent` 에만
   나온다.
3. **serve 도 같다.** `mcp-serve` 는 인프라다.
4. **internal 도 같다.** 비밀 probe 를 빈 곳으로 세지 않는다.
5. **알 수 없는 category 는 어디에도 안 넣는다.** 분모에도 제외
   목록에도 없다. 모르는 것을 빈 곳으로 승격하지 않는다.
6. **분모 0 은 100.** 잴 게 없으면 빈 곳도 없다. 0 나누기 대신
   100. 빈 used + 빈 분모도 100.
7. **빈 used + 비지 않은 분모는 0%.** 에이전트-대면이 있는데
   gym 이 하나도 안 부르면 커버 0.
8. **이름 겹침은 한 명령.** 같은 `name` 이 두 번 나와도 분모는
   1. category 가 `query` 와 `export` 로 갈라져도 이름은 하나다.
9. **분모 밖 used 는 covered 를 부풀리지 않는다.**
   `hwp5-inventory` 를 used 에 넣어도 `coveredCommands` 에 안
   들어간다.
10. **격자 행은 pack.json 폴더와 같다.** 스캔이 없는 pack 행을
    지어내지 않는다.
11. **격자 안 명령은 정렬·중복 없음.**
12. **미사용 연산자는 REGISTRY − 과제 op.** 기준풀이 op 는 안
    센다. 목록은 정렬된다.
13. **기존 키 의미는 그대로.** `kind=gymCoverage`,
    `schemaVersion=1.0`, `agentFacingTotal`, `covered`,
    `uncovered`, `coveragePercent`, `uncoveredByCategory`,
    `coveredCommands`, `excludedNonAgent`. 새 키는 `packs` 와
    `unusedOperators` 뿐이다.
14. **사람용 출력은 격자와 미사용 연산자를 빠뜨리지 않는다.**
    JSON 만 보면 사람이 빈 행을 놓친다.

이 열네 줄이 이 PR 의 커버리지 계약이다. 과제를 백 개 늘려도
분모 규칙은 바뀌지 않는다. 바뀌면 사고가 다시 난다.

## 3. 이번 확장이 분모에 하는 일 / 안 하는 일

하는 일:

- 이미 노출된 명령의 **종류·형식·표본 격자**를 촘촘히 한다.
- 미사용에 가깝던 플래그(`--bom`, `--dry-run`, `--verify`,
  `--name-field`, `--kind amount|number|all`, `--limit`)를
  과제로 고정한다.
- pack README 와 예외 테스트와 working 문서를 남긴다.

안 하는 일:

- 분모에 명령을 더하지 않는다. 새 CLI 가 없다.
- 제외 카테고리를 분모로 끌어오지 않는다.
- `coveragePercent` 를 올리려고 진단 과제를 만들지 않는다.
- `unusedOperators` 를 0으로 만들려고 억지 과제를 만들지 않는다.
  `deep_contains` 는 편집 축에서 금지이고, 조회 축에서도 경로
  지목이 더 정직하다. 미사용으로 남는 것이 맞다.
- profiles 에 신규 pack 을 끼워 점수를 뭉치지 않는다. 세 pack
  은 원래 있었다.

그래서 이 PR 을 "커버리지 % 올리기"로 읽으면 틀린다. 격자의
**빈 칸을 같은 명령으로 채운 것**이다. % 는 거의 그대로다.
그것이 정직하다.

## 4. 확장 전 상태 (얇은 pack)

확장 전 세 pack 은 살아 있었으나 격자가 얇았다.

### extraction (EX01–EX04)

| ID | 명령 | 묻는 것 |
|---|---|---|
| EX01 | chart-to-csv --chart 1 | rowCount |
| EX02 | export-text | pageCount |
| EX03 | extract-data --kind date | itemCount |
| EX04 | export-text -p 0 | pages[0].text 길이 |

금액·수량·전종류·limit·빈 결과·HWPX 쌍·다른 차트 가족이 없었다.

### table-csv (TC01–TC03)

| ID | 명령 | 묻는 것 |
|---|---|---|
| TC01 | csv-to-table | 재정표 적립금 999 |
| TC02 | table-to-csv | (0,0)=1, (1,2)=7 |
| TC03 | csv-to-table | (0,0)=100 |

BOM·dry-run 선확인·치수 계약·표 선택·HWPX 쌍·다른 칸·`--verify` 가
없었다. `--dry-run` 은 TC01 채점에만 쓰였다.

### batch-ops (BO01–BO03)

| ID | 조합 |
|---|---|
| BO01 | form-01.hwpx + CSV 3행 순번 |
| BO02 | form-01.hwpx + CSV --name-field myMsg01 |
| BO03 | form-01.hwp + JSONL 순번 |

`--dry-run` · `--verify` · `outname` 열 · form-02 · 형식 교차
(HWP5+CSV, HWPX+JSONL) · 1행/4행이 없었다.

세 pack 모두 명령 자체는 노출돼 있었다. 커버리지 도구가
`[extraction] chart-to-csv, export-text, extract-data` 를 찍어도
"금액 추출을 재봤는가"에는 답하지 못한다. 격자의 칸이 명령이지
플래그가 아니기 때문이다. 과제가 그 플래그를 물어야 한다.

## 5. 확장 후 지도 — extraction EX05–EX28

읽기 축. 연산자는 `answer_eq` / `len_answer_eq` 만. 제출은
`answer.json` 만.

### 5.1 종류 격자 (extract-data --kind)

| kind | 홍보문 | 시험지 HWP | 시험지 HWPX | 빈 HWPX |
|---|---|---|---|---|
| date | EX03, EX21(limit 1) | EX08 | EX09 | EX19 |
| amount | EX05 | EX10 | (EX26 all 에 포함) | EX20 |
| number | EX06 | EX28 | (EX26 all) | — |
| all | EX07 | — | EX26 | — |

EX23 은 별도 표본(`2010-01-06.hwp`)의 date.

빈 칸: 빈 HWPX 의 number/all, 시험지 HWP 의 all. 고의다. 0건
계약은 date/amount 두 종류로 충분하고, all 은 홍보문·시험지
HWPX 로 닫았다.

### 5.2 형식 쌍

| 축 | HWP | HWPX |
|---|---|---|
| 시험지 1쪽 날짜 | EX08 | EX09 |
| 시험지 2쪽 쪽수 | EX11 | EX12 |
| 차트 묶은세로막대 | EX01 | EX18 |
| 시험지 전종류 | (EX08+EX10+EX28) | EX26 |

같은 본문처럼 보여도 파서가 다르다. 답을 복사하면 안 된다.
오라클이 각 입력을 다시 돈다.

### 5.3 차트 가족

| 가족 | 과제 |
|---|---|
| 묶은세로막대형 | EX01 (HWP), EX18 (HWPX) |
| 누적세로막대형 | EX24 |
| 묶은가로막대형 | EX15 |
| 꺽은선형 | EX16 |
| 2차원원형 | EX17 |

차트 번호는 1부터. 표 `--table 0` 과 섞으면 실패다.

### 5.4 쪽수·글자 수

| 묻는 것 | 과제 |
|---|---|
| pageCount | EX02 홍보, EX11/12 2쪽, EX14 3쪽, EX22 table-001, EX27 4쪽 |
| 첫 쪽 글자 | EX04 1쪽, EX13 2쪽, EX25 3쪽 |

`info.pageCount` 와 섞지 않는다. 오라클은 `export-text` 다.

### 5.5 0건과 limit

EX19·EX20 은 빈 문서. `itemCount=0`, exit 0. 파이프라인이 0건을
오류로 승격하면 과제가 아니라 도구가 틀린다.

EX21 은 `--limit 1`. 묻는 것은 이번 응답 `itemCount` 다.
`totalItemCount` 를 내면 실패다. 절단 총량은 다른 과제다. 만들지
않았다.

### 5.6 extraction 이 남긴 빈 곳

- 항목 주소(`section`/`paragraph`/`page`/`charOffset`)를 묻는
  과제. 지금은 개수만.
- `normalized` 값 대조. 박제하면 표기 진화에 깨진다.
- `chart-to-csv --bom`, `--chart` 생략(전량).
- `export-text -p 1` (둘째 쪽).
- `csv-to-chart` 왕복 — `studio-e2e` ST01 의 축.
- 한글 수사 금액, 두 자리 연도 — 코어가 `null` 로 두는 범위.

주소 과제를 넣으려면 표본마다 좌표가 안정적이어야 한다. 홍보문은
실문서라 편집되면 좌표가 흔들린다. 개수 오라클이 더 정직하다.

## 6. 확장 후 지도 — table-csv TC04–TC25

편집 축. `deep_contains` 금지. 좌표 지목.

### 6.1 추출 격자

| ID | 표본 | 지목 |
|---|---|---|
| TC02 | basic-table-01 | (0,0)=1, (1,2)=7 |
| TC04 | table-001 | (0,0)=구 분 |
| TC14 | 재정표 HWP | (0,0)=구분 |

`구 분` 과 `구분` 은 다른 문자열이다.

### 6.2 BOM

TC05 basic-table-01, TC06 table-001, TC23 재정표. 파일 선두
EF BB BF + 첫 칸. 봉투 `csv` 문자열에는 BOM 이 없다.

### 6.3 치수

| ID | 표본 | 묻는 것 |
|---|---|---|
| TC10 | basic-table-01 | rows, cols |
| TC11 | table-001 | rows, cols |
| TC12 | 재정표 HWPX | rows |
| TC13 | multi-table 표 0 | rows |
| TC15 | table-004 | rows |
| TC17 | table-text.hwpx | rows |
| TC18 | 재정표 HWP | rows, cols |
| TC21 | multi-table | tableCount (선택 없음) |
| TC22 | hwp_table_test | rows |
| TC25 | multi-table 표 1 | rows |

`--table N` 을 주면 선택된 표가 봉투의 `tables[0]` 이다.
`tables[1]` 경로를 읽으면 안 된다. TC21 에 `--table` 을 붙이면
개수가 1이 된다.

### 6.4 dry-run

| ID | 의미 |
|---|---|
| TC07 | TC03-edit 적용 시 변경 칸 수 (라이브) |
| TC08 | TC01-edit 적용 시 변경 칸 수 (라이브) |
| TC19 | 동일 격자 → 계약 0 |

TC19 는 `value_eq` 와 `json_value_eq` 둘 다 0 이어야 한다.

### 6.5 되쓰기

| ID | 칸 | 검증 |
|---|---|---|
| TC01 | 적립금 328→999 | dry-run 재적용 0 |
| TC03 | (0,0)→100, (1,2)=7 | cell_text_eq |
| TC09 | (1,2)→77, (0,0)=1 | cell_text_eq |
| TC16 | (0,1)→200, 1과 7 유지 | cell_text_eq 3점 |
| TC20 | 수입 50→55 | dry-run 재적용 0 |
| TC24 | TC03 + --verify | cell_text_eq |

옆칸 오염을 거르는 것이 이 여정의 핵심이다. 전역 치환은 실패다.

### 6.6 자산 치수

`basic-table-01` 은 3×4.

```
1,2,3,4
5,6,7,8
9,10,11,12
```

재정표는 TC01 과 같은 5열 격자. 자산을 손으로 늘리면 exit 2.
이 pack 은 치수 불일치 실패를 과제로 두지 않았다. 러너의
`expect_exit` 를 이 확장에서 열지 않기 때문이다.

### 6.7 table-csv 가 남긴 빈 곳

- 치수 불일치 exit 2.
- 병합 덮인 칸 `coveredCellNotEmpty`.
- 중첩 표 (v1 범위 밖).
- `table-to-csv` 폴더 출력(표 여러 개).
- `edit set-cell` 혼합 — `table-editing` 의 축.
- 표 2 이상 (`--table 2`). multi-table 표본의 표 개수를 박제하지
  않으려고 0과 1만 골랐다.

## 7. 확장 후 지도 — batch-ops BO04–BO20

자동화 축. 산출물 + `search`. dry-run 만 `json_value_eq`.

### 7.1 형식 × 데이터 격자

```
           CSV                    JSONL
HWPX       BO01, BO05, BO07,      BO10, BO18
           BO13, BO14
HWP5       BO09                   BO03, BO08, BO20
```

이름 전략은 별 축이다.

```
순번              BO01, BO03, BO05, BO07–BO10, BO13, BO14, BO18, BO20
name-field 본문   BO02, BO12, BO15, BO17
name-field 열     BO06, BO16
dry-run           BO04, BO11, BO19
verify            BO05, BO15, BO20
```

서식은 form-01 / form-02, 둘 다 `myMsg01` 하나.

### 7.2 행 수

| 행 | 과제 |
|---|---|
| 1 | BO13 |
| 2 | 대부분이 2 |
| 3 | BO01, BO05, BO18 |
| 4 | BO14 |

1행은 "대량"의 하한이다. 0행은 레시피에만 있고 과제에 없다.

### 7.3 dry-run 채점의 정직성

`batch fill --json` 은 NDJSON 이다. gym 러너는 단일 JSON 만
파싱한다. 그래서 dry-run 을 `answer_eq` 라이브 오라클로 두지
않았다. 자산의 데이터 행 수를 `planned` 계약으로 둔다. 문서
파생 값이 아니라 손수 자산의 계약이다. 골든 쪽수와 다르다.
CSV 를 고치면 과제·기준(`const`)·테스트를 같이 고친다.

`--out-dir` 는 dry-run 에도 필수다. 선검증이 실행 명령줄에서
`--dry-run` 하나만 빼면 되게 하려는 코어 계약이다.

### 7.4 outname 과 notFound

`--name-field outname` 이면 봉투에 `notFound: ["outname"]` 이
실릴 수 있다. 누름틀이 아니기 때문이다. 본문 채움은 `myMsg01`
한 건이다. 채점기는 봉투를 보지 않고 본문을 검색한다. 파일
이름은 제출 경로로 본다.

### 7.5 T07 과 축

T07 은 단건 `fill-fields`. 이 pack 은 N부 `batch fill`. 힌트에
`fill-fields` 를 넣지 않는다. 값이 맞아도 축을 속인 풀이다.
테스트가 문자열을 금한다.

### 7.6 batch-ops 가 남긴 빈 곳

- `batch info` / `export-text` / `convert` (stdin 축). 러너가
  stdin 을 물리지 않는다. 별도 장치 없이 과제로 넣으면 기준
  풀이가 실패한다.
- 빈 데이터 0행.
- 이름 충돌 `_2`.
- 따옴표·쉼표 CSV (`"김철수, 대표"`). 레시피에 있고 과제에 없다.
  `search` 바늘이 흔들린다.
- `--threads`. 이름 예약 계약은 코어 테스트의 몫.
- 필드가 여럿인 서식. 표본이 `myMsg01` 하나다. 새 서식을 만들지
  않았다.

stdin 축을 넣으려면 러너 확장이 필요하다. 그것은 새 CLI 에
가깝고, 이 PR 의 금지 목록이다.

## 8. 커버리지 격자에 보이는 것 / 안 보이는 것

도구가 찍는 것:

```
[extraction] chart-to-csv, export-text, extract-data
[table-csv]  csv-to-table, export-tables, table-to-csv
[batch-ops]  batch, search
```

안 찍는 것:

- `--kind amount` 가 있는지
- `--bom` 이 있는지
- `--name-field outname` 이 있는지
- HWPX 쌍이 있는지
- 0건 계약이 있는지

격자 칸은 명령이다. 플래그·표본·형식은 과제 수와 README 가
말한다. 이 working 문서가 그 간극을 메운다. 커버리지 % 를
올리고 싶어서 명령을 더 노출한 것이 아니다. 이미 노출된
명령의 **안쪽**을 과제화한 것이다.

미사용 연산자 목록에 `deep_contains` 가 남는 것은 성공이다.
편집 축이 전역 훑기를 쓰면 #4600 이 다시 난다. 조회 축도
경로 지목이 더 낫다. 목록을 0으로 비우는 것이 목표가 아니다.

## 9. 표본 정책

새 픽스처를 추가하지 않았다. 쓴 경로는 전부 `samples/` 기존
파일이다.

extraction: 차트 가족 다섯, 홍보문, 시험지 1–4쪽 HWP/HWPX,
table-001, 2010-01-06, blank_hwpx.

table-csv: 재정표 HWP/HWPX, basic-table-01, table-001,
multi-table-001, table-004, table-text.hwpx, hwp_table_test.

batch-ops: form-01 / form-02 의 HWP/HWPX 네 파일.

한글 경로 차트 표본은 이미 저장소에 있다. 새 파일을 만들지
않았다. 자산 CSV/JSONL 만 pack `assets/` 에 손수 두었다. 치수와
센티넬은 이 문서와 README 에 적었다.

금지 표본: 이 세 pack 밖에서 쓰던 이슈 회귀 HWP 를 끌어와
축을 흐리지 않는다. `issue2007_nested_cell_pagination_42065.hwp`
는 `table-editing` / `text-editing` 의 것이다.

## 10. 테스트 피라미드 (바이너리 없음)

순수 테스트만 추가했다. rhwp 바이너리를 부르지 않는다.

### 10.1 test_gym_coverage.py

기존 분모 계약 + 격자 + 미사용 연산자 + CLI `--json`/`--capabilities`.
이번 추가:

- 알 수 없는 category
- 빈 used
- 빈 used + 빈 분모
- 이름 겹침 (같은 category / 다른 category)
- 분모 밖 used 가 covered 를 부풀리지 않음
- 실제 gym 스캔이 확장 pack 의 명령·연산자를 포함하는지

### 10.2 test_gym_extraction_pack.py

- EX01–EX04 잔존, EX05+ 존재
- 과제↔기준 1:1, 고아 없음
- op / cmd 화이트리스트
- `--kind` 값 집합, 차트 1 기준
- 표본 prefix
- `runner` 신원 고정
- 스키마 `validate_task`
- README / working 존재와 최소 길이

### 10.3 test_gym_table_csv_pack.py

- TC01–TC03 잔존
- 자산 경로 실재
- 편집 산출물의 `differs_from_input`
- BOM 과제의 `utf8_bom`
- 전역 훑기 금지
- `runner` 신원 고정

### 10.4 test_gym_batch_ops_pack.py

- BO01–BO03 잔존
- `batch` 의 둘째 토큰은 `fill` 뿐
- dry-run 은 `json_value_eq` planned, `const` 기준
- 산출 과제는 `search` + `matchCount >= 1`
- `--data` 자산 실재
- `fill-fields` 문자열 부재

`test_gym_packs.py` 의 전 pack 계약(유일 id, 기준 풀이, 이름 있는
검사, 스키마)도 신규 과제를 같이 본다. 여기 전용 테스트는 축
화이트리스트를 더 좁힌다.

바이너리 왕복(`build_baseline` + `score`)은 admission 이다. CI
기본 경로가 바이너리를 요구하지 않게 두었다. 로컬에서:

```
python gym/tools/build_baseline.py --agent baseline --pack extraction --bin target/debug/rhwp
python gym/score.py --agent baseline --pack extraction --bin target/debug/rhwp
```

table-csv · batch-ops 도 같다.

## 11. runner 신원

세 pack 의 `runner` 는 확장 전 값 그대로다.

- `rhwpVersion`: 0.8.4
- `rhwpCommit`: 4324eb0e4cf1a65f7efb305993a79ac44859a7ca
- `capabilitiesSha256`: 4767e61c3af751bb2f97af9d0b3e5ffa5cbb5dc70a89cf3ae85987132fa5473d

과제만 늘리면서 신원을 갈아끼우면 "이 점수가 어느 바이너리에서
났는가"가 거짓말이다. 테스트가 해시를 고정한다.

## 12. 하지 않은 것 (다시)

1. 새 CLI 동사. `extract-data2` 같은 것을 만들지 않았다.
2. 새 pack 폴더. 세 pack 안에 과제를 더했다.
3. `gym/README.md` 과제 수 표. 운동장 지도의 존 배치는 그대로.
   숫자가 어긋날 수 있다. 고의다. profiles/README 미수정과 같은
   이유 — 확장 PR 이 운동장 문서를 흔들지 않는다.
4. `PARK.md` · `INVITE.md` · `profiles/*.json`.
5. `checks.py` / `schema.py` / `runner.py`. 판정 어휘를 늘리지
   않았다. 있는 연산자만 골랐다.
6. `cargo fmt --all`. 러스트를 안 고쳤다.
7. 골든 숫자 박제 (dry-run planned 와 TC19 의 0 제외 — 자산
   계약).
8. 다른 PR 의 pack 을 살찌우기. 이 문서는 #5212 만.

## 13. 실패 모드 카탈로그 (교차)

세 pack 이 공유하는 착각.

### 13.1 형식 쌍을 같다고 가정

`.hwp` 답을 `.hwpx` 에 복사. 오라클이 각 입력을 다시 돈다.
같을 수도 있다. 달라도 각각이 정답이다.

### 13.2 0 기준 / 1 기준 혼동

- 표 `--table 0`, 차트 `--chart 1`, 쪽 `-p 0`, 메일머지 순번
  `0001` (1 기준, 4자리).
- 한 과제의 습관을 다음 과제에 가져가면 실패한다.

### 13.3 단건 도구로 대량을 흉내

`fill-fields` N번, `edit set-cell` 로 CSV 왕복 흉내. 값이 맞아도
축이 아니다. 힌트와 기준 풀이와 금지 문자열이 막는다.

### 13.4 dry-run 인데 파일을 씀

원본을 `--in-place` 로 더럽히면 저장소 문제다. 과제는
`answer.json` 만 받는다.

### 13.5 전역 훑기

`deep_contains` 는 편집 축에서 스키마가 막는다. 조회 축에서도
안 쓴다. `search` 의 `matchCount` 는 바늘이 있는지를 보는 것이지
문서 전체를 긁어 "어딘가에 있으면 통과"가 아니다. 산출 파일
하나를 연다.

### 13.6 커버리지 % 강박

명령을 하나 더 노출하려고 진단 과제를 만들면 분모 계약과
싸운다. 이 PR 은 % 를 올리지 않는다. 칸을 채운다.

## 14. 과제 수와 만점

pack 만점은 `sum(tier)` 다. 신규는 대부분 tier 2(조회·치수·
dry-run) 또는 tier 3(되쓰기·메일머지).

대략:

- extraction 28과제, 거의 tier 2 → 만점 ~56
- table-csv 25과제, 2와 3 혼재 → 만점 ~60
- batch-ops 20과제, dry-run 2 / 산출 3 → 만점 ~56

숫자는 과제가 늘면 같이 는다. profiles 가 이 pack 을 고르지
않으면 family/boss 점수에는 안 잡힌다. 고의다. 운동장 입구를
흔들지 않는다. pack 단독 왕복으로 admission 한다.

## 15. 레시피와 정본

명령의 정본은 `mydocs/manual/cli_commands.md` 다.

- extract-data § `--kind date|amount|number|all`, `--limit`,
  0건 = 성공, 정규화 규약.
- export-text § `-p` 0 기준.
- chart-to-csv § `--chart` 1 기준, 행=카테고리.
- table-to-csv § `--table` 0 기준, `--bom` 은 파일만.
- csv-to-table § 치수 계약, `--dry-run`, `--verify`, exit 2/3.
- batch fill § `--form`/`--data`/`--out-dir`, `--name-field`,
  `--dry-run` 에도 out-dir, NDJSON.

메일머지 실측은 `mydocs/manual/recipes/05_mail_merge_batch_fill.md`.
이 pack 의 자산 센티넬은 레시피의 한글 문장 대신 ASCII 식별자를
쓴다. `search` 가 줄바꿈에 흔들리지 않게.

## 16. 남은 빈 곳 — 우선순위

다음에 손댄다면, 새 CLI 없이, 러너를 바꾸지 않고 할 수 있는
것부터.

1. **extraction 항목 주소.** 안정 표본(시험지)에서
   `items[0].page` 를 `answer_eq` 로. 실문서 홍보문은 피한다.
2. **table-csv `--table 2`.** `tableCount` 가 3 이상인 표본을
   라이브로 확인한 뒤.
3. **batch-ops 쉼표 값.** `"김철수, 대표"` 를 본문에 넣고
   `search` 바늘을 `김철수` 로 짧게.
4. **export-text 둘째 쪽.** `-p 1` + `len_answer_eq`. 쪽 번호
   혼동을 한 번 더 고정.
5. **chart-to-csv 전량.** `--chart` 없이 `charts` 길이.

러너가 필요한 것 (이 PR 밖):

- stdin batch 축을 gym 에 넣기.
- NDJSON 을 `answer_eq` 로 읽는 장치 (dry-run batch 라이브화).
- `expect_exit` 과제로 치수 불일치 exit 2 를 고정.

진단 명령을 분모에 넣는 것은 **하지 않는다.** 그것이 이 문서의
첫 줄이다.

## 17. 커버리지 도구 사용

```
python gym/tools/coverage.py --bin target/debug/rhwp
python gym/tools/coverage.py --capabilities cap.json --json
```

`--bin` 과 `--capabilities` 둘 다 없으면 exit 2. 테스트가
고정한다. JSON 봉투의 키는 §2 의 목록과 같다. 사람용 출력은
미노출을 카테고리별로 찍고, 격자 행을 전부 찍고, 미사용
연산자를 한 줄로 찍는다. 빈 격자는 `(없음)`, 스캔 없으면
`(pack 스캔 없음)`.

측정 함수 `measure` 는 순수하다. 파일도 바이너리도 안 본다.
가드가 픽스처로 시험한다. `report` 는 capabilities 목록과
gym 스캔을 합친다. 역시 바이너리 없음.

`commands_in_doc` 은 빈 cmd, 비-dict answer, 비-spec 값을
무시한다. 깨진 과제 JSON 이 스캔을 죽이지 않게. 스키마
검증은 `test_gym_packs` 의 몫이다.

## 18. 작업 순서 (이 PR)

1. `coverage.py` 에 격자·미사용 연산자·사람용 출력.
2. 얇은 과제 EX03–EX04, TC02–TC03, BO02–BO03.
3. 이 문서가 기록하는 2차 확장: README 3종, EX05–EX28,
   TC04–TC25, BO04–BO20, 자산, 예외 테스트, working 노트.
4. 순수 테스트로 스키마·화이트리스트·자산 실재를 고정.
5. 분모 불변식에 엣지 케이스를 더함.

러스트 변경 없음. `cargo fmt` 없음. 같은 브랜치
`feat/gym-coverage-and-extract` 에만 쌓는다. 새 PR 을 열지
않는다.

## 19. 읽을 순서

에이전트가 이 pack 을 처음 보면:

1. `gym/README.md` 30초 입장 (운동장 규칙).
2. 이 문서 §2 분모, §3 하는 일/안 하는 일.
3. 해당 pack README 의 여정 지도.
4. 과제 JSON 의 `instructions`.
5. `reference/` 는 풀이 중에 보지 않는다. 채점 재현용이다.

유지보수자가 과제를 더 넣으면:

1. 이 문서 §16 남은 빈 곳.
2. 기존 과제 JSON 을 복사해 표본·플래그만 바꾼다.
3. 기준 풀이를 같은 명령으로 둔다. `const` 는 자산 계약에만.
4. pack 테스트의 화이트리스트·표본 목록을 갱신한다.
5. README 여정 표에 한 줄을 더한다.
6. `runner` 를 건드리지 않는다.
7. 새 CLI 이름 문자열 검사를 돌린다.

## 20. 한 줄 요약

분모는 에이전트-대면만, 진단은 빈 곳이 아니다. 세 pack 은
이미 있던 명령의 안쪽을 과제화했다. 새 CLI 는 없다. 0건은
성공이다. 좌표를 지목하라. 신원을 갈아끼우지 마라. 커버리지
% 를 올리려고 분모를 더럽히지 마라.

이 문서는 그 문장을 600줄로 풀어 쓴 것이다. 줄 수가 목표가
아니라, 다음에 과제를 늘릴 사람이 같은 실수를 하지 않게
빈 곳과 하지 않을 일을 적는 것이 목표다. 적지 않으면
#4781 이 다시 난다.

---

## 부록 A. 과제 ID 전체 목록

extraction: EX01 차트행, EX02 홍보쪽수, EX03 홍보날짜, EX04 1쪽글자,
EX05 홍보금액, EX06 홍보수량, EX07 홍보전종류, EX08 시험날짜HWP,
EX09 시험날짜HWPX, EX10 시험금액, EX11 2쪽HWP, EX12 2쪽HWPX,
EX13 2쪽첫글자, EX14 3쪽수, EX15 가로막대, EX16 꺽은선, EX17 원형,
EX18 세로막대HWPX, EX19 빈날짜0, EX20 빈금액0, EX21 limit1,
EX22 table001쪽수, EX23 20100106날짜, EX24 누적세로, EX25 3쪽첫글자,
EX26 시험HWPX전종류, EX27 4쪽수, EX28 시험수량.

table-csv: TC01 적립금999, TC02 격자추출, TC03 첫칸100,
TC04 table001첫칸, TC05 BOM기본표, TC06 BOM table001,
TC07 dry-run TC03, TC08 dry-run TC01, TC09 (1,2)=77,
TC10 치수기본표, TC11 치수table001, TC12 재정HWPX행,
TC13 다중표0, TC14 재정머리, TC15 table004행, TC16 (0,1)=200,
TC17 table-text행, TC18 재정HWP치수, TC19 동일0,
TC20 수입55, TC21 tableCount, TC22 hwp_table_test행,
TC23 BOM재정, TC24 verify100, TC25 다중표1.

batch-ops: BO01 순번3, BO02 name본문, BO03 HWP5 JSONL,
BO04 dry-run2, BO05 verify3, BO06 outname, BO07 form02 CSV,
BO08 form02 JSONL, BO09 HWP5 CSV, BO10 HWPX JSONL,
BO11 dry-run JSONL, BO12 HWP5 name, BO13 1행, BO14 4행,
BO15 verify+name, BO16 HWP5 outname, BO17 form02 name,
BO18 JSONL3, BO19 form02 dry-run, BO20 HWP5 verify.

## 부록 B. 허용 명령 화이트리스트 (이 PR)

```
extract-data
export-text
chart-to-csv
table-to-csv
csv-to-table
export-tables
batch
search
```

이 여덟 개 외의 첫 토큰이 세 pack 의 과제·기준에 있으면
테스트가 실패한다. `edit` 도 여기 없다. `fill-fields` 도 없다.
`info` 도 없다. 축을 지키려면 목록을 짧게 유지한다.

## 부록 C. 허용 연산자

extraction: `answer_eq`, `len_answer_eq`.

table-csv: `file_exists`, `differs_from_input`, `csv_cell_eq`,
`cell_text_eq`, `utf8_bom`, `value_eq`, `answer_eq`,
`json_value_eq`.

batch-ops: `file_exists`, `differs_from_input`, `value_ge`,
`json_value_eq`.

공통 금지: `deep_contains`, `not_contains` (편집 축은 스키마도
막음).

## 부록 D. 자산 파일

table-csv/assets:

- TC01-edit.csv (기존)
- TC03-edit.csv (기존)
- TC09-edit.csv
- TC16-edit.csv
- TC19-identity.csv
- TC20-edit.csv
- TC24-edit.csv

batch-ops/assets:

- BO01-data.csv (기존)
- BO02-data.csv (기존)
- BO03-data.jsonl (기존)
- BO06-data.csv
- BO07-data.csv
- BO08-data.jsonl
- BO09-data.csv
- BO10-data.jsonl
- BO12-data.csv
- BO13-data.csv
- BO14-data.csv
- BO15-data.csv
- BO16-data.csv
- BO17-data.csv
- BO18-data.jsonl
- BO20-data.jsonl

extraction 은 자산이 없다. 읽기만 한다.

## 부록 E. 분모 시나리오 표

| commands | used | agentFacing | covered | % | 비고 |
|---|---|---|---|---|---|
| query info, edit csv-to-table, diag x, serve y | {info} | 2 | 1 | 50 | 기본 픽스처 |
| diag only | {} | 0 | 0 | 100 | 분모 0 |
| [] | {} | 0 | 0 | 100 | 빈 입력 |
| 픽스처 | {} | 2 | 0 | 0 | 빈 used |
| info×2, search query+export | {info,search} | 2 | 2 | 100 | 이름 겹침 |
| info + mystery unknown | {info,mystery} | 1 | 1 | 100 | mystery 무시 |
| 픽스처 | {info, diag, serve} | 2 | 1 | 50 | 분모 밖 used 무시 |

이 표의 행이 테스트 이름과 1:1 로 대응한다. 행을 더할 때
테스트를 더한다. 테스트 없이 표를 고치지 마라.

## 부록 F. 관련 이슈·PR

- #4781 중복 fill-fields pack 철회 — 분모 도구의 동기.
- #4600 전역 훑기 오검출 — 편집 축 좌표 지목.
- #4653 pack 스키마·라이브 오라클.
- #4689 core-cli 기준 풀이 전건.
- #3719 extract-data / table-csv / batch fill 도입.
- #4100 chart-to-csv.
- #5208 coverage 격자·미사용 연산자 (이 브랜치 1차).
- #5212 이 문서가 기록하는 확장.
- #5230 / #5240 table-editing 과제 확장 — README·테스트 형식의
  거울. 이 PR 은 그 형식을 세 pack 에 적용했다. 그 PR 의
  과제를 건드리지 않는다.

## 부록 G. 자기점검 질문

과제를 하나 더 넣기 전에 묻는다.

1. 이미 같은 명령·같은 플래그·같은 표본이 있는가? 있으면 만들지
   않는다. #4781.
2. 새 동사가 필요한가? 필요하면 이 PR 이 아니다.
3. 분모에 진단이 들어가는가? 들어가면 도구를 깨는 것이다.
4. 숫자를 박제하는가? 라이브 경로가 있으면 박제하지 않는다.
5. 편집인데 `deep_contains` 인가? 스키마가 거절한다.
6. 자산 치수가 원본과 같은가? 다르면 exit 2.
7. 기준 풀이가 같은 명령을 부르는가? 안 부르면 재현이 아니다.
8. 테스트 화이트리스트를 갱신했는가?
9. `runner` 해시를 바꿨는가? 바꿨으면 되돌린다.
10. 다른 브랜치·다른 PR 을 살찌웠는가? 이 문서는 #5212 만.

열 질문에 모두 "문제 없음"이어야 커밋한다.

끝.
