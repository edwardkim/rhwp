# 호스트별 `.mcp.json` 붙여넣기 예 — PATH 있을 때 / 절대 경로

닥터 `--host` 가 내는 `mcpHost.snippet` 과 같은 모양이다.
아래 `C:\\src\\rhwp\\target\\release\\rhwp.exe` 와 `/src/rhwp/target/release/rhwp` 는
자리표시자다. 리포트 `binary.path` 로 바꾼다.

포트·토큰·URL 은 없다.

## claude-code — `.mcp.json` (A형, 저장소 실증)

PATH 있음:

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "rhwp",
      "args": ["mcp-serve"]
    }
  }
}
```

Windows 절대 경로:

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "C:\\src\\rhwp\\target\\release\\rhwp.exe",
      "args": ["mcp-serve"]
    }
  }
}
```

Unix 절대 경로:

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "/src/rhwp/target/release/rhwp",
      "args": ["mcp-serve"]
    }
  }
}
```

이미 다른 서버가 있으면 `mcpServers` 아래에 `rhwp` 키만 더한다. 파일을 통째로
지우지 않는다. 닥터 `--write .mcp.json` 은 파일이 있으면 exit 2.

## claude-desktop — `claude_desktop_config.json` (A형)

Windows: `%APPDATA%\Claude\claude_desktop_config.json`

macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

모양은 claude-code 와 같다. 호스트를 재시작한다.

## cursor — `.cursor/mcp.json` (A형)

프로젝트: `.cursor/mcp.json`
전역: `~/.cursor/mcp.json`

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "rhwp",
      "args": ["mcp-serve"]
    }
  }
}
```

프로필로 좁히기 (기존 계약, 새 플래그 아님):

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "rhwp",
      "args": ["mcp-serve", "--profile", "행정서식"]
    }
  }
}
```

## cline — `cline_mcp_settings.json` (A형)

UI 가 `disabled` / `autoApprove` 를 붙일 수 있다. 붙여넣을 핵심은 같다.

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "rhwp",
      "args": ["mcp-serve"],
      "disabled": false
    }
  }
}
```

`disabled:true` 로 남아 있으면 도구가 안 보인다. 온보딩 실패가 아니라 호스트 설정.

## windsurf — `~/.codeium/windsurf/mcp_config.json` (A형)

A형과 동일. 경로가 릴리스마다 옮을 수 있다. 확신도 high.

## vscode — `.vscode/mcp.json` (B형)

최상위 키가 `mcpServers` 가 아니라 `servers` 다. `type: stdio` 가 필요하다.

```json
{
  "servers": {
    "rhwp": {
      "type": "stdio",
      "command": "rhwp",
      "args": ["mcp-serve"]
    }
  }
}
```

Windows 절대 경로:

```json
{
  "servers": {
    "rhwp": {
      "type": "stdio",
      "command": "C:\\src\\rhwp\\target\\release\\rhwp.exe",
      "args": ["mcp-serve"]
    }
  }
}
```

사용자 `settings.json` 의 `mcp.servers` 도 같은 B형이다. A형을 그대로 넣으면
호스트가 무시한다.

## gemini-cli — `~/.gemini/settings.json` (A형)

A형. `mcpServers` 키.

## qwen-code — `~/.qwen/settings.json` (A형)

Gemini CLI 포크. 같은 A형.

## roo — `.roo/mcp.json` (A형)

프로젝트 파일. 전역 `mcp_settings.json` 도 A형.

## kilo — `.kilocode/mcp.json` (A형)

roo 와 같은 모양.

## kiro — `.kiro/settings/mcp.json` (A형)

작업공간 / `~/.kiro/settings/mcp.json`.

## amazon-q — `.amazonq/mcp.json` (A형)

전역 `~/.aws/amazonq/mcp.json`.

## zed — `settings.json` → `context_servers`

```json
{
  "context_servers": {
    "rhwp": {
      "source": "custom",
      "command": {
        "path": "rhwp",
        "args": ["mcp-serve"]
      }
    }
  }
}
```

절대 경로:

```json
{
  "context_servers": {
    "rhwp": {
      "source": "custom",
      "command": {
        "path": "/src/rhwp/target/release/rhwp",
        "args": ["mcp-serve"]
      }
    }
  }
}
```

스키마가 릴리스마다 바뀔 수 있다. 확신도 medium. 최신 Zed 문서를 한 번 본다.

## goose — `~/.config/goose/config.yaml`

```yaml
rhwp:
  type: stdio
  cmd: rhwp
  args: [mcp-serve]
```

Windows 절대 경로:

```yaml
rhwp:
  type: stdio
  cmd: C:\src\rhwp\target\release\rhwp.exe
  args: [mcp-serve]
```

키 이름이 `command` 가 아니라 `cmd` 다.

## continue — `~/.continue/config.yaml`

```yaml
mcpServers:
  - name: rhwp
    command: rhwp
    args: [mcp-serve]
```

목록이지 맵이 아니다. `name` 이 빠지면 호스트가 거부할 수 있다.

## 닥터로 같은 조각을 받기

```bash
python tools/agent_onboarding/rhwp_doctor.py --host vscode --json
python tools/agent_onboarding/rhwp_doctor.py --host zed --json
python tools/agent_onboarding/rhwp_doctor.py --host goose --json
python tools/agent_onboarding/rhwp_doctor.py --host continue --json
```

`mcpHost.snippet` 을 설정 파일에 병합한다. `--write` 는 A형만 쓰므로
VS Code/Zed/YAML 에 `--write` 하지 않는다.

## 병합 규칙

1. 파일이 없으면 A형은 `--write` 가능.
2. 파일이 있으면 해당 키만 추가. 다른 서버를 지우지 않는다.
3. 기존 `rhwp` 키가 있으면 사람이 비교한다. `--force` 는 명시적 덮어쓰기.
4. JSON/YAML 을 손으로 고친 뒤 구문 오류면 호스트가 서버를 안 띄운다.

## 손 검증

```bash
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | rhwp mcp-serve
```

PowerShell 에서 한글/UTF-8 파이프가 깨지면 임시 파일로 JSON 줄을 넘긴다.
서버는 stdin EOF 에서 끝난다.

## 실패와 온보딩 예외의 구분

| 증상 | 온보딩 예외인가 | 다음 |
|---|---|---|
| 호스트가 command 를 못 찾음 | 아님. 바이너리 자리 | [binary-discovery.md](binary-discovery.md) |
| 파일이 이미 있어 `--write` 거부 | `write_exists` exit 2 | `--force` 또는 병합 |
| 도구 목록이 비어 있음 | 아님. 프로필/disabled | `rhwp-mcp-session` |
| 네트워크 없음 | `no_network` (비임계) | 로컬 stdio 는 동작 |

## 관련 테스트

- `TestMcpHostShapes.test_every_catalog_host_builds`
- `TestMcpHostShapes.test_no_host_invents_a_port`
- `TestMcpHostShapes.test_vscode_is_shape_b`
- `TestMcpHostShapes.test_zed_uses_context_servers`
- `TestMcpHostShapes.test_goose_uses_cmd_key`
- `TestMcpHostShapes.test_continue_is_list`
