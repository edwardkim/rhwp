# MCP 재측정과 세션 경계

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

`capabilities --mcp` 는 무상태 선언이다. 세션 도구는
`mcp-serve` 의 `tools/list` 에만 있다. 지도 §6-2.

세션의 유일한 기록 지점은 `hwp_doc_save` 라는 문장은 지도
§1-1 (자)에 있다. 여기 절차를 늘여 쓰지 않는다.

NDJSON 도구는 `structuredContent` 가 null 이다 (§6-3).
호스트 부착 절차는 `rhwp-mcp-session` + MCP 가이드.

재측정 JSON-RPC 순서는 지도 §0 코드 블록.
