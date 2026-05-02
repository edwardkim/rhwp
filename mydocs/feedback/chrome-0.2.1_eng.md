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

[v0.2.1 Changes — 2026-04-19]

■ v0.2.1 (2026-04-19) Highlights

[Bug Fixes]
• Restored "remember last save location" for general file downloads —
  no longer defaults to Desktop when the extension is active
  (chrome-fd-001 user report, #198)
• Options page now works correctly (CSP fix, #166)
• Blocked empty viewer tabs on certain Korean government download
  handlers (DEXT5-style, #198)
• Fixed Korean file path handling on Windows
  (external contribution by @dreamworker0, PR #152)
• Mobile dropdown menu icon/label overlap fix
  (external contribution by @seunghan91, PR #161)
• Thumbnail loading spinner cleanup + options CSP compatibility
  (external contribution by @postmelee, PR #168)

[Improvements]
• HWP files: Ctrl+S now overwrites the same file directly — no save
  dialog every time (external contribution by @ahnbu, PR #189)
• Rotated shape resize cursor + Flip handling improved
  (external contribution by @bapdodi, PR #192)
• HWPX files now show a beta notice with direct save disabled
  (prevents data loss, #196)
• HWPX Serializer — Document IR → HWPX save
  (external contribution by @seunghan91, PR #170)
• HWP image effects (grayscale / black-and-white) reflected more
  accurately in SVG (external contribution by @marsimon, PR #149)
• HWPX ZIP entry decompression cap + strikeout shape whitelist
  + shape resize clamp
  (external contribution by @seunghan91, PR #153, #154, #163)
• About dialog version display fix

[Known Limitations]
• Direct HWPX save is in beta and disabled for now
  (until the HWPX→HWP full converter #197 lands)
• If the print preview window appears too large, press Ctrl+0 to
  reset zoom (#199)

[Thanks to Contributors]
@ahnbu (PR #189)
@bapdodi (PR #192)
@dreamworker0 (PR #152)
@marsimon (PR #149)
@postmelee (PR #168)
@seunghan91 (PR #149, #153, #154, #161, #163, #170)


■ v0.1.0 (previous)

• Initial public beta


[Full Changelog]
https://github.com/edwardkim/rhwp/releases

[Source Code]
https://github.com/edwardkim/rhwp