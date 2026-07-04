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

[v0.2.5 Changes — 2026-06-19]

■ v0.2.5 (2026-06-19) Highlights

This update bundles rhwp core v0.7.16 and adds a security confirmation step to local drag-and-drop file handling.

[Security / UX]
• Drag-and-drop local file loading is no longer automatic. Dropping a file now shows a confirmation dialog first, and the file loads only after you click "Open" (explicit opt-in).
• Dark theme support and UI contrast cleanup
• No new permissions
• No new external network endpoints

[rhwp core v0.7.16]
• Fixed the ClickHere (click-to-type) guide-text save format so it binds correctly in the Hancom editor
• Improved HWPX save fidelity: preserves cell/text-box controls, captions, picture sizes, table page breaks, and more
• Improved rendering / table / picture placement alignment and endnote flow

[Known Limitations]
• Direct save back to the original HWPX format remains limited as a beta feature
• Complex HWPX roundtrip visual fidelity will continue to improve in later releases

[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp
