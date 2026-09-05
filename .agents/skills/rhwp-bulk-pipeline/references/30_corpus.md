# 30 — 표본 코퍼스

새 HWP 바이너리를 만들지 않는다. 저장소 `samples/` 와 레시피 9 실측만 인용.

| id | path | 비고 |
| --- | --- | --- |
| `plan2022` | `samples/2022년 국립국어원 업무계획.hwp` | 레시피 9 실측. info/export-text/extract-data 첫 행. |
| `trade` | `samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp` | 레시피 9 실측. 금액 키 0 은 '없다'가 아니라 kind=all 에서 금액 미검출. |
| `field01` | `samples/field-01.hwp` | 누름틀 11. extract-data 0건은 오류가 아니다. |
| `hwp3` | `samples/hwp3-sample.hwp` | HWP3 표본. fields 0 은 축 전환 신호. |
| `missing` | `samples/없는파일.hwp` | 레시피 9 가 일부러 섞은 없는 파일. 실패 봉투 원형. |
| `form01` | `samples/form-01.hwp` | 메일머지 서식 최소 표본. |
| `table001` | `samples/table-001.hwp` | batch_axes_contract 표 병합 표본. |
| `handbook` | `samples/2025 행정업무운영 편람(최종).hwpx` | 레시피 9 convert 실측. 387쪽 / 428ms / 9_083_392 bytes. |
| `handbookHwp` | `samples/2025 행정업무운영 편람(최종).hwp` | extract-data 계약 오라클과 같은 편람 HWP5. |

없는 파일 `samples/없는파일.hwp` 는 저장소에 없다. 실패 봉투를 내기 위해
목록에만 넣는다.

편람 HWPX convert 실측은 파일이 있을 때만 재현된다. 스킬 테스트는
커밋된 NDJSON 전사를 읽고 바이너리 없이 계약을 검사한다.

기계 원본: `fixtures/samples.json`, `fixtures/recipe9_gate.json`.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `30_corpus.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
