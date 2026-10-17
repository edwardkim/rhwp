# #7974 — public PDF source-mask controls

Small, self-authored PDF inputs; these are source documents, not renderer goldens.
The source dictionaries and packed bytes are the independent test contract.

- `colour-key`, `gray-packed-decode`, `indexed-key`, `gray16-key`, `rgb-all-components`:
  mask bounds compare all original sample components before Decode/colour conversion.
- `standalone-stencil`, `stencil-form-reuse`, `stencil-reverse-padding`, `stencil-alpha`:
  current nonstroking colour/alpha, invocation ownership, bit polarity and row padding.
- `smask-overrides-key`: SMask takes precedence over the deliberately invalid Mask array.
- `low-resolution-mask`, `unmasked-control`, `linear-control`: final image sampling,
  including the existing Interpolate=true smoothing control.

The actual public import/save/reopen tests are in
`tests/cases/issue_7974_pdf_source_masks.rs`. Source-texture sample positions are
sample indices; no document layout position or renderer hash is pinned.

Validation: `mydocs/working/task_m100_7974_stage1.md`. Original PDFs were independently
rasterized at 96 DPI with Poppler, then compared with Native Skia and fresh Docker
WASM Canvas/SVG. `gray16-key` uses MuPDF as the reference: Poppler collapses adjacent
16-bit values (32767/32768) in this case and fails to preserve their different alpha.
MuPDF preserves the distinction required by the PDF source-sample contract.

References: [Adobe PDF Reference 1.7, 4.8.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf),
[W3C PNG chunk naming](https://www.w3.org/TR/png-3/#5Chunk-naming-conventions).
TSV, execution logs, PNGs and source hashes remain in ignored review output.
