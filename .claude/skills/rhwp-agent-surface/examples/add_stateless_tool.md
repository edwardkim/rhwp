# 레시피 — MCP 무상태 도구를 더할 때

SSOT: `mcp_tool_definitions()`.

## 전제

대응하는 `rhwp <cmd> --json` 이 이미 있다. CLI 가 없으면 CLI 를 먼저.

## 한 곳

```
tool(
    "hwp_example",
    "한 줄 설명 — 에이전트가 고르는 문장",
    path_schema(...),          // required 에 path
    "example",                 // cli.command = capabilities 이름
    json!(["example", "--json", "{path}"]),
    &["schemaVersion", "source", "...판정필드"],
)
```

선택 인자는 `tool_with_optional_args` + `when`.

## 검증

```
rhwp capabilities --mcp   # 새 이름이 보이는지
```

가드:

- `capabilities_mcp_tool_definitions_contract` — name/schema/cli
- `capabilities_mcp_covers_every_json_command` — json 명령 누락 없음
- `tools_list_matches_capabilities_manifest` — 서버가 같은 배열

## 암호

자식 CLI 가 `--password-stdin` 을 받으면 `supports_password_stdin` 매치
목록에 이름을 더한다. 응답·세션에 비밀번호를 남기지 않는다.

## 하지 말 것

- 서버 안에서 표를 다시 파싱하기
- `hwp_doc_example` 를 동시에 발명하기 (세션은 짝이 난 다음)
- `batch convert` 처럼 `batch.mcp.excluded` 에 적힌 축을 몰래 열기
