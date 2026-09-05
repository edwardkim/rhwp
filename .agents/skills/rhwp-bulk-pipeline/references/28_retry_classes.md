# 28 — 재시도 부류

실패 행을 무조건 다시 돌리지 않는다. 부류를 가른다.

| ID | 신호 | 재시도 | 행동 |
| --- | --- | --- | --- |
| `R-PATH` | os error 2 | 아니오 | 목록 경로 수정 |
| `R-PERM` | os error 13 / Access is denied | 아니오 | 권한·잠금 해제 |
| `R-PASS` | 암호 / encrypted | 아니오 | 단건 --password 로 분리. batch 플래그 금지 |
| `R-PARSE` | 문서를 열 수 없습니다 (파서) | 아니오 | 손상 파일. 코퍼스에서 격리 |
| `R-TRANSIENT` | 일시적 IO / sharing violation | 예 | 같은 축으로 실패 목록만 재시도 |
| `R-USAGE` | exit 2, 레코드 없음 | 아니오 | 플래그·이름 예약·쿼리부터 수정 |
| `R-VERIFY` | exit 3/4, error 키 없음 | 아니오 | 행별 verify 봉투. 재시도로 해결되지 않음 |

정지 규칙과의 연결:

- R-PATH / R-PARSE → B05 (목록 수정, 손상 격리)
- R-PASS → B03 (단건 분리)
- R-USAGE → B03/B09/B11/B12/B18
- R-VERIFY → B15/B16
- R-TRANSIENT 만 같은 축 재시도

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `28_retry_classes.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
