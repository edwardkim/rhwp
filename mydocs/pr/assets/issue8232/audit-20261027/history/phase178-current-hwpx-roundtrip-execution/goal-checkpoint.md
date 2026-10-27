---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase178-current-hwpx-roundtrip-execution/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase178-current-hwpx-roundtrip-execution/goal-checkpoint.md` · 원문 SHA-256: `846741d49927f56513ce5e71b62d71c7ba5c2ecf46175490f4c7b84eee406372`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 178차 기록

137차에서 소스·독립 XML·공개 저장 경로까지 확인했으나 현재 통합 실행이 없어 보류했던 21개를 현재 head의 suite 020으로 실행했다. 21 PASS/0 FAIL/0 ignore이며 현재 실행 보류 21개를 해소했다. 이미 조사한 함수이므로 직접 점검 수에 중복 가산하지 않았다.

기존 histogram·경고 측정 두 개는 실제로도 PASS하지만 실패 가능한 수치 assertion이 없는 진단이다. 원래 측정 의도와 원 PR 출처를 보존하고 보정 필요 판정 두 개를 유지했다. 모든 PASS를 회귀 검출력이나 한컴 시각 일치로 확대하지 않는다.

176차에서 준비한 현재 실행 파일을 재사용했고 이 단계의 추가 빌드는 없다. 제품/테스트 수정·개명·외부 게시가 없고 보정 필요 누계는 205개다. 전수 감사와 166차 전쪽 검증은 미완료다.
