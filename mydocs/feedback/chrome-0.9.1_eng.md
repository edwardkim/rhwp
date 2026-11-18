# Chrome Web Store — 0.9.1 (English)

Status: pre-submission candidate. Final artifacts, source SHA and SHA-256 are recorded after packaging.

- Includes the 0.9.1 layout, font, round-trip and small-viewport dialog improvements.
- Documents are processed locally with browser WASM. Existing URL/host access supports HWP/HWPX link detection, previews, context menus and local file opening.
- Actual manifests match 0.9.0 in every field except version. No new permissions, host access or CSP changes. Packaged external endpoints and runtime behavior are checked separately.
- Chrome and Edge share one ZIP. Firefox also requires a reproducible source ZIP.
- PDF import follows the CLI/WASM/Studio contract; the extension does not intercept every web PDF.
- Store approval and public installation verification remain pending.

[Release notes and credits](../release/release_notes_v0.9.1_EN.md)
