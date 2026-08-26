---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4959 검토 - HWP5-origin 각주·미주 vpos 보정 분리

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4959](https://github.com/edwardkim/rhwp/pull/4959) |
| 작성자 / source | @planet6897 / `fix/note-vpos-hwp5-origin` |
| 원 source head | `9ffe4878d3cf3c2adf4bbd20b13f02fa92eb7078` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `9135f037a`, `40954853e` |
| 관련 issue | #4916, #4660, #3531 |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

HWP5-origin marker가 있는 자체 산출 HWPX에는 실물 한컴 HWPX의 `vpos=0` 복원 보정을 적용하지 않도록
thread-local RAII guard를 둔다. 저장된 note subList의 정당한 후속 줄 `vpos=0`이 합성값으로 바뀌지 않으며,
marker가 없는 실물 HWPX의 기존 보정은 유지한다. 후속 source commit은 #3915 IR 실패 표본을 이미 정상화된
`pic-crop-01`에서 `hwp3-sample10`으로 교체한다.

## 검증과 판단

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| HWP5-origin note | `cargo test --profile release-test --target-dir target/pr-review --test issue_4916_note_vpos_roundtrip` | 1 passed |
| 실물 HWPX 보정 | `cargo test --profile release-test --target-dir target/pr-review --test issue_1692` | 11 passed |
| 최신 후속 test | `cargo test --profile release-test --target-dir target/pr-review --test issue_3915_verify_axes_both_reported` | 3 passed |
| 누적 전체 Rust | release-test nextest 전체 | 6,519 passed, 38 skipped |
| 품질 | fmt, clippy, diff 검사 | 통과 |

HWP5 marker가 존재하는 경우만 보정을 분기해 기존 실물 HWPX 의미론을 보존한다. **통합 수용 권고.**
