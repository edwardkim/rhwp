rhwp is a free and open-source extension that lets you open, edit, and print HWP/HWPX documents directly in your browser. No separate software installation required.

Key Features:

Auto-open HWP/HWPX files in the viewer when downloading from the web
Document editing: text input/modification, table editing, formatting
Printing: Ctrl+P for print preview, save as PDF or send to printer
Save edited documents as HWP files
Open files via drag & drop
Auto-detect HWP links on web pages and display an icon (badge)
Document info preview card on mouse hover
Right-click menu: "Open with rhwp"

⚠️ Important Notice:

Direct HWPX save is currently disabled (beta). You can view and edit
HWPX files but cannot save them yet — full support is coming in the
next update. (HWP file save works normally.)

Please back up important HWPX documents before editing.

Privacy:

All processing happens in the browser via WebAssembly (WASM)
Files are never sent to any external server
No ads, no tracking, no sign-up required
We do not collect any personal information
Web Developer Support:

HWP link integration via the data-hwp-* protocol
Built-in developer tools (rhwpDev) for debugging
Developer guide provided
Recommended for:

Citizens viewing government and public-sector documents
Parents checking school newsletters
Office workers reviewing contracts and reports
macOS/Linux users without Hancom Office
Anyone who does not want to install separate software just to open HWP files
MIT licensed — free for personal and commercial use.

[v0.2.2 Changes — 2026-05-01]

■ v0.2.2 (2026-05-01) Highlights

This update bumps the library core from v0.7.3 → v0.7.9 (4 cycles
combined). Includes typesetting/rendering fixes from many external
contributors and maintainer regression fixes.

[Typesetting / Rendering]
• Multi-column section accumulation formula fix — branched by
  single/multi-column, suppresses trailing line_spacing inflation
  (external contribution by @planet6897, PR #391)
• Multi-column right-column single-line paragraph line spacing fix
  (external contribution by @planet6897, PR #424)
• Table page split: row-unit split allowed for large rowspans —
  Hancom-compatible (external contribution by @planet6897, PR #401)
• Inline image pagination fix in the same paragraph — fixed second
  image being drawn at the same y-coordinate (external contribution
  by @planet6897, PR #406)
• heading-orphan vpos-based 5-condition guard — heading is now
  placed together with the subsequent table (external contribution
  by @planet6897, PR #408)
• Image auto-crop (FitToSize+crop) formula fix + cell-internal image
  clamp (external contribution by @planet6897 + maintainer follow-up)
• Hancom-defensive-logic mimic guard for the case where cell padding
  exceeds cell.height — fixes missing cell text and cell-entry
  failure (mel-001 personnel-table regression)

[Equation Rendering]
• Inline equation height set based on HWP authoritative value with
  simultaneous X/Y scaling (external contribution by @oksure, PR #396)
• Separated ATOP / OVER semantics + AST parsing (external contribution
  by @cskwork, PR #397 — first external contributor to this repository)
• Equation tokenizer font-style keyword prefix split + renderer italic
  honor (external contribution by @planet6897)
• Alignment applied to empty-text + TAC equation cells — fixed
  left-anchored equations (external contribution by @planet6897)
• HWPX equation serialization preservation (external contribution
  by @cskwork, PR #400)

[Images / Shapes]
• Picture+Square wrap host paragraph text now respects LINE_SEG
  cs/sw to avoid overlapping the image area (external contribution
  by @planet6897)
• Picture serialization within group implemented — fixes data loss
  for group-pictures when saving HWP (external contribution by
  @oksure, PR #428)
• HWP image effects (brightness/contrast) reflected in SVG (external
  contribution by @oksure, PR #395)
• wrap=Square table paragraph margin reflected in x coordinate
  (external contribution by @planet6897)

[Footnotes / Pagination]
• Footnote multi-paragraph line_spacing alignment (external
  contribution by @planet6897)
• PartialParagraph inline Shape page routing fix (external
  contribution by @planet6897)
• TypesetEngine pagination fit accumulation drift fix (maintainer
  regression fix)

[API · Tooling]
• Public Paragraph char_idx conversion method exposed — supports
  Python/Node bindings (external contribution by @DanMeon, PR #494)
• PageLayerTree generation API introduced — common foundation for
  multiple renderer backends (external contribution by @seo-rii,
  PR #419)
• Canvas visual diff regression verification infrastructure — legacy
  Canvas ↔ PageLayerTree replay automated pixel diff (external
  contribution by @seo-rii, PR #498)
• rhwp-studio PWA support — offline usage available (external
  contribution by @dyjung150605, PR #413)
• editor.exportHwp() API added (external contribution by @ggoban,
  PR #411)

[Web Page Integration]
• Better URL handling for some Korean government download flows —
  fixes cases where the viewer could not fetch the file directly

[Known Limitations]
• Direct HWPX save is in beta and disabled for now (until the
  HWPX→HWP full converter lands)
• If the print preview window appears too large, press Ctrl+0 to
  reset zoom

[Thanks to Contributors]
@cskwork — first external contributor to this repository (PR #397, #400)
@DanMeon (PR #405, #494)
@dyjung150605 — first-time contributor (PR #413)
@ggoban — first-time contributor (PR #411)
@oksure (PR #395, #396, #427, #428)
@planet6897 (PR #391, #401, #406, #408, #410, #415, #424, #434, #478)
@seo-rii (PR #419, #498)


■ v0.2.1 (previous)

Library v0.7.3 — multiple external contributors + maintainer
regression fixes consolidated. Hotfix for the v0.2.0 cycle's missing
version sync in dev-tools-inject.js / content-script.js.


[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp
