# E10 표가 잘린다 ≠ 전략

- 코퍼스 시나리오: `—`
- 정지 규칙: `ST-LAYER-MIX`
- 픽스처: `fixtures/handoff.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

고객이 전략 회의 중 '이 과업지시서 표가 잘려요'를 덧붙인다.

## 절차

1. 고객이 전략 회의 중 '이 과업지시서 표가 잘려요'를 덧붙인다.

2. 표 절단은 FDE 증상. 이 스킬이 고치지 않는다.

3. 전략 엔게이지먼트는 계속하되, 표 이슈는 rhwp-fde 로 인계한다.

4. FDE 추측('렌더러 버그일 듯')을 CLAIM 으로 쓰지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-LAYER-MIX` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/handoff.json
