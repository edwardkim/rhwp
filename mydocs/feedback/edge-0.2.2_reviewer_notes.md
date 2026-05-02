--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.2.2) ---

Length: 1,996 / 2,000 characters

---

# What the extension does

rhwp opens HWP / HWPX (Hancom Hangul) documents directly in the browser. All processing runs in WebAssembly, on-device. No server upload, no telemetry, no sign-up.

# How to test (5 minutes)

1. Install and open https://github.com/edwardkim/rhwp/tree/main/samples
2. Click any *.hwp link — viewer opens in a new tab.
3. Try the toolbar: zoom, page navigation, edit text, save as HWP, print preview.
4. Right-click any HWP link on the web → "Open with rhwp".

Sample files are MIT-licensed (in the same repo). Recommended: KTX.hwp, biz_plan.hwp.

# Permissions justification

- activeTab: open the viewer tab when the user clicks an HWP link.
- downloads: intercept HWP file downloads to open them in the viewer instead of the OS shell. Restored "remember last save location" for non-HWP downloads (#198).
- contextMenus: "Open with rhwp" right-click item.
- clipboardWrite: copy selected text from the viewer.
- storage: user preferences (zoom level, last save location, auto-open toggle).
- host_permissions <all_urls>: required to detect HWP links on arbitrary websites and intercept their download requests. Not used for tracking.

# WASM safety

- Compiled from Rust, sandboxed by the browser.
- No `eval` / no remote code. CSP: "script-src 'self' 'wasm-unsafe-eval'".
- Source: https://github.com/edwardkim/rhwp (MIT)
- Submitted commit SHA: see manifest.json `version` 0.2.2.

# Data collection

None. No analytics, no fingerprinting. Privacy policy: https://github.com/edwardkim/rhwp/blob/main/rhwp-chrome/PRIVACY.md

# v0.2.2 highlights (vs v0.2.1)

Library core upgrade v0.7.3 → v0.7.9 (4 cycles). Typesetting fixes (multi-column accumulation, table row-split, heading-orphan, cell padding edge case), equation renderer (italic, ATOP semantics), Picture serialization in groups, PageLayerTree API, automated Canvas pixel-diff CI. No new permissions, no new network endpoints.
