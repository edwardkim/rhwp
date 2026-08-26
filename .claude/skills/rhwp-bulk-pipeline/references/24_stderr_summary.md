# 24 — stderr 사람용 요약

stdout 은 NDJSON 만. 요약은 stderr. 에이전트는 요약을 힌트로만 쓴다.

| 축 | stderr 예 | stdout 줄 | exit |
| --- | --- | --- | --- |
| `export-text` | batch: 5건 중 4 성공, 1 실패 | 5 | 1 |
| `info` | batch: 5건 중 4 성공, 1 실패 | 5 | 1 |
| `convert` | batch convert: 이름 충돌 — 산출을 쓰지 않습니다 | 0 | 2 |
| `search` | error: --query 가 필요합니다 | 0 | 2 |
| `info` | error: batch 는 --password 를 지원하지 않습니다 | 0 | 2 |
| `fill` | batch fill: 3행 중 3 성공 | 3 | 0 |
| `convert` | batch: 1건 중 1 성공, 검증 판정 차이 | 1 | 3 |
| `convert` | batch: 1건 중 1 성공, 페이지 검증 불일치 | 1 | 4 |

## 규칙

- 요약을 `결과.ndjson` 에 리다이렉트하지 않는다.
- `2>&1` 금지. PowerShell 은 `2> 요약.err`.
- "N건 중 M 성공"만 보고 행을 건너뛰지 않는다 (B14).
- exit 2 요약은 레코드가 없음을 뜻한다. 게이트 공식에 넣지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `24_stderr_summary.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
