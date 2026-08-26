# E07 플레이스홀더 납품 거부

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-PLACEHOLDER`
- 픽스처: `fixtures/validate/placeholder.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

시간 부족으로 CLAIM-2 가 `[CLAIM-2: 에이전트가 근거 EV-4 로 작성]` 그대로다.

## 절차

1. 시간 부족으로 CLAIM-2 가 `[CLAIM-2: 에이전트가 근거 EV-4 로 작성]` 그대로다.

2. PLACEHOLDER_RE 가 잡아 kind=placeholder.

3. 선택지: 문장을 쓰거나, 그 CLAIM 을 제거하고 미작성을 3부에 적는다.

4. 부분 통과를 '거의 됨'으로 납품하지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-PLACEHOLDER` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/validate/placeholder.json
