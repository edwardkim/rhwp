# 예 05_table_report.md — 표 3개·병합 1개 보고서

종류: `table` · 정지 `X10` · gym 아님.

## 첫 수

```bash
rhwp explore samples/report-tables.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=6` ·
`field_count=0` ·
`table_count=3` ·
`chart_count=0` ·
`injection=0` ·
`hidden=0` ·
`encrypted=False`

## 메뉴

`table-extract → triage-overview`

첫 명령: `rhwp export-tables <file> --json`

전체 봉투는 `fixtures/envelopes/S04.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

table-extract why 에 병합 셀이 드러난다.

why 에 병합 셀이 드러난다. 다음 스킬은 rhwp-table-exchange.
