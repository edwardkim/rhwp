---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase232-cellzone-line-ownership/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase232-cellzone-line-ownership/goal-checkpoint.md` · 원문 SHA-256: `fc06aa38791024f09aab64ef6c2001970f8d2ab6d9a8777d038ee16dbdd40c4e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 232차 기록

셀 영역 대각선의 보류 9건을 판정했다. 같은 HEAD의 원 Native 검사 8개는 모두 PASS이며, 공용 helper는 그 실행 경로에서 소비됐다. 실제 공개 편집/원본 입력 10조건의 Native·WASM 선 좌표도 같았다. 추가 빌드는 하지 않았다.

기존 조건은 raw line 좌표만 사용한다. 실제 표시를 숨기거나 첫 셀 중심선을 다른 셀로 이동한 진단 출력에서도 조건이 참이었다. 80셀의 가로 중심선 80개와 세로 70개를 제거해도 세로선 10개 조건이 통과했다. 금지된 이전 X를 path로 추가한 진단 사본도 기존 부재 조건을 만족했다. 원 Rust 반환값에 변형을 주입해 통과시킨 실행이나 제품의 자연 발생 결함으로 보고하지 않는다.

작성 HU, 원본 zone·셀 속성, own/effective 편집 결과, 1×1 override와 HWP LIST_HEADER 저장 계약은 유지한다. 이 유효한 계약을 source 셀 소속·모서리/중점·실제 표시와 연결하는 보정 대상이다. 숫자가 있다는 이유로 모든 단언을 없애거나 ±2px로 완화하지 않는다.

원본 대각선 문서의 앞선 전쪽 미달은 해결되지 않았다. 이번 SVG/rsvg 진단은 독립 Print·Visual Sweep 통과나 98% 좌표 자격이 아니다. 최초 package 검증 자료형 오류는 실제 실행 전에 수정했고, HWPX 보조 저장의 ID 거부는 원 검사의 HWP 저장 경로로 연결했다. 이미 완료된 Native 8개 검사는 반복하지 않았다.

증거 보류는 405→396개, 보정·정리 함수 지점은 336→345개다. 직접 검토 15,847개는 중복 증가시키지 않았다. 소스 수정·개명·외부 게시 0이며 전체 감사는 진행 중이다.
