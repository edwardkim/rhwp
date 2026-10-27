---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase149-renderer-comparison-evidence-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase149-renderer-comparison-evidence-contracts/goal-checkpoint.md` · 원문 SHA-256: `96a32be0cebf4b374f32fa513eb6509f3a79bb2c82457938692b6a2132f061b4`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 149차 기록

렌더러 root 21개와 helper 9개를 판정했다. libtest 21 PASS는 합성 관계 검사 11개·수동 측정 7개·잘못된 실물 비교 3개를 포함하므로 모두 의미 회귀 통과로 부르지 않는다. 원본과 같은 입력 80개를 별도 cwd에 복사해 읽었고 p222 결측 1개는 그대로 기록했다. 추가 빌드는 없다.

기존 Paginator와 비교한다는 실물 root 3개가 현재 DocumentCore의 TypesetEngine 기본 산출을 old_result로 읽는다. p222는 입력 자체가 없어 비교 없이 PASS다. root 3개와 공통 helper를 보정 대상으로 추가했다. Task 398의 원래 LineSeg 리포트는 측정 도구이며 페이지별 raw Visual Sweep이 아니므로 출력 수치만 보고 잘못된 px 핀으로 분류하지 않았다.

보정 필요는 166개 함수 항목이다. 전수 조사·최종 이슈 갱신·제품/test 보정·issue_8232 새 명명은 아직 미완료다.
