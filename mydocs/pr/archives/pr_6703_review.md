# PR #6703 review - HWP5 near-top reset narrowing

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6703](https://github.com/edwardkim/rhwp/pull/6703) |
| Author | `planet6897` |
| Base / source head | `devel` / `219868e86f94b47f0b033bf2b50d64ca655ef8d0` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `ceadaf94a`, `2dd41febf` (`-x`) |
| Scope | `src/renderer/typeset.rs`, one external-corpus regression case |
| Related issue | [#5941](https://github.com/edwardkim/rhwp/issues/5941) |

## Review

- Reviewer `jangster77` was assigned before local integration.
- The selected source head was `MERGEABLE/CLEAN`; CI, CodeQL, Render Diff,
  Adapter inter-diff, and Proptest were completed successfully or policy-skipped.
- The narrowing is profile-plus-fill based: native HWP5 stored-pagination pages
  only retain the saved near-top reset once the page is no longer nearly empty;
  the original HWPX and empty-page contracts remain explicit negative controls.
- The primary 202-page corpus case is intentionally external. Its test returns
  only when the fixture is unavailable; it must be run with
  `RHWP_ISSUE5941_SAMPLE` set to the identified private source file.
- No such integration-head run or direct visual comparison has been executed.

## Final decision

- Decision: **머지 보류**
- Release condition: run the required integration lint/test gates with the
  `#5941` corpus input, then retain current-head representative visual evidence
  for the filled-page and empty-page controls.
- Remote action: none.
