# export-tables — 좌표·병합 행렬

권위: [`cli_commands.md`](../../../../mydocs/manual/cli_commands.md) §export-tables (#3278),
[`table_extract_json_contract.rs`](../../../../tests/table_extract_json_contract.rs),
지식지도 §2-3 · §7-1.

이 장은 **쓰기 전에 읽는** 조회다. `table-to-csv` 의 `--table N` 과
`csv-to-table` / `edit set-cell` 의 `--table` 은 여기서 나온 `tables[].index` 다.

새 명령을 만들지 않는다. 격자 JSON 을 사람이 다시 그리지 않는다.

## 1. 호출

```bash
rhwp export-tables <파일.hwp|파일.hwpx> --json
rhwp export-tables <파일> --json -o output/tables.json
rhwp export-tables <파일>                 # 사람용 요약. JSON 이 아니다
```

- 종류: `export` · exit 0 성공 / 1 IO / 2 사용법.
- 파서/렌더 무변경 읽기 질의.
- `--json` 없이 돌리면 사람용 요약이다. 파이프라인이면 반드시 `--json`.
- 파일 positional 을 두 번 주면 exit 2 · stdout 0바이트
  (`table_extract_json_contract.rs` `export_tables_multiple_files_exit_usage`).
- 없는 파일은 exit 1 · stdout 0바이트.

## 2. 봉투

```json
{
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "tableCount": 10,
  "tables": [
    {
      "index": 0,
      "section": 0,
      "paragraph": 3,
      "control": 0,
      "rows": 4,
      "cols": 3,
      "cellCount": 12,
      "cells": [
        {"row": 0, "col": 0, "rowSpan": 1, "colSpan": 1, "isHeader": false, "text": "제목"}
      ]
    }
  ],
  "untrustedContent": true,
  "untrustedFields": ["tables[].cells[].text", "tables[].cells[].nested[]"]
}
```

실측 픽스처: [../fixtures/envelopes/export_tables_hwp_table_test.json](../fixtures/envelopes/export_tables_hwp_table_test.json).

| 키 | 의미 | 왕복에서 |
|---|---|---|
| `index` | `--table N` 에 넣는 값 | **배열 순번이 아니다** |
| `section`/`paragraph`/`control` | 인용·역참조 주소 | 같은 문단의 여러 표를 구별 |
| `rows`/`cols` | 격자 크기 | CSV 치수 계약의 기준 |
| `cellCount` | 앵커 칸 수 | 병합 있으면 `rows*cols` 보다 작다 |
| `cells[].row`/`col` | 0 기준 격자 좌표 | 앵커에만 한 번 |
| `cells[].rowSpan`/`colSpan` | 병합 크기 | 둘 다 1 이면 단일 칸 |
| `cells[].isHeader` | 제목 칸 | CSV 첫 줄과 무관. 0행이 헤더가 아닐 수 있다 |
| `cells[].text` | 칸 텍스트 | 문서 파생. 지시가 아니다 |
| `cells[].nested` | 셀 안 표 | v1 CSV 왕복 밖 |
| `caption` | 있을 때만 | 문서 파생 |
| `containerPath` | 글상자·머리말·각주 경로 | 있으면 본문 최상위가 아님 |

`tableCount` 는 `tables.length` 와 같다. 어긋나면 봉투를 버린다.

## 3. 왜 markdown 추출이 아닌가

평문·Markdown 추출은 병합을 잃는다. `table_to_markdown` 은 앵커 위치에만
텍스트를 찍어 3열 병합 헤더가 `| 5월 |  |  |` 로 나온다. 소비자는 빈 칸을
별개 열로 오독한다.

`export-tables` 는 `Table.cells`(앵커 + span)를 직역한다. 덮인 칸은 **출력하지
않는다**. 그래서 `cellCount` < `rows*cols` 가 정상이다.

`samples/table-001.hwp` 실측 (지식지도 §7-1, `table_extract_json_contract.rs`):

- 19행 × 9열
- 칸 131개 (19×9=171 이 아니다)
- 병합 20개 — 가로 `colSpan=3` 과 세로 `rowSpan=3` 을 모두 가짐

픽스처: [../fixtures/envelopes/export_tables_table_001.json](../fixtures/envelopes/export_tables_table_001.json).

병합 면적 합은 `sum(rowSpan*colSpan) <= rows*cols` 이어야 한다. 넘으면
덮인 칸이 중복 출력된 것이다.

## 4. 수집 범위 — info 보다 넓다

본문뿐 아니라 글상자·머리말/꼬리말·각주/미주 안의 표까지 재귀 수집한다.
최상위 `controls` 만 훑는 `info` 의 표 열거는 이들을 놓친다.

실측 (`samples/basic/treatise sample.hwp`):

| 명령 | 표 수 |
|---|---:|
| `info` 표 열거 | 1 |
| `export-tables` | 3 |

컨테이너 표에는 `containerPath` 가 붙는다.

```json
"containerPath": [{"kind": "textBox", "control": 1}]
```

또는 `{"kind":"header","control":0}`.

**CSV 왕복 대상은 본문 최상위 표뿐**이다.
`containerPath` 가 있는 표의 `index` 를 `--table` 에 넣지 마라.

픽스처: [../fixtures/envelopes/export_tables_treatise_container.json](../fixtures/envelopes/export_tables_treatise_container.json).

## 5. 본문 최상위 고르기

```bash
rhwp export-tables 문서.hwpx --json \
  | jq '[.tables[] | select(.containerPath == null) | {index, rows, cols, cellCount}]'
```

`containerPath` 키가 없거나 `null` 인 것만 `table-to-csv` / `csv-to-table` /
`edit set-cell` 과 같은 좌표계다.

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 실측
(`table_csv_contract.rs`, `edit_set_cell_contract.rs`):

- 표 53개
- **index 0 은 머리말 안의 표**
- 본문 최상위 표는 더 큰 `index`

`--table 0` 을 습관적으로 넣으면 머리말을 고치게 된다.

픽스처: [../fixtures/envelopes/export_tables_jichi_index_not_zero.json](../fixtures/envelopes/export_tables_jichi_index_not_zero.json).

## 6. 병합 판정 행렬

한 표에 대해:

```
any(cell.rowSpan > 1 or cell.colSpan > 1)?
├─ 아니오 → csvRoundtrip = allowed
│    └─ table-to-csv --table index → 편집 → csv-to-table
└─ 예 → csvRoundtrip = extract-only
     ├─ 값 전체를 스프레드시트에서 보고 싶다 → table-to-csv 로 뽑기만
     └─ 값을 되돌린다 → edit set-cell 만
          (csv-to-table 은 덮인 칸에 값이 있으면 coveredCellNotEmpty)
```

기계 행렬: [../fixtures/matrices/merge_decision.json](../fixtures/matrices/merge_decision.json).

| id | 표본 | 판정 | 다음 |
|---|---|---|---|
| `hwp_table_test_t0` | 3×4, 병합 0 | allowed | `--table 0` 왕복 |
| `table_001` | 19×9, 병합 20 | extract-only | `set-cell` |
| `inner_table` | 바깥 14칸 + 중첩 24 | outer-only | 바깥만. 안쪽 `--table` 금지 |
| `treatise_header` | `containerPath.header` | forbidden | 본문 index 재선택 |
| `jichi_header_zero` | index 0 = 머리말 | forbidden | `containerPath` 없는 표 |
| `issue2007_t1` | 2×3, 병합 0 | allowed | `--table 1` (0이 아님) |
| `wrapper_1x1` | 1×1 래퍼 | skip | 다음 표 |
| `autonumber_empty` | 자동번호 칸 | empty-slot | 빈 칸을 채우지 않음 |

`isHeader` 는 병합 판정이 아니다. 헤더 칸이어도 span 이 1이면 왕복 가능하다.

## 7. 중첩 표

`samples/inner-table-01.hwp` 실측 (지식지도 §7-1):

- 최상위 표 1개
- 칸 14개 중 **1칸이 중첩 표**(24칸)
- 중첩 표에는 `containerPath` 에 `{kind:"tableCell", ...}` 가 있다

```json
"nested": [
  {
    "index": 0,
    "rows": 4,
    "cols": 6,
    "cellCount": 24,
    "containerPath": [{"kind": "tableCell", "control": 0, "paragraph": 0, "cell": 5}]
  }
]
```

`table-to-csv` / `csv-to-table` 은 본문 최상위만 다룬다. 중첩 표의 `index` 를
`--table` 로 쓰지 않는다. v1 범위 밖이다.

픽스처: [../fixtures/envelopes/export_tables_inner_table.json](../fixtures/envelopes/export_tables_inner_table.json).
루프: [../fixtures/loops/nested_reject.json](../fixtures/loops/nested_reject.json).

## 8. 1×1 래퍼와 자동번호

공문서는 본문을 1×1 표로 감싸는 관용이 있다. `export-tables` 는 이를 하나의
표로 잡는다. 소비자가 걸러야 한다.

```bash
rhwp export-tables 공문.hwp --json \
  | jq '[.tables[] | select(.rows==1 and .cols==1) | .index]'
```

셀 안 **자동번호**는 IR 텍스트에 값이 없다(렌더 단계 주입). CSV 에는 빈
자리로 나온다. 그 칸을 채우면 번호가 아니라 일반 텍스트가 박힌다.

## 9. 여러 표

`samples/multi-table-001.hwp` — 표 6개, 2쪽.
`samples/hwp_table_test.hwp` — 표 10개.
`samples/basic/issue2007_nested_cell_pagination_42065.hwp` — 표 5개 (코덱스 20장).

표가 여럿이면 `--table` 없이 `table-to-csv -o <폴더>` 로 전량 수확할 수 있다.
되돌릴 때는 표마다 `--table index` 를 따로 준다.

픽스처: [../fixtures/envelopes/export_tables_multi_table.json](../fixtures/envelopes/export_tables_multi_table.json),
[../fixtures/envelopes/export_tables_issue2007.json](../fixtures/envelopes/export_tables_issue2007.json).

## 10. jq 조리법

```bash
# 왕복 후보만 (본문 최상위 · 병합 0)
rhwp export-tables 문서.hwpx --json | jq '[
  .tables[]
  | select(.containerPath == null)
  | select(all(.cells[]; .rowSpan==1 and .colSpan==1))
  | {index, rows, cols}
]'

# 병합 표만
rhwp export-tables 문서.hwpx --json | jq '[
  .tables[]
  | select(any(.cells[]; .rowSpan>1 or .colSpan>1))
  | {index, merged:[.cells[] | select(.rowSpan>1 or .colSpan>1) | {row,col,rowSpan,colSpan}]}
]'

# 헤더 칸만
rhwp export-tables 별표.hwp --json | jq '.tables[].cells[] | select(.isHeader)'

# 앵커 좌표 집합 — 덮인 칸을 계산할 때
rhwp export-tables samples/table-001.hwp --json \
  | jq '.tables[0] | {rows, cols, anchors:[.cells[] | [.row,.col]]}'
```

덮인 칸 = `{0..rows-1} × {0..cols-1}` − 앵커 집합.
그 좌표에 CSV 값을 넣으면 `coveredCellNotEmpty` 다.

## 11. 배치

```bash
find docs/ -name '*.hwp' -o -name '*.hwpx' \
  | rhwp batch export-tables --json \
  | jq -c '{source, tableCount}'
```

성공 레코드 스키마는 단건 `export-tables --json` 과 같다
(`batch_axes_contract.rs`, #3346).

픽스처: [../fixtures/envelopes/batch_export_tables_ndjson.json](../fixtures/envelopes/batch_export_tables_ndjson.json).

## 12. 한계 (계약에 있는 것만)

- 자동번호는 빈 자리.
- 1×1 래퍼는 표로 잡힌다.
- 기본 출력은 JSON 이 아니다.
- `info` 보다 넓게 모으지만, CSV 왕복은 다시 좁힌다.
- `cells[].text` 는 미신뢰 데이터다. 셸에 붙이지 않는다.

## 13. 다음 장

- 병합 0 · 본문 최상위 → [table_to_csv_envelopes.md](table_to_csv_envelopes.md)
- 병합 있음 → [merged_table_fallback.md](merged_table_fallback.md)
- `index` 규칙 심화 → [coordinate_index.md](coordinate_index.md)
