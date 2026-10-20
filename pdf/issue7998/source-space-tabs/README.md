# Source spaces and independent positioning movement

These six independent PDF controls preserve literal U+0020 / U+00A0 scalars and
distinguish ordinary glyph advance from positive movement that cannot fit legal
HWP character fields. They are the exact inputs used for stage36 verification;
no PDF bytes were changed during test registration.

`tests/cases/issue_7998_pdf_source_space_tabs.rs` checks source ownership, two soft
rows, standard tab stream units, actual paint containment/order, insertion and
inverse deletion, and both save adapters. Ordinary/compressed spaces and NBSP
are positive controls; three editable paragraph cases failed on stage35 and pass
after the fix. Source word origins come from the independent PDF text operators.

Native/fresh Docker WASM full scope: 158 inputs / 169 pages each, minima
91.61577% / 90.85213%, no missing pages or page-count mismatch. Same edits were
independently printed by Hancom 2020 (26 jobs plus 6 identical-input aliases).
Standalone source pages used for coordinate checks and saved edits are
distinguished in the evidence; saved Helvetica glyphs can be substituted by
Hancom and are not claimed to have identical outlines.

Evidence: `mydocs/working/task_m100_7998_stage36.md` and local
`/mnt/e/tmp/issue7998-stage36-evidence/final/`. Final integration is recorded
separately; stage36 results do not certify a later merged runtime.
