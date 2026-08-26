# 규칙 1 — 선언·실행·문서는 한 곳에서 갈라진다

플레이북 §1 규칙 1. 픽스처: [`../fixtures/rules.json`](../fixtures/rules.json).

## 한 곳

무상태 도구의 이름·설명·`inputSchema`·`cli.args`·`outputFields` 는
`mcp_tool_definitions()` **함수 하나**가 만든다.

```
mcp_tool_definitions()
        ├─ rhwp capabilities --mcp          선언
        ├─ mcp-serve tools/list (무상태)    실행
        └─ rhwp://capabilities/mcp          리소스
```

세션 이름은 이 함수에 넣지 않는다. 세션은 `ALL_SESSION_TOOLS` 가 한 곳.

CLI 목록은 `capabilities_command_entries()` 가 한 곳이다.
`capabilities_value()` 가 그 배열을 감싸 `rhwp capabilities` 와
`export-agent-manifest` 가 같이 쓴다.

## 하지 말 것

- 호스트 `.mcp.json` 에 도구 배열을 손으로 적기
- 스킬·위키에 "현재 39종"처럼 개수를 계약으로 박기
- `tools/list` 결과를 저장소에 골든으로 얼리기
- capabilities 에만 명령을 넣고 MCP 를 잊기
- MCP 에만 도구를 넣고 capabilities 를 잊기

어느 쪽이든 드리프트 가드가 붉다. 가드를 느슨하게 만들지 말고 원천을 맞춘다.

## 자리표시자 1:1

`tool()` 의 `cli.args` 자리표시자 `{name}` 은 `inputSchema.required` 와
같은 집합이어야 한다. 선택 인자는 `tool_with_optional_args()` 의
`optionalArgs` + `when` 조건으로만 붙인다.

사고 사례: `{output}` 을 required 가 아닌데 args 에 넣으면, 호출자가 생략했을 때
글자 그대로 `{output}` 파일이 생긴다.

## 프로필도 한 곳

`src/agent_profiles.rs` `PROFILES` 가 `capabilities --mcp --profile` 과
`mcp-serve --profile` 을 같이 구동한다. `tools` 배열을 다른 파일에
복제하지 않는다. 프로필은 추천 목록이 아니라 **서버가 실제로 제공하는
집합의 경계**다 (`allows_tool` / `allows_session_tool`).

## 문서

`cli_commands.md` 해당 절과 지식 지도 행은 **구현과 같은 PR** 에서 갱신한다.
문서만 앞서 가면 가드가 못 잡는다 — 문서 드리프트는 리뷰 항목이다.
이 스킬은 문서 본문을 대신 쓰지 않고, "어디에 한 줄을 더하라"만 가리킨다.
