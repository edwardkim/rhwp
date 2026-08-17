---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4942 검토 - TAC 표 유효 높이 계산 공통화

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4942](https://github.com/edwardkim/rhwp/pull/4942) |
| 작성자 / source | @kevin9327 / `fix/effective-h-dedup` |
| 원 source head | `7c0bfcd1d7e50db068b21c0cd22284466b8a32e5` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `5aae6d3e8` |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

`typeset`와 paginator가 중복 보유하던 `seg_lh.max(mt_h)` 판단을 renderer 공통 함수로 모았다. 계산식과
호출 순서는 바꾸지 않는 구조 보정이며, paginator 경로의 존폐 자체는 #4605 범위로 남는다.

## 검증과 판단

| 범위 | 근거 | 결과 |
| --- | --- | --- |
| 누적 Rust 회귀 | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,519 passed, 38 skipped, 7 slow, 389.510초 |
| 출력 baseline | 같은 전체 run의 `visual_baseline_all_samples` | 13.558초, 통과 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |

renderer 경로에는 닿지만 순수 식 추출이고 전체 visual baseline이 통과했다. 작업지시자의 추가 준비 생략 지시에
따라 별도 Native Skia, WASM build, PDF pixel sweep은 반복하지 않았다. **통합 수용 권고.**
