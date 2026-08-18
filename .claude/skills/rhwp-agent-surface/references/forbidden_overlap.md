# 이 스킬이 다시 쓰지 않는 것

이슈 #5326 범위. 세 스킬은 **덮어쓰지 않는다.**

## `rhwp-mcp-session` — 호스트 부착

그 스킬의 축:

- `.mcp.json` / VS Code `servers` / 절대 경로
- `initialize` 핸드셰이크 줄
- 무상태 vs 세션 **선택** (한 방이면 무상태, 반복이면 세션)
- resources URI 소비
- 호스트별 부착 킷

여기의 축:

- 세션 조각의 **SSOT** (`ALL_SESSION_TOOLS` + `served_tools`)
- 닫힌 핸들은 런타임 `isError` + `nextCall`
- 세션을 `--mcp` 선언에 복제하지 말 것

`.mcp.json` 예시를 이 스킬에 또 적지 않는다. 부착이 필요하면
`rhwp-mcp-session` 을 연다.

## `rhwp-cli` — 명령 매핑

그 스킬의 축:

- "SVG 로 내보내" → `export-svg`
- 레이아웃 버그 디버깅 순서 (`--debug-overlay` → `dump-pages` → `dump`)
- HWPX→HWP 저장 계약 분석 (`hwp5-inventory-diff`)

여기의 축:

- 그 명령이 **어느 층**에 있고, `--json` 계약이 있는지
- `rhwp capabilities --search` 로 이름을 찾는 법
- 새 명령을 더할 때 `capabilities_command_entries()` 에 등재

요청→명령 표를 여기 복제하지 않는다.

## `rhwp-codex` — 대전 항해

그 스킬의 축:

- `mydocs/manual/agent_codex/` 장 순서
- `tools/gen_agent_codex.py` 재생성·신선도
- 철학 4규약 입장

여기의 축:

- 대전이 가리키는 **표면 계약**(판정=데이터, 단일 출처)을 구현/운용 규칙으로 닫기
- 대전 장을 재작성하거나 생성기를 돌리지 않음

## gym

gym 팩·트레이스·입장 티켓은 이 스킬의 실행 경로가 아니다.
단어 "gym" 은 "금지"와 함께만 쓴다. 그 트리 경로를 실행 경로로 안내하지 않는다.
