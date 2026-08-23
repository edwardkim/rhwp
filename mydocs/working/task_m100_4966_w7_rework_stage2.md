---
kind: report
status: active
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-R2·R3 소유권·회귀 경계 정정

## 1. 판정

W1은 현재 source의 유지 조건이 아니라 `sourceCommit`에 고정된 역사 증거로 전환했다. W7 현재 authority는
canonical registry·migration manifest·generated projection과 공개 runtime trace가 담당한다. 제품
`src/**`에 남았던 전환 전 수기 mapping 4개와 신규 oracle helper 2개는 제거했고, 회귀 원본은
`CONTRIBUTING.md`에 따라 `tests/cases/issue_4966_font_rule_projection.rs`로 옮겼다.

PR head에 collaborator가 추가한 `745660467`은 수기 oracle을 기존 `#[cfg(test)] mod tests` 내부로 옮겨
top-level unit-tier 탐지를 피했다. 그러나 제품 source에 새 test support를 두지 않는 현재 기여 규칙과
소유권 단일화 목적에는 맞지 않는다. 해당 커밋은 이력에 포함하되 최종 tree에서는 integration 원본으로
대체한다.

## 2. 역사 증거와 현재 authority 분리

- W1 30개 boundary·1,352개 candidate는 `795e7b5f`의 Git blob을 직접 읽어 path·SHA-256·selector
  match count를 검증한다.
- 현재 checkout에서 이관 완료된 과거 selector의 존속을 요구하지 않는다.
- W3 candidate 의미 비교에서는 `sourceLocation`을 제외하고, path·symbol·selector 이동은 별도
  `ownershipDriftCandidateIds`로 보고한다.
- W6 metric-table 600개는 의미 변경 0건·ownership 이동 600건으로 분류된다.
- 현재 metric lineage의 67개 alias 입력은 수기 Rust 함수가 아니라 canonical registry의
  `rust-layout-metric` projection에서 구성한다.

## 3. Rust 회귀 경계

새 integration 원본은 다음 두 축을 공개 API로 검증한다.

| 축 | 범위 | 결과 |
| --- | ---: | --- |
| layout-name 공개 trace | 직접 관측 137 rule + legacy 우선순위 shadow 34 rule | 171개 폐쇄 |
| layout-metric 공개 lookup | alias 67 rule × bold·italic 4조합 | 전건 일치 |

review 전용 detached worktree에서 `--prepare`와 manifest `--check`를 수행했다. 원본은
`regression_suite_014`에 자동 배정됐고 실제 2개 테스트가 통과했다. generated harness와
`tests/suites/manifest.json`은 작업 branch에 포함하지 않았다.

## 4. focused 검증

| 검사 | 결과 |
| --- | --- |
| PR-base unit-tier | 4,221 tests / 299 modules / cfg support 28, 통과 |
| W1·W2·W3·W6·W7 Node contract | 87/87 |
| W6 baseline·lineage check | 통과 |
| `style_resolver` source test | 26/26 |
| `font_metrics_data` source test | 9/9 |
| 새 integration source | 2/2 |
| prepared manifest | 891 source / 4,158 static test attrs / 32 suite + 9 exception |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo fmt --all -- --check` | 통과 |

## 5. W7-R4 인계

원격 `task_m100_4966`의 collaborator commit을 로컬 이력에 포함하고 정책에 맞는 최종 tree로 충돌을
해소한다. 그 최신 후보에서 전체 release-test·Native Skia·Docker WASM·공개 native/WASM parity를 다시
검증한다. remote push와 PR 본문 완료 체크는 별도 승인 전에는 수행하지 않는다.
