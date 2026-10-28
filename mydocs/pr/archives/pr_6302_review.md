---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6302 검토 - 투명선 병합 경계 안내선

- PR: [#6302](https://github.com/edwardkim/rhwp/pull/6302)
- 이슈: [#6301](https://github.com/edwardkim/rhwp/issues/6301)
- 작성자: `@t2c-lab` (rhwp 첫 기여)
- 코드 후보 head: `9c1bb890a890af45057c958a6a3294697ab2c3a8`

## 변경 검토

기여 구현은 병합된 셀 내부에 남아 있던 투명 테두리 안내선을 제거하도록 table border 렌더링과
span 내부 커버 범위를 조정한다. 변경 경로는 `src/renderer/layout/border_rendering.rs`,
`table_cell_content.rs`, `table_layout.rs`, `table_partial.rs` 및 이슈 회귀 테스트다.

원 기여 구현은 수용한다. maintainer는 제품 코드를 바꾸지 않고
`tests/cases/issue_6301_transparent_border_merge_guide.rs`만 보완했다.

## maintainer 보정 사유

원 테스트는 가로 병합 셀의 안내선 제거만 확인했다. 이슈의 병합 경계 처리를 안전하게 유지하려면
세로 span 내부도 안내선 없이 처리되는지와, 실제 `none` 테두리는 편집 안내선이 계속 표시되는지의
역방향 경우도 함께 보호해야 한다.

- 보정 commit: `9c1bb890a890af45057c958a6a3294697ab2c3a8`
- 추가 범위: 가로·세로 span 내부의 안내선 제거, 실제 `none` 테두리의 안내선 유지
- 제품 코드 변경: 없음

## 검증

- GitHub Actions: 코드 후보 최신 head에서 CI, CodeQL, Render Diff, Adapter inter-diff,
  Proptest roundtrip, Native Skia를 포함한 required check가 성공했다.
- 로컬 focused 회귀: `issue_6301_transparent_border_merge_guide` 2건 통과.
- 로컬 형식·계약: `cargo fmt --all -- --check`, Rust test suite manifest prepare/check 통과.
- 동작 검증: 작업지시자가 실제 동작을 확인했다.
- PDF·visual sweep: 원본 HWP/HWPX와 기준 PDF가 제공되지 않아 이번 merge 판단 근거로 실행하지 않았다.

## 결론과 후속 처리

PR #6302는 수용 가능하다. 이 trailing commit은 첫 기여자 외부 PR 공식 절차, review 기록과
오늘할일을 함께 보존한다. 최신 trailing head의 문서 CI가 성공하고 merge 가능 상태를 재확인한 뒤
merge 후속 처리와 첫 기여자 감사 comment를 진행한다.
