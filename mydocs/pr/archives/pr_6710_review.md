# PR #6710 review - profile-agnostic first-fragment allowance

## Metadata

| Item | Value |
| --- | --- |
| Source PR | [#6710](https://github.com/edwardkim/rhwp/pull/6710) |
| Author | `planet6897` |
| Base / source head | `devel` / `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb` |
| Integration branch | `review/green-ci-batch-20260904-full` |
| Applied commits | `c340bd7a8`, `61cd71fb9` (`-x`) |
| Maintainer correction | `872f3d4c5` |
| Scope | stored first-fragment allowance, `#4658` test mediation, two source PNGs |
| Related issue | [#5057](https://github.com/edwardkim/rhwp/issues/5057) |

## Review

- Reviewer `jangster77` was assigned before local integration. The source head
  was `MERGEABLE/CLEAN`; latest required CI checks were success or
  policy-expected skips.
- The implementation makes the saved first-fragment allowance available to the
  direct-HWPX stored-layout path as well as native HWP5, and updates the
  page-count-based `#4658` mediation so its different-page contract remains
  meaningful.
- `before_p7.png` and `after_p7.png` were cherry-picked as source references,
  not accepted as integration-head visual evidence.
- The contributor test skipped on a missing private fixture as intended, but
  also returned successfully after HWPX export/ZIP processing failures.
  Maintainer correction `872f3d4c5` makes every post-fixture failure explicit.
- The local `21484591` corpus file is available for
  `RHWP_ISSUE5057_SAMPLE`; no resulting integration-head test or visual output
  has yet been produced.

## Final decision

- Decision: **메인터너 보정 후 수용 가능**
- Original head: `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb` can hide failed
  regression-test setup after a fixture is found.
- Corrected integration head: includes `872f3d4c5`; acceptance still requires
  the integration lint/test gates and direct HWP5/direct-HWPX page-visible
  evidence with stable review assets.
- Remote action: none.
