# rhwp-agent-surface 레퍼런스

정본: [`mydocs/manual/agent_surface_playbook.md`](../../../../mydocs/manual/agent_surface_playbook.md).
이 폴더는 스킬이 한 화면에서 닫히도록 층을 나눈 안내다. 숫자를 계약으로 적지 않는다.

## 읽기 순서

1. [three_layers.md](three_layers.md) — 3층과 각 층의 단일 출처
2. [rule1_single_source.md](rule1_single_source.md) — 선언·실행·문서 한 곳
3. [rule2_reuse_core.md](rule2_reuse_core.md) — 편집 로직 발명 금지
4. [rule3_judgment_is_data.md](rule3_judgment_is_data.md) — 판정은 데이터
5. [capabilities_how_to.md](capabilities_how_to.md) — `rhwp capabilities` · `--mcp` · `--search`
6. [add_surface_piece.md](add_surface_piece.md) — 조각을 더하는 절차
7. [acceptance_checklist.md](acceptance_checklist.md) — 수용 기준
8. [exception_paths.md](exception_paths.md) — 키 부재·가드·닫힌 핸들·프로필
9. [drift_guards.md](drift_guards.md) — 가드 이름과 고치는 곳
10. [forbidden_overlap.md](forbidden_overlap.md) — mcp-session / cli / codex 와 경계

## 생성기

[`_gen_pack.py`](_gen_pack.py) 가 `src/main.rs` `mcp_tool_definitions()` ·
`capabilities_command_entries()`, `src/agent_profiles.rs` `ALL_SESSION_TOOLS` ·
`PROFILES` 에서 이름을 읽어 `../fixtures/` 를 다시 쓴다.

```bash
python .claude/skills/rhwp-agent-surface/references/_gen_pack.py
python -m unittest scripts.tests.test_agent_surface
```

개수는 출력에 실려도 계약이 아니다. 소스에서 빠진 이름을 픽스처에 넣으면
계약 시험이 실패한다.
