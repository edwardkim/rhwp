# 19 — 함정 (실측·계약)

## P01 — batch --password

올바름: exit 2. 단건 --password

## P02 — convert 이름 충돌을 덮어쓰기

올바름: 한 파일도 안 쓰고 exit 2

## P03 — fill 에 stdin 목록

올바름: --form + --data

## P04 — 실패 행 삭제 후 성공만 저장

올바름: 게이트가 깨짐

## P05 — limit 을 배치 전체 상한으로 해석

올바름: 문서마다

## P06 — stderr 요약을 stdout 으로 파싱

올바름: 2> 분리

## P07 — head 로 미리보고 게이트

올바름: 원본 목록 줄 수 기준

## P08 — search 없이 --query

올바름: exit 2

## P09 — --out-dir -결과

올바름: ./-결과

## P10 — tableCount 0 을 실패

올바름: 빈 표는 성공

## P11 — fieldCount 0 을 실패

올바름: 축 전환 신호

## P12 — itemCount 0 을 실패

올바름: 추출 0건은 exit 0

## P13 — matchCount 0 을 실패

올바름: 검색 0은 성공

## P14 — exit 1 이면 전부 실패

올바름: 행별 봉투

## P15 — 병렬이면 순서 뒤섞임

올바름: 입력 순서 보존

## P16 — MCP 로 convert

올바름: CLI 전용

## P17 — CP949 --data

올바름: UTF-8

## P18 — counts == itemCount 항상

올바름: 절단 전이 counts

## P19 — 성공 행 재처리

올바름: 실패만 재시도

## P20 — 새 batch 서브커맨드 발명

올바름: 기존 9축만

## P21 — gym pack 작성

올바름: 금지

## P22 — 목록을 argv 로

올바름: stdin

## P23 — 2>&1 로 섞기

올바름: NDJSON 오염

## P24 — verify 실패면 산출 없음

올바름: 산출은 남고 exit 3

## P25 — verify 와 verify-pages 코드 혼동

올바름: 3 vs 4

## P26 — 같은 이름 다른 대소문자 허용

올바름: 충돌

## P27 — fill dry-run 에서 out-dir 생략

올바름: 필수

## P28 — Select-String error

올바름: 본문 error 오탐. jq

## P29 — 단건 실패처럼 stdout 빈 줄 기대

올바름: 배치는 실패 레코드

## P30 — extract-data 축이 없다 판단

올바름: capabilities + 레시피 9

함정 픽스처: `fixtures/pitfalls.json`.
## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `19_pitfalls.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
