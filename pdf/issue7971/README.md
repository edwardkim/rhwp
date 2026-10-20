# Scan-sized JPEG buffer controls

Generated 5-page PDF for #7971, included in PR #7970. This file contains only generated
colour bands, labels and borders. It has no pixels or text from the private maintainer input.
The private input, its hash and comparison images remain local validation evidence.

Generate with `python3 pdf/issue7971/generate.py` (Pillow 10.2.0).
SHA-256: `d056da61de9235369c99f7b21c8dd538a016455b7f49b646692e504083172cba`.
Size: 2,107,269 bytes. QPDF check exits 0.

Pages 1–3 contain one 1,928×2,725 RGB baseline JPEG each, page 4 uses
1,929×2,728 and page 5 uses 1,929×2,729. At those sizes an RGB/RGBA buffer fits
32 MiB while a hypothetical eight-byte plane does not. The PDF has one image paint per
page and retains every original generated sample; no downsampling is needed for import.

The independent rendering oracle is Poppler at 96 DPI, compared with Native Skia and
fresh Docker WASM Canvas at Print/compat 2022. Full raw TSV and enlarged representative
comparisons are recorded in the PR body and task stage report after execution.
These resource-boundary fixtures are not a claim of general PDF codec allocation safety.
