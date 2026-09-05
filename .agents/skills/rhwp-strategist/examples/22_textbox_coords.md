# E22 글상자 좌표는 textbox 키

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-INVENT-PAGE`
- 픽스처: `fixtures/envelopes/search_textbox.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

공고 표지의 글상자에 '필수기능'이 있다.

## 절차

1. 공고 표지의 글상자에 '필수기능'이 있다.

2. 매치에 textbox 식별자가 있으면 복사, 없으면 생략.

3. 글상자 텍스트를 본문 paragraph 로 옮긴 척하지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-INVENT-PAGE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/search_textbox.json
