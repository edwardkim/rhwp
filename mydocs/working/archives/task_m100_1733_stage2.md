# Task #1733 Stage 2 — PR급 회귀 검증 보강

## 배경

Stage 1 변경으로 `samples/task1725/text_footnote_tail_overpagination.{hwp,hwpx}` 는 기준 PDF와 같은
242쪽까지 맞췄다. 다만 PR 준비 과정에서 승인 전 PR #1856 이 생성되어 사용자가 close 했고, 이후
로컬에서 PR급 전체 회귀 검증을 다시 수행했다.

## 발견한 회귀

첫 전체 통합 검증에서 다음 회귀가 확인됐다.

- `issue_1073_nested_table_split::kps_ai_nested_table_split_no_title_duplication`
  - 원인: `saved_bounds_fit_at_flow_tail` 의 공통 tolerance 를 #1733 tail 보정용 128px 로 넓혀,
    nested table split/title duplication 가드에 과도하게 적용됐다.
- `issue_rowbreak_chart_overlap` 3건
  - 원인: 하단 빈 문단 bridge 흡수와 saved tail split 완화가 RowBreak 표 인접 흐름에도 적용되어,
    rowbreak chart/nested table 케이스의 tail 배치를 흔들었다.

## 변경

- `saved_bounds_fit_at_flow_tail` 의 기존 공통 tolerance 는 16px 로 복원했다.
- #1733 전용 경로만 `saved_bounds_fit_at_flow_tail_with_tolerance(..., 128px)` 를 사용하도록 분리했다.
- 하단 빈 문단 bridge 흡수는 현재 페이지에 `PartialTable` 이 이미 있으면 적용하지 않도록 제한했다.
- partial paragraph tail split 완화는 `para_near_rowbreak_table(...)` 인접 문단에는 적용하지 않도록 제한했다.

## 검증

집중 회귀:

```bash
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1733
# ok: 2 passed
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1073_nested_table_split
# ok: 3 passed
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap
# ok: 20 passed
```

전체 검증:

```bash
cargo fmt --check
# ok
env CARGO_INCREMENTAL=0 cargo test --release --lib
# ok: 2075 passed; 0 failed; 6 ignored
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
# ok
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
# ok
cargo test --doc
# ok: 0 passed; 0 failed; 1 ignored
```

## 결론

#1733 의 242쪽 정합은 유지하면서, `saved_bounds_fit_at_flow_tail` 의 공통 확산과 RowBreak 인접 흐름 회귀를
차단했다. 이번 커밋은 로컬 커밋까지만 수행하고, remote push/PR 은 작업지시자 승인 전까지 대기한다.
