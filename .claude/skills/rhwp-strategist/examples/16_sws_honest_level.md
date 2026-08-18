# E16 SWS 낮은 레벨은 정직한 현황

- 코퍼스 시나리오: `gov_rfp`
- 정지 규칙: `ST-GATE-FAIL`
- 픽스처: `fixtures/validate/pass_with_sws.json`
- 엔진: `tools/strategist/engagement.py`
- 정본: `mydocs/manual/strategist_playbook.md` §1–§5

## 상황

validate pass + swsAudit.attained=L2.

## 절차

1. validate pass + swsAudit.attained=L2.

2. L3 미달을 숨기거나 exit 을 실패로 바꾸지 않는다.

3. 회신: '연결 게이트 통과, SWS L2, L3 이상은 spec 미기재'.

4. --no-sws-audit 를 기본값으로 쓰지 않는다. 숨기려는 신호가 된다.

## 기대

- 새 CLI 를 치지 않는다 (`strategy`/`forecast`/`claim-check` 없음).
- gym 경로를 인용하지 않는다.
- 좌표 키를 발명하지 않는다.
- 위반 시 `ST-GATE-FAIL` 로 멈춘다.

## 관련

- SKILL.md 절차 1–6
- fixtures/catalog.json
- fixtures/validate/pass_with_sws.json
