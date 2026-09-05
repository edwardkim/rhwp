# E03 실패한 문서는 실패로 남긴다

- 코퍼스 시나리오: `mixed_failed`
- 정지 규칙: `ST-DROP-FAILED`
- 픽스처: `fixtures/corpus_maps/mixed_failed.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

코퍼스 4건 중 암호 1, 손상 1. info 가 실패한다.

## 절차

1. 코퍼스 4건 중 암호 1, 손상 1. info 가 실패한다.

2. corpus_map.documents 길이는 4, mappedCount 는 2, failed 두 행에 infoExit 가 있다.

3. 에이전트가 암호 파일을 지우고 다시 돌리면 ST-DROP-FAILED.

4. search 단계의 추가 실패는 evidence.failures 에 쌓인다. 지도 행은 그대로다.

5. 회신 1부: '읽음 2 / 선언 4 / 실패 암호_내부.hwp exit 2, 손상_백업.hwpx exit 1'.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-DROP-FAILED` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/corpus_maps/mixed_failed.json
