# 단일 출처 — `capabilities --mcp`

도구 정의는 `src/main.rs` 의 `mcp_tool_definitions()` **한 곳**에서 나온다.

| 표면 | 무엇이 나오나 | 무엇이 안 나오나 |
|---|---|---|
| `rhwp capabilities --mcp` | 무상태 도구 선언(name/description/inputSchema/cli/annotations) | 세션 도구 |
| `mcp-serve` `tools/list` | 위 선언 + 세션 도구 | 손으로 베낀 옛 목록 |
| `rhwp://capabilities/mcp` | 매니페스트 리소스(= `--mcp`) | 세션 도구 |

계약 테스트 `tests/mcp_server_contract.rs::tools_list_matches_capabilities_manifest`
가 두 표면의 무상태 부분을 같게 유지한다. `--json` 명령이 늘었는데 MCP 에서 빠지면
`capabilities_mcp_covers_every_json_command` 가 잡는다.

## 에이전트가 할 일

1. 추측하지 말고 `rhwp capabilities --mcp` 또는 리소스 `rhwp://capabilities/mcp` 를 1회 캐시한다.
2. 세션이 필요하면 `tools/list` 를 한 번 더 본다. 세션 이름은 여기에만 있다.
3. 이름이 없으면 `didYouMean` → `tools/list` → 무상태 CLI. **새 이름을 만들지 않는다.**
4. 문서(본 스킬 포함)의 개수가 바이너리와 다르면 바이너리가 이긴다.

## 함수콜 클라이언트 (경로 ①)

MCP 호스트가 아니면 선언을 직접 소비한다.

1. `cli.args` 의 `{키}` 를 `inputSchema` 같은 이름 값으로 치환한다.
2. 객체·숫자는 JSON 문자열로 넣는다 (`hwp_fill_fields` 의 `{data}`).
3. `invocation.stdinTools` (`hwp_batch` · `hwp_batch_search` · `hwp_batch_extract_data`)는
   `paths` 를 stdin 한 줄씩 흘린다.
4. `cli.passwordStdin` 이 있으면 `password` 를 `--password-stdin` 첫 줄로만 넘긴다.

## 프로필

`--profile <이름>` 은 추천이 아니라 **서버가 제공하는 집합의 경계**다.
목록에 없는 세션 도구는 `tools/call` 로도 우회할 수 없다.
없는 프로필 이름은 실행 전에 막힌다 (`오류: 알 수 없는 프로필`).

실측 프로필: `경영보고` · `행정서식` · `데이터분석` · `콘텐츠제작` ·
`아카이브검색` · `품질검증` · `개발통합`.

`개발통합` 만 필터가 없다. 작은 모델은 직무에 맞는 프로필로 물린다.

## 확인 명령

```bash
rhwp capabilities --mcp | jq '.tools[] | {name, description}'
rhwp capabilities --mcp --profile 행정서식
# 세션 포함 목록
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | rhwp mcp-serve
```
