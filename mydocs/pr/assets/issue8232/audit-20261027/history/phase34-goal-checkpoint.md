---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase34-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase34-goal-checkpoint.md` · 원문 SHA-256: `6688a9065132342aa6138fcfd5145caf1ce78d77cbbd158fc0007979bc7d173a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 34차

## 목표·권한·상태

전수 조사·독립 판정 → #8232 반영 → 부적합 검사와 필요한 공통 구현 보정 → 실제 보정 파일·함수를 `issue_8232`로 명명하고 helper/module/suite manifest/생성/스크립트/E2E 호출 갱신 → 실제 필수 재검증을 완료한다. 잘못된 기존 판정은 보존할 의무가 없고 독립 근거가 있는 내용·순서·소속·저장 목적을 재구성한다. 정상 검사/원본 단위를 일괄 삭제·개명하지 않으며 이전 이름 wrapper·중복 등록을 만들지 않는다. PR은 나중이며 현재 push/PR 승인은 없다. #8232 수정·필요한 MCP Print는 승인됐다. agent/task 메시지·예약·메모리 쓰기는 금지돼 있다.

R=`/Users/tsjang/rhwp`, branch=`audit/8232-px-regressions-20261025`, HEAD/base=`238de6de41a208648f4ade6b494fe433a81c1fce`, Git clean. 제품·검사 수정 0건, 개명 0건. 모든 임시 자료는 ignored `output/issue8232-audit/`에 있다. jobs 8/threads 8/캡처 합계 8/Cargo 순차를 유지하고 원본·PDF·공유 target·다른 작업장을 보존했다. 이번은 합성 자료·격리 코드 사본의 실행이며 실제 HWP/한컴 Print PDF·fresh WASM·브라우저·전쪽 시각 캡처는 없다. 목표는 ACTIVE다. 직전 turn과 이번 turn 모두 실제 판정·실행·새 반례·이슈 반영이라는 진전이 있었으며 전체 완료나 차단 상태가 아니다.

## 이번 직접 판정

두 검사 파일 전체를 읽었다: `tools/pdf-ir-poc/tests/nested_tables.rs` 324줄, `tests/cases/issue_4764_pdf_raster_fidelity.rs` 483줄. 생성기 `scripts/generate_issue_4764_residual_fixtures.py` 전체와 POC topology/hierarchy 및 root raster fingerprint/wrap/classify/contract/mod/report/isolate의 직접 관련 함수 body 9파일을 읽었다. 원장 추가는 검사 23개/helper 65개, 합계 88함수/12파일이며 중복은 없다. `record_integration_batch47.py`, `integration-semantics-batch-47.json`, `non-numeric-helper-semantics-40.json`이 함수별 정본이다. recognition.rs와 pdf_page/catalog 일부는 관련 구간만 읽었으며 그 전체 함수를 직접 판정한 것으로 세지 않았다.

작성 raster 사각형·원본 pixel·정수 lattice·정규화 embedding·논리 source identity·합성 metadata와 실물 페이지 원점 고정을 구분했다. 82/17과 과거383은 catalog/API 상수이며 현재 검사는 실제 원본을 열지 않는다. 실제 편람/sample16 검사는 복원하지 않았고 생성기 현재 7corpus에도 제외 문서가 없다. 과거 이슈/작업 문서의 전쪽 완료 주장을 현재 수용으로 재인용하지 않는다.

## 실제 실행: 원본 잠금 실패와 진단 사본 분리

루트는 현재 generated suite023/nextest metadata·실제 바이너리 전후 SHA를 고정해 기존11개를 실행했다. `phase34-root-tests.log`: 11 PASS / 0 FAIL, Summary0.026초, 선택target258skipped. 같은 순차 driver의 원본 POC `cargo test --locked`는 lock 갱신 요구로 exit101이며 test binary 실행 전 실패했다. `phase34-existing-tests-producer.json`에 두 결과가 분리돼 있고 source/fixture/바이너리는 보존됐다.

원본 lock을 바꾸지 않고 동일 `.rs`를 `phase34-poc-project/`로 복사했다. manifest의 rhwp 상대 path만 원래 R 절대 경로로 바꿨다. offline 사본 잠금 생성은 선택 가능한 zip 부족으로 실패했고, 같은 사본의 online 잠금 생성은 exit0이었다. 원본과 사본의 잠금 package 차이는 추가42/제거24이며 root 의존성의 새 vendored backend/서브패키지 등이 반영된다. 같은 잠금이라고 주장하지 않는다. `phase34-poc-copy-provenance.json`은 offline 실패 당시 snapshot이고, 최종 source/lock/실행 출처는 `phase34-poc-copy-tests-producer.json`이다.

사본은 `--locked`, jobs8, threads8로 기존 중첩 표12개를 실행해12 PASS/0 FAIL/0ignore/0.10초였다. wrapper79.220392791초다. 원본 source와 lock은 그대로다. root와 Cargo 실행은 겹치지 않았다. `target/issue8232-pdf-ir-poc`는 이 감사에서 처음 생성한 전용 target이며 shared `target/pr-review`를 지우거나 덮어쓰지 않았다.

이후 원본의 `cargo metadata --locked`도 다시 exit101이었다. stderr는 `phase34-poc-original-metadata.log`, 빈 stdout은 `phase34-poc-original-metadata-empty-stdout.txt`다. 빈 stdout을 JSON으로 파싱하지 않는다. `phase34-original-lock-recheck.json`에 명령·exit·hash를 연결했다. 진단 사본의12PASS로 원본 locked gate 실패를 숨기지 않는다. 지금 모든 Cargo/Rustc/진단 실행은 terminal이며 자동 중복 시작하지 않는다.

## 생성기·표본 범위·새 검출력 반례

생성기 source는 변경하지 않고 실행 namespace의 OUT만 ignored 경로로 바꿔 재생산했다.12개 파일이 커밋된 fixture와 byte 동일하고 원본 SHA도 그대로다. catalog185행, 격리1480행이며 source의 선택 조건에 따라 실제 casebook 본문은454행만 실행하고1026행은 건너뛴다. PDF builder는185행 중12개 작성 표본만 사용한다. seed histogram/bbox/MediaBox와 과거 잔여 tag를 실제 한컴 PDF 측정값이나 현재 전쪽 수용으로 쓰지 않는다.

`reg_76076` index1의8행을 index2의8행으로 대체한 분석 fixture는 전체1480행과 locked corpus 행 수는 같지만 고유 identity가1472개다. 기존 casebook·well-formed 두 root는 정상/변형에서 각각2PASS였다. 원본 단언은 그대로이고 fixture 경로만 ignored 사본으로 바꿨다. 개수/일부flag만으로 페이지별 누락·중복을 검출하지 못한다. loader와 두 caller의 같은 원인이며3개 독립 결함으로 세지 않는다.

POC 보류 root에서는 실제 recognition 결과의 후보3개를0개로 바꾸고 bodyRegions도 비운 분석 사본이 기존 `.all(status==Deferred)`/body-empty 단언을 통과했다. 정상/변형 각각1PASS다. 빈 배열의 공허 통과가 family 소실을 검출하지 못하므로 source candidate 존재·identity·보류 소유를 보호하도록 보정할 대상이다. `phase34-predicate-counterexamples.json/log`와 `phase34-predicate-probes/`를 따른다. 분석 사본은 CI/Cargo 검사에 새로 등록한 wrapper가 아니며, 결과를 제품 원본의 실제 오류나 새 정식 회귀 FAIL→PASS로 보고하지 않는다.

새 원인은 합계2개이며 관련 직접 기록4개(공통loader/두caller/POCroot)다. 아직 실제 검사를 고치거나 개명하지 않았다.

## 원 PR과 독립 산술

POC는 PR #7913의 실제 merge4662c069e557e928ed7968d061121fde90b1f027에서 같은 검사가 들어왔고 현재 조상이다. root 원 PR #5464는 미병합 CLOSED이며 head0f5ee8617bf518eab6892ea419975347ee529c91의 source를 API로 보존했다. 실제 통합은 #5530/bef280b001b290768874ff164b8314fced6e25ea이고, 현재 corpus 축소·개수 보정은 #8228/3e02fe2749866a8453af3eb3901b9c1a674cccaf다. 실제 merge tree와 현재 조상을 확인했다. `phase34-origin-chains.json`과 PR metadata/source 사본이 정본이다. #7913 commit에 연결된 다른 OPEN PR은 기원 병합으로 취급하지 않았다.

초기 full PR 응답을 shell 인자에 넣어 저장하려던 시도는 큰 diff 때문에 실행 프로세스 생성 전에 길이 제한으로 실패했다. metadata만 골라 저장하거나 REST metadata를 파일로 바로 받아 정정했다. source/test 실패가 아니며 `phase34-pr-metadata-write-error.txt`에 기록했다.

독립 F2 경계 행렬은 full3×3에서 V16/E24/F9, rank15/9, Betti[1,0,0]이다. hole은[1,1,0], 분리는[2,0,0], 중복face는[1,0,1], 병합3face는[1,0,0]이다. 정규화 embedding의 두 합성 순서는 유리수[29/100,8/25,3/25,9/100]로 같다. `phase34-independent-topology-math.json/log`를 따른다.1e-6은 f32 대수 오차이며 실물 PDF/HWP page 좌표 허용폭이 아니다. 독립 Print를 새로 생성·변경하지 않았다.

## 원장·이슈·다음

직접 판정은 7,081개/755소스 파일, 원장은 11,335그룹/2,127파일이다. 미분류 3,924개와 메모만 있는 330개를 합친 분류 미확정은 4,254개다. 독립 근거 pending 508개는 직접 판정 기록 안에 있어 단순 합산하지 않는다. `validate_phase34_ledger.py`가 현재 source SHA·전체 연결·루트 11 PASS·원본 잠금 실패·사본 12 PASS·두 반례·기원·fixture/lock 보존을 분리해 검증했다. 전체 감사·적합 판정·호출 경로 검증은 미완료다.

#8232에 이번 결과 6개 bullet을 추가하고 API 재조회에서 말미 개행 포함 byte 동일을 확인했다. 상태는 OPEN이며 updated_at·본문 SHA는 `phase34-remote-verification.json`을 따른다. 완료 checkbox·close·PR 생성은 없다. 실제 부적합 검사 보정·issue_8232 명명·모든 호출/등록 갱신·원본 잠금 gate 해소와 필수 재검증은 아직 남는다.

이전 phase32의 profile guard와 phase30의 암호 HWP3 vpos/배경·전 24쪽/WASM 자격 pending도 계속 유효하다. 이전 checkpoint·독립 JSON을 따르고 이번 진단을 그 완료로 대체하지 않는다. 나머지 미분류·독립 근거와 source cfg/전체 tests/Studio E2E·동적/생성/공용 경로를 계속 조사한다. 다음 후보는 현재 원장의 ruler-resize.cases.mjs, font_metric_coverage, stored_partial_frame_alignment 등이다. 끝난 이번 루트 11개·사본 12개·분석 2/1개와 이전 46/65/41/37/60개를 자동 재시작하지 않는다. 전체 목표는 ACTIVE다.
