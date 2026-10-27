---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-20261026-independent-contract.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 독립 기대 계약 감사 체크포인트

- 작업장 `/Users/tsjang/rhwp`, branch `audit/8232-px-regressions-20261025`, HEAD `238de6de41a208648f4ade6b494fe433a81c1fce`. 제품·테스트 변경/rename/commit/push/PR 없음. 마지막 git status clean.
- 목표 ACTIVE. 전수 조사·이슈 업데이트 후 지침에 맞지 않는 실제 검사를 모두 보정하고 실제 보정 파일/함수를 issue_8232로 명명한다. 이전 wrapper/중복 등록 없음. 기존 잘못된 기대값/동작은 무조건 보존하지 않는다. 정상 결함 검출 목표는 독립 근거에서 다시 정한다. PR은 나중이며 push/PR 승인 없음. 다른 task/subagent/예약/메모리 변경 없음.
- 현재 turn 진척: #8232 본문 실제 재검증 절의 '원래 반례'를 '독립 근거로 유효성이 확인된 결함 반례'로 정정, 후보9666개로 갱신. phase8 댓글6401391347 게시·수정·exact body 검증. 저장본 `issue-body-20261026-independent-contract.md`, `phase8-nearfit-report.md`, `phase8-published-receipt.json`.
- phase8 기존3검사는 이미 exit0 종료,3PASS/271filtered/0.528초. Recipe는 명시적으로 잘못된 body width 축소를 통과시키는 검사. old PR7120 CLOSED 미병합→실제 통합7138 commit63547ddf ancestor. 새 paired3조건/쪽: Book100/100, Movie97.58688/97.58688, Recipe90.09535/90.05374. source raw35719HU vs body35149HU. 독립 stroke width475.79068. 기존 source-probe/quantitative/origin-history JSON과 기존 test log 보존. 본 turn 캡처/빌드 중복 실행 없음.
- integration batch06=100:63국소 원본/상대,37독립근거 보류. batch07=170:83국소 원본/상대,87독립근거 보류. 각각 `record_integration_batch06.py`, `record_integration_batch07.py`가 source SHA·전체 assertion site와 의미 기록을 생성. 단언 형태의 직접 분류이며 전체 helper/caller 폐쇄·독립 시각 수용이 아님.
- `integration-decimal-direct-coverage.json`: tests/의 unreviewed 함수에 직접 소수와 abs/approx/assert_near 등이 있는 검색 조건의 남은 후보0. 전수 완료가 아니며 상수·정수·helper·비수치 selector·생성/동적/E2E·증거 미연결은 남음.
- source batch21=80: 변환/bit/byte/인덱스·nonce/CSV·cursor path/perf·데이터 추출. 직접 수치 의미상 실물 절대 px 핀 아님. `record_source_batch21.py`, `source-semantics-batch-21.json`. full_conformance_verified=false. source batch22 아직 준비하지 않음.
- 최신 ledger9666그룹2103파일: unreviewed7548, function_px_policy_adjudicated1346, semantic_notes133, nested_helpers_pending70, function_evidence_pending357, file_context_only212. 이 숫자를 위반 수/전체 완료로 해석하지 않는다.
- archived37 fixture의 실제 자동 소비 대조는 미완료. tests/suite policy root는 tests nonrecursive/tests/cases recursive/tests/suites/issue_regression_pilot nonrecursive + .rs 필터. module graph3291엣지와 공개 collector boundaries(samples/hwpx·samples, golden root tests/golden_svg, renderer manifest samples 제약, oracle_profile의issue4963 exact paths)는 확인. archived 경로 직접 코드 검색0만으로 inactive 최종 판정하지 않는다. `record_other_fixture_consumers.py`는 아직37개 false를 생성한다.
- 동적 함수 추가 backstop `dynamic-collection-functions-backstop.json`1064그룹: walk에는 RenderTree 순회도 섞임. 실제 filesystem collector로 확정한 수가 아님. `rhwp-studio/scripts/e2e_runner.cjs`는 현 tree에 없는 잘못된 이전 추정 경로. 현재 `rhwp-studio/e2e/scenario-runner.mjs`는 전달받은 scenario.steps만 실행하며 전역 mydocs 수집기가 아님. E2E 진입 manifest는 scripts/check_e2e_manifest.py 및 package/workflow를 대조.
- 기존 PNG/TSV catalog83조건2333쪽/backend(기존 producer673c1… 포함). old673→현재238의 src/crates/rhwp-studio diff없음 재확인. 동일 출처 재사용과 새 실행을 혼동하지 않는다. 각종 low raw 미해결과 full sweep/실제 수정은 앞선 checkpoint·issue 댓글에 남음.
- 다음: 독립 근거 보류·상수/비수치 helper/수집 경로·남은 source/integration 범위 감사 계속. 조사 완료를 먼저 입증/공개한 뒤 실제 출력/검사 보정·rename·정상 대조/전쪽 양쪽시각·필수 회귀/게이트. 허용폭만 넓히거나 이름만 바꿔 완료로 처리하지 않는다.
