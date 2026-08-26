# E02 분기 전략 보고서

- 코퍼스 시나리오: `quarterly`
- 정지 규칙: `ST-FORECAST`
- 픽스처: `fixtures/engagements/quarterly.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

2Q 실적·파이프·인력·리스크 문서가 있다. 목표는 3Q 전략 보고서.

## 절차

1. 2Q 실적·파이프·인력·리스크 문서가 있다. 목표는 3Q 전략 보고서.

2. 질문: 수주액/파이프, 공수, 리스크. 시장 점유율 전망은 질문이 아니다.

3. 엔진 실행 후 kind=data 금액 EV 가 있으면 그 normalized 만 인용한다.

4. 리스크 절에 문서에 없는 '낙관 시나리오'를 덧붙이지 않는다.

5. validate pass 후 납품. SWS 레벨을 회신에 명시한다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-FORECAST` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/engagements/quarterly.json
