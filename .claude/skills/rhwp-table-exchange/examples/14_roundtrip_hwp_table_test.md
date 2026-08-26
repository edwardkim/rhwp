# 14 — 레시피 02 왕복 전체

표본: `samples/hwp_table_test.hwp` 표 0.

## 순서

```bash
rhwp export-tables samples/hwp_table_test.hwp --json
# index 0, 4x3, merged 0

rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0.csv --json
# csv = 제목,담당자,세부 내용\r\n,,\r\n,,\r\n,,\r\n

# table0_edited.csv 작성 (헤더 유지, 3행 채움)

rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 --dry-run --json
# changedCount 9, invalid [], changedPages null

rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 \
  -o table_updated.hwp --verify --json
# changedCount 9, verify.identical true, outputFormat hwp5

rhwp export-tables table_updated.hwp --json \
  | jq '.tables[0].cells[] | select(.row==1)'
# 서버 이관 / 홍길동 / 1차 완료
```

닫힘 조건: `invalid==[]` ∧ `verify.identical==true` ∧ 재독 값이 CSV 와 같음.

픽스처: [../fixtures/transcripts/recipe02_roundtrip.json](../fixtures/transcripts/recipe02_roundtrip.json),
[../fixtures/loops/roundtrip_plain.json](../fixtures/loops/roundtrip_plain.json).

트랜스크립트 전문: [../references/sample_transcripts.md](../references/sample_transcripts.md) §1.
