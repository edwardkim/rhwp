--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.8.0) ---

# What it does

rhwp opens HWP/HWPX (Hancom Hangul) documents in the browser. Processing runs locally in WebAssembly. Documents are not uploaded. No analytics, tracking, or sign-up.

# Version numbering note

Starting with this release, the extension version is unified with the bundled document-engine version. The previous store version was 0.2.8; this submission is 0.8.0. The version number remains strictly increasing; this is a numbering-scheme change only.

# How to test

1. Install the extension.
2. Open https://github.com/edwardkim/rhwp/tree/main/samples and click any *.hwp or *.hwpx link.
3. The document opens in the rhwp viewer tab.
4. Try zoom, page navigation, edit, Ctrl+P print, and save as HWP.
5. Right-click an HWP/HWPX link → "Open with rhwp".
6. Drag a local .hwp/.hwpx file into the viewer — a confirmation dialog appears first; the file loads only after you click "열기 (Open)".

# Permissions / host justification

- activeTab: opens the viewer tab from a user action.
- downloads: opens HWP/HWPX downloads in the viewer.
- contextMenus: adds "Open with rhwp".
- clipboardWrite: copies selected document text.
- storage: stores user preferences only.
- host_permissions `<all_urls>` and content_scripts `matches: ["<all_urls>"]`: HWP/HWPX links can appear on arbitrary sites, including public-sector portals with unpredictable download URLs. The content script only inspects anchor/link metadata locally to detect HWP/HWPX candidates and add a badge/hover card. It does not read document contents, collect page data, or track browsing.

# Changes in v0.8.0

Major update of the bundled WebAssembly document engine (v0.7.19 → v0.8.0), plus extension-side hardening.

- Engine: save-reliability overhaul (edits such as form values, bookmarks, styles, and table borders no longer revert on save), dozens of HWPX attribute round-trip preservations, precise undo for footnote/equation/header-footer/table-cell operations, unified font handling with rendering-accuracy verification over a 10,000-document corpus, parser hardening against corrupt or malicious files, and improved editing responsiveness.
- Extension: hardened settings persistence/recovery and download auto-open behavior.
- All processing remains local (WebAssembly). No document upload.

**No new permissions and no new external network endpoints were added.**

# WASM safety

All JavaScript and WebAssembly are bundled. No remote code is loaded. CSP uses `wasm-unsafe-eval` only for browser WebAssembly execution.

Source code: https://github.com/edwardkim/rhwp
