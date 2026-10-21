# #8109 font provenance controls

These handwritten PDF controls test the distinction between a missing standard
Type1 program and a declared, unreadable, or unknown program. They are synthetic
resource contracts, not samples saved by Hancom. Each successful PDF is its own
independent visual reference. `unresolved-descriptor.pdf` intentionally keeps the
existing bounded parser rejection and is not a renderable fixture.

`generate_controls.py` writes the eleven PDFs using explicit object dictionaries,
content streams, and xref offsets. It requires fontTools and the system
`fonts-urw-base35` package. `manifest.json` records the source font and PDF hashes.
The generator does not write `standard-substitute.hwpx`.

`standard-substitute.hwpx` was made from `pdf/issue7998/body-list.pdf` with the
pre-fix importer and an explicit model declaration `Helvetica` plus TrueType
substitute `Arial`. It independently exercises HWP's alternative face/type byte
and HWPX → HWP → HWPX preservation without relying on the new PDF policy.
Its generation and independent Hancom 2020 Print are recorded in
`mydocs/working/task_m100_8109_stage1.md`; current qualification is in
`mydocs/working/task_m100_8109_stage2.md`.

The embedded-program controls include only a CFF program inside their PDF text
document. Source: Artifex/URW `NimbusSans-Regular.otf`, distributed by Debian's
`fonts-urw-base35` package, copyright 2015 URW Software and 2013–2014 (URW)++
Design & Development. License: GNU AGPL version 3 with this font exception:

> As a special exception, permission is granted to include these font
> programs in a Postscript or PDF file that consists of a document that
> contains text to be displayed or printed using this font, regardless
> of the conditions or license applying to the document itself.

Upstream source and license: https://github.com/ArtifexSoftware/urw-base35-fonts.
No standalone font program or commercial font is included.
