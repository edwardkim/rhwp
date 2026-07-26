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

[v0.8.1 Changes — 2026-07-26]

▣ v0.8.1 (2026-07-26) Highlights

A patch release focused on document rendering accuracy.

[Rendering accuracy]
• WordArt (decorative title text) in legacy HWP 3.0 documents now displays correctly. Previously an image embedded inside the document was misidentified as an external file and left blank.
• Fixed holes inside shapes being filled in instead of left open.
• Fixed paragraph borders set to "no line" being drawn as solid lines. Side rules on shaded boxes are corrected as well.

[Editing]
• Style creation, modification, and deletion can now be undone with Ctrl+Z. Previously these actions were not recorded in the undo history.

[Extension changes]
• Fixed unnecessary error messages appearing in the console.
• No new permissions
• No new external network endpoints

[Full changelog]
https://github.com/edwardkim/rhwp/releases

[Source code]
https://github.com/edwardkim/rhwp
