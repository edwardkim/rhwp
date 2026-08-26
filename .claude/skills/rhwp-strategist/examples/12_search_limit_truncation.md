# E12 searchLimit 절단은 숨기지 않는다

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-TRUNCATE-HIDE`
- 픽스처: `fixtures/envelopes/search_truncated.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

engagement.searchLimit=5, '데이터' 키워드 totalMatchCount=41.

## 절차

1. engagement.searchLimit=5, '데이터' 키워드 totalMatchCount=41.

2. 대장 truncatedSearches 에 omittedCount=36 이 있다.

3. 5건만 보고 '전수 검색했다'고 쓰지 않는다. ST-TRUNCATE-HIDE.

4. 선택: limit 제거 후 재실행, 또는 키워드를 '데이터 플랫폼'으로 좁힘.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-TRUNCATE-HIDE` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/envelopes/search_truncated.json
