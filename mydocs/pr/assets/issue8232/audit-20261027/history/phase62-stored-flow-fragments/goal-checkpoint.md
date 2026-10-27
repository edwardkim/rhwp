---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase62-stored-flow-fragments/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase62-stored-flow-fragments/goal-checkpoint.md` · 원문 SHA-256: `b52a2bb930d01e7183fbb54c176f9635bdce3951e26d84e3296a2094b32dac80`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 감사 62차 체크포인트

- 현재 브랜치 `audit/8232-px-regressions-20261025`, head `01ced3d42724285c8f9765a453393ab6af326617`입니다. 최신 동기화의 삭제 검사와 신규 95개 Rust/2개 JS case는 감사 및 PASS 계수에서 제외했습니다.
- 기존 48함수/28검사를 판정했습니다. 신규 직접 점검 36함수/17검사, 기존 점검 재확인 12함수입니다. 누적 직접 점검 8,340함수, Rust 검사 5,848개/751파일입니다. 전체 감사는 미완료입니다.
- 공식 Native 28 PASS/0 FAIL, 0.145초, 빌드·등록 포함 24.354초입니다. 763개는 필터 제외이며 PASS로 세지 않았습니다.
- 원 CFB/XML의 저장 위치·여백·셀 높이로 pending 4건을 해소했습니다. 원 HU를 px로 환산했다는 이유로 원본 관계 계약을 제거하지 않습니다.
- 실제 PDF 폭 ±1px 고정은 Native 96.27348%로 좌표 자격 미달이며 보정 대상으로 남겼습니다. 밑줄 입력은 Native 전2쪽 raw100이지만 fresh WASM 자격은 확인하지 않았습니다.
- 밑줄 검사 본문 그대로 실제 Screen SVG/선 defs 숨김/부모 200px 이동 대조군이 모두 PASS했습니다. 독립 rsvg는 숨김·200px 이동을 확인했습니다. 실제 제품 선 누락을 관측했다는 주장이 아닙니다.
- 두 원본의 review/standalone overlay를 직접 확인했습니다. 원 PDF 6개/21쪽의 내용·쪽수 확인은 전21쪽 시각 통과와 구분했습니다.
- ledger/summary/입력/공식 실행/대조군/새로 추가된 검사 제외의 일관성 validation PASS입니다. #8232 본문 갱신 뒤 EOF까지 동일한 readback을 확인했습니다.
- 제품/검사 수정 0, 개명 0, PR·push 없음입니다. 별도 dirty `rhwp-studio/public/rhwp.js`는 SHA `c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6`로 보존했습니다.
- 다음에는 남은 기존 함수의 독립 판정을 계속합니다. 모든 조사가 끝나기 전 완료 처리하지 않습니다. 새 PR은 나중에 진행합니다.
