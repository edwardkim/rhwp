# 02 — 표 하나 CSV (`--table` + 파일 `-o`)

권위: [table_to_csv_envelopes.md](../references/table_to_csv_envelopes.md).
표본: `samples/hwp_table_test.hwp` 레시피 02.

## 명령

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o output/table0.csv --json
```

## 기대 봉투

```json
{
  "bom": false,
  "outputFormat": "csv",
  "tableCount": 1,
  "tables": [
    {
      "index": 0,
      "rowCount": 4,
      "colCount": 3,
      "csv": "제목,담당자,세부 내용\r\n,,\r\n,,\r\n,,\r\n"
    }
  ],
  "untrustedContent": true,
  "untrustedFields": ["tables[].csv"]
}
```

파일 `output/table0.csv` 내용이 `tables[0].csv` 와 같다 (BOM 없음).

## 확인

- `tables[0].index == 0`
- CSV 레코드 4, 필드 3
- 빈 세 줄은 데이터 칸이지 헤더 아래 "무시 행"이 아니다

픽스처: [../fixtures/envelopes/table_to_csv_hwp_table_test_t0.json](../fixtures/envelopes/table_to_csv_hwp_table_test_t0.json).

다음: 편집 후 [05](05_dry_run_preview.md).
