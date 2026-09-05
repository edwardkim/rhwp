# 07 — 암호 / 빈 파일 / 특수 어포던스 없음

세 갈래를 한 경로로 뭉개지 않는다.

## 암호 (encrypted)

| 상태 | exit | stdout | 다음 |
| --- | --- | --- | --- |
| 비밀번호 없음 | 2 | 비움 | `--password` / `--password-stdin` 후 같은 explore |
| 비밀번호 불일치 | 1 | 비움 | 비밀번호 확인. 메뉴 추정 금지 |
| 맞아서 로드됨 | 0 | `encrypted:true` | 후속 command 에도 비밀번호 (X04) |

`explore` 자체는 `--password` 를 하위 옵션으로 파싱하지 않는다.
전역 pre-scan 이 `load_document` 에 전달한다.

로드된 암호의 `triage-overview.why` 는
`암호 보호 — 후속 명령에 --password 필요` 를 포함한다.

## 빈 파일·로드 실패

| 상태 | exit | 메뉴 |
| --- | --- | --- |
| 경로 없음·읽기 실패 | 1 | 없음 |
| 0바이트·파싱 실패 | 1 | 없음 |
| 로드 성공, 쪽 0·문단 0 | 0 | triage-overview 하나 |
| detect_format Empty 이고 로드 성공 | 0 | format=`빈 파일`, 개요 하나 |

파싱 실패에 가짜 개요를 지어내지 않는다 (P09).

## 특수 어포던스 없음

표·누름틀·차트·조문·각주·장문(≥10쪽)·보안 신호가 없으면 메뉴는
`triage-overview` 한 줄이다. 이것은 성공이다 (X05). `digest --json`
으로 파악하고 멈출 수 있다.

9쪽은 장문을 켜지 않는다. 10쪽부터다 (`LONG_DOC_PAGES`).
