# 예 10_mixed_kitchen.md — 전체 어포던스

종류: `mixed` · 정지 `X03` · gym 아님.

## 첫 수

```bash
rhwp explore samples/kitchen-sink.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=22` ·
`field_count=4` ·
`table_count=2` ·
`chart_count=1` ·
`injection=1` ·
`hidden=1` ·
`encrypted=False`

## 메뉴

`security-sweep → form-fill → table-extract → structure-outline → chart-extract → note-structure → long-doc-digest → triage-overview`

첫 명령: `rhwp inspect injection <file> --json`

전체 봉투는 `fixtures/envelopes/S26.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

8개가 모두 켜지고 보안이 1번.

여덟 어포던스가 모두 켜진 합성 표본. 보안이 1번인지 확인하는 계약용.
