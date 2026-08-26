# 레시피 — 드리프트 가드가 붉을 때

픽스처: [`../fixtures/exceptions/drift_guard_fail.json`](../fixtures/exceptions/drift_guard_fail.json).

## 흔한 실패

```
--json 계약 명령인데 MCP 도구로 안 나오는 것: ["word-count"]
```

`capabilities_mcp_covers_every_json_command`.

뜻: `capabilities_command_entries()` 에는 `json:true` 인데
`mcp_tool_definitions()` 에 `cli.command == "word-count"` 가 없다.

## 고치는 곳

`src/main.rs` `mcp_tool_definitions()` 에 `tool("hwp_word_count", ...)` 한 줄.
목록을 테스트나 문서에 베끼지 않는다.

코어를 새로 짜지 않는다. 이미 CLI 가 쓰는 함수를 그대로 연결한다.

## 다른 가드

| 메시지 느낌 | 가드 | 고치는 곳 |
|---|---|---|
| tools/list 와 --mcp 가 다름 | `tools_list_matches_capabilities_manifest` | 서버가 같은 함수를 쓰는지 |
| name/inputSchema 누락 | `capabilities_mcp_tool_definitions_contract` | `tool()` helper |
| help 명령이 capabilities 에 없음 | `capabilities_covers_every_help_command` | `capabilities_command_entries()` |
| 스킬이 죽은 명령 안내 | `skills_reference_only_real_commands` | SKILL.md 의 `rhwp <토큰>` |

## 하지 말 것

- missing 을 제외 배열에 슬쩍 넣기
- 가드 assert 를 `>=` 로 느슨하게 만들기
- 테스트 골든에 도구 개수를 박기
