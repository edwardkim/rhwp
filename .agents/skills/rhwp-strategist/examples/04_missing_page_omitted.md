# E04 page 가 없으면 키를 만들지 않는다

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-INVENT-PAGE`
- 픽스처: `fixtures/envelopes/search_missing_page.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

선행연구 중간본의 어떤 문단은 조판 전이라 search 매치에 page 가 없다.

## 절차

1. 선행연구 중간본의 어떤 문단은 조판 전이라 search 매치에 page 가 없다.

2. copy_coords 가 page 키를 생략한 EV 가 대장에 있다.

3. 에이전트가 '아마 12쪽'을 넣으면 ST-INVENT-PAGE.

4. 인용 라벨: `선행연구_중간.hwpx (section=0, paragraph=88, charOffset=14)`.

5. 재독은 command 를 다시 쳐 같은 키 집합이 나오는지 본다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-INVENT-PAGE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/search_missing_page.json
