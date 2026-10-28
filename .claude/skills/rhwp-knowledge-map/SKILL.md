---
name: rhwp-knowledge-map
description: 실 에이전트가 rhwp 참조 문서에 들어갈 때 llms.txt → mydocs/manual/agent_knowledge_map.md → 요청에 필요한 canonical 하나 순으로 읽게 합니다. 지도는 요약·앵커만. 지도와 상세가 다르면 상세. 봉투 필드 이름은 지도 §2 사전에서만. 트리거 — "지식 지도", "어디 문서부터", "이 필드가 뭐야", "llms.txt 다음", "capabilities 재측정", "last_verified stale", "바이너리 버전 불일치". rhwp-codex(대전 장 항해)·rhwp-agent-surface(3층 계약) 를 다시 쓰지 않습니다.
---

# rhwp-knowledge-map — 지식 지도 진입점 Skill

에이전트가 rhwp 문서를 **어디서 읽을지** 만 고른다.
이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 CLI 도 새 편집
로직도 없다. 정본은 이미 있는 `llms.txt` 와
`mydocs/manual/agent_knowledge_map.md` 다.

이웃 스킬 `rhwp-codex`(대전 장 항해)와 `rhwp-agent-surface`(3층 계약)를
여기서 재작성하지 않는다. 지도는 요약·앵커만 담는다. 기존 행을
다시 쓰지 않는다. 지도와 canonical 이 다르면 **canonical 을 따른다**.

상세는 `references/` 를 연다. 이 파일은 첫 읽기·재측정·§2 사전·예외·
정지·점프만 담는다.

## 바이너리

이 스킬은 새 명령을 만들지 않는다. 재측정은 기존 표면이다.

```bash
rhwp capabilities
rhwp capabilities --mcp
rhwp mcp-serve
```

`mcp-serve` 는 initialize → notifications/initialized → `tools/list`.
세션 도구는 `--mcp` 매니페스트에 없다. 네이티브는 로컬 cargo.
Docker 는 WASM 전용.

## 첫 읽기 순서

| 순서 | 문서 | 범위 |
| --- | --- | --- |
| 1 | `llms.txt` | 머리 + 시작하기. 레시피 전체를 여기서 소화하지 않는다 |
| 2 | `mydocs/manual/agent_knowledge_map.md` | §0 과 요청에 맞는 **한 절** |
| 3 | 그 절 권위 열의 canonical **하나** | 상세. 지도와 다르면 이쪽 |

ROADMAP·대전·표면 플레이북을 첫 문서로 열지 않는다.
지도를 처음부터 끝까지 읽지 않는다 (R10).

## 재측정

지도 §0 숫자를 암기하지 않는다. 손에 든 바이너리로 다시 찍는다.

| ID | 명령 | 보는 것 |
| --- | --- | --- |
| RM01 | `rhwp capabilities` | 명령·플래그·recordFields·종료 코드 |
| RM02 | `rhwp capabilities --mcp` | MCP 무상태 선언 |
| RM03 | `rhwp mcp-serve` + `tools/list` | 세션 포함 실제 목록 |

버전이 다르면 **바이너리가 이긴다**. 검색은
`rhwp capabilities --search <낱말> [--json]`.

## 봉투 필드 사전

필드 이름은 지도 **§2** 가 사전이다. 철자 변형
(`schema_version`, `page_count`, `is_error`)을 만들지 않는다.
뜻은 지도 표 셀에 있고, 이 스킬은 그 행을 옮기지 않는다.
대전·표면 스킬에 필드 뜻을 다시 쓰지 않는다 (R15).

## 지도를 그만 읽고 스킬로 점프할 때

절과 정본을 골랐으면 이 스킬을 닫는다.

| 작업 | 다음 스킬 | 정지 |
| --- | --- | --- |
| 서식·메일머지 | rhwp-form-fill | R08 |
| 표 CSV | rhwp-table-exchange | R08 |
| inspect·redact | rhwp-security-sweep | R08 |
| 폴더 일괄 | rhwp-bulk-pipeline | R08 |
| render-diff | rhwp-visual-regression | R08 |
| MCP 부착 | rhwp-mcp-session | R08 |
| CLI 분석 | rhwp-cli | R08 |
| 긴 문서 좁히기 | rhwp-doc-triage | R08 |
| run·dry-run | rhwp-safe-edit | R08 |
| untrusted* | rhwp-provenance | R08 |
| 영수증 | rhwp-work-receipt | R08 |
| 온보딩 | rhwp-onboarding | R08 |
| 대전 장 항해 | rhwp-codex | R09 |
| 3층 계약·조각 추가 | rhwp-agent-surface | R09 |

채움·표·스윕·배치·세션을 이 스킬 안에서 재구현하지 않는다.

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| R01 | 한 절·정본 하나로 닫힘 | llms.txt → 지도 절 → canonical 하나 |
| R02 | 숫자·개수가 필요 | capabilities / --mcp / tools/list |
| R03 | 필드 이름 | 지도 §2. 없으면 중단 |
| R04 | last_verified 가 30일보다 오래 | 날짜를 보여 주고 중단 |
| R05 | 바이너리 버전 ≠ 지도 §0 | 바이너리. 재측정 |
| R06 | 지도 ≠ canonical | canonical |
| R07 | §2 에 없는 이름 | 발명 금지 |
| R08 | 실무 작업 | 이웃 스킬로 점프 |
| R09 | 대전 항해 또는 3층 계약 | 해당 스킬. 재작성 금지 |
| R10 | 지도 통독 | 거부 |
| R11 | 새 rhwp 하위명령 | 금지 |
| R12 | gym 으로 대체 | 금지 |
| R13 | 지도 행 재서술 | 금지 |
| R14 | §0·§2·§7 수치를 손보기 | 재측정으로만 |
| R15 | 필드 사전을 다른 스킬에 재정의 | 지도 §2 |
| R16 | 이웃 스킬 본문 수정 | 금지 |

**금지 기본값**

- `knowledge-map` / `map` / `field-dict` / `first-read` 전용 명령
- gym pack / gym 과제
- `rhwp-codex` / `rhwp-agent-surface` 본문을 이 PR 에서 재작성
- DocumentCore 편집 로직
- 정본에 없는 필드 이름
- 지도 기존 행을 더 길게 풀어 쓰기

## 예외 네 갈래

- **last_verified stale** — 30일 초과. 사다리를 기억으로 메우지 않음.
  2026-08-18 기준 지도는 `2026-08-11` 이라 신선하다. 시뮬레이션은
  `2025-01-01`.
- **binary version mismatch** — 지도 §0 은 `v0.8.3`, 이 나무
  `Cargo.toml` 은 `0.8.4` 일 수 있다. 바이너리가 이긴다.
- **map vs canonical** — 플래그·종료 코드는 `cli_commands.md`.
- **invented field name** — §2 에 없으면 쓰지 않는다.

[10_stale_last_verified.md](references/10_stale_last_verified.md) ·
[11_version_mismatch.md](references/11_version_mismatch.md) ·
[12_map_vs_canonical.md](references/12_map_vs_canonical.md).

## 인계

- 문서 위치만 물으면 이 스킬에서 끝 (R01)
- 실무는 위 표의 이웃 스킬 (R08)
- 장 항해는 `rhwp-codex` (R09)
- 표면 추가는 `rhwp-agent-surface` (R09)

상세: [14_handoff.md](references/14_handoff.md).

## 봉투

`fixtures/transcripts.json` 은 `llms.txt` 와 지도에서 **발췌**했다.
살아 있는 CLI 를 다시 돌린 결과가 아니다.

## 레퍼런스 목차

1. [00_first_read.md](references/00_first_read.md) — 첫 읽기
2. [01_remeasure.md](references/01_remeasure.md) — 재측정
3. [02_tree.md](references/02_tree.md) — 판단 나무
4. [03_request_map.md](references/03_request_map.md) — 요청 대조
5. [04_boundary.md](references/04_boundary.md) — 대전·표면 경계
6. [05_envelope_dict.md](references/05_envelope_dict.md) — §2 사전
7. [06_canonicals.md](references/06_canonicals.md) — §9 권위
8. [07_section_index.md](references/07_section_index.md) — 절 인덱스
9. [08_jump_to_skill.md](references/08_jump_to_skill.md) — 점프
10. [09_exceptions.md](references/09_exceptions.md) — 예외
11. [10_stale_last_verified.md](references/10_stale_last_verified.md) — 낡은 날짜
12. [11_version_mismatch.md](references/11_version_mismatch.md) — 버전
13. [12_map_vs_canonical.md](references/12_map_vs_canonical.md) — 상세가 이김
14. [13_stop_conditions.md](references/13_stop_conditions.md) — 정지
15. [14_handoff.md](references/14_handoff.md) — 인계
16. [15_pitfalls.md](references/15_pitfalls.md) — 함정
17. [16_journeys.md](references/16_journeys.md) — 여정
18. [17_intent_matrix.md](references/17_intent_matrix.md) — 발화
19. [18_field_lookup.md](references/18_field_lookup.md) — 필드 조회
20. [19_three_questions.md](references/19_three_questions.md) — 3문
21. [20_samples_index.md](references/20_samples_index.md) — 표본
22. [21_contract_tests_index.md](references/21_contract_tests_index.md) — 계약 테스트
23. [22_mcp_remeasure.md](references/22_mcp_remeasure.md) — MCP 재측정
24. [23_transcripts.md](references/23_transcripts.md) — 발췌
25. [24_decision_table.md](references/24_decision_table.md) — 결정표
26. [25_sibling_boundary.md](references/25_sibling_boundary.md) — 이웃
27. [README.md](references/README.md)

기계 가독 픽스처: `fixtures/`.
일한 예: `examples/`.

## 권위

- [`llms.txt`](../../../llms.txt)
- [`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md)
- 처리 결과: [`mydocs/working/agent_knowledge_map_skill.md`](../../../mydocs/working/archives/agent_knowledge_map_skill.md)
