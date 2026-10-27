---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase187-competitive-bench-boundary-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase187-competitive-bench-boundary-contracts/goal-checkpoint.md` · 원문 SHA-256: `23c212c8e8d2e917a02b052ef6dab9140f0a00753db2625abed6822de9f7867a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 187차 기록

경쟁 벤치의 예외/추가 검사 68개와 helper 2개를 확인했다. Python 68 PASS/0 FAIL/0 skip이며 실제 출력 경로는 임시 output 안에서 격리했다. 오류·schema·agent ID·scorecard·문자수 비율·경로 선택을 페이지 px와 구분했다.

대문자 B.HWP를 후보에서 빼도 기존 uppercase 검사는 1 PASS다. 현재 Mac 원본 실행도 B.HWP를 선택하지 않으며, 대문자 HWP와 HWPX만 둔 독립 입력에서는 HWPX만 반환한다. 소문자 정상 대조는 둘 다 선택한다. 검사 한 개의 검출 누락과 현재 glob 경로를 보정할 대상이며 실제 HWP 파싱/다른 OS의 출력으로 확대하지 않는다.

70개를 새로 판정하고 보정 필요 누계는 208개다. 원 PR #5239·통합 #5429·현재 두 파일의 byte를 확인했다. 추가 빌드·외부 프로세스·시각 캡처·제품/테스트 수정·개명·외부 게시가 없고 전수 감사는 미완료다.
