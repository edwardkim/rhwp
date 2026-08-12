rhwp is a free and open-source extension that lets you open, edit, and print HWP/HWPX documents directly in your browser. No separate software installation is required.

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

Notice
This product was developed with reference to the HWP (.hwp) file format specification published by Hancom Inc.

Trademark
"Hangul", "Hancom", "HWP", and "HWPX" are registered trademarks of Hancom Inc. This package is an independent open-source project with no affiliation, sponsorship, or endorsement by Hancom Inc.

[v0.8.4 Changes — 2026-08-12]

▣ v0.8.4 (2026-08-12) Highlights

A cumulative patch release improving encrypted documents, nested tables, special-character rendering, and editing performance.

[Open and save]
• Added password entry when opening supported encrypted HWP/HWPX documents.
• Expanded preservation of document properties that could previously be lost or corrupted during HWP/HWPX save operations.

[Rendering accuracy]
• Improved nested tables spanning pages, including cell content, empty lines, continued tables, and final borders.
• Fixed Hangul-specific square-number and small right-triangle characters appearing as tofu glyphs.

[Editing and performance]
• Fixed text and table selection/copy behavior inside nested tables.
• Reduced unnecessary full rerenders while navigating or editing large tables.

[Extension changes]
• No new permissions compared with v0.8.2
• No new external network endpoints

[Full changelog]
https://github.com/edwardkim/rhwp/releases

[Detailed changes and contributors]
https://github.com/edwardkim/rhwp/blob/main/CHANGELOG.md

[Source code]
https://github.com/edwardkim/rhwp
