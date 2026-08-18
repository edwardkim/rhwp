# 01 권위 — playbook 이 정본이다

canonical: `mydocs/manual/strategist_playbook.md`
capability: `CAP-4903` (등록 이슈 #4903, 스킬 고도화 #5335)
엔진: `tools/strategist/engagement.py`

## 왜 이 장이 먼저인가

스킬·에이전트·픽스처가 서로 다른 말을 하면 게이트가 흔들린다. 충돌이 나면
**playbook 을 따른다.** 이 스킬은 playbook 을 에이전트가 30초 안에 조립
하도록 나눈 운영 계약이지, 새 규칙을 만들지 않는다.

playbook §1 이 고정한 세 보장(전수성·좌표·연결 검증)과 §5 게이트,
§7 하지 않는 것을 스킬이 약화하지 않는다.

## 권위 사슬

```
mydocs/manual/strategist_playbook.md          정본 (사람)
tools/strategist/engagement.py               결정적 엔진 (기계)
mydocs/tech/standards/strategy_work_standard.md   SWS/1.0 공개 포맷
.claude/agents/rhwp-strategist.md            에이전트 절차
.claude/skills/rhwp-strategist/SKILL.md      이 스킬 라우터
```

SWS 감사는 `--validate` 가 자동 호출한다. 감사 도달 레벨은 납품 가부를
바꾸지 않는다(playbook §5.1). §5 연결 게이트만 exit 3 을 만든다.

## playbook 절 ↔ 스킬 장

| playbook | 스킬 |
| --- | --- |
| §1 정직한 경계 | 00_tree, 09_out_of_scope, 10_fde_chief_boundary |
| §2 엔게이지먼트 프로토콜 | 02_engagement_protocol, 20_question_design |
| §3 파이프라인 A→C | 03_corpus_map, 04_evidence_ledger |
| §4 근거 대장 스키마 | 04_evidence_ledger, 06_coordinate_rules, 07_search_extract_envelopes |
| §5 주장-근거 게이트 | 05_claim_gate, 08_validate_exit |
| §5.1 SWS 자동 감사 | 11_sws_audit |
| §6 종료 코드 | 08_validate_exit |
| §7 하지 않는 것 | 09_out_of_scope, 12_pitfalls |

## 에이전트와의 분업

에이전트 파일은 "누가 판단하는가"를 적는다. 스킬은 "어느 파일을 읽고
어느 명령을 치는가"를 적는다. 둘 다 playbook 을 가리킨다.

- 질문 설계·CLAIM 문장 = 에이전트
- 전수 수집·좌표 복사·게이트 = 엔진
- 라우팅·정지 규칙·레시피 = 이 스킬

## 변경 규율

- 엔진 동작을 바꾸고 싶으면 engagement.py 의 별도 이슈다. 이 PR 은
  스킬·픽스처·계약 시험만 다룬다.
- 새 CLI 플래그·새 하위명령을 스킬에 적지 않는다.
- playbook 문장을 스킬에서 재정의하지 않는다. 요약만 한다.

## 검증 질문

1. 이 장이 가리키는 playbook 경로가 저장소에 있는가.
2. SKILL.md 가 에이전트 파일을 가리키는가.
3. 엔진 경로가 `tools/strategist/engagement.py` 인가.
4. gym 경로를 권위로 인용하지 않는가.

다음: [02_engagement_protocol.md](02_engagement_protocol.md).
