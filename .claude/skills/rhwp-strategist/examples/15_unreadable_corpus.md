# E15 부분 가독 코퍼스

- 코퍼스 시나리오: `mixed_failed`
- 정지 규칙: `ST-DROP-FAILED`
- 픽스처: `fixtures/corpus_maps/mixed_failed.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

공개 공고와 평가기준만 ok. 내부 문서는 암호.

## 절차

1. 공개 공고와 평가기준만 ok. 내부 문서는 암호.

2. 가독 문서의 EV 만으로 주장한다. 암호 문서 내용을 추측하지 않는다.

3. L2 는 unreadable 을 남긴 것이 준수다. 실패를 지워야 L2 가 아니다.

4. 고객이 암호를 주면 같은 objective 로 엔진을 다시 돈다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-DROP-FAILED` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/corpus_maps/mixed_failed.json
