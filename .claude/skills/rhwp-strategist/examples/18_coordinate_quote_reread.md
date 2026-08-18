# E18 재독 — 좌표에서 인용이 나와야 한다

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-INVENT-PAGE`
- 픽스처: `fixtures/transcripts/reread_ev3.txt`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

EV-3 command 를 그대로 실행한다.

## 절차

1. EV-3 command 를 그대로 실행한다.

2. 매치 text 가 quote 와 같고, 좌표 키가 같다.

3. 어긋나면 대장을 손으로 고치지 않고 엔진을 다시 돈다.

4. 이것이 SWS L1 이다. 재독 없이 '맞을 것'이라고 쓰지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-INVENT-PAGE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/transcripts/reread_ev3.txt
