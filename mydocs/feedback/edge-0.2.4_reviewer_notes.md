--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.2.4) ---

# What it does

rhwp opens HWP/HWPX (Hancom Hangul) documents in the browser. Processing runs locally in WebAssembly. Documents are not uploaded. No analytics, tracking, or sign-up.

# How to test

1. Install the extension.
2. Open https://github.com/edwardkim/rhwp/tree/main/samples and click any *.hwp or *.hwpx link.
3. The document opens in the rhwp viewer tab.
4. Try zoom, page navigation, edit, Ctrl+P print, and save as HWP.
5. Right-click an HWP/HWPX link → "Open with rhwp".

# Permissions / host justification

- activeTab: opens the viewer tab from a user action.
- downloads: opens HWP/HWPX downloads in the viewer.
- contextMenus: adds "Open with rhwp".
- clipboardWrite: copies selected document text.
- storage: stores user preferences only.
- host_permissions `<all_urls>` and content_scripts `matches: ["<all_urls>"]`: HWP/HWPX links can appear on arbitrary sites, including public-sector portals with unpredictable download URLs. The content script only inspects anchor/link metadata locally to detect HWP/HWPX candidates and add a badge/hover card. It does not read document contents, collect page data, or track browsing.

# Security changes in v0.2.4

This release hardens extension-side document fetch paths:

- service worker sender validation for document fetch requests
- localhost, loopback, private-network, link-local, and internal-host URLs are blocked
- redirect final URLs are revalidated with the same policy
- fetch uses `credentials: "omit"`
- automatically extracted thumbnail data is kept out of the page DOM

No new permissions and no new external network endpoints were added.

# WASM safety

All JavaScript and WebAssembly are bundled. No remote code is loaded. CSP uses `wasm-unsafe-eval` only for browser WebAssembly execution.

Source code: https://github.com/edwardkim/rhwp
License: MIT

# Data collection

None: no personal data, analytics, or fingerprinting.
Privacy policy: https://github.com/edwardkim/rhwp/blob/main/rhwp-chrome/PRIVACY.md

# v0.2.4 highlights

Core updated to v0.7.15. This release hardens service-worker document fetch security and includes equation TAC flow/caret movement fixes plus HWPX save-contract follow-ups. No new permissions or network endpoints.
