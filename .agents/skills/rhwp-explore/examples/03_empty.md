# 예 03_empty.md — 0쪽·문단 0 — 로드는 됐으나 비어 보임

종류: `empty-loaded` · 정지 `X05` · gym 아님.

## 첫 수

```bash
rhwp explore samples/empty-body.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=0` ·
`field_count=0` ·
`table_count=0` ·
`chart_count=0` ·
`injection=0` ·
`hidden=0` ·
`encrypted=False`

## 메뉴

`triage-overview`

첫 명령: `rhwp digest <file> --json`

전체 봉투는 `fixtures/envelopes/S22.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

로드 성공이면 메뉴는 개요 하나. 파싱 실패와 구별.

로드 성공·본문 없음. 파싱 실패(exit 1)와 구별한다. 가짜 표를 넣지 않는다.
