# E11 Chief 목표형 요청 인계

- 코퍼스 시나리오: `—`
- 정지 규칙: `ST-LAYER-MIX`
- 픽스처: `fixtures/handoff.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

chief 큐 항목: '이 자료로 수주하고 싶다' needs-agent.

## 절차

1. chief 큐 항목: '이 자료로 수주하고 싶다' needs-agent.

2. 이 스킬이 objective/corpus 초안을 받아 engagement.json 을 닫는다.

3. 큐 상태 머신·우선순위는 여기서 구현하지 않는다.

4. 끝나면 확인/산출/다음 3부를 chief 회신 형식으로 되돌린다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-LAYER-MIX` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/handoff.json
