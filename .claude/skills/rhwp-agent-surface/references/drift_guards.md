# 드리프트 가드 — 이름이 가리키는 원천

카드: [`../fixtures/drift/`](../fixtures/drift/).

가드는 "목록을 두 벌 유지하지 마라"는 규칙 1 의 이빨이다.
가드가 붉으면 원천을 맞춘다. 가드를 약화하지 않는다.

## 무상태 표면

| 가드 | 파일 | 왼쪽 | 오른쪽 |
|---|---|---|---|
| `capabilities_mcp_covers_every_json_command` | `tests/cli_json_contract.rs` | `capabilities` 의 `json:true` 명령 | `--mcp` 도구의 `cli.command` |
| `capabilities_mcp_tool_definitions_contract` | 같은 파일 | `--mcp` 각 도구 | `name`/`description`/`inputSchema`/`cli.command` |
| `tools_list_matches_capabilities_manifest` | `tests/mcp_server_contract.rs` | `--mcp` 도구 이름 | `mcp-serve` `tools/list` (세션 제외) |

제외(사유가 코드 주석에 있음):

- `capabilities` — 도구가 아니라 도구 목록의 원천
- `dump-pages` — 진단 계약만, #3608 1-C 가 MCP 짝을 요구하지 않음

새 제외는 이슈와 주석 없이 넣지 않는다.

## CLI 목록

| 가드 | 왼쪽 | 오른쪽 |
|---|---|---|
| `capabilities_covers_every_help_command` | `rhwp --help` | `capabilities.commands[].name` |
| `capabilities_version_matches_version_flag` | `capabilities.version` | `rhwp --version` |

## 스킬 표류

| 가드 | 왼쪽 | 오른쪽 |
|---|---|---|
| `skills_reference_only_real_commands` | `.claude/skills/**/SKILL.md` 의 `rhwp <토큰>` | capabilities ∪ --help |

이 스킬이 존재하지 않는 명령을 안내하면 이 가드가 붉다.
`rhwp capabilities` · `rhwp search` · `rhwp ir-diff` 처럼 실명만 적는다.

## 출처 표지

`tests/provenance_contract.rs` 가 선언이 아니라 **실제 봉투 값**을 보고
누락을 잡는다. 새 `--json` 봉투는 이 가드가 통과할 때까지 표지를 단다.

## 실패했을 때 하는 일

1. 테스트 출력의 missing 배열을 읽는다.
2. 빠진 이름을 `mcp_tool_definitions()` 또는 `capabilities_command_entries()` 에 추가.
3. 서버 전용 로직을 넣지 말고 기존 코어+helper 를 연결 (규칙 2).
4. 로컬에서 그 테스트만 다시 돌린다.

가드 메시지를 "너무 빡세다"고 완화하는 PR 은 표면 계약이 아니다.
