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

[v0.8.2 Changes — 2026-07-27]

▣ v0.8.2 (2026-07-27) Highlights

An urgent fix release that restores printing.

[Printing restored]
• Fixed the "file not found" error when printing with Ctrl+P. The print preview page was not included in the extension package; printing and Save as PDF now work correctly.

[Rendering accuracy]
• Fixed left/right outer margins not being applied to tables placed inline within text.

[Extension changes]
• No new permissions
• No new external network endpoints

[Full changelog]
https://github.com/edwardkim/rhwp/releases

[Source code]
https://github.com/edwardkim/rhwp
