# 2026-07-13

## PR #2219 review feedback - public API restoration and middle-anchor guard

- Finding 1 (independent re-review): the middle-anchor fix changed the public
  `is_tac_table_inline` signature from `(table, seg_width, text, controls)` to
  `(table, seg_width, para)`, breaking Rust API compatibility for library consumers.
- Decision 1: restore the original four-argument function byte-compatible with HEAD and move the
  middle-anchor rule into a new `is_tac_table_inline_in_para(table, seg_width, para)` that
  delegates to the original. All internal call sites already hold a `&Paragraph`, so they switch to
  the new function; the public surface only grows, never changes.
- Alternative rejected: keeping the three-argument signature and bumping semver — the crate is
  consumed as a renderer library and the review explicitly required signature preservation.
- Finding 2 (full-suite verification): the middle-anchor rule declared any table anchored strictly
  inside the text range inline. HWP receipts pad TAC paragraphs with U+F081C PUA fillers
  (복학원서.hwp pi=16: 99 filler chars around a 93%-width table), so the rule forced those tables
  inline and broke the synthesized seal line —
  `issue_2020_bokhak_receipt_seal_line_and_stamp_align` failed.
- Decision 2: the middle anchor only counts when at least one real text character
  (`char::is_alphanumeric`, the same predicate Issue #842 established for this exact sample)
  exists on each side of the anchor. Filler-only, whitespace-only, and object-marker-only
  neighborhoods fall back to the original width rules.
- Alternative rejected: excluding a hard-coded PUA list — `is_alphanumeric` already excludes PUA,
  whitespace, and markers, and matches the precedent in `src/renderer/layout.rs` (#842 comment).
- Verification: `cargo test` 3125 passed / 254 suites, `cargo clippy --all-targets --all-features
  -- -D warnings` clean, `cargo fmt --all -- --check` clean, native SVG re-export of
  `formatting_table.hml`, `aligns.hml`, `복학원서.hwp` with `abc`→table→`efg` coordinates verified.
