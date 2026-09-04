# PR #6698 review - NBSP advance width

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6698](https://github.com/edwardkim/rhwp/pull/6698) |
| Author | `jeong-sik` |
| Base / source head | `devel` / `7e47ef6914edfed1852c7fff99cd04cdc71713a4` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commit | `c0145ec66` (`-x`) |
| Scope | NBSP measurement and one `exam_eng.hwp` regression case |
| Related issue | [#6646](https://github.com/edwardkim/rhwp/issues/6646) |

## Review

- Reviewer `jangster77` was assigned before local integration.
- The source head was `MERGEABLE/CLEAN` at selection and its required CI checks
  were green or policy-expected skips.
- The change reuses the established ordinary-space measurement path for `U+00A0`;
  it does not introduce a font-specific constant. The test locks the affected
  line's punctuation-to-text advance rather than a broad document coordinate.
- Integration-head local lint/test and current-head visual confirmation remain
  unrun, so source CI is not recorded as an integration result.

## Final decision

- Decision: **머지 보류**
- Release condition: complete the integration lint/test gates and inspect the
  relevant `exam_eng.hwp` output against the cited Hancom spacing before approval.
- Remote action: none.
