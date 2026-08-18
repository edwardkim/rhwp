# E09 출처 없는 시장 전망은 비범위

- 코퍼스 시나리오: `—`
- 정지 규칙: `ST-FORECAST`
- 픽스처: `fixtures/stop_rules.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

고객: '그래서 내년 시장이 어떻게 될지도 한 단락'. 코퍼스에 전망 문서 없음.

## 절차

1. 고객: '그래서 내년 시장이 어떻게 될지도 한 단락'. 코퍼스에 전망 문서 없음.

2. 거부. ST-FORECAST. 새 CLI 를 만들어 전망하지 않는다.

3. 대안: '성장률/전년대비' 질문을 추가해 엔진을 다시 돌리자. 0건이면 공란.

4. 회신 3부에 키워드 후보만 적는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-FORECAST` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/stop_rules.json
