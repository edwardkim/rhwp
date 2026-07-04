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

[v0.2.6 Changes — 2026-06-23]

■ v0.2.6 (2026-06-23) Highlights

This update bundles rhwp core v0.7.17, brings the viewer in line with the Content Security Policy, and fixes a download-handling conflict with other extensions.

[Security / Stability]
• Moved the viewer's inline script out of the page so it complies with the Content Security Policy (CSP) — resolves cases where the viewer failed to open in some environments.
• Reworked download handling so it no longer interferes with other extensions' downloads (subfolder/filename choices).
• Added the missing dark-mode icon asset.
• No new permissions
• No new external network endpoints

[rhwp core v0.7.17]
• Rendering support for some OOXML charts (3D bar / 3D pie / ofPie) + stacked/percent bar alignment
• Fixed missing shape description (shapeComment) serialization; improved HWPX save fidelity
• Regression fix so table height is preserved when adding/removing table rows/columns
• Autosave & recovery for unsaved documents, local-font consent, picture/cursor editing fidelity

[Known Limitations]
• Direct save back to the original HWPX format remains limited as a beta feature
• Complex HWPX roundtrip visual fidelity will continue to improve in later releases

[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp
