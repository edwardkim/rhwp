---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase137-hwpx-roundtrip-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase137-hwpx-roundtrip-contracts/goal-checkpoint.md` · 원문 SHA-256: `dc6706ff25f03b35873ec053eef6c22aab3e39858c269cc6390386ac32c5b287`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 137차 기록

HWPX 저장·IR 왕복 검사 63개와 helper 21개를 직접 분석했다. helper 1개는 기존 기록을 재판정해 중복 합산하지 않았다. 누적 직접 점검 12,187개 함수·Rust 검사 7,300개이며 미점검 9,468개·증적 대기 573개다.

현재 소스와 동일한 기존 Native 단위 실행에서 42 PASS, 현재 CLI의 원본 16개 저장 경로·독립 ZIP/XML 대조를 완료했다. 별도 통합 root 21개는 등록/소스와 공개 소비 대조를 확인했으나 현재 Native 통합 실행 증적은 대기로 남겼다. 과거 238de 바이너리나 공개 저장 exit 0을 통합 21개 PASS로 재인용하지 않았다. 새 빌드는 없다.

histogram·false_positive_measurement 두 함수는 변화를 실패시키는 assertion 없이 수치만 출력한다. 원 commit 7945e7cb73과 통합 PR #200·당시 보고서를 대조해 원 측정 의도와 회귀 검사 집계의 차이를 기록했다. 보정 필요 함수 항목은 125→127개다. 실제 보정은 전수 조사 후 독립 원본 관계를 검출하는 issue_8232 검사와 output 진단 경로로 구분하며, 현재 출력 수치를 golden으로 복제하지 않는다.

원본 HU·문자 슬롯·저장 XML 보존과 실제 페이지 px를 구분했다. source 제품/테스트 수정·개명·최종 이슈 갱신은 아직 하지 않았으며 전체 감사는 미완료다. 사용자 Studio JS·원본·PDF·공유 target을 보존했다.
