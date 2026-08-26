# E20 scaffold 미광고면 spec 까지가 산출

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-SCAFFOLD-GUESS`
- 픽스처: `fixtures/envelopes/engagement_summary_no_scaffold.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

devel 일부 빌드는 scaffold 가 capabilities 에 없다.

## 절차

1. devel 일부 빌드는 scaffold 가 capabilities 에 없다.

2. 결과 봉투 scaffoldAdvertised=false, artifacts 에 hwpx 없음.

3. `rhwp scaffold` 를 추측 실행하면 ST-SCAFFOLD-GUESS.

4. 검증된 spec.json + evidence.json 을 함께 납품하고 그 사실을 명시한다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-SCAFFOLD-GUESS` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/engagement_summary_no_scaffold.json
