---
kind: guide
status: active
canonical: mydocs/manual/agent_codex/README.md
last_verified: 2026-08-11
---

# 에이전트 대전(Codex) — 입장

**30초 규약**: [00_서문](00_서문.md)에서 철학 4개(판정=데이터·결정론·출처
표지·원본 무훼손)를 읽고 → [01_판단트리](01_판단트리.md)로 요청을 갈래
지어 → 해당 장(10~85)의 **실측 표본**을 그대로 흉내내면 된다. 장의 예시는
전부 이 저장소 픽스처에 실제로 돌린 봉투다.

- 생성: `python tools/gen_agent_codex.py` (표본 재실행 포함)
- 신선도 검사: `python tools/gen_agent_codex.py --check` (차이 → exit 3)
- 커버리지 가드: `tests/agent_codex_contract.rs` (전 명령 장 보유 판정)
- 필드 사전: [지식지도 §2-2](../agent_knowledge_map.md) · 스킬 진입:
  `rhwp-codex`
