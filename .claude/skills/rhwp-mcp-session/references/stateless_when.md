# 언제 무상태 도구를 쓰는가

무상태 도구는 `rhwp capabilities --mcp` 가 내는 선언이다. 실행은 `mcp-serve` 가
같은 선언의 `cli.args` 를 해석해 rhwp 자신을 서브프로세스로 돌린다. 따라서
**CLI `--json` 계약이 곧 도구 계약**이다.

현재 소스에서 추출한 무상태 도구는 162종이다. 이 숫자를 외우지 마라.
손에 든 바이너리의 `capabilities --mcp` 가 이긴다.

## 선택 규칙

| 질문 | 예 | 선택 |
|---|---|---|
| 호출 하나면 끝나는가? | 쪽수, 검색 1회, PDF 1회 | **무상태** |
| 같은 파일을 두 번 이상 파싱하게 되는가? | 검색 3회 + 본문 + 채움 | **세션** |
| 대상이 파일 목록인가? | 폴더 스윕, 메일머지 | **무상태 배치** (`hwp_batch*`) |
| 세션에 짝이 없는가? | PDF, redact, run, ir-diff, scan | **무상태만** |
| 워크스페이스 코퍼스인가? | `--workspace` 인벤토리 | **세션** (`hwp_ws_*`) |

## 무상태가 항상 이기는 작업

- 변환·발행: `hwp_export_pdf` · `hwp_export_markdown` · `hwp_convert_hwpx` · `hwp_convert_hwp5`
- 검증 사다리: `hwp_ir_diff` · `hwp_verify` · `hwp_replay` · `hwp_audit` · `hwp_lineage`
- 보안 스윕: `hwp_threat_scan` · `hwp_inspect_*` · `hwp_redact` · `hwp_sanitize`
- 대량: `hwp_scan` · `hwp_batch` · `hwp_batch_search` · `hwp_batch_extract_data` · `hwp_batch_fill`
- 표/차트 왕복: `hwp_table_to_csv` · `hwp_csv_to_table` · `hwp_chart_to_csv` · `hwp_csv_to_chart`
- 원자 다중 편집: `hwp_run_plan` (세션 누적과 다른 축 — 선검증 후 한 파일)

## 세션이 이기는 작업

- 수백 쪽 문서를 검색·발췌·재검색
- 채움/치환/칸 수정을 누적한 뒤 `changedPages` 만 렌더하고 한 번 저장
- 조문 계층을 따라가며 같은 핸들에서 본문·금액을 반복 조회
- 미리보기만 하고 저장하지 않고 폐기

실측(지식 지도): 387쪽 문서에서 검색 3회+info 가 세션 310ms vs 무상태 810ms.

## 배치 함정

`hwp_batch` 계열은 `structuredContent` 가 `null` 이다. `content[0].text` 를 줄 단위
NDJSON 으로 파싱한다. `batch convert` 는 MCP 에 없다 —
`capabilities.batch.mcp.excluded` 가 이유를 문자열로 준다.

## 금지

- 무상태 도구 이름을 손으로 베껴 호스트에 고정하지 않는다.
- `hwp_doc_*` 를 무상태처럼 `path` 로 부르지 않는다. 세션 도구는 `docId` 다.
- 세션에 없는 동사를 만들어 붙이지 않는다.
