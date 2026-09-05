# #5326 에이전트 표면 플레이북 스킬 신설 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5326
브랜치: `feat/agent-surface` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-agent-surface/` · `scripts/tests/test_agent_surface.py` ·
`tests/agent_surface_skill_contract.rs` · 본 문서
비범위: `gym/` · `rhwp-mcp-session` · `rhwp-cli` · `rhwp-codex` · 다른 스킬 ·
새 CLI · DocumentCore 편집 로직 · 열린 PR 이 만지는 공유 파일

## 무엇을

실 에이전트가 새 CLI/MCP 조각을 더하거나 기존 표면을 굴릴 때
**3층 계약**(CLI JSON · MCP 무상태 · MCP 세션)과 `rhwp capabilities` 가
도구 정의의 단일 출처라는 규칙을 스킬로 닫는다.

mcp-session(호스트 부착) · cli(명령 매핑) · codex(대전 항해) 와 겹치지 않는
**표면 계약** 축이다. 그 세 스킬을 다시 쓰지 않는다.

## 왜

플레이북 `mydocs/manual/agent_surface_playbook.md` 는 정본이지만 길다.
에이전트가 층을 섞어 더하고, `mcp_tool_definitions()` 가 아닌 곳에 목록을
복제하고, `identical:false` 를 오류로 재시도하는 사고가 난다.
스킬이 30초 판단 + 규칙 3줄 + 예외 4바늘 + 수용 체크리스트를 한 입구로 모은다.

이슈 DoD: additions 5000–10000, 최소 5000, PR 전 `cargo fmt --all -- --check`,
gym 금지, 새 CLI 금지.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-surface` 에
   `feat/agent-surface` 를 `upstream/devel` 에서 분기.
2. 이름은 소스에서만 추출.
   생성기 `.agents/skills/rhwp-agent-surface/references/_gen_pack.py`.
   - `mcp_tool_definitions()` → 무상태
   - `ALL_SESSION_TOOLS` → 세션 18종
   - `capabilities_command_entries()` → CLI
   - `PROFILES` → 프로필 경계
3. `SKILL.md` 를 내비게이터로 두고 `references/` 에 층·규칙·검색·추가 절차·
   수용 기준·예외·가드·경계, `examples/` 에 레시피, `fixtures/` 에 기계 카드.
4. `scripts/tests/test_agent_surface.py` 가 소스와 픽스처를 대조하고
   마크다운의 `hwp_*` 가 allowlist 밖이면 실패.
5. `tests/agent_surface_skill_contract.rs` 가 스킬 트리 존재·규칙 문장·
   금지 겹침·예외 바늘을 파일만으로 고정 (바이너리 호출 없음 — 새 CLI 없음).

새 CLI 없음. `mcp_serve.rs` / `document_core` / 다른 스킬을 바꾸지 않음.

## 3층 (스킬에 옮긴 것)

| 층 | SSOT |
|---|---|
| CLI JSON | 명령 구현 + `*_json_value` + `capabilities_command_entries()` |
| MCP 무상태 | `mcp_tool_definitions()` |
| MCP 세션 | `ALL_SESSION_TOOLS` + `served_tools()` |

세션은 `--mcp` 선언에 없다. `tools/list` 가 정본.

## 규칙 3줄

1. 선언·실행·문서는 한 곳에서 갈라진다. 가드:
   `capabilities_mcp_covers_every_json_command`,
   `tools_list_matches_capabilities_manifest`.
2. 새 편집 로직 금지. `set_field_value_by_name_at` · `replace_all_native` ·
   `grep` · `collect_field_records` · `extract_tables` · `edit_serialize`.
3. 판정은 데이터. `identical:false` / `replacedCount:0` / `notFound` 는 필드.
   `isError` 는 런타임만.

## 예외 4바늘

1. `untrustedContent` 키 부재 → 미표기. `false` 가 아님.
2. 드리프트 가드 실패 → `mcp_tool_definitions()` 한 줄. 제외 목록 감 금지.
3. 닫힌 핸들 → `isError` + `nextCall.hwp_open`. 옛 docId 재사용 금지.
4. 프로필 차단 → 경계. `tools/call` 우회 불가. 없는 프로필 exit 2.

## 쓰지 않은 것

- `rhwp-mcp-session` 의 `.mcp.json` 부착 절차
- `rhwp-cli` 의 요청→명령 표
- `rhwp-codex` 의 대전 장 순서
- gym 팩
- `mydocs/manual/agent_capability_registry.md` 행
  (열린 형제 PR 이 같은 파일을 만지므로 충돌을 피함)

## 검증

```
python .agents/skills/rhwp-agent-surface/references/_gen_pack.py
python -m unittest scripts.tests.test_agent_surface
cargo fmt --all -- --check
```

Rust 계약 시험은 `cargo test --test agent_surface_skill_contract` 로 돌릴 수 있다.
바이너리를 부르지 않으므로 새 명령을 요구하지 않는다.
