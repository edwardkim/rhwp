---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5718 검토 - TopAndBottom 도형의 흐름 후퇴 방지

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5718](https://github.com/edwardkim/rhwp/pull/5718) / `planet6897` |
| base / 원 PR head | `devel` / `451e3d132d2df0cae58576cd7a9a806cf4dc8530` |
| 변경 규모 | 3 files, +124 / 0 |
| 통합 검토 branch | `review/planet6897-20260820` |
| local cherry-pick | `fde038b9b` |
| 통합 기준 | `upstream/devel@cfe2c351e` 위에 #5709 → #5710 → #5718 적용 |
| 관련 issue | #5699 H2 |

원 PR은 비 draft이며 작성 시점 확인에서 Full CI·CodeQL·Render Diff·Native Skia와 관련
필수 검사가 통과했다. GitHub의 mergeability와 check 상태는 merge 직전에 최신 head로 다시
확인해야 한다.

## 변경 범위와 검토 결과

`src/renderer/layout.rs`의 메인 layout loop에서 비-tac `TopAndBottom`인 Picture·Shape·Equation
항목이 진입 흐름보다 작은 y를 반환할 때 흐름을 진입 위치로 고정한다. Square/Tight 등 다른
text-wrap과 `treat_as_char` 항목은 대상에서 제외된다.

이 분리는 TopAndBottom 텍스트가 도형 아래에서만 이어져야 한다는 레이아웃 계약에 맞고, Square의
기존 앵커 복귀 동작을 건드리지 않는다. 샘플 기반 테스트는 서로 다른 문단의 본문 줄이 같은
y 대역에 겹치는 문제만 계약으로 잠근다.

검토 결과 조건 범위가 좁고 기존 wrap 동작을 보존한다. 추가 메인터너 보정은 필요하지 않았다.

## 체리픽 및 충돌

- 최신 `upstream/devel@cfe2c351e` 기반 가시성 branch에 앞선 #5709·#5710을 먼저 적용했다.
- #5718 source head `451e3d13`를 `fde038b9b`로 누적 적용했다.
- 세 PR 모두 충돌 없이 적용됐다.

## 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- 집중 테스트 `issue_5699_shape_flow_rewind`: 1/1 통과
- 통합 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: **8,001 통과, 38 skip**
- 전체 실행에서 H2 샘플의 pi2·pi3 줄 겹침 방지 계약도 다시 통과했다.
- 원 PR의 샘플 기반 Canvas Render Diff와 Native Skia 결과는 통과했다. 통합 branch에서는
  동일 renderer 경로에 대해 집중 테스트와 전체 Rust 회귀를 추가 확인했다.

## 판정

차단 결함과 추가 메인터너 보정 필요 사항은 발견하지 못했다. #5699 H2 범위는 통합 branch에서
수용 권고다. H3 및 후속 시각 fidelity 범위는 별도 과제로 남기며, 원격 push·승인·merge는
수행하지 않았다. merge 전 최신 head와 required check를 다시 확인해야 한다.
