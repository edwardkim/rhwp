# Chrome Web Store — 0.9.1 (English)

Status: verified package ready for manual maintainer submission.

- Includes the 0.9.1 layout, font, round-trip and small-viewport dialog improvements.
- Documents are processed locally with browser WASM. Existing URL/host access supports HWP/HWPX link detection, previews, context menus and local file opening.
- Actual manifests match 0.9.0 in every field except version. No new permissions, host access or CSP changes. The scoped source review found no new network endpoints. Packaged Chromium E2E passed three suites.
- Chrome and Edge share one ZIP. Firefox also requires a reproducible source ZIP.
- PDF import follows the CLI/WASM/Studio contract; the extension does not intercept every web PDF.
- Store approval and public installation are verified after submission.

[Release notes and credits](../release/release_notes_v0.9.1_EN.md)

[최종 소스·파일별 SHA-256 기록](../pr/assets/release-0.9.1-execution/delivery-evidence.json)
