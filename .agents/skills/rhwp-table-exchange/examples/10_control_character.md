# 10 — 줄바꿈·탭 (`controlCharacter`)

엑셀 Alt+Enter 또는 탭 구분 잔재.

## 명령

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv fixtures/csv/table0_control_lf.csv --table 0 --dry-run --json
```

## 기대

exit 2.

```json
{
  "reason": "controlCharacter",
  "row": 1,
  "col": 0,
  "message": "셀 값에 줄바꿈·탭은 v1 에서 허용하지 않습니다."
}
```

탭도 같다 (`table0_control_tab.csv`).

인용(`"서버\n이관"`)이 되어 있어도 거부한다. 파싱된 값을 본다.

처방: `서버 이관` 처럼 공백으로 치환 후 dry-run.
여러 줄 셀 편집 로직을 만들지 않는다.

픽스처: [../fixtures/envelopes/csv_to_table_control_lf.json](../fixtures/envelopes/csv_to_table_control_lf.json),
[../fixtures/envelopes/csv_to_table_control_tab.json](../fixtures/envelopes/csv_to_table_control_tab.json).
