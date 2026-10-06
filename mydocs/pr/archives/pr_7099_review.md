---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7099 빈 마지막 셀 문단 trailing 줄간격 검토

**판정: 메인터너 보정 후 수용 가능.** 수용 범위는 테스트 추가분뿐이다. 로컬 체리픽 통합본에 대한 검토다. 원 PR merge/close 또는 GitHub approve를 완료했다는 의미가 아니다.

## 출처와 통합 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#7099](https://github.com/edwardkim/rhwp/pull/7099) / lpaiu-cs |
| 원 head | `465534ececc6a57702005aef50821f6e871f50eb` |
| 최신 devel 시작점 | `410d22cdf77e3e9e45159999cab19719c621c025` |
| 통합 code head | `b40c2953cf526b86319f2841c5b5584d7a83d735` |
| 작업 branch | `codex/non-draft-integration-20260914` |
| reviewer | jangster77, 검토 전 요청 등록 확인 |
| source → local | `465534ece` → `06a9322bc` |

사용자 지시로 #7141·#7142를 제외했다. draft #6670·#7118도 제외했다. 현재 대상은 #7099·#7109·#7134·#7136·#7137·#7139의 6개 PR, 원 commit 9개다. 원 작성자와 cherry-pick -x 출처를 보존했다.

## 코드와 증거 판독

원 PR은 `trim().is_empty()`로 공백만 있는 문단도 마지막 줄간격 제외 대상으로 만들고 KTX golden을 함께 바꾼다. 최신 devel에는 이미 `0a6224e86`의 완전한 빈 문단 처리와 공백 문단 보존이 들어 있다. 이번 충돌 해결은 height_measurer와 KTX golden의 devel 쪽을 보존하고 원 기여 테스트 2개만 추가했다. 따라서 원 PR 전체를 그대로 승인하거나 신규 렌더 개선이라고 표현하지 않는다.

Visual Sweep로 `36382471_masked.hwpx` 1쪽과 KTX 2쪽을 PDF와 직접 비교했다. 이전 renderer는 `6e5dc6f52`이며 base와 `src/**`, Cargo.toml/lock이 동일함을 확인했다. 통합 전후 render tree는 masked 2/2쪽, KTX 27/27쪽이 동일하다. masked의 `중랑물재생센터` y는 반올림 tree 기준 912.6px이다.

[masked 이전](../assets/pr7099_masked_before_p001.png) · [masked 통합](../assets/pr7099_masked_native_p001.png) · [KTX 이전](../assets/pr7099_ktx_before_p002.png) · [KTX 통합](../assets/pr7099_ktx_native_p002.png).

검토자가 위 비교/overlay 이미지를 직접 판독했다. masked의 placeholder·제목 글꼴, KTX 목차 글꼴·세로 위치 등 기존 차이가 남는다. KTX 정본 대비 +0.95px라는 선행 커밋 설명을 이번 실측으로 재확인한 것처럼 인용하지 않는다. 전후 무변경은 회귀 없음의 제한된 근거이며 한컴 출력 일치의 증거가 아니다.

원 PR을 통째로 merge하면 통합에서 제외한 동작과 golden 변경까지 들어오므로, 최종 통합 PR에서 테스트 추가분의 provenance를 설명하는 방식으로 처리한다.

## CI와 검증 범위

조회한 원 PR head는 위 source SHA와 일치하고, CI에 실패·대기 항목이 없다. skipped/neutral을 실제 검사 통과 수로 합산하지 않는다. [Lint (fmt, clippy, WASM check): SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34750312826/job/103705627519) · [Frontend package gates: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34750312826/job/103705627514) · [Build & Test: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34750312826/job/103707033088).

이 통합 head에는 별도 GitHub CI가 아직 없다. 원 PR들의 녹색 상태를 새 통합 head의 CI 성공으로 대신하지 않는다. 사용자 지시에 따라 완료된 CI 전체 회귀를 로컬에서 반복하지 않았다. macOS 전용 `CARGO_TARGET_DIR=target/nondraft-review-20260914`의 native build를 새 head에서 완료했고, fresh WASM web 빌드도 성공했다(host `--no-opt`, 5분 12초). Docker/wasm-opt 표준 빌드 및 성능 검증 성공으로 확대하지 않는다. [WASM 빌드 로그](../assets/non_draft_20260914_wasm-build-six-pr.log.txt). PR 생성 전 fmt·native/WASM32/workspace all-target Clippy·workspace build·suite manifest·unit tier 검사를 모두 통과했다. [명령·exit code·로그](../assets/non_draft_20260914_pre_pr_validation.json). 검증 head는 `9ac32f05b`이며 이후 변경은 검토 기록·오늘할일·로그뿐이다. 통합 GitHub CI는 PR 생성 후 확인한다.

## 공통 조판 원칙 준수

| 항목 | 판정·범위 |
| --- | --- |
| 독립 근거·원인 계층 | 위 코드/입력 계약에 근거. contributor 주장과 직접 관찰을 구분 |
| 문서별 예외·좌표 clamp | 이번 수용 코드에 문서 ID 분기나 픽셀 맞춤 상수 추가 없음 |
| 측정·배치 공통 결과 | #7109는 공통 사다리 helper. #7099는 devel 유지. 나머지는 레이아웃 산식 변경 비해당 |
| 줄 소속·높이 | 변경 범위와 반례는 본문 참조. 모든 중첩 표·모든 문서 보장으로 확대하지 않음 |
| golden·허용치 | 기존 baseline을 완화하지 않음. #7099 원 golden 변경은 수용하지 않음 |
| 실제 시각 증거 | 실행한 Visual Sweep/Studio만 기록. 미공개 문서/미실행 호스트는 미검증 |
| 성능·전체 fidelity | 이번 검토에서 측정·보증하지 않음 |

입력 SHA-256·Git blob 일치·source CI·원 commit 매핑은 [공통 증적](../assets/non_draft_20260914_evidence.json)에 있다. 이미 Git에 있는 HWP/HWPX/PDF를 이름만 바꿔 중복 추가하지 않았다.

## 최종 통합 이후 comment 계획

통합 PR이 승인·CI 완료·merge된 뒤 원 PR에 구현 출처, 실제 수용 범위, 보정 사유, 검증 결과와 미검증 범위를 설명한다. Visual Sweep가 적용된 PR에는 [사용법](../../manual/verification/visual_sweep_guide.md)과 확정 merge SHA의 증적 링크를 포함한다. 원 PR close는 구현 반영을 확인한 뒤 진행하고 contributor fork branch는 보존한다. 현재 원격 comment/close/통합 PR 생성은 하지 않았다.

## Fresh WASM Visual Sweep 확인

[masked 1쪽](../assets/pr7099_masked_wasm_p001.png) · [KTX 2쪽](../assets/pr7099_ktx_wasm_p002.png)의 실제 비교/overlay를 직접 판독했다. Native와 같은 기존 차이를 확인했다. masked pixel_match 96.728%, ink_match 36.604%, KTX pixel_match 93.824%, ink_match 25.069%다. 높은 전체 픽셀 점수는 빈 영역의 영향이 크므로 통과 지표로 사용하지 않았다.

fidelity의 전 페이지 text-only/export-all-svg/layout-ledger 원장도 [masked](../assets/non_draft_20260914_ledger/masked/run-state.tsv)와 [KTX](../assets/non_draft_20260914_ledger/ktx/run-state.tsv)에 남겼다. table fragment·경계 clip 후보는 존재하며 후보 0이나 전 페이지 시각 일치를 주장하지 않는다. 여기서 직접 PDF와 육안 대조한 페이지는 각 1쪽·2쪽뿐이다.
