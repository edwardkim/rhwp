# PR #3264 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3264](https://github.com/edwardkim/rhwp/pull/3264) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `d8b8fee39149926be6338f65b9c582c5921aebe7` (2026-07-25 조회 참고값) |
| 규모 | +1867/-10, 9 files, 6 commits |
| 관련 이슈 | #3263 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 통합 보정 | `31e3d5c36` — JSON command와 MCP tool inventory 정합 |
| 판단 | v2 통합 PR 수용 후보 — 최종 처리 |

## 범위와 검토

- `capabilities --json`과 `--mcp`로 CLI command 및 MCP tool schema를 자기서술한다. 원 feature
  `e8e2ea08e`, `14f7fad2a`를 누적 적용했다.
- 실제 JSON CLI의 `export-svg`, `export-tables`, `search`, `fields`, `ir-diff`가 capabilities `jsonContract`에
  빠졌고 MCP tool 목록에도 노출되지 않았다. 공개 manifest의 누락은 지원되지 않는 기능으로 보이므로 blocking이다.

## 보정과 검증

- v2의 `31e3d5c36`는 누락한 command의 JSON capability와 `hwp_export_svg`, `hwp_export_tables`, `hwp_search`,
  `hwp_fields`, `hwp_ir_diff` MCP schema를 추가한다. 기존 test가 모든 JSON command의 MCP 노출을 가드한다.
- direct CLI에서 JSON command 10개와 MCP tool 9개가 모두 확인됐고, `cli_json_contract` 22 passed와 누적
  full release-test 전체 성공을 확인했다.
- CLI manifest만 변경하며 renderer·layout·fixture·golden은 바뀌지 않아 visual sweep과 baseline 등록은 불필요하다.

## 권고와 다음 조건

- **권고: v2 통합 PR의 최종 보정으로 수용.** #3258, #3262, #3276, #3282, #3280, #3285, #3288 feature가 누적된
  v2에서 capability manifest를 확정한다. 1,000줄 초과 PR의 보정 범위는 공개 command/MCP inventory 정합으로 한정한다.
- source head에는 push하지 않고 latest full CI·통합 PR mergeability·사용자 PR·merge 승인을 확인한다.
