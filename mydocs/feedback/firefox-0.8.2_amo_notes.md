--- Firefox Add-ons / AMO — Reviewer notes (v0.8.2) ---

# What the extension does

rhwp opens HWP/HWPX documents directly in Firefox. The parser, renderer, editor, and save/export paths run locally in WebAssembly. The extension does not upload documents, does not call analytics services, and does not collect personal data.

# Build artifacts

- Extension package: `rhwp-firefox-0.8.2.zip`
- Source package for AMO review: `rhwp-source-0.8.2-amo.zip`

AMO source uploads are limited to 200 MB. Do not upload a full-repository archive
because large fixtures in `samples/` and `pdf-large/` exceed that limit. The review
source package is a filtered Git archive containing only the Firefox extension,
rhwp-studio viewer source, Rust/WASM source, shared extension code, fonts, and build
scripts needed to reproduce the submitted extension:

```bash
git archive --format=zip --prefix=rhwp-source/ --output=rhwp-firefox/rhwp-source-0.8.2-amo.zip HEAD Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml Dockerfile docker-compose.yml .env.docker.example LICENSE README.md README_EN.md CHANGELOG.md CHANGELOG_EN.md THIRD_PARTY_LICENSES.md src rhwp-studio rhwp-firefox rhwp-shared assets/fonts scripts npm/README.md npm/editor
zip -d rhwp-firefox/rhwp-source-0.8.2-amo.zip "rhwp-source/rhwp-studio/public/samples/*"
```

The generated archive excludes top-level `samples/`, `pdf-large/`, `output/`, `target/`,
`node_modules/`, extension `dist/` output, and the bundled `rhwp-studio/public/samples/`
demo documents. `Cargo.lock` is included for a reproducible build.

# Permissions justification

Unchanged from v0.8.1.

- activeTab: open the viewer tab from a user action.
- downloads: open HWP/HWPX downloads in the viewer.
- contextMenus: add "Open with rhwp".
- clipboardWrite: copy selected document text.
- storage: store user preferences only.
- host_permissions <all_urls>: HWP/HWPX links may appear on any domain. Detection is performed locally and is not used for tracking.

# Changes in v0.8.2

Hotfix release. Bundled WebAssembly document engine updated v0.8.1 → v0.8.2.

- **Printing restored**: Ctrl+P failed with "file not found" because the print preview page (`print.html`) was not included in the extension package. It was omitted from the build's asset copy list when the print path was introduced. Printing and Save as PDF now work.
- Build hardening: added a required-artifact check so a missing runtime file fails the build instead of only emitting a warning.
- Rendering: left/right outer margins are now applied to the x-origin of inline tables.
- No dependency changes since v0.8.1.
- All processing remains local (WebAssembly). No document upload.

**No new permissions and no new external network endpoints were added.**

# Security notes

The extension uses bundled WebAssembly generated from Rust. No remote JavaScript is loaded.
The CSP contains `wasm-unsafe-eval` only for WebAssembly execution.

`browser_specific_settings.gecko.data_collection_permissions.required` is set to `["none"]`.

Source code: https://github.com/edwardkim/rhwp
