# E14 연결 없는 CLAIM

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-UNLINKED`
- 픽스처: `fixtures/validate/unlinked.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

문장에 CLAIM-1 은 있으나 EV- 토큰이 없다.

## 절차

1. 문장에 CLAIM-1 은 있으나 EV- 토큰이 없다.

2. kind=unlinked, exit 3.

3. 같은 문단 끝에 [근거: EV-1] 을 붙이거나 CLAIM 을 제거한다.

4. 다른 문단의 EV 는 동거로 치지 않는다. 단위는 문단·표 행.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-UNLINKED` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/validate/unlinked.json
