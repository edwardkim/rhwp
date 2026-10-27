---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase153-remaining-renderer-root-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase153-remaining-renderer-root-contracts/goal-checkpoint.md` · 원문 SHA-256: `c2aef23e4d825e62a6404acb3a57a3350dcfe77e70e9eb487cdd37612ec9651a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 153차 기록

남은 renderer root 30개와 helper 10개를 직접 판정했다. 정상 작업장 libtest 30 PASS에는 hwpspec-w 결측으로 인한 무검증 성공도 포함한다. 별도 output에 손상 입력 세 개를 작성했는데 통합 root 3개가 모두 PASS하여 loader의 parse 오류 은폐를 실제 확인했다. 추가 빌드는 없다.

SVG 두 경로 자체는 서로 다르지만 Err를 빈 문자열로 바꾸는 equality가 있고, 넘침 검사의 유일한 단언은 비어 있지 않음이라 한 페이지 반례도 통과한다. root 4개·공통 helper 1개를 보정 대상으로 추가했다. 공통 loader의 다른 caller는 새 근거로 재점검할 별도 목록을 남겼다.

원본 TTF의 AVATAR hmtx와 작성 grid 기대값 6개도 독립 대조했다. HU/source frame·font chain·작성 bbox·상태 관계를 실물 px pin으로 오인하지 않았다. 보정 필요 171개 함수 항목이며 전수 조사·최종 이슈 갱신·소스 보정·개명은 미완료다.
