# PR #6705 review - continued-paragraph floating picture anchor

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6705](https://github.com/edwardkim/rhwp/pull/6705) |
| Author | `jeong-sik` |
| Base / source head | `devel` / `05325df7c4350b101276580803a208c62709c05a` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `7b15d9582`, `dda7902e5` (`-x`) |
| Scope | `src/renderer/layout.rs`, `hwp3-sample.hwp` SVG assertions |
| Related issue | [#6704](https://github.com/edwardkim/rhwp/issues/6704) |

## Review

- Reviewer `jangster77` was assigned before local integration.
- The source head was `MERGEABLE/CLEAN` with required CI checks green or
  policy-expected skips at selection time.
- The change reuses the existing `PartialParagraph { start_line > 0 }` meaning
  for a `vert=Para` picture anchored in a paragraph that continues from an
  earlier page. The new case checks both the corrected 519px picture and an
  unchanged local-anchor footer picture.
- The PR claims a page-visible geometry correction, so source CI is insufficient
  to approve the combined integration head without direct current-head output.

## Final decision

- Decision: **머지 보류**
- Release condition: required integration Rust gates and a current-head
  `hwp3-sample.hwp` page-7 visual comparison against the stated Hancom geometry.
- Remote action: none.
