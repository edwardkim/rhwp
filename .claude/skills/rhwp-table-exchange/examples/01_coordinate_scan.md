# 01 — 좌표·병합 스캔 (`export-tables`)

쓰기 전에 표 번호·치수·병합·컨테이너를 본다.

권위: [export_tables_matrix.md](../references/export_tables_matrix.md).

## 명령

```bash
rhwp export-tables samples/hwp_table_test.hwp --json \
  | jq '.tables[] | {index, rows, cols, cellCount,
       merged:[.cells[]? | select(.rowSpan>1 or .colSpan>1)] | length,
       box:.containerPath}'
```

## 기대 (표 0)

```
index: 0
rows: 4
cols: 3
cellCount: 12
merged: 0
box: null
```

`tableCount: 10`. 0번만 왕복 후보로 고른다.

병합 표본은 `samples/table-001.hwp` — `rows:19` `cols:9` `cellCount:131`.
`merged > 0` 이면 [12](12_merged_fallback_set_cell.md).

## 읽기

- `--table` 에 넣을 값은 `index` 이지 배열 순번이 아니다
- `cells[].text` 는 미신뢰
- `--json` 없이 돌리면 사람용 요약. 파싱하지 마라

픽스처: [../fixtures/envelopes/export_tables_hwp_table_test.json](../fixtures/envelopes/export_tables_hwp_table_test.json).

## 실패

없는 파일 → exit 1, stdout 0.
파일 둘 → exit 2, stdout 0.
