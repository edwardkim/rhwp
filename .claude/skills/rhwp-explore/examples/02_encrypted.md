# 예 02_encrypted.md — 암호가 풀린 짧은 문서

종류: `encrypted` · 정지 `X04` · gym 아님.

## 첫 수

```bash
rhwp explore secret/memo.hwp --json
```

## 엔진 개수 (본문 아님)

`page_count=2` ·
`field_count=0` ·
`table_count=0` ·
`chart_count=0` ·
`injection=0` ·
`hidden=0` ·
`encrypted=True`

## 메뉴

`triage-overview`

첫 명령: `rhwp digest <file> --json`

전체 봉투는 `fixtures/envelopes/S17.json`.
`<file>` 만 실제 경로로 치환한다. 새 명령을 만들지 않는다.

메뉴는 나온다. why 가 후속 --password 를 상기.

비밀번호 없이 치면 exit 2 이고 이 봉투는 없다. 풀린 뒤에만 encrypted why 가 보인다.
