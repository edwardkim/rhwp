---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase236-form-fill-model-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase236-form-fill-model-contracts/goal-checkpoint.md` · 원문 SHA-256: `e0ca6dc1f13ead5b6f223defab94a294f0cd6f305635f98d5b556fbf9808bd3e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 236차 기록

누름틀 Python 모델의 원 검사 31개를 직접 읽고 실행해 31 PASS를 확인했다. 모델 구현 649줄과 선택한 catalog 선언·실제 호출/읽기 경계를 연결했다.

숫자는 필드 개수·반복 순번·문자열 값/목록 보존·CSV/JSONL 행·상태 코드다. sample 경로는 문자열이며 원본 HWP/HWPX/PDF를 읽지 않았다. output은 문자열이고 verify는 Python before/after 모델 비교다. 이를 실제 문서 저장·재파싱·Native/WASM 시각 통과로 주장하지 않는다. 유효한 모델 계약을 화면 px 핀으로 오인해 바꾸지 않는다.

직접 검토는 16,016→16,047함수, 미검토는 6,347→6,316개다. 보정 지점 346개·증거 보류 396개는 그대로다. 추가 빌드·캡처·소스 수정·개명·외부 게시 0이며 전체 감사는 진행 중이다.
