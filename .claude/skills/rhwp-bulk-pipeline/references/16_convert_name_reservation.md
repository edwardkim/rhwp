# 16 — convert 이름 예약

`batch convert` 는 쓰기 전에 모든 산출 이름을 예약한다.

규칙: `<out-dir>/<입력 stem>.hwp`

충돌 (같은 이름, 대소문자만 다름, 다른 디렉터리의 같은 stem):

- **exit 2**
- 산출 파일을 **하나도 쓰지 않는다**
- stdout 레코드 없음
- stderr 에 충돌 이유

macOS/Windows 기본 FS 와 Linux 재실행이 같은 결과를 내게 하려는
보수적 규약이다 (cli_commands 명문).

## 사례

| 입력 | 결과 |
| --- | --- |
| `A.hwp`, `B.hwpx` | 통과. `A.hwp`, `B.hwp` |
| `A.hwp`, `A.hwpx` | 충돌. stem `A` |
| `Report.HWP`, `report.hwp` | 충돌. 대소문자 |
| `dir1/x.hwp`, `dir2/x.hwp` | 충돌. 이름만 봄 |

픽스처: `fixtures/convert_names.json`.
전사(빈 stdout): `examples/transcripts/T08.ndjson`.

## 에이전트 처방

1. 충돌 쌍을 jq/그룹으로 가른다.
2. `--out-dir` 를 접두 폴더로 나눈다 (`out/dir1`, `out/dir2`).
3. 또는 목록을 두 번 나눈다.
4. 그 전에는 한 파일도 없다고 가정한다 — "절반 성공" 폴더를 쓰지 않는다.

## MCP

`hwp_batch` 에 convert 쓰기 축이 없다. 에이전트가 MCP 세션에서
변환을 요청하면 CLI 로 내려가거나 "CLI 전용"이라고 답한다.
`rhwp-mcp-session` 스킬을 이 스킬 안에서 재작성하지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `16_convert_name_reservation.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
