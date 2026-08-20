---
kind: pr-review
status: superseded-by-integrated-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5745 검토 - 첫 쪽만 배경/테두리

- source head `734b1f4ee28c0292ee7463e9f8dffd3ec446a281`는 `Lint (fmt, clippy, WASM check)` 실패로
  `MERGEABLE/BLOCKED`다. `Build & Test` 실패는 이 lint aggregate를 따른 결과다.
- 같은 #5717 수정은 후속 PR #5762가 HWPX `SHOW_FIRST` 파싱·직렬화 왕복, 첫 쪽/전 쪽 렌더 회귀와 함께
  포함한다. 따라서 이 통합에는 #5745를 별도 cherry-pick하지 않고 #5762를 적용했다.
- #5762를 포함한 통합 후보에서는 fmt, clippy, standard WASM build, 전체 8,025 nextest 및 native-Skia가
  모두 통과했다. 중복 적용이나 실패 head의 직접 수용은 권고하지 않는다.
