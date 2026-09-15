---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7137 진단의 저장 줄 귀속 이탈 축 검토

**판정: 승인 — 공개 CLI 도움말 누락은 비차단 보완 권고.** 로컬 체리픽 통합본에 대한 검토다. 원 PR merge/close 또는 GitHub approve를 완료했다는 의미가 아니다.

## 출처와 통합 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#7137](https://github.com/edwardkim/rhwp/pull/7137) / planet6897 |
| 원 head | `0d7a3292ca58e0b65a5b26f13d8ddb4e3fc58bff` |
| 최신 devel 시작점 | `410d22cdf77e3e9e45159999cab19719c621c025` |
| 통합 code head | `b40c2953cf526b86319f2841c5b5584d7a83d735` |
| 작업 branch | `codex/non-draft-integration-20260914` |
| reviewer | jangster77, 검토 전 요청 등록 확인 |
| source → local | `0d7a3292c` → `82b419368` |

사용자 지시로 #7141·#7142를 제외했다. draft #6670·#7118도 제외했다. 현재 대상은 #7099·#7109·#7134·#7136·#7137·#7139의 6개 PR, 원 commit 9개다. 원 작성자와 cherry-pick -x 출처를 보존했다.

## 코드와 증거 판독

새 `storedLineEscape`는 저장 줄 baseline과 렌더 baseline의 일반적인 차이를 모두 오류로 취급하지 않는다. 줄 소속을 특정할 수 있고, 렌더 baseline이 자기 줄 상자 밖이며 같은 문단의 다른 저장 줄 baseline과 일치하는 좁은 신호다. 원본 IR이 없는 `scan_page` 호출은 기존 5축 동작을 유지하고 `scan_page_with_source`/`scan_document`에서 새 축을 적용한다. JSON 집계와 --strict에 연결되어 있다.

렌더러를 바꾸는 PR이 아니므로 조판·paint 수치 변화와 PDF 개선 주장은 비해당이다. 기대값은 contributor가 제시한 저장 LineSeg 계약 및 합성 음성/양성 검사로 검토했다. samples 961건·코퍼스 1,986건 무신호 및 revert 대조 165.37px 검출은 contributor 기록이며 여기서 전수 재실행하지 않았다.

직접 CLI 진입점을 실행해 exam_eng 1쪽, issue7018 2쪽, tac-case-003 1쪽을 확인했다(`-p`는 0-based). 세 입력 모두 exit 0, storedLineEscapeCount 0이다. 현재 이미 수정된 입력의 무신호 검증이며 결함 검출률 검증이 아니다. 처음 1-based로 요청한 명령은 범위 오류여서 제외하고 올바른 페이지로 다시 실행했다. [실행 명령](../assets/pr7137_diagnostic-runs.json).

**비차단 보완:** `rhwp layout-anomaly --help`의 공통 CLI metadata는 여전히 5종만 설명하고 새 `--stored-line-tolerance` 및 strict 신호를 누락한다. 실제 parser에서는 옵션이 동작한다. `src/cli/catalog.rs`, `src/cli/metadata/help/public.rs`, capabilities의 자기서술을 맞추는 후속 보완이 필요하다.

이 축은 글자와 그림 겹침을 검출하지 않는다. #7061의 제안 수식을 그대로 구현한 것이 아니라 정본 반례 때문에 범위를 좁힌 진단이다. 최종 통합 PR에서 #7061 전체를 자동 close하기 전에 이 범위를 명시해야 한다.

## CI와 검증 범위

조회한 원 PR head는 위 source SHA와 일치하고, CI에 실패·대기 항목이 없다. skipped/neutral을 실제 검사 통과 수로 합산하지 않는다. [Lint (fmt, clippy, WASM check): SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34823701309/job/103910939402) · [Frontend package gates: SKIPPED](https://github.com/edwardkim/rhwp/actions/runs/34823701309/job/103910941032) · [Build & Test: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34823701309/job/103914301395).

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
