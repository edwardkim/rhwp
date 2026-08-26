# 26 — 재현 트레이스

트레이스 20개. 각 항목은 `examples/transcripts/<id>.ndjson` 과
`fixtures/traces/<id>.json` 을 가리킨다.

| ID | 제목 | 축 | 입력 | 성공 | 실패 | exit | 정지 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T01 | info 5=4+1 | `info` | 5 | 4 | 1 | 1 | B05 |
| T02 | export-text 5=4+1 | `export-text` | 5 | 4 | 1 | 1 | B05 |
| T03 | extract-data --limit 3 | `extract-data` | 5 | 4 | 1 | 1 | B10 |
| T04 | convert 편람 1건 | `convert` | 1 | 1 | 0 | 0 | B17 |
| T05 | search --query 의 | `search` | 2 | 2 | 0 | 0 | B17 |
| T06 | search 쿼리 없음 | `search` | 0 | 0 | 0 | 2 | B09 |
| T07 | password 거부 | `info` | 0 | 0 | 0 | 2 | B03 |
| T08 | convert 이름 충돌 | `convert` | 0 | 0 | 0 | 2 | B11 |
| T09 | fields 조사 | `fields` | 2 | 2 | 0 | 0 | B08 |
| T10 | export-tables 병합 | `export-tables` | 1 | 1 | 0 | 0 | B07 |
| T11 | export-structure auto | `export-structure` | 2 | 2 | 0 | 0 | B17 |
| T12 | fill dry-run | `fill` | 3 | 3 | 0 | 0 | B17 |
| T13 | fill 실행 | `fill` | 3 | 3 | 0 | 0 | B17 |
| T14 | fill 빈 CSV | `fill` | 0 | 0 | 0 | 2 | B12 |
| T15 | fill stdin 오용 | `fill` | 0 | 0 | 0 | 2 | B12 |
| T16 | threads 8 순서 | `export-text` | 5 | 4 | 1 | 1 | B05 |
| T17 | 게이트 증발 | `export-text` | 5 | 1 | 0 | None | B13 |
| T18 | verify 차이 | `convert` | 1 | 1 | 0 | 3 | B15 |
| T19 | verify-pages | `convert` | 1 | 1 | 0 | 4 | B16 |
| T20 | info 선별 75건 | `info` | 270 | 270 | 0 | 0 | B04 |

T01–T04 는 레시피 9 실측 숫자를 따른다 (5=4+1, 편람 convert 9_083_392 bytes).
T05 는 `batch_axes_contract` 의 같은 파일 두 줄 검색 형태.
T06–T08·T14–T15 는 사용법 오류라 전사가 비어 있다 (stdout 0줄).

## 권위

- `mydocs/manual/cli_commands.md` §batch
- 이슈 #5311. gym 아님. 새 CLI 아님.
