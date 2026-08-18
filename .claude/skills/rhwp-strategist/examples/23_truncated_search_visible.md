# E23 omittedCount 를 대장에 남긴다

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-TRUNCATE-HIDE`
- 픽스처: `fixtures/ledgers/gov_rfp_truncated.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

절단 행은 매치 EV 와 별개 배열이다. EV 를 줄여 숨기지 않는다.

## 절차

1. 절단 행은 매치 EV 와 별개 배열이다. EV 를 줄여 숨기지 않는다.

2. 회신에 totalMatchCount 와 omittedCount 를 적는다.

3. 절단된 나머지를 에이전트 기억으로 채우지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-TRUNCATE-HIDE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/ledgers/gov_rfp_truncated.json
