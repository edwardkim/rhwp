--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.2.6) ---

# What it does

rhwp opens HWP/HWPX (Hancom Hangul) documents in the browser. Processing runs locally in WebAssembly. Documents are not uploaded. No analytics, tracking, or sign-up.

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

# Changes in v0.2.6

- Security/Stability: moved the viewer's inline script into a separate file so the viewer page complies with the Content Security Policy (`script-src 'self' 'wasm-unsafe-eval'`). This removes an inline-script CSP violation; it does not add any capability.
- Download handling: replaced the `downloads.onDeterminingFilename` listener with `downloads.onCreated`/`onChanged` observers. The previous listener registration had a global side effect that could disrupt other extensions' `downloads.download({ filename })` subfolder/filename choices. rhwp no longer participates in the filename-determination step; it only observes downloads to offer opening HWP/HWPX in the viewer.
- Bundled rhwp core updated to library v0.7.17 (OOXML chart routing, shapeComment serialization, table row/column edit regression fix, rendering/table/picture fixes). Document processing still runs entirely in local WebAssembly.

**No new permissions and no new external network endpoints were added.**

# WASM safety

All JavaScript and WebAssembly are bundled. No remote code is loaded. CSP uses `wasm-unsafe-eval` only for browser WebAssembly execution.

Source code: https://github.com/edwardkim/rhwp
