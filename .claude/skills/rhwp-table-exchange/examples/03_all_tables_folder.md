# 03 — 전량 수확 (`-o` 폴더)

`--table` 없이 `-o` 는 폴더다. 각 파일 이름 `table<index>.csv`.

## 명령

```bash
rhwp table-to-csv samples/hwp_table_test.hwp -o output/tables --json
ls output/tables
```

## 기대

- `tableCount: 10`
- `tables[i].output` 이 `output/tables/table{index}.csv`
- `tables[i].index` 가 파일 이름의 번호

`samples/multi-table-001.hwp` 도 같다 (표 6).

## 되돌리기

폴더를 통째로 넣는 명령은 없다. 표마다:

```bash
rhwp csv-to-table 문서.hwp --csv output/tables/table0.csv --table 0 --dry-run --json
```

`--table` 은 파일 이름의 숫자가 아니라 봉투의 `index` 다. 둘이 같길
바라지만, 머리말 표가 빠진 전량이면 파일 번호와 어긋날 수 있다.
`export-tables` 로 대조한다.

픽스처: [../fixtures/envelopes/table_to_csv_all_tables.json](../fixtures/envelopes/table_to_csv_all_tables.json).
