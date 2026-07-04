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

[v0.2.8 Changes — 2026-06-24]

■ v0.2.8 (2026-06-24) Highlights

Follow-up hotfix for v0.2.7.

[Bug Fix]
• Fixed an additional case where previously completed download records could be delivered as if they were new downloads after Chrome restart or extension service worker restart, causing HWP/HWPX viewer tabs to open again.
• The extension now checks download start/end timestamps and ignores old download history entries that had already completed before the extension started.
• Normal auto-open behavior for newly started HWP/HWPX downloads is preserved.
• No new permissions
• No new external network endpoints

[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp
