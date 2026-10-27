---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase167-relative-size-diagnostic-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase167-relative-size-diagnostic-contracts/goal-checkpoint.md` · 원문 SHA-256: `e62f58ad4ae598b855b0bedb1d03b32e100a8e3fca209790eb3bf3fb3e633cc8`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사167차 기록

기존 보류 검사2개를 현재 동일 소스 Native에서 재실행해2 PASS를 확인했다. 작성한100/10/0% 입력을 parser 모델·SVG·은닉 CLI까지 대조했다. 정상10%는1pt로 출력되지만 진단은10pt로 보아 threshold2pt에서 놓친다. 공식 PDF29쪽의 유효 상대 크기10..250%와 대조해 과거 렌더러 전제를 고정한 검사1개를 보정 대상으로 확정했다.

0%는 예전 문서의 입력 내성을 보호하던 조건이므로 현재SVG0pt에 맞춰 기대값을 바꾸지 않는다. 올바른 독립 정규화 계약은 추가 확인한다. 두 함수는 재검토이므로 직접 점검 누계에 다시 더하지 않았고 보정 필요항목은200개다. 제품/테스트 수정·개명·추가 빌드·PNG 캡처 없음.

동시에166차 시험지 현재 전쪽 Native/fresh WASM 수집은 별도 프로세스에서 진행 중이다. 원장 판정은165차에서 이167차로 전진했으므로166차 최종 반영 시 시작 원장을 덮어쓰지 않고 현재 원장에 재검토 결과를 병합해야 한다. 전체 감사는 미완료다.
