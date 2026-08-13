---
kind: guide
status: active
canonical: mydocs/manual/mcp_attach_kit.md
last_verified: 2026-08-13
---

# rhwp 붙이기 킷 — 아무 에이전트 도구에나 MCP 로 붙인다

`rhwp mcp-serve`(stdio JSON-RPC)를 여러분이 쓰는 에이전트 호스트에 **한 번 붙이면**,
그 호스트의 에이전트가 rhwp 의 도구·리소스(로드맵·gym·표준)를 그대로 쓴다. 통합
계약의 정본은 [`mcp_integration_guide.md`](mcp_integration_guide.md)이고, 이 문서는
**호스트별 붙이기 설정**만 모은 킷이다.

전송은 stdio 뿐이다(포트·인증 없음). `rhwp` 가 PATH 에 없으면 `command` 를 절대
경로로 준다. 저장소 안에서 Claude Code 로 작업하면 루트 [`.mcp.json`](../../.mcp.json)이
이미 rhwp 를 붙여 둔다(승인은 클라이언트가 묻는다).

## 설정 모양 세 갈래

대부분의 호스트는 **A형**(`mcpServers` 맵)을 쓴다:

```json
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

VS Code/Copilot 은 **B형**(`servers` + `type`)이다:

```json
{ "servers": { "rhwp": { "type": "stdio", "command": "rhwp", "args": ["mcp-serve"] } } }
```

Zed·Goose·Continue 는 각자 다른 스키마다(아래 표 참고).

## 호스트별 붙이기

| 호스트 | 설정 파일 | 모양 | 확신도 |
|---|---|---|---|
| **Claude Code** | `.mcp.json`(프로젝트) · `~/.claude.json`(사용자) · `claude mcp add rhwp -- rhwp mcp-serve` | A | 저장소 실증 |
| **Claude Desktop** | Win `%APPDATA%\Claude\claude_desktop_config.json` · mac `~/Library/Application Support/Claude/claude_desktop_config.json` | A | 높음 |
| **Cursor** | `.cursor/mcp.json`(프로젝트) · `~/.cursor/mcp.json`(전역) | A | 높음 |
| **Cline**(VS Code 확장) | MCP 설정 UI → `cline_mcp_settings.json` | A(+옵션 `disabled`·`autoApprove`) | 높음 |
| **Continue** | `~/.continue/config.yaml` → `mcpServers:` 목록(`name`·`command`·`args`) | YAML | 중 — YAML·버전 분기 |
| **Windsurf**(Cascade) | `~/.codeium/windsurf/mcp_config.json` | A | 높음 |
| **VS Code / Copilot**(에이전트 모드) | `.vscode/mcp.json`(작업공간) · 사용자 `settings.json` → `mcp.servers` | **B** | 높음 — 최상위 키 다름 |
| **Zed** | `settings.json` → `context_servers` | Zed형(아래) | 중 — 스키마 진화 |
| **Goose** | `~/.config/goose/config.yaml` → `extensions:`(또는 `goose configure`) | YAML(`type: stdio`·`cmd`·`args`) | 중 |
| **Roo Code** | `.roo/mcp.json`(프로젝트) · 전역 `mcp_settings.json` | A | 중 |
| **Kilo Code** | `.kilocode/mcp.json`(프로젝트) · 전역 `mcp_settings.json` | A | 중 |
| **Kiro**(AWS) | `.kiro/settings/mcp.json`(작업공간) · `~/.kiro/settings/mcp.json`(사용자) | A | 중 |
| **Amazon Q Dev CLI** | `.amazonq/mcp.json`(작업공간) · `~/.aws/amazonq/mcp.json`(전역) | A | 중 |
| **Gemini CLI** | `~/.gemini/settings.json` → `mcpServers` | A | 높음 |
| **Qwen Code** | `~/.qwen/settings.json` → `mcpServers`(Gemini CLI 포크) | A | 중 |
| **JetBrains Junie / AI Assistant** | IDE 설정 → MCP(‎`mcpServers` JSON 임포트) | A(UI) | 낮음 — 버전 의존 |
| **Trae** | MCP 패널(‎`mcpServers` JSON 임포트) | A(UI) | 낮음 |
| **Augment** | 설정 UI(‎`mcpServers` 임포트) | A(UI) | 낮음 |

> **확신도 "중·낮음" 호스트는 각 도구의 최신 문서로 파일 경로를 확인한 뒤 쓴다** —
> 경로가 릴리스마다 옮겨 다닌다. 붙이는 JSON 모양(A/B)은 안정적이다.

### Zed — `context_servers`

```json
{ "context_servers": { "rhwp": { "source": "custom", "command": { "path": "rhwp", "args": ["mcp-serve"] } } } }
```

### Goose / Continue — YAML

```yaml
# Goose ~/.config/goose/config.yaml (extensions 아래)
rhwp:
  type: stdio
  cmd: rhwp
  args: [mcp-serve]
```

```yaml
# Continue ~/.continue/config.yaml
mcpServers:
  - name: rhwp
    command: rhwp
    args: [mcp-serve]
```

## Windows 주의

`rhwp` 가 PATH 에 없으면 `command` 를 `"C:\\path\\to\\rhwp.exe"` 절대 경로로 준다.
일부 호스트는 `"command": "cmd"`, `"args": ["/c", "rhwp", "mcp-serve"]` 가 필요하다.

## 붙였는지 확인

```bash
# 서버가 초기화·리소스 목록에 응답하는지 직접 확인
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}' \
  | rhwp mcp-serve
```

`tools/list`·`resources/list` 가 rhwp 의 도구와 문서 리소스(`rhwp://docs/…`)를 내면
성공이다. 세션 도구·프로필로 좁히는 법은 [`mcp_integration_guide.md`](mcp_integration_guide.md).

## 붙고 나면 — 표준으로

rhwp 를 붙인 에이전트는 [에이전트 작업 표준(AWS)](../tech/standards/agent_work_standard.md)의
레퍼런스 구현을 손에 쥔다: 편집을 `replay --capsule` 로 증명(AW-L1)하고, gym 으로
실력을 재고, 로드맵으로 방향을 본다. 붙이기는 강요가 아니라 문을 여는 것이다.
