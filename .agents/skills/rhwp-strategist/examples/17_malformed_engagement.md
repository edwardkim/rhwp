# E17 깨진 engagement 는 exit 2

- 코퍼스 시나리오: `—`
- 정지 규칙: `ST-SKIP-ENGINE`
- 픽스처: `fixtures/engagements/invalid_missing_questions.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

questions 누락, corpus 폴더 없음, 빈 배열, 깨진 JSON.

## 절차

1. questions 누락, corpus 폴더 없음, 빈 배열, 깨진 JSON.

2. 엔진은 대장을 만들지 않는다. 부분 산출을 쓰지 않는다.

3. 픽스처 invalid_* 를 보고 필드를 고친다.

4. exit 2 를 '바이너리 고장'으로 오해하지 않는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-SKIP-ENGINE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/engagements/invalid_missing_questions.json
