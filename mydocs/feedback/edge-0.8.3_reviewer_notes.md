--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.8.3) ---

# What it does

rhwp opens HWP/HWPX (Hancom Hangul) documents in the browser. Processing runs locally in WebAssembly. Documents are not uploaded. No analytics, tracking, or sign-up.

# How to test

1. Install the extension.
2. Open https://github.com/edwardkim/rhwp/tree/main/samples and click any *.hwp or *.hwpx link.
3. The document opens in the rhwp viewer tab.
4. Try zoom, page navigation, edit, Ctrl+P print, and save as HWP.
5. Right-click an HWP/HWPX link → "Open with rhwp".
6. Drag a local .hwp/.hwpx file into the viewer — a confirmation dialog appears first; the file loads only after you click "열기 (Open)".
7. Open a supported encrypted HWP/HWPX file. A password prompt appears; the document is processed locally after successful entry.

# Permissions / host justification

Unchanged from v0.8.2.

- activeTab: opens the viewer tab from a user action.
- downloads: opens HWP/HWPX downloads in the viewer.
- contextMenus: adds "Open with rhwp".
- clipboardWrite: copies selected document text.
- storage: stores user preferences only.
- host_permissions `<all_urls>` and content_scripts `matches: ["<all_urls>"]`: HWP/HWPX links can appear on arbitrary sites, including public-sector portals with unpredictable download URLs. The content script only inspects anchor/link metadata locally to detect HWP/HWPX candidates and add a badge/hover card. It does not read document contents, collect page data, or track browsing.

# Changes in v0.8.3

The bundled WebAssembly document engine was updated from v0.8.2 to v0.8.3.

- Encrypted documents: added a local password-entry flow for supported encrypted HWP/HWPX files and expanded encrypted save/reopen compatibility.
- Nested tables: improved page splitting, empty cell paragraphs, continued child-table flow, and final bottom borders.
- Special characters: restored square-number and small right-triangle glyphs that could appear as tofu.
- Editing: fixed text/table selection and copy paths inside nested tables.
- Performance: reduced redundant full rerenders for large tables and post-edit updates.
- All processing remains local in WebAssembly. Documents and passwords are not uploaded.

**No new permissions and no new external network endpoints were added.** The v0.8.2 and v0.8.3 extension manifests have identical permission, host-permission, and content-script declarations.

# WASM safety

All JavaScript and WebAssembly are bundled. No remote code is loaded. CSP uses `wasm-unsafe-eval` only for browser WebAssembly execution.

Source code: https://github.com/edwardkim/rhwp
