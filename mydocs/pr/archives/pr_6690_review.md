# PR #6690 review - object-only trailing line spacing

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6690](https://github.com/edwardkim/rhwp/pull/6690) |
| Author | `jeong-sik` |
| Base / source head | `devel` / `c379257716458c30028dbd44f84ce8b463c0b96d` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `eb84bbbc7`, `9f0455b6f` (`-x`) |
| Scope | `src/renderer/height_measurer.rs`, one regression case |
| Related issue | [#6681](https://github.com/edwardkim/rhwp/issues/6681) |

## Review

- Reviewer `jangster77` was assigned before local integration.
- The source head was `MERGEABLE/CLEAN` when selected; its latest required CI
  gates were success or policy-expected skips.
- The implementation narrows the existing trailing-line-spacing exception to
  multi-paragraph, inline-table cells whose final line is not object-only. The
  regression test anchors the first text baseline after the affected table.
- Both source commits were applied in original order; cherry-picking only the
  final clippy cleanup commit would have omitted the behavior change.
- Integration-head lint/test and direct visual inspection remain unrun.

## Final decision

- Decision: **머지 보류**
- Release condition: required integration-head Rust gates plus current-head page-4
  visual evidence for `exam_science.hwp` showing the post-table baseline.
- Remote action: none.
