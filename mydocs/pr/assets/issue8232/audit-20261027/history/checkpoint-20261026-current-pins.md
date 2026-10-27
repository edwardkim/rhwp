---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-20261026-current-pins.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 이어받기 — 2026-10-26 현재 고정 위치 검사 추가 조사

작업 목표는 ACTIVE다. 전수 조사와 이슈 업데이트 뒤 부적합 기존 검사 보정·`issue_8232` 명명 및 실제 검증까지 필요하다. PR은 나중이며 push/PR은 하지 않는다. 원래 부적합한 px 판정을 무조건 보존하지 않는다. 유효한 검증 목표는 독립 근거에서 다시 정한다. 다른 task/subagent/예약/메모리 쓰기 금지.

## 현재 권위 상태

- 작업장 `/Users/tsjang/rhwp`, branch `audit/8232-px-regressions-20261025`, HEAD `238de6de41a208648f4ade6b494fe433a81c1fce`.
- 마지막 git status는 깨끗하다. 제품/테스트 수정·명명·commit은 아직 없다. 모든 새 자료는 ignored `output/issue8232-audit/`에만 있다.
- 마지막 골 상태 조회는 ACTIVE였다. 완료/blocked/paused로 변경하지 않았다.
- 이전 목표 턴은 단순 설명이었지만 이번 턴에는 아래 실측·감사 원장·공개 이슈 상태를 전진시켰다. 현재 진짜 blocker는 없다.
- 최신 repository 지침을 확인했다. PDFium5.13/96CSSpx/2px raw, Native+fresh WASM, 같은 쪽 양쪽98%·독립 좌표·직접 판독·±2CSSpx 및 의미 보호를 구분한다. Capture total8, Cargo sequential/jobs8/tests8.

## 이번 턴의 실행 종료 상태

- 과거 session69803은 없었으며 Cargo 빌드 로그는 성공이었다. `font-probe-root-library.json`은 fresh한 정확한 root rlib를 지정했다.
- `font-observer-probe.rs`를 해당 rlib로 연결했다. session94244는 compile0 및 입력2건 probe exit0으로 종료했다. producer는 `font-observer-producer.json`.
- phase7 첫 session61337은 HWP3의 `lastSavedWith=None` 처리 누락으로 4조건 뒤 exit1. 실패 로그 `phase7-current-pin-driver-before-hwp3-resume.log` 보존.
- driver의 실제 format/저장제품별 engine 분기를 보정하고 source/native/pkg/script/input/PDF hash를 확인해 나머지만 재개했다. 완료4조건을 다시 캡처하지 않았다.
- 재개 session68834/PID63524는 정상 exit0. `phase7-current-pin-evidence/state.json` complete=true,8조건 모두 종료 기록. 마지막 조건의 두 backend 시각 command exit1은 미달이며 숨기지 않았다.
- 현재 이 턴이 시작한 Cargo/캡처는 모두 끝났다. 다음 턴도 실제 ps·로그부터 확인한다. 종료 session을 새 실행의 근거로 쓰지 않는다.

## 실제 font observer

`hy001-hwp-native-font-trace.json`, `hy001-hwpx-native-font-trace.json`, 각 `-declarations.txt`, `phase5-hy001-native-font-selection-summary.json`.

두 입력 모두 p2 258문자/누락0. 본문123문자와 로고 줄 공백54자는 휴먼명조 요청→휴먼명조(system) 선택. HCI Poppy9문자는 Palatino Linotype 선택. HWP source 선언도 휴먼명조다. `fc-match` 결과를 실제 Native 결정으로 대용하지 않았다. Canvas2D/CanvasKit은 이 관측에서 unsupported이며 검증했다고 하지 않는다.

독립 HWP2020 PDF의 Gulim 선택과 HWPX2024의 휴먼명조 subset, 공통 로고 gap539.0 vs532.99334px는 별도 원인이다. 원격 private Human font 파일은 앞선 v2 재고에서 Mac과 같은 d033 hash였지만 MCP worker의 실제 글꼴 선택 경로는 아직 입증하지 않았다. 원격 설치/등록/재시작은 하지 않았다.

## 원장 추가

- source-semantics-batch-20:70그룹의 단언/호출 직접 수치 의미를 기록했다. 원문 인덱스·모양ID·원본HU/bit/byte·편집 단조 관계와 화면 핀을 구분했다.
- integration-semantics-batch-02:70그룹, 국소 비절대 목적45 / 독립 근거 연결25로 최초 분류했다. 이 중 #1133 assert_frame은 현재 독립 자격·부적합 공차를 확인해 `qualified_page_requires_tolerance_and_semantic_rewrite`, change_required=true로 갱신했다.
- integration-semantics-batch-03:70그룹, 국소 비절대 목적25 / 독립 근거 연결45. 같은 helper도 호출별로 심사한다.
- 전체 ledger9664그룹/2103파일. 최신 review_state: unreviewed8098, function_px_policy_adjudicated1030, semantic_notes_present_requires_adjudication133, local_meaning_reviewed_nested_helpers_pending42, function_evidence_pending149, file_context_only212.
- 이 수치는 위반 수·PASS·전체 적합 완료 수가 아니다. 모든 full_conformance_verified=false이며 호출 폐쇄 미검증을 보존한다.
- `prepare_integration_review_batch.py 4 70`부터 다음 묶음을 만들 수 있다. 직전 선택 시 미검토 direct decimal tolerance540중70을 batch03으로 읽었다. 다음은470예상이며 실제 ledger에서 계산한다. 원본 숫자·기하 후보와 추가 숫자 helper backstop은 구분한다.

## phase7 독립 Print와 전쪽 결과

`phase7-current-pin-quantitative-summary.json`, canonical source/run manifest·입력/PDF SHA·job은 각 state와 case 폴더.

각 backend38쪽. 7조건 전쪽 양쪽98% 이상:

- frame1133-hwp/hwpx/bottom15/footer15 각3쪽: Native98.05890 / WASM98.05930.
- signature1950:1쪽99.99743/99.96141.
- number265-inline:16쪽99.64682/99.62131.
- wrapper5585:7쪽98.31237/98.22089.
- anchor6787:2쪽 중 **1쪽89.72029/89.64814**, 두 시각 명령 exit1. 해당 고정 x 배열 ±0.6을 적합으로 처리하지 않는다.

anchor6787의 Native p1 review+overlay를 직접 읽었다. 내부 후보 상자의 상하 경계, 사진/후보자 문구·기호 및 아래 설명문의 위치/간격 차이가 보인다. 글꼴 하나로 단정하지 않았다. 출력을 먼저/기대값 변경과 구분한다. 원본·PDF·TSV·PNG 보존.

## #1133 공차 검출력

- 실제 목표 p2/p3 Native review와 WASM overlay를 직접 읽었다. 두 형식의 원본 출력/독립PDF PNG는 backend별3쪽 전부 동일hash였다. 연결은 `phase7-frame1133-direct-review.json`.
- `phase7-independent-path-probe.py`, `phase7-frame1133-independent-paths.json`: PDF level0의 실제 stroke path와 matrix를 사용했다. p2 상단440.478678/하단1042.539998, p3 상단77.356038/하단296.156006CSSpx.
- 원 해결 PR7382의 로컬 review 기록과 실제 f07affc360/fff5310762를 읽었다.
- 기존 ±0.6/±1.3을 ±2로만 바꾸면, 보정 전 기록의 잘못된 y439.066667/75.573333도 약1.4/1.8px 차이여서 통과한다. `phase7-frame1133-tolerance-counterexample.json`은 **단언 수준의 수학적 대조**이며 과거 engine 실행이라고 주장하지 않는다.
- 재작성에는 원본141HU 바깥 위여백의 첫/이어받기 프레임 소유·예약/paint 공통 소비와 원문 행/후속 문단 보존 관계가 필요하다. 자격이 있는 독립 좌표만±2로 보호한다. 아직 실제 테스트 수정은 하지 않았다.

## 카탈로그·이슈

- `build_paired_evidence_catalog.py`에 phase7 추가. 현재80조건·각backend2330쪽·입력/PDF159파일 hash확인. 기존72조건 catalog는 `paired-evidence-catalog-before-phase7.json` 보존.
- producer673c→current238의 src/crates/rhwp-studio diff없음 확인은 이전35조건 재사용 근거다. 모든2330쪽을 이번에 새로 캡처했다고 하지 않는다.
- 공개 추가 결과6400913080: `phase5-priority-predicate-report.md`, exact-body receipt `phase5-published-receipt.json`.
- 최신 공개 추가 결과6401073862: `phase7-current-pin-report.md`, exact-body receipt `phase7-published-receipt.json`.
- https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6401073862
- 현재 이슈 본문은 부적합 기존 단언 무조건 보존을 전제로 하지 않으며 실제 보정한 이름만issue_8232로 바꾼다. 이 턴에 본문을 추가로 바꾸지는 않았다.

남은 전수 판정·원본/간접 helper/동적 수집(보존37자료 포함)·조건부 실행 확인을 계속한다. 감사 완료와 실제 보정·수집·검출력/시각/회귀 검증 전에는 목표 완료 또는 PR 준비 완료로 판정하지 않는다.
