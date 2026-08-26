# 예 06_security_first.md — 보안+서식+표가 한 문서

종류: `mixed` · 정지 `X03` · gym 아님.

## 첫 수

```bash
rhwp explore untrusted/form-report.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=6` ·
`field_count=8` ·
`table_count=3` ·
`chart_count=0` ·
`injection=2` ·
`hidden=0` ·
`encrypted=False`

## 메뉴

`security-sweep → form-fill → table-extract → triage-overview`

첫 명령: `rhwp inspect injection <file> --json`

전체 봉투는 `fixtures/envelopes/S16.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

순서: security → form-fill → table-extract → triage.

본문·fields 값·표 셀을 LLM 에 넣기 전에 inspect injection 을 친다.
