# PR #6709 review - paper-relative Square float reflow

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6709](https://github.com/edwardkim/rhwp/pull/6709) |
| Author | `planet6897` |
| Base / source head | `devel` / `36b5500891e750be7680c2559e2c278d4cbbe175` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `e6b9a3ed5`, `ffd47191e` (`-x`) |
| Maintainer correction | `872f3d4c5` |
| Scope | paper/page coordinate conversion and picture-band re-projection |
| Related issue | [#6202](https://github.com/edwardkim/rhwp/issues/6202) |

## Review

- Reviewer `jangster77` was assigned before local integration. The source head
  was `MERGEABLE/CLEAN` and all selected required checks were success or
  policy-expected skips.
- The two source commits calculate a `PaperOrigin` and convert paper-relative
  vertical positions to the host band's local frame before re-projecting the
  body after picture edits. The static path remains constrained to non-TAC
  `Square` pictures with supported anchor modes.
- The contributor regression case conditionally skips only if its private
  fixture is absent, but previously treated a supplied-fixture
  `set_picture_properties_native` error as success. Maintainer correction
  `872f3d4c5` changes that path to an explicit failure.
- The private `156483689` source file exists locally and must be supplied as
  `RHWP_ISSUE6202_SAMPLE` for actual validation. No integration-head run or
  visual sweep has been performed yet.

## Final decision

- Decision: **메인터너 보정 후 수용 가능**
- Original head: `36b5500891e750be7680c2559e2c278d4cbbe175` lacks a
  fail-closed assertion for a failed picture edit.
- Corrected integration head: includes `872f3d4c5`; it is a candidate only
  after required integration lint/test gates and current-head moved-picture
  visual evidence confirm both reflow and no stale exclusion band.
- Remote action: none.
