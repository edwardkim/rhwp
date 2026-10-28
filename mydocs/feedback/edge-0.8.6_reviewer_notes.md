--- Edge Add-ons / Microsoft Partner Center — Notes for certification (v0.8.6) ---

# What it does

rhwp opens HWP/HWPX documents in the browser. Parsing, rendering, editing, and save/export run locally in
WebAssembly. Documents are not uploaded. The extension has no analytics, tracking, or sign-up.

# How to test

1. Install the extension.
2. Open https://github.com/edwardkim/rhwp/tree/main/samples and click an `.hwp` or `.hwpx` link.
3. Confirm that it opens in the rhwp viewer, then try zoom, editing, printing, and saving.
4. Right-click an HWP/HWPX link and select "Open with rhwp".
5. Drag a local HWP/HWPX file into the viewer and confirm the file-open prompt.
6. As a negative case, download an `.xlsx` response from a URL whose path ends in `.hwp`; v0.8.6 must leave
   the spreadsheet to the browser instead of opening the rhwp viewer.

# Permissions and host justification

Permissions are unchanged from v0.8.4.

- `activeTab`: opens the viewer from a user action.
- `downloads`: observes HWP/HWPX downloads and opens validated candidates in the viewer.
- `contextMenus`: adds "Open with rhwp".
- `clipboardWrite`: copies selected document text.
- `storage`: stores local user preferences only.
- `<all_urls>` host permission and content-script match: public-sector and other HWP/HWPX links may occur on
  arbitrary domains. The content script inspects link metadata locally for badges and preview cards. It does
  not collect page data or track browsing.

# Changes in v0.8.6

- Download interception now waits for finalized filename/MIME evidence and rejects `.xlsx` even when a
  provisional URL resembles HWP.
- Preview-image decompression now enforces a bounded output and exact declared length.
- The bundled v0.8.6 WebAssembly engine improves typesetting, font handling, save preservation, and editing.

**No new permissions and no new external network endpoints were added.** Processing remains local in bundled
JavaScript and WebAssembly; no remote code is loaded. CSP uses `wasm-unsafe-eval` only for WebAssembly.

Source code: https://github.com/edwardkim/rhwp
