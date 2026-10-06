# Firefox AMO reviewer notes — v0.8.7

rhwp is a free HWP/HWPX document viewer and editor. This updates the existing add-on
`rhwp-firefox@edwardkim.github.io`; Firefox 142 or later is required.

## Submitted files

- Extension: `rhwp-firefox-0.8.7.zip`
- Sources: `rhwp-source-0.8.7-amo.zip` (filtered tracked Git tree, less than 200 MB)

The release record identifies the exact source commit and SHA-256 of both files. Sources include
root `build.rs`, all Cargo workspace members, the vendored dependency, embedded production assets,
Firefox/Studio/shared browser source, fonts/licenses, lockfiles, and build scripts. Large corpus,
output, generated packages, local credentials, `target`, `node_modules`, and extension `dist` are excluded.
Source symlinks are preserved; the extension build dereferences the shared service-worker files.

## Rebuild

Use Linux, the Rust toolchain pinned in `rust-toolchain.toml`, Node.js 22 or later, npm 10 or later,
and wasm-pack 0.15.0. Extract the source ZIP, then run from `rhwp-source/`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
npm --prefix rhwp-studio ci --no-audit
npm --prefix rhwp-firefox ci --no-audit
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
npm --prefix rhwp-firefox run build
cd rhwp-firefox/dist
zip -r ../rhwp-firefox-0.8.7.zip .
```

Use `unzip` to preserve archive symlinks. The bundled NotoSansKR TTF and its OFL notice are included.
The output folder is `rhwp-firefox/dist`. Different ZIP timestamps/compression may change archive
bytes; compare extracted files with the submitted package. The release record lists local tool versions
and the executed source-package rebuild. No prebuilt WASM is required in the source upload.

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

`browser_specific_settings.gecko.data_collection_permissions.required` is `["none"]`.
Source: https://github.com/edwardkim/rhwp
