# E19 command 필드로 제3자 재현

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-SKIP-ENGINE`
- 픽스처: `fixtures/ledgers/gov_rfp.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

고객: '이 문장 어디서 왔나'.

## 절차

1. 고객: '이 문장 어디서 왔나'.

2. 답: EV-1 → 과업지시서.hwp section=0 paragraph=12 page=2 → command.

3. command 를 다시 치도록 안내한다. 스크린샷만 보내지 않는다.

4. 경로가 상대면 corpus 루트를 같이 준다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-SKIP-ENGINE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/ledgers/gov_rfp.json
