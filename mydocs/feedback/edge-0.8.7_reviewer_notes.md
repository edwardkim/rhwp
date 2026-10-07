# Edge Add-ons certification notes — v0.8.7

rhwp is a free HWP/HWPX document viewer and editor. The Edge package is a byte-for-byte copy of
`rhwp-chrome-0.8.7.zip`, named `rhwp-edge-0.8.7.zip`. It updates the existing store listing.

## How to test

1. Open an HWP/HWPX link from https://github.com/edwardkim/rhwp/tree/main/samples.
2. Check the automatically opened viewer, zoom, editing, printing, and saving.
3. Right-click a document link and select “Open with rhwp”.
4. Drag a local HWP/HWPX file into the viewer.
5. Save an edited document and confirm that automatic download handling does not create duplicate viewer tabs.
6. As a negative control, a downloaded `.xlsx` file must remain a normal browser download even if its URL ends in `.hwp`.

## Permissions and host justification

No new permissions: the v0.8.7 manifest differs from v0.8.6 only in its version.

- `activeTab`: opens the viewer from a user action.
- `downloads`: observes document downloads and opens validated HWP/HWPX candidates.
- `contextMenus`: provides “Open with rhwp” for document links.
- `clipboardWrite`: copies selected document text.
- `storage`: stores local preferences.
- `<all_urls>` host permission and content-script matches: HWP/HWPX links occur on arbitrary
  domains. Local link inspection provides badges and previews and does not collect browsing data.

No new external network endpoints were added to the extension download/link-handling code.
Document parsing, rendering, editing, and export run locally in bundled JavaScript and WebAssembly.
Documents and passwords are not uploaded. There is no analytics or tracking. The extension loads
no remote JavaScript. `wasm-unsafe-eval` is used for bundled WebAssembly.

## Changes in v0.8.7

- Corrected inline/nested table layout and page continuation.
- Improved English UI, hyperlink editing/preservation, search counts, and table editing.
- Strengthened document input boundaries and restored compatible thumbnail decompression.
- Corrected download completion and reopening of saved documents.

Source: https://github.com/edwardkim/rhwp
