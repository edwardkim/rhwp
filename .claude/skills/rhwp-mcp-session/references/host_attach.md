# 호스트에 붙이기 — 세션을 살리는 설정

정본 킷: `mydocs/manual/mcp_attach_kit.md`. 여기서는 세션 관점만 옮긴다.

## 최소 등록

```json
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- `rhwp` 가 PATH 에 없으면 `command` 를 절대 경로로 (`C:/…/target/release/rhwp.exe`).
- 전송은 stdio JSON-RPC 뿐이다. 포트·인증·URL 설정은 없다.
- 서버는 stdin EOF 에서 종료한다. 호스트가 프로세스를 죽이면 핸들은 전부 사라진다.

## 세션을 살리려면

1. 호스트가 대화마다 새 `mcp-serve` 를 띄우면 세션 이득이 없다. 같은 서버 프로세스가
   여러 `tools/call` 을 받아야 한다.
2. 역할이 정해져 있으면 `"args": ["mcp-serve", "--profile", "행정서식"]`.
3. 코퍼스 인벤토리가 필요하면 `"args": ["mcp-serve", "--workspace", "C:/abs/corpus"]`.
4. 상대 경로를 피하려면 호스트 cwd 와 무관하게 **절대 경로만** 넘긴다.

## 호스트 모양

| 형 | 호스트 | 키 |
|---|---|---|
| A | Claude Code · Desktop · Cursor · Windsurf · Gemini CLI | `mcpServers` |
| B | VS Code / Copilot | `servers` + `type: stdio` |
| YAML | Goose · Continue | `cmd`/`command` + `args` |

저장소 루트 `.mcp.json` 은 Claude Code 용 A형이다.

## 핸드셰이크 후 확인

1. `initialize` → `protocolVersion` / `serverInfo.name=rhwp`
2. `notifications/initialized`
3. `tools/list` — 무상태 + 세션. 프로필이면 축소된 목록.
4. `resources/list` — `rhwp://capabilities/mcp` 가 있는지.

## 하지 않는 것

- 도구 목록을 호스트 설정에 하드코딩하기.
- 세션 도구를 호스트가 임의로 추가하기.
- gym 트레이스를 실사용 세션으로 오인하기. 이 스킬은 실 에이전트 부착용이다.
