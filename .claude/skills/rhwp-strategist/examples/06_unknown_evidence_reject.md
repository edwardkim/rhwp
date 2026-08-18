# E06 지어낸 EV 는 거부

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-UNKNOWN-EV`
- 픽스처: `fixtures/validate/unknown_evidence.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

에이전트가 인용하고 싶은 문장에 번호가 없어 EV-99 를 붙였다.

## 절차

1. 에이전트가 인용하고 싶은 문장에 번호가 없어 EV-99 를 붙였다.

2. validate kind=unknown-evidence, exit 3.

3. 고침: EV-99 를 지우고, 필요하면 keywords 에 그 어휘를 넣어 엔진 재실행.

4. 재실행 후 새 EV-n 이 생기면 그 id 만 쓴다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-UNKNOWN-EV` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/validate/unknown_evidence.json
