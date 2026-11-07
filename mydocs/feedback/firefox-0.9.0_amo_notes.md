# Firefox AMO reviewer notes — v0.9.0 preparation

rhwp is an HWP/HWPX document viewer and editor. This updates the existing add-on
`rhwp-firefox@edwardkim.github.io`; the manifest requires Firefox 142 or later.
These are submission notes for the 0.9.0 candidate. Store publication is still pending.

## Submitted files and source provenance

- Extension: `rhwp-firefox-0.9.0.zip`
- Sources: `rhwp-source-0.9.0-amo.zip`

Record the final release source commit, file sizes and SHA-256 values for both uploads.
The source ZIP includes root `build.rs`, Cargo workspace members, vendored dependencies,
embedded production assets, Firefox/Studio/shared browser sources, fonts and notices,
lockfiles and build scripts. It excludes the large test corpus, credentials, generated
packages, `target`, `node_modules` and extension `dist`. Preserve source symlinks when extracting;
the extension build dereferences its shared service-worker files.

The release preparation already rebuilt the filtered source ZIP and exercised Firefox download,
editing and saving. Those results describe the recorded preparation source. Rebuild and verify
the final release source before submitting its archives; do not reuse an older package solely
because its manifest also says 0.9.0.

## Rebuild

Use Linux, Rust 1.93.1 from `rust-toolchain.toml`, Node.js 22 or later, npm 10 or later,
and wasm-pack 0.15.0. Extract with `unzip`, then run from `rhwp-source/`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
npm --prefix rhwp-studio ci --no-audit
npm --prefix rhwp-firefox ci --no-audit
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
npm --prefix rhwp-firefox run build
release_out="$PWD/output/release-v0.9.0/packages"
mkdir -p "$release_out"
(cd rhwp-firefox/dist && zip -qr "${release_out}/rhwp-firefox-0.9.0.zip" .)
```

The output directory is `rhwp-firefox/dist`. No prebuilt RHWP WASM is required in the source upload.
Font licenses and third-party notices accompany the bundled assets. ZIP timestamps/compression may
differ; inspect extracted files and record the actual toolchain rather than claiming identical archive bytes.

## Reviewer journey

1. Open an HWP/HWPX sample link from the repository's `samples` directory and check the viewer.
2. If font consent is requested, choose the available fallback fonts or supply fonts you may use.
   The dialog is an expected user choice, not a loading failure.
3. Check zoom, text editing, printing and saving. Reopen the saved document and inspect the edit.
4. Use “Open with rhwp” on a document link, then open a local HWP/HWPX file in the viewer.
5. Move a picture, use Undo/Redo and check the saved document. Check continuous highlighting as well.
6. Confirm edited downloads do not create duplicate viewer tabs. A downloaded `.xlsx` must remain
   an ordinary download even when its source URL ends in `.hwp`.

## Changes in 0.9.0

- Nested and continued tables, cell pictures, text boxes, headers/footers, equations and font fallback.
- Picture insertion and movement, continuous highlighting, paragraph formatting and round-trip preservation.
- Memo APIs and preservation of author/creation time; a new memo editing UI is not part of this claim.
- The shared engine adds PDF import, opening-password authentication and page progress. CLI/Studio
  integration is documented in the developer guide. Automatic web download/link handling remains
  focused on HWP/HWPX; this is not a request to intercept arbitrary PDF downloads.
- The same engine starts crates.io distribution with 0.9.0 for Rust and CLI developers. This is a
  separate package channel within the release, not an additional browser permission.

## Permissions and processing

The source manifest's permissions, host permissions, CSP and Firefox minimum version are unchanged
from v0.8.7. Inspect the final packaged manifest before submission.

- `activeTab`: open the viewer from a user action.
- `downloads`: observe document downloads and open validated HWP/HWPX candidates.
- `contextMenus`: offer “Open with rhwp” for document links.
- `clipboardWrite`: copy selected document text.
- `storage`: store local preferences.
- `<all_urls>` host permission and content-script matches: document links occur on arbitrary sites;
  link inspection supports badges, previews and the open action.

Document parsing, rendering, editing and export run in bundled JavaScript and WebAssembly.
Document bytes and opening passwords are not uploaded for conversion. `wasm-unsafe-eval` permits
the bundled WebAssembly; `data_collection_permissions.required` remains `["none"]`.

[PDF developer guide](../manual/pdf_developer_onboarding_EN.md) ·
[Release notes and contributor credits](../release/release_notes_v0.9.0_EN.md) ·
[Packaging and submission procedure](../manual/publish_guide.md) ·
[Privacy policy](https://edwardkim.github.io/rhwp/privacy.html)
