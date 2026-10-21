# Editable PDF text frames

Synthetic one-page PDF inputs created from independent affine matrices and the
installed ArialMT font widths. No font program or user PDF is included.
`rotation-outside-origin` has a baseline outside the MediaBox and visible glyphs
inside it. `rotation-clip` adds a rectangular clip; `rotation-form` paints through
an XObject Form. Negative fixtures retain explicit geometry rejection.

The producing script and SHA-256 manifest, unchanged input references, 96 DPI
Native/fresh Docker WASM TSVs and direct comparisons are linked from
`mydocs/working/task_m100_7998_stage37.md` (2026-10-21 text transformation).
All nine accepted inputs were qualified before regression registration: Native
minimum 97.21048%, fresh WASM minimum 95.70248%. Assertions protect editable
ownership, orientation and transformed subtree content through HWP/HWPX reopen;
they do not pin pixel coordinates or claim Hancom export fidelity.
