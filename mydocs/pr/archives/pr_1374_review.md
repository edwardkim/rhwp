# PR #1374 검토 - 미주 측정 SSOT 게이트 재보정

- PR: https://github.com/edwardkim/rhwp/pull/1374
- 제목: Task #1370: 미주 측정 SSOT 게이트 재보정 — A3 break-높이 디커플로 hancom 배치 6/13 무회귀 해결
- 작성일: 2026-06-12
- 작성자: `planet6897`
- 관련 이슈: #1370 "미주 측정 SSOT 게이트 재보정 — 정확 sim 위 hancom 배치 13건 재현 (#1363 v3 후속)"
- base: `devel`
- head: `planet6897:task1370`
- 처리 상태: merged

## 1. 요약 판단

**merge 가능**으로 판단했고, 작업지시자 지시에 따라 merge 완료했다.

이번 PR은 PR #1371의 A3 SSOT opt-in 위에서 이어지는 후속 재보정이다. 핵심 구현은
`src/renderer/typeset.rs`에서 A3가 정확 스냅샷 기반 break 높이 대신 compact 누적 경로를
사용하도록 게이트를 좁히는 것이다. 그 결과 한컴 기준 문제집 13건 중 6건 재현 상태가
회귀하지 않도록 보정한다.

문서 처리도 함께 확인했다. 닫힌 문서 PR #1397로 분리되면 안 되는 PR #1371 / #1395 처리
기록을 이번 정상 merge PR에 동반했고, Task #1370 작업 문서도 #1367 처리 방식처럼
archives 경로로 정리되어 포함됐다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 최종 상태 | merged |
| draft | false |
| base | `devel` |
| head | `planet6897:task1370` |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `CLEAN` |
| merge commit | `2c5ee854f2eff810a67550f663c11428d3376fa5` |
| mergedAt | 2026-06-12 21:20:48 KST |
| 연결 이슈 | #1370 |

## 3. 변경 범위

### 3.1 렌더링 코드

- `src/renderer/typeset.rs`
  - A3 SSOT 경로에서 exact snapshot break 높이를 쓰던 조건을 A2 한정으로 좁혔다.
  - A3는 compact 누적 높이 경로를 사용해 한컴 배치와 맞지 않던 일부 미주 page break
    회귀를 피하도록 했다.

### 3.2 Task #1370 문서

다음 작업 문서가 active 경로가 아니라 archives 경로로 포함됐다.

- `mydocs/plans/archives/task_m100_1370.md`
- `mydocs/plans/archives/task_m100_1370_impl.md`
- `mydocs/report/archives/task_m100_1370_report.md`
- `mydocs/working/archives/task_m100_1370_stage1.md`
- `mydocs/working/archives/task_m100_1370_stage2.md`
- `mydocs/working/archives/task_m100_1370_stage3.md`

### 3.3 PR 처리 기록 동반 반영

다음 리뷰 문서는 이전 문서 전용 PR로 분리하지 않고 이번 PR에 동반했다.

- `mydocs/pr/archives/pr_1371_review.md`
- `mydocs/pr/archives/pr_1371_review_impl.md`
- `mydocs/pr/archives/pr_1395_review.md`
- `mydocs/pr/archives/pr_1395_review_impl.md`

## 4. 검증 결과

### 4.1 GitHub Actions

최종 GitHub 체크는 모두 통과했다.

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Canvas visual diff | pass |
| WASM Build | skipped |

### 4.2 macOS 로컬 검증

`mydocs/manual/dev_environment_guide.md`의 "macOS 로컬 빌드/테스트 검증" 절 기준으로
다음 명령을 수행했다.

| 명령 | 결과 |
|---|---|
| `cargo build --release` | 통과, 6m 36s |
| `cargo test --release --lib` | 통과, 1724 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |

추가로 PR 리뷰 과정에서 다음 확인도 완료했다.

| 명령 | 결과 |
|---|---|
| `cargo clippy -- -D warnings` | 통과 |
| `cargo test --doc` | 통과 |
| `cargo test --test svg_snapshot` | 통과, 8 passed |

merge 후 `local/devel`을 `upstream/devel`에 동기화한 뒤 렌더 영향 PR 후속 확인으로
`cargo test --test svg_snapshot`를 다시 수행했고 8건 모두 통과했다.

## 5. 리스크

| 리스크 | 평가 |
|---|---|
| 미주 공통 로직 근본 해결 | 중간. 이 PR은 A3 게이트 재보정이며 Task #1293 계열 근본 미주 해석과는 별개 |
| 기존 PR #1371 회귀 | 낮음. 관련 unit/integration/svg 검증 통과 |
| 문서 누락 | 낮음. #1371/#1395 처리 기록과 #1370 archives 문서가 PR에 포함됨 |
| golden SVG 회귀 | 낮음. merge 전후 `svg_snapshot` 통과 |

## 6. 후속 처리 결과

- merge 완료: PR #1374, merge commit `2c5ee854f2eff810a67550f663c11428d3376fa5`
- 이슈 #1370: GitHub auto-close가 동작하지 않아 수동 close
- PR 코멘트: merge 완료와 검증 결과 요약 등록
- `local/devel`: `upstream/devel` merge commit 기준으로 동기화 완료
- 렌더 영향 후속 확인: `cargo test --test svg_snapshot` 통과
