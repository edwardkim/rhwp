# E01 정부과제 수주 근거 — 전 경로

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-SKIP-ENGINE`
- 픽스처: `fixtures/engagements/gov_rfp.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

고객이 공고·과업지시서·예산서·선행연구·평가표 더미를 주고 "이걸로 수주 근거 보고서를 만들어 줘"라고 한다. 증상도 큐도 아니다.

## 절차

1. 고객이 공고·과업지시서·예산서·선행연구·평가표 더미를 주고 "이걸로 수주 근거 보고서를 만들어 줘"라고 한다. 증상도 큐도 아니다.

2. 질문을 기능·예산·배점 셋으로 쪼개 engagement.json 을 쓴다. 수주 확률 질문은 넣지 않는다.

3. `python3 tools/strategist/engagement.py engagement.json --bin $RHWP_BIN` — 즉흥 search 금지.

4. corpus_map 이 문서 5건, mappedCount 5 인지 확인한다.

5. evidence.json 에서 Q1/Q2/Q3 EV 를 읽고 CLAIM 을 작성한다. 각 문단에 [근거: EV-n] 을 남긴다.

6. --validate 가 verdict=pass 일 때만 spec 을 납품한다.

7. 회신 1부에 documentCount·evidenceCount·verdict 를 숫자로 적는다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-SKIP-ENGINE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/engagements/gov_rfp.json
