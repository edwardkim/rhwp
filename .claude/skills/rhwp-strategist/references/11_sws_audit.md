# 11 SWS/1.0 자동 감사

정본: playbook §5.1, `mydocs/tech/standards/strategy_work_standard.md`,
`tools/strategist/sws_audit.py`.

## 왜 여기 붙어 있나

예전에 감사기는 독립 CLI 라서 호출을 잊으면 검증 안 된 산출물이 나갔다.
지금은 `--validate` 가 spec·ledger·corpus_map 을 SWS 포맷으로 옮겨
자동 채점한다. `--no-sws-audit` 로만 생략한다.

## 엔진이 채우는 레벨

| 레벨 | 이름 | 엔진 보장 |
| --- | --- | --- |
| SW-L1 | 근거 좌표 | ledger 좌표 + 재독 |
| SW-L2 | 전수 수집 | corpus_map 선언/읽음/실패 |
| SW-L3 | 반증 생존 | spec 에 challenge 자리가 없음 → 미달 |
| SW-L4 | 반증가능성 | falsifier/confidence 자리 없음 → 미달 |
| SW-L5 | 정산·감사 | AWS 접합. 자동으로 올리지 않음 |

낮은 도달은 실패가 아니다. 골격의 현재 한계를 정직하게 드러낸다.
에이전트가 L3 이상을 원하면 CLAIM 문장에 반증 시도와 확신도를
구조화해 싣고 감사가 읽게 해야 한다 — 이 스킬 파동은 그 필드를
엔진에 발명하지 않는다.

## exit 와의 관계

SWS 도달 레벨은 exit 를 바꾸지 않는다. §5 게이트만 납품 가부.
감사 실행이 예외여도 `swsAudit.error` 만 남긴다.

## 회신에 적을 것

"SWS 도달: L2 (L3 이상은 spec 미기재, 정직한 미달)".
고객에게 낮은 레벨을 숨기지 않는다(SWS §legitimacy).

예제: [examples/16_sws_honest_level.md](../examples/16_sws_honest_level.md).

다음: [12_pitfalls.md](12_pitfalls.md).
