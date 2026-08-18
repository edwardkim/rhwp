# 실측 봉투 트랜스크립트

이 장은 저장소에 있는 표본으로 **이미 문서화된** 실행을 한곳에 모은다.
출처는 레시피 02, 코덱스 20장, playbook §10-5, 지식지도 §7-1,
`table_csv_contract.rs`, `table_extract_json_contract.rs`.

이 파동이 바이너리를 다시 돌리지 않았다. 숫자는 위 정본을 옮긴 것이다.
정본과 어긋나면 정본이 이긴다.

기계 사본: [../fixtures/transcripts/](../fixtures/transcripts/).

## 1. `samples/hwp_table_test.hwp` — 레시피 02 왕복

표 편집 기능 안내 문서. 표 10개. 0번 표는 3열×4행, 머리글만 있고 나머지 빈 칸.
모든 span=1. CSV 왕복에 적합.

### 1-1. export-tables

```bash
rhwp export-tables samples/hwp_table_test.hwp --json
```

```json
{
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "tableCount": 10,
  "tables": [
    {
      "cellCount": 12,
      "cells": [
        {"col": 0, "colSpan": 1, "isHeader": false, "row": 0, "rowSpan": 1, "text": "제목"},
        {"col": 1, "colSpan": 1, "isHeader": false, "row": 0, "rowSpan": 1, "text": "담당자"},
        {"col": 2, "colSpan": 1, "isHeader": false, "row": 0, "rowSpan": 1, "text": "세부 내용"},
        {"col": 0, "colSpan": 1, "isHeader": false, "row": 1, "rowSpan": 1, "text": ""}
      ],
      "cols": 3,
      "control": 0,
      "index": 0,
      "paragraph": 3,
      "rows": 4,
      "section": 0
    }
  ]
}
```

읽는 법: `--table 0`. `cols:3` `rows:4`. 병합 없음.

픽스처: [../fixtures/envelopes/export_tables_hwp_table_test.json](../fixtures/envelopes/export_tables_hwp_table_test.json).

### 1-2. table-to-csv --table 0

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0.csv --json
```

```json
{
  "bom": false,
  "output": "…/table0.csv",
  "outputFormat": "csv",
  "schemaVersion": "1.0",
  "source": "…/hwp_table_test.hwp",
  "tableCount": 1,
  "tables": [
    {
      "colCount": 3,
      "csv": "제목,담당자,세부 내용\r\n,,\r\n,,\r\n,,\r\n",
      "index": 0,
      "output": "…/table0.csv",
      "rowCount": 4
    }
  ],
  "untrustedContent": true,
  "untrustedFields": ["tables[].csv"]
}
```

파일 내용:

```csv
제목,담당자,세부 내용
,,
,,
,,
```

### 1-3. 외부 편집 (재현용)

```csv
제목,담당자,세부 내용
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기
```

픽스처 CSV: [../fixtures/csv/table0_edited.csv](../fixtures/csv/table0_edited.csv).

### 1-4. csv-to-table --verify

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 \
  -o table_updated.hwp --verify --json
```

```json
{
  "changed": [
    {"col": 0, "newText": "서버 이관", "oldText": "", "row": 1},
    {"col": 1, "newText": "홍길동", "oldText": "", "row": 1},
    {"col": 2, "newText": "1차 완료", "oldText": "", "row": 1},
    {"col": 0, "newText": "DB 백업", "oldText": "", "row": 2},
    {"col": 1, "newText": "김철수", "oldText": "", "row": 2},
    {"col": 2, "newText": "진행중", "oldText": "", "row": 2},
    {"col": 0, "newText": "문서 정리", "oldText": "", "row": 3},
    {"col": 1, "newText": "박영희", "oldText": "", "row": 3},
    {"col": 2, "newText": "대기", "oldText": "", "row": 3}
  ],
  "changedCount": 9,
  "changedPages": [0],
  "colCount": 3,
  "dryRun": false,
  "invalid": [],
  "output": "…/table_updated.hwp",
  "outputFormat": "hwp5",
  "rowCount": 4,
  "schemaVersion": "1.0",
  "table": 0,
  "verify": {"diffCount": 0, "identical": true}
}
```

`changedCount: 9` = 3열×3행. 헤더 3칸은 동일.

### 1-5. 재독

```bash
rhwp export-tables table_updated.hwp --json \
  | jq '.tables[0].cells[] | select(.row==1)'
```

```json
{"col": 0, "colSpan": 1, "isHeader": false, "row": 1, "rowSpan": 1, "text": "서버 이관"}
{"col": 1, "colSpan": 1, "isHeader": false, "row": 1, "rowSpan": 1, "text": "홍길동"}
{"col": 2, "colSpan": 1, "isHeader": false, "row": 1, "rowSpan": 1, "text": "1차 완료"}
```

픽스처: [../fixtures/envelopes/reread_export_tables_row1.json](../fixtures/envelopes/reread_export_tables_row1.json),
[../fixtures/transcripts/recipe02_roundtrip.json](../fixtures/transcripts/recipe02_roundtrip.json).

워크스루: [../examples/14_roundtrip_hwp_table_test.md](../examples/14_roundtrip_hwp_table_test.md).

## 2. `samples/table-001.hwp` — 병합 19×9 · 치수 거부

지식지도 §7-1: 표 1개, 19×9, 칸 131, 병합 20.
`table_extract_json_contract.rs` 가 가로·세로 병합을 모두 확인한다.

### 2-1. 구조

```json
{
  "schemaVersion": "1.0",
  "source": "samples/table-001.hwp",
  "tableCount": 1,
  "tables": [
    {
      "index": 0,
      "paragraph": 1,
      "rows": 19,
      "cols": 9,
      "cellCount": 131,
      "cells": [
        {"row": 0, "col": 0, "rowSpan": 1, "colSpan": 1, "text": "구 분"},
        {"row": 0, "col": 1, "rowSpan": 1, "colSpan": 3, "isHeader": true, "text": "5월"}
      ]
    }
  ]
}
```

`cellCount` 131 < 171. 병합 면적. CSV 되돌리기 금지. 추출은 가능.

픽스처: [../fixtures/envelopes/export_tables_table_001.json](../fixtures/envelopes/export_tables_table_001.json).

### 2-2. 잘못된 CSV (playbook §10-5 실측)

```
$ rhwp csv-to-table samples/table-001.hwp --csv out/bad.csv --table 0 --dry-run --json
{"changed":[],"changedCount":0,"colCount":9,"rowCount":19,
 "invalid":[{"actual":2,"expected":19,"reason":"rowCountMismatch",
             "message":"CSV 행 수 2 가 표 0 의 행 수 19 와 다릅니다 — 표 크기는 바꾸지 않습니다."},
            {"actual":2,"expected":9,"reason":"colCountMismatch","row":0},
            {"actual":2,"expected":9,"reason":"colCountMismatch","row":1}]}
exit=2
```

처방: `table-to-csv --table 0` 산출을 고친다. 직접 만들지 않는다.

픽스처: [../fixtures/transcripts/playbook_table001_mismatch.json](../fixtures/transcripts/playbook_table001_mismatch.json),
[../fixtures/envelopes/csv_to_table_table001_both_mismatch.json](../fixtures/envelopes/csv_to_table_table001_both_mismatch.json).

### 2-3. 덮인 칸 set-cell

playbook: `(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.`

`--col 1` (`5월`) 으로 다시 부른다.

## 3. `samples/basic/issue2007_nested_cell_pagination_42065.hwp` — 코덱스 20장

표 5개. 0번은 1열×2행 개요, 1번은 3열 규제 표.

### 3-1. export-tables (코덱스 원문)

```bash
rhwp export-tables samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
```

```json
{
  "schemaVersion": "1.0",
  "source": "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
  "tableCount": 5,
  "tables": [
    {
      "cellCount": 2,
      "cells": [
        {"col": 0, "colSpan": 1, "isHeader": false, "row": 0, "rowSpan": 1, "text": "Ⅰ. 규제 심사(안) 개요"},
        {"col": 0, "colSpan": 1, "isHeader": false, "row": 1, "rowSpan": 1, "text": "□ 요  약"}
      ],
      "cols": 1,
      "control": 4,
      "index": 0,
      "paragraph": 0,
      "rows": 2,
      "section": 0
    },
    {
      "cellCount": 6,
      "cols": 3,
      "control": 2,
      "index": 1,
      "paragraph": 1,
      "rows": 2,
      "section": 0,
      "cells": [
        {"col": 0, "colSpan": 1, "row": 0, "rowSpan": 1, "text": "규제 사무명"},
        {"col": 1, "colSpan": 1, "row": 0, "rowSpan": 1, "text": "현행 규제내용"},
        {"col": 2, "colSpan": 1, "row": 0, "rowSpan": 1, "text": "변경(또는 신설) 규제내용"}
      ]
    }
  ],
  "untrustedContent": true,
  "untrustedFields": ["tables[].cells[].text", "tables[].cells[].nested[]"]
}
```

`--table 0` 은 개요다. 규제 표를 원하면 `--table 1`.

### 3-2. table-to-csv --table 1

```bash
rhwp table-to-csv samples/basic/issue2007_nested_cell_pagination_42065.hwp --table 1 -o t1.csv --json
```

```json
{
  "bom": false,
  "output": "<tmp>/t1.csv",
  "outputFormat": "csv",
  "schemaVersion": "1.0",
  "source": "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
  "tableCount": 1,
  "tables": [
    {
      "colCount": 3,
      "index": 1,
      "rowCount": 2,
      "output": "<tmp>/t1.csv"
    }
  ],
  "untrustedContent": true,
  "untrustedFields": ["tables[].csv"]
}
```

CSV 첫 줄: `규제 사무명,현행 규제내용,변경(또는 신설) 규제내용`.
값에 쉼표가 있어 RFC 인용된다.

픽스처: [../fixtures/transcripts/codex20_issue2007.json](../fixtures/transcripts/codex20_issue2007.json),
[../fixtures/csv/issue2007_table1.csv](../fixtures/csv/issue2007_table1.csv).

코덱스 20장은 `csv-to-table` 표본 실행을 싣지 않는다(입력 합성 비용).
계약만 있다 — 이 스킬의 되돌리기 표본은 레시피 02 가 정본.

## 4. `samples/inner-table-01.hwp` — 중첩

지식지도: 최상위 1, 칸 14 중 1칸이 중첩 24칸.
`export_tables_expresses_nested_tables` 가 `nested` + `containerPath.kind==tableCell` 을 요구.

왕복: 바깥 `--table` (보통 0) 만. 안쪽은 v1 밖.

픽스처: [../fixtures/envelopes/export_tables_inner_table.json](../fixtures/envelopes/export_tables_inner_table.json).

## 5. `samples/basic/treatise sample.hwp` — 컨테이너

`info` 1개, `export-tables` 3개. 글상자·머리말에 `containerPath`.

CSV 왕복은 `containerPath` 없는 표만.

픽스처: [../fixtures/envelopes/export_tables_treatise_container.json](../fixtures/envelopes/export_tables_treatise_container.json).

## 6. `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`

30쪽, 표 53, 누름틀 0. `table_csv_contract.rs` 의 기본 표본.
본문 최상위 번호가 0에서 시작하지 않는다 — 0번은 머리말.

이 문서에서 `--table 0` 은 거의 항상 잘못이다.

계약이 여기서 확인하는 것:

- `table-to-csv --json` 기본 `bom: false`, `untrustedFields: ["tables[].csv"]`
- 병합 표 CSV 는 직사각
- `--bom` 은 파일만
- `--table 99999` → exit 1, stdout 0
- 인자 누락 → exit 2, stdout 0
- 동일 CSV 되돌리기 → `changedCount: 0`, `verify.identical: true`, `outputFormat: hwpx`
- 행/열 불일치·덮인 칸 값·깨진 CSV → `invalid` + exit 2, 파일 없음
- dry-run → 파일 없음, `changedPages: null`

픽스처: [../fixtures/envelopes/export_tables_jichi_index_not_zero.json](../fixtures/envelopes/export_tables_jichi_index_not_zero.json).

## 7. `samples/multi-table-001.hwp`

표 6개, 2쪽. `--table` 지목 연습.

```bash
rhwp export-tables samples/multi-table-001.hwp --json \
  | jq '[.tables[] | {index, rows, cols, section, paragraph}]'
rhwp table-to-csv samples/multi-table-001.hwp -o output/mt --json
# output/mt/table<index>.csv
```

픽스처: [../fixtures/envelopes/export_tables_multi_table.json](../fixtures/envelopes/export_tables_multi_table.json).

## 8. `samples/hwpx/basic-table-01.hwpx`

`cli_commands.md` 사용 예.

```bash
rhwp table-to-csv samples/hwpx/basic-table-01.hwpx --json \
  | jq '.tables[] | {index, rowCount, colCount}'
rhwp table-to-csv samples/hwpx/basic-table-01.hwpx --table 0 -o /tmp/표0.csv
rhwp csv-to-table samples/hwpx/basic-table-01.hwpx --csv /tmp/표0.csv --table 0 -o 작성본.hwpx --json
```

형식 보존: 입력 hwpx → `outputFormat: hwpx`.

## 9. `samples/복학원서.hwp` · `samples/추진일정.hwp`

- 복학원서: 표 3, 누름틀 0 → set-cell 축
- 추진일정: 표 1, 누름틀 1, 싼 왕복·렌더 표본 (`export-svg` 82KB)

누름틀이 있으면 `rhwp-form-fill` 과 역할을 나눈다. 표 칸 값은 이 스킬.

## 10. RFC 4180 왕복 (계약 트랜스크립트)

지자체 양식의 본문 최상위 병합 표에서:

1. `edit set-cell --table I --row 0 --col 0 --text '가,나"다'`
2. `table-to-csv --table I --json`
3. `tables[0].csv` 가 `"가,나""다"` 로 시작
4. 독립 판독기가 `가,나"다` 를 복원
5. 그 행 필드 수 = `colCount`

픽스처: [../fixtures/envelopes/table_to_csv_rfc4180.json](../fixtures/envelopes/table_to_csv_rfc4180.json).

## 11. BOM 왕복 (계약 트랜스크립트)

같은 표:

1. `table-to-csv --table I -o t.csv --bom --json`
2. 봉투 `bom: true`, `csv` 는 FEFF 로 시작하지 않음
3. 파일 바이트 `[0xEF, 0xBB, 0xBF, …]`

픽스처: [../fixtures/envelopes/table_to_csv_bom_file.json](../fixtures/envelopes/table_to_csv_bom_file.json).

## 12. 트랜스크립트를 읽는 규칙

- `source` 경로의 `…` 는 레시피 원문이 줄인 것. 실제는 저장소 상대 경로.
- 코덱스 표본의 배열은 2원소로 절단된다. `tableCount` 가 더 크다.
- `csv-to-table` 의 `untrustedContent` 는 변경 전 셀이 있을 때 true
  (`changed[].oldText`). 레시피 02 원문은 `false` 로 찍힌 판본이 있다 —
  지식지도·계약(`untrustedFields: ["changed[].oldText"]`)이 이긴다.
- 이 스킬 픽스처는 계약 쪽을 따른다.

## 13. 다음

워크스루 18편이 위 트랜스크립트를 한 장면씩 재현한다.
[../examples/README.md](../examples/README.md).
