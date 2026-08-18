# E13 금액은 extract-data EV 로만

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-AMOUNT-REWRITE`
- 픽스처: `fixtures/envelopes/extract_amount.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

extract-data --kind amount 가 '3,180백만원' → normalized 3180000000.

## 절차

1. extract-data --kind amount 가 '3,180백만원' → normalized 3180000000.

2. CLAIM 은 raw 와 normalized 를 그대로 적는다.

3. 에이전트가 31.8억/3.2천억으로 다시 쓰면 ST-AMOUNT-REWRITE.

4. currency=KRW 가 있으면 복사한다. 없으면 붙이지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-AMOUNT-REWRITE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/extract_amount.json
