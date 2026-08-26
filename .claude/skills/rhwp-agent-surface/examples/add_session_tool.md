# 레시피 — MCP 세션 도구를 더할 때

SSOT: `ALL_SESSION_TOOLS` + `served_tools()`.

## 전제

무상태 짝 `hwp_*` 가 `--mcp` 선언에 있다. 짝이 없으면 무상태를 먼저.
`hwp_doc_redact` · `hwp_doc_insert_row` 처럼 짝 없는 이름을 만들지 마라.

## 한 곳

1. `src/agent_profiles.rs` `ALL_SESSION_TOOLS` 에 이름.
2. 조회 전용이면 `SESSION_READ_TOOLS` 에도.
3. `mcp_serve.rs` 디스패치가 **무상태와 같은 코어**를 열린 문서에 적용.
4. 필수 인자 `docId`. 모르거나 닫힌 id → `isError` + `nextCall.hwp_open`.
5. 디스크를 쓰지 않는다. 쓰기는 `hwp_doc_save`.
6. 판정 필드 이름(`notFound`, `replacedCount`, `identical`)은 무상태와 동형.

## 프로필

조회 전용 프로필(`아카이브검색`)에 변이·save 를 넣지 않는다.
`allows_session_tool` 이 `tools/list` 와 `tools/call` 을 같이 막는다.

## 확인

`capabilities --mcp` 에 새 세션 이름이 **없어야** 한다.
`tools/list` 에 **있어야** 한다. 반대면 규칙 1 위반.

## 닫힌 핸들 바늘

[`closed_handle_recovery.md`](closed_handle_recovery.md).
