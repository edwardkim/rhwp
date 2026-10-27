---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase215-crooked-diagonal-independent-print/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase215-crooked-diagonal-independent-print/goal-checkpoint.md` · 원문 SHA-256: `f972c0b7cfc1a12d39385e975b570563b25af267431bc7b8e015f7c34f114055`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 215차 기록

대각선 원본 HWP/HWPX와 독립 한컴 PDF 세 개를 대조했다. 첫 행 두 번째 셀의 꺾임 비율은 약 40%·60%로 확인되어 합성 셀 단위 검사의 근거 보류를 해소했다. 실물 검사는 고정 화면 영역과 길이만 고르고 셀 소유·선 연결을 확인하지 않으므로 보정 필요로 판정했다.

기존 Native 검사 2개는 PASS다. 그러나 두 원본의 전쪽 Visual Sweep은 Native 79.44272%·fresh WASM 79.54196%로 네 조건 모두 미달이다. 직접 판독에서 아래쪽 셀의 붉은 복합 X 누락을 확인했으며, 이 원인의 구현 보정이나 검사 제거·이관은 아직 하지 않았다.

보류 533→531개, 보정 필요 218→219개다. 직접 점검 함수 15,514개·Rust 테스트 7,882개/888개 파일은 중복 집계하지 않았다. 추가 빌드·소스/검사 수정·개명·외부 게시 모두 0건이며 전체 감사는 미완료다.
