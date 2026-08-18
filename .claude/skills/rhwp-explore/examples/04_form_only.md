# 예 04_form_only.md — 누름틀만 있는 신청서

종류: `form` · 정지 `X10` · gym 아님.

## 첫 수

```bash
rhwp explore samples/form-01.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=1` ·
`field_count=1` ·
`table_count=0` ·
`chart_count=0` ·
`injection=0` ·
`hidden=0` ·
`encrypted=False`

## 메뉴

`form-fill → triage-overview`

첫 명령: `rhwp fields <file> --json`

전체 봉투는 `fixtures/envelopes/S02.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

form-fill 이 개요보다 위. fields --json 으로 인계.

다음 스킬은 rhwp-form-fill. 이 예가 fill-fields 를 실행하지 않는다.
