--- Firefox Add-ons / AMO — Reviewer notes (v0.8.6) ---

# What the extension does

rhwp opens HWP/HWPX documents directly in Firefox. Parsing, rendering, editing, and save/export run locally in
WebAssembly. The extension does not upload documents or passwords, call analytics services, or collect personal
data.

# Build artifacts

- Extension package: `rhwp-firefox-0.8.6.zip`
- Source package: `rhwp-source-0.8.6-amo.zip`

The AMO source package is a filtered Git archive containing the Firefox extension, Studio source, Rust/WASM
source, embedded production resources, workspace manifests, fonts, and build scripts needed to reproduce the
submission. It excludes top-level large fixtures and generated/local directories.

```bash
git archive --format=zip --prefix=rhwp-source/ --output=rhwp-firefox/rhwp-source-0.8.6-amo.zip HEAD Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml Dockerfile docker-compose.yml .env.docker.example LICENSE README.md README_EN.md CHANGELOG.md CHANGELOG_EN.md THIRD_PARTY_LICENSES.md llms.txt src rhwp-studio rhwp-firefox rhwp-shared assets/fonts assets/logo/logo-32.png saved/blank2010.hwp ttfs/opensource/NotoSansKR-Regular.ttf scripts npm/README.md npm/editor bindings/Native tools/rhwp-subsecond tools/batch-convert mydocs/manual/agent_knowledge_map.md mydocs/manual/agent_troubleshooting_guide.md mydocs/manual/recipes
zip -d rhwp-firefox/rhwp-source-0.8.6-amo.zip "rhwp-source/rhwp-studio/public/samples/*"
```

Do not upload a full-repository archive. The filtered archive excludes top-level `samples/`, `pdf-large/`,
`output/`, `target/`, `node_modules/`, extension `dist/`, and bundled Studio demo documents.

# Permissions justification

Permissions are unchanged from v0.8.4.

- `activeTab`: opens the viewer from a user action.
- `downloads`: observes HWP/HWPX downloads and opens validated candidates.
- `contextMenus`: adds "Open with rhwp".
- `clipboardWrite`: copies selected document text.
- `storage`: stores local preferences only.
- `<all_urls>`: HWP/HWPX links may occur on any domain; local link detection is not used for tracking.

# Changes in v0.8.6

- Download interception waits for finalized filename/MIME evidence and rejects `.xlsx` responses even when a
  provisional URL resembles HWP.
- Preview-image decompression enforces a bounded output and exact declared length.
- The bundled v0.8.6 WebAssembly engine improves typesetting, font handling, save preservation, and editing.

**No new permissions and no new external network endpoints were added.** No remote JavaScript is loaded. CSP
contains `wasm-unsafe-eval` only for WebAssembly execution.

`browser_specific_settings.gecko.data_collection_permissions.required` is set to `["none"]`.

Source code: https://github.com/edwardkim/rhwp
