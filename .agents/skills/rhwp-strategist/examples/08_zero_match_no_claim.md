# E08 0건 질문에는 CLAIM 이 없다

- 코퍼스 시나리오: `quarterly`
- 정지 규칙: `ST-FORECAST`
- 픽스처: `fixtures/ledgers/quarterly_zero_q3.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

Q3 키워드 '시장점유율' 은 코퍼스에 없다. search 0건.

## 절차

1. Q3 키워드 '시장점유율' 은 코퍼스에 없다. search 0건.

2. 골격은 그 절에 '근거 없음'만 적고 CLAIM 을 만들지 않는다.

3. 에이전트가 그 절에 전망을 쓰면 ST-FORECAST.

4. noEvidenceQuestions 에 Q3 가 들어 있는지 확인한다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-FORECAST` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/ledgers/quarterly_zero_q3.json
