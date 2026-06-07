rhwp is a free and open-source extension that lets you open, edit, and print HWP/HWPX documents directly in your browser. No separate software installation required.

Key Features:

Auto-open HWP/HWPX files in the viewer when downloading from the web
Document editing: text input/modification, table editing, formatting
Printing: Ctrl+P for print preview, save as PDF or send to printer
Save edited documents as HWP files
Open files via drag & drop
Auto-detect HWP links on web pages and display an icon badge
Document info preview card on mouse hover
Right-click menu: "Open with rhwp"

Privacy:

All processing happens in the browser via WebAssembly (WASM)
Files are never sent to any external server
No ads, no tracking, no sign-up required
We do not collect any personal information

[v0.2.4 Changes — 2026-06-06]

■ v0.2.4 (2026-06-06) Highlights

This update bundles rhwp core v0.7.15 and hardens the browser extension document-fetch path.

[Security Hardening]
• Added sender validation for HWP/HWPX document fetch requests handled by the service worker
• Blocks localhost, loopback, private-network, link-local, and internal-host URLs
• Revalidates the final URL after redirects with the same policy
• Uses credentials: "omit" for extension-side fetches
• Keeps automatically extracted thumbnail data out of the page DOM
• No new permissions
• No new external network endpoints

[rhwp core v0.7.15]
• Improved wrapping and paragraph-indent handling for equation TAC-only lines
• Improved caret movement after forced line breaks and in endnote areas
• Fixed HWPX picture serialization for flip/rotation and isEmbeded output
• Preserved HWPX diagonal cell-border slash/backSlash type values
• Preserved zero-length HWPX field ordering

[Known Limitations]
• Direct save back to the original HWPX format remains limited as a beta feature
• Complex HWPX roundtrip visual fidelity will continue to improve in later releases

[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp
