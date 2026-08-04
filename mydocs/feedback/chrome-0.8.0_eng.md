rhwp is a free and open-source extension that lets you open, edit, and print HWP/HWPX documents directly in your browser. No separate software installation required.

Key Features:

Auto-open HWP/HWPX files in the viewer when downloading from the web
Document editing: text input/modification, table editing, formatting
Printing: Ctrl+P for print preview, save as PDF or send to printer
Save edited documents as HWP files
Open files via drag & drop (with a confirmation step)
Auto-detect HWP links on web pages and display an icon badge
Document info preview card on mouse hover
Right-click menu: "Open with rhwp"

Privacy:

All processing happens in the browser via WebAssembly (WASM)
Files are never sent to any external server
No ads, no tracking, no sign-up required
We do not collect any personal information

[v0.8.0 Changes — 2026-07-26]

■ v0.8.0 (2026-07-26) Highlights

Starting with this release, the extension version matches the document engine version (which is why 0.2.8 is followed by 0.8.0).

[Major document engine update (v0.8.0)]
• Save reliability overhaul: fixed a large batch of issues where edits (form values, bookmarks, click-here fields, styles, table borders, equation properties, and more) were lost or reverted on save.
• HWPX save compatibility: dozens of attributes — vertical writing, table overlap, 3D/shadow borders, arrowhead styles, checkbox states, and more — are now preserved across save.
• Undo stability: footnote/endnote/equation insertion, header/footer operations, and table-cell paragraph merges now undo precisely.
• Rendering accuracy: unified font handling and improved table/footnote/object placement; verified zero page-count regressions across a 10,000-document real-world corpus.
• Hardened parsers against corrupt or malicious document files.
• Improved editing responsiveness and Korean input latency.

[Extension-side changes]
• Hardened settings persistence/recovery and download auto-open behavior.
• No new permissions
• No new external network endpoints

[Full changelog]
https://github.com/edwardkim/rhwp/releases

[Source code]
https://github.com/edwardkim/rhwp
