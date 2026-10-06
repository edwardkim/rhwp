---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7134 Studio 기본 한 쪽 보기 검토

**판정: 승인.** 로컬 체리픽 통합본에 대한 검토다. 원 PR merge/close 또는 GitHub approve를 완료했다는 의미가 아니다.

## 출처와 통합 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#7134](https://github.com/edwardkim/rhwp/pull/7134) / davindev |
| 원 head | `7a9c9eb0cae6df74bd912f67328f004c13510860` |
| 최신 devel 시작점 | `410d22cdf77e3e9e45159999cab19719c621c025` |
| 통합 code head | `b40c2953cf526b86319f2841c5b5584d7a83d735` |
| 작업 branch | `codex/non-draft-integration-20260914` |
| reviewer | jangster77, 검토 전 요청 등록 확인 |
| source → local | `7a9c9eb0c` → `0fca3bba6` |

사용자 지시로 #7141·#7142를 제외했다. draft #6670·#7118도 제외했다. 현재 대상은 #7099·#7109·#7134·#7136·#7137·#7139의 6개 PR, 원 commit 9개다. 원 작성자와 cherry-pick -x 출처를 보존했다.

## 코드와 증거 판독

새 사용자 기본값을 `single`로 바꾸고, 저장 설정에 pageArrangement 항목이 없을 때 같은 기본값을 적용한다. 사용자가 명시적으로 저장한 auto/two/facing/multiple 설정과 공통 resolver의 손상값 처리 계약은 보존한다. `DEFAULT_PAGE_ARRANGEMENT`를 전역 변경하지 않아 기존 그리드 API의 하위 호환 범위를 건드리지 않는다.

관련 #7133은 기본 쪽 배열 요구이며 문서 조판이나 PDF 페이지 구성을 바꾸는 요청이 아니다. 이 PR의 렌더러/PDF fidelity 검증은 비해당이다. 실제 Studio의 40% 배율·넓은 화면과 저장된 auto 유지 결과는 아래에 기록한다. davindev는 기존 merged PR #7013·#6591이 있어 첫 기여자가 아니다.

## CI와 검증 범위

조회한 원 PR head는 위 source SHA와 일치하고, CI에 실패·대기 항목이 없다. skipped/neutral을 실제 검사 통과 수로 합산하지 않는다. [Lint (fmt, clippy, WASM check): SKIPPED](https://github.com/edwardkim/rhwp/actions/runs/34815730511/job/103885960256) · [Frontend package gates: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34815730511/job/103885959054) · [Build & Test: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34815730511/job/103887534777).

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

## 실제 Studio 확인

macOS / Chrome 152.0.7977.83 / 1600×1100 viewport / fresh WASM의 Studio에서 저장값이 없는 새 세션을 열었다. `pageArrangement=single`, 세로 방향을 확인했다. 원본 11쪽 문서를 40%로 표시해 [한 열로 이어지는 화면](../assets/pr7134_single_40pct.png)을 직접 판독했다. `auto`를 저장한 뒤 같은 origin의 새 Studio 페이지를 초기화했을 때 `auto`가 유지됐다. [원시 결과](../assets/non_draft_20260914_studio_result.json).

검증 harness의 첫 실행은 WASM 생성 전 시작했던 Vite의 실패한 경로 캐시 때문에 500 오류가 났고, 서버 재시작 후 해결했다. `activate` 호출 오기도 실제 API `activateWithCaretPosition`으로 수정했다. 같은 페이지 reload는 navigation timeout이 있어 성공 증거에서 제외하고 저장 설정 재초기화는 새 Studio 페이지에서 확인했다. 최종 실행의 pageerror/console error는 0개다. 이 harness 준비 실패를 PR 회귀로 분류하지 않았다.
