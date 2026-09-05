# E21 표 칸 좌표는 cell 키

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-INVENT-PAGE`
- 픽스처: `fixtures/envelopes/search_cell.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

평가표 배점이 표 칸에 있다. search 매치에 cell 이 붙는다.

## 절차

1. 평가표 배점이 표 칸에 있다. search 매치에 cell 이 붙는다.

2. copy_coords 가 cell 을 복사한 EV 를 인용한다.

3. 칸 좌표를 paragraph 로 환산하지 않는다.

4. 연결표 라벨에 cell 을 그대로 적는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-INVENT-PAGE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/search_cell.json
