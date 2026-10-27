---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase142-equation-svg-semantic-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase142-equation-svg-semantic-contracts/goal-checkpoint.md` · 원문 SHA-256: `bdc6bbb065c675241aac99dc687ffe04747605edceeffedc75135d339d16ae78`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 142차 기록

작성한 수식의 내용·문자 순서·rm/it 스타일·괄호/분수/적분 SVG 표현에 관한 12개 검사와 helper 2개를 판정했다. 실제 Native 12 PASS이며 compiler 입력 동등성을 재확인한 기존 바이너리를 사용해 추가 빌드는 없다.

20은 helper의 합성 글자 크기 입력이다. 실물 페이지의 px 기대값이 아니며 Unicode 이탤릭 내용과 실제 글꼴 윤곽을 구분했다. 추가 보정 대상은 없고 누적 직접 점검 12,475개, Rust 검사 7,371개, 보정 필요 159개 함수 항목이다. 미점검 9,247개가 남아 전수 감사·최종 이슈 갱신·구현 보정·개명은 미완료다.
