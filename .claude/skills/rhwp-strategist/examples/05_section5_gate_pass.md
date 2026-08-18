# E05 §5 게이트 통과

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-GATE-FAIL`
- 픽스처: `fixtures/validate/pass.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

골격 CLAIM-1 플레이스홀더를 실제 문장으로 바꾼다.

## 절차

1. 골격 CLAIM-1 플레이스홀더를 실제 문장으로 바꾼다.

2. 같은 문단에 EV-1, EV-7 이 실존한다.

3. 근거 연결표 행을 같은 id 로 맞춘다.

4. validate: verdict=pass, violationCount=0, exit 0.

5. 이 상태가 아니면 납품하지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-GATE-FAIL` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/validate/pass.json
