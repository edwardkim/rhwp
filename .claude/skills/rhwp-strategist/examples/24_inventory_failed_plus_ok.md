# E24 지도 수치가 선언과 같다

- 코퍼스 시나리오: `mixed_failed`
- 정지 규칙: `ST-DROP-FAILED`
- 픽스처: `fixtures/corpus_maps/mixed_failed.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

documentCount == documents.length 계약.

## 절차

1. documentCount == documents.length 계약.

2. ok + failed == documentCount.

3. 계약 시험이 픽스처마다 이 등식을 본다.

4. 에이전트가 요약에서 failed 를 빼면 등식이 깨진 회신이 된다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-DROP-FAILED` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/corpus_maps/mixed_failed.json
