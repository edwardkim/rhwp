# 레시피 — CLI `--json` 조각을 더할 때

이 레시피는 **절차**다. 이 저장소 작업에서 새 명령을 만들지 않는다.

## 전제

코어 함수가 이미 검증돼 있다. 없으면 코어 이슈가 먼저다 (규칙 2).

## 순서

1. 이슈를 잠근다. `gh issue view` 로 assignee / 열린 PR.
2. `tests/<name>_contract.rs` 를 **신설**한다. 구현 전 FAILED 를 확인.
   기존 `cli_json_contract.rs` 를 고치지 않는 편이 병렬에 안전하다.
3. 명령이 `*_json_value` 를 stdout 에만 내게 한다. 진단은 stderr.
4. 실패 시 stdout 0바이트 (run 예외는 `jsonContract.failure` 에 이미 적혀 있다).
5. `schemaVersion` + `untrustedContent` + `untrustedFields` 를 dry-run 포함 모든 모드에.
6. `capabilities_command_entries()` 에 `cmd_json("이름", "가족", "요약", true, ...)`.
7. `--json` 이면 같은 PR 에서 `mcp_tool_definitions()` 에 `tool(...)`.
   빼먹으면 `capabilities_mcp_covers_every_json_command` 가 붉다.
8. `cli_commands.md` 절 + 지식 지도 행.
9. `cargo fmt --all -- --check` 후 한글 PR.

## 자리표시자

무상태 짝을 같이 달 때 `{path}` 는 `required` 에 있어야 한다.
선택 플래그는 `optionalArgs`.

## 하지 말 것

- `--help` 문자열만 추가하고 capabilities 를 잊기
- MCP 목록을 다른 파일에 손으로 적기
- DocumentCore 에 새 편집 알고리즘을 이 PR 에 섞기
