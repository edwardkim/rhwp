# PR #6683 review - object-only cell height

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6683](https://github.com/edwardkim/rhwp/pull/6683) |
| Author | `jeong-sik` |
| Base / source head | `devel` / `e5dde4373ed0c8d26543482c8031b0e2aa556baa` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `dd8ca73a2`, `4c333ab94` (`-x`) |
| Scope | `src/renderer/height_measurer.rs`, one regression case |
| Related issue | [#6660](https://github.com/edwardkim/rhwp/issues/6660) |

## Review

- Reviewer `jangster77` was assigned before local integration.
- The source head was `MERGEABLE/CLEAN` when selected. CI, CodeQL, Render Diff,
  Adapter inter-diff, and Proptest completed successfully or with policy-expected skips.
- The two commits narrow the subtraction to a one-paragraph cell whose non-inline
  object exceeds declared cell height, preserving the documented in-cell `#6312`
  counterexample.
- The new case measures the affected `exam_science.hwp` row height rather than a
  position already known to contain another error axis.
- No local lint, test, or integration-head visual sweep has been run in this review.

## Visual evidence

The issue makes a table-row geometry claim. Source-side images are reference-only;
no integration-head PDF/SVG/PNG or stable review asset has yet been generated.

## Final decision

- Decision: **머지 보류**
- Release condition: run the required integration-head Rust lint/test gates and
  inspect a current-head visual result for `exam_science.hwp` page 4 against the
  Hancom reference before changing this decision to approval.
- Remote action: no source comment, close, push, PR creation, approval, or merge
  was performed by this review.
