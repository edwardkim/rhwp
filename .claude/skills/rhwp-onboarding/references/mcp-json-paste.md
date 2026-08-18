# `.mcp.json` 붙여넣기 — 호스트별 실물 조각

전송은 **stdio JSON-RPC** 뿐이다. 포트·토큰·TLS 설정이 없다.
`rhwp` 가 PATH 에 없으면 `command` 에 절대 경로를 넣는다.
닥터가 그 절대 경로를 채워 준다.

정본: [`mydocs/manual/mcp_attach_kit.md`](../../../../mydocs/manual/mcp_attach_kit.md),
[`mydocs/manual/mcp_integration_guide.md`](../../../../mydocs/manual/mcp_integration_guide.md).

## 닥터로 조각 얻기

```bash
python tools/agent_onboarding/rhwp_doctor.py --json
python tools/agent_onboarding/rhwp_doctor.py --host cursor --json
python tools/agent_onboarding/rhwp_doctor.py --list-hosts
python tools/agent_onboarding/rhwp_doctor.py --write .mcp.json          # 없을 때만
python tools/agent_onboarding/rhwp_doctor.py --write .mcp.json --force  # 덮어쓰기
```

`--write` 는 항상 A형(`.mcp.json`)만 기록한다. VS Code/Zed/Goose 모양은
리포트 `mcpHost.snippet` 을 사람이 해당 설정 파일에 병합한다.
기존 파일이 있으면 `--force` 없이 종료 코드 2 (`write_exists`).

## 공통 규칙

1. 키를 통째로 덮지 말고 `mcpServers.rhwp`(또는 호스트 해당 키)만 병합한다.
2. `args` 는 `["mcp-serve"]`. 새 서버 바이너리를 만들지 않는다.
3. Windows 절대 경로는 역슬래시를 JSON 에서 이스케이프한다.
   예: `"C:\\repo\\target\\release\\rhwp.exe"`.
4. 일부 호스트는 `cmd /c` 래퍼가 필요하다. 기본은 직접 실행이다.
5. 서버는 stdin EOF 에서 종료한다.

## 손 검증 (호스트 없이)

```bash
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | rhwp mcp-serve
```

`tools/list` 가 도구 이름을 내면 배선은 된 것이다. 세션 도구 선택은
`rhwp-mcp-session` 으로 넘긴다.

## 호스트 표

| id | 파일 | 모양 |
|---|---|---|
| `claude-code` | `.mcp.json` | A |
| `claude-desktop` | `%APPDATA%/Claude/claude_desktop_config.json` | A |
| `cursor` | `.cursor/mcp.json` | A |
| `cline` | `cline_mcp_settings.json` | A |
| `windsurf` | `~/.codeium/windsurf/mcp_config.json` | A |
| `vscode` | `.vscode/mcp.json` | B |
| `gemini-cli` | `~/.gemini/settings.json` | A |
| `qwen-code` | `~/.qwen/settings.json` | A |
| `roo` | `.roo/mcp.json` | A |
| `kilo` | `.kilocode/mcp.json` | A |
| `kiro` | `.kiro/settings/mcp.json` | A |
| `amazon-q` | `.amazonq/mcp.json` | A |
| `zed` | `settings.json → context_servers` | zed |
| `goose` | `~/.config/goose/config.yaml` | goose |
| `continue` | `~/.continue/config.yaml` | continue |

## claude-code

- 파일: `.mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host claude-code --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## claude-desktop

- 파일: `%APPDATA%/Claude/claude_desktop_config.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host claude-desktop --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## cursor

- 파일: `.cursor/mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host cursor --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## cline

- 파일: `cline_mcp_settings.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host cline --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## windsurf

- 파일: `~/.codeium/windsurf/mcp_config.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host windsurf --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## vscode

- 파일: `.vscode/mcp.json`
- 모양: `B`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host vscode --json`

```
{ "servers": { "rhwp": { "type": "stdio", "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## gemini-cli

- 파일: `~/.gemini/settings.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host gemini-cli --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## qwen-code

- 파일: `~/.qwen/settings.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host qwen-code --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## roo

- 파일: `.roo/mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host roo --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## kilo

- 파일: `.kilocode/mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host kilo --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## kiro

- 파일: `.kiro/settings/mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host kiro --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## amazon-q

- 파일: `.amazonq/mcp.json`
- 모양: `A`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host amazon-q --json`

```
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## zed

- 파일: `settings.json → context_servers`
- 모양: `zed`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host zed --json`

```
{ "context_servers": { "rhwp": { "source": "custom", "command": { "path": "rhwp", "args": ["mcp-serve"] } } } }
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## goose

- 파일: `~/.config/goose/config.yaml`
- 모양: `goose`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host goose --json`

```
rhwp:
  type: stdio
  cmd: rhwp
  args: [mcp-serve]
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## continue

- 파일: `~/.continue/config.yaml`
- 모양: `continue`
- 닥터: `python tools/agent_onboarding/rhwp_doctor.py --host continue --json`

```
mcpServers:
  - name: rhwp
    command: rhwp
    args: [mcp-serve]
```

PATH 에 없으면 `command`(또는 Zed `path`, Goose `cmd`)를 닥터가 준
절대 경로로 바꾼다. 프로필로 도구를 좁히려면 기존 계약대로
`args` 에 `"--profile"`, `"행정서식"` 을 덧붙인다. 새 플래그를 만들지 않는다.

## Windows 메모

- 실행 파일 이름은 `rhwp.exe` 다. 닥터 `_exe_name()` 이 OS 를 본다.
- 콘솔 코드페이지와 무관하게 스니펫 JSON 은 UTF-8 이다.
- 경로에 공백이 있으면 절대 경로 한 문자열로 둔다. argv 를 쪼개지 않는다.

## 성공 판정

1. 호스트가 `initialize` 에 `serverInfo.name == "rhwp"` 로 답한다.
2. `tools/list` 가 비어 있지 않다.
3. 포트 리스너를 열지 않았다.

세션 도구(`hwp_open` …) 사용법은 이 문서가 아니라 `rhwp-mcp-session` 이다.
