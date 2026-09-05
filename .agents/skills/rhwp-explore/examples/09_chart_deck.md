# 예 09_chart_deck.md — 차트 2개 설명회 자료

종류: `chart` · 정지 `X10` · gym 아님.

## 첫 수

```bash
rhwp explore samples/charts.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=5` ·
`field_count=0` ·
`table_count=0` ·
`chart_count=2` ·
`injection=0` ·
`hidden=0` ·
`encrypted=False`

## 메뉴

`chart-extract → triage-overview`

첫 명령: `rhwp chart-to-csv <file> --json`

전체 봉투는 `fixtures/envelopes/S08.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

chart-to-csv 로 인계. 표가 없으면 table-extract 는 없다.

표가 없으면 table-extract 가 없다. 차트만 chart-to-csv.
