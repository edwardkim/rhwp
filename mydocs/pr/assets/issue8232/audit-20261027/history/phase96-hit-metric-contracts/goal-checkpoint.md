---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase96-hit-metric-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase96-hit-metric-contracts/goal-checkpoint.md` · 원문 SHA-256: `fc1a9aa9fc83ff90a693c1a2016b87a64195bdb4a82c3d261df800c365cadfb9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 96차 기록

직접 검토 10,197개 함수, Rust 테스트 6,524개/841개 파일이다. 미검토 그룹 10,986개, 추가 증거 대기 513개, 보정 필요 확정 항목 87개로 전수 조사는 미완료다.

이번에는 중첩 셀 hit-test와 Canvas 측정 세션 31개 함수를 검토했다. 기존 바이너리의 Core Canvas 11개는 PASS이며 hit-test 7개는 미실행이다. 원본 Native 전 4쪽 TSV의 최저는 74.03720%이고 1·3·4쪽이 90% 미만이다. 복사 클릭 위치 2건은 근거 확인을 위해 보류했다. Canvas 주입 advance와 1e-9 세션 전후 비교는 실물 배치 절대 핀과 구분했다.

추가 빌드, 제품/테스트 소스 변경, 이름 변경은 0이다. 원본·PDF·사전 변경 JS를 보존했다. 다음은 미검토 검사와 helper의 단위·기대값·호출 관계 조사이며, 전수 판정 뒤 이슈 갱신과 필요한 보정을 수행한다.
