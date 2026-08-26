--- Firefox Add-ons / AMO — Reviewer notes (v0.8.4) ---

# What the extension does

rhwp opens HWP/HWPX documents directly in Firefox. The parser, renderer, editor, and save/export paths run locally in WebAssembly. The extension does not upload documents or passwords, does not call analytics services, and does not collect personal data.

# Build artifacts

- Extension package: `rhwp-firefox-0.8.4.zip`
- Source package for AMO review: `rhwp-source-0.8.4-amo.zip`

AMO source uploads are limited to 200 MB. Do not upload a full-repository archive because large fixtures in `samples/` and `pdf-large/` exceed that limit. The review source package is a filtered Git archive containing the Firefox extension, Studio viewer source, Rust/WASM source, embedded runtime resources, workspace manifests, fonts, and build scripts required to reproduce the submitted extension:

```bash
git archive --format=zip --prefix=rhwp-source/ --output=rhwp-firefox/rhwp-source-0.8.4-amo.zip HEAD Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml Dockerfile docker-compose.yml .env.docker.example LICENSE README.md README_EN.md CHANGELOG.md CHANGELOG_EN.md THIRD_PARTY_LICENSES.md llms.txt src rhwp-studio rhwp-firefox rhwp-shared assets/fonts assets/logo/logo-32.png saved/blank2010.hwp ttfs/opensource/NotoSansKR-Regular.ttf scripts npm/README.md npm/editor bindings/Native tools/rhwp-subsecond tools/batch-convert mydocs/manual/agent_knowledge_map.md mydocs/manual/agent_troubleshooting_guide.md mydocs/manual/recipes
zip -d rhwp-firefox/rhwp-source-0.8.4-amo.zip "rhwp-source/rhwp-studio/public/samples/*"
```

The generated archive excludes top-level `samples/`, `pdf-large/`, `output/`, `target/`, `node_modules/`, extension `dist/` output, and bundled Studio demo documents. `Cargo.lock` and every production `include_str!`/`include_bytes!` resource are included for a reproducible build; test-only fixtures are excluded.

# Permissions justification

Unchanged from v0.8.2.

- activeTab: open the viewer tab from a user action.
- downloads: open HWP/HWPX downloads in the viewer.
- contextMenus: add "Open with rhwp".
- clipboardWrite: copy selected document text.
- storage: store user preferences only.
- host_permissions <all_urls>: HWP/HWPX links may appear on any domain. Detection is performed locally and is not used for tracking.

# Changes in v0.8.4

The bundled WebAssembly document engine was updated from v0.8.2 to v0.8.4.

- Encrypted documents: added a local password-entry flow for supported encrypted HWP/HWPX files and expanded encrypted save/reopen compatibility.
- Nested tables: improved page splitting, empty cell paragraphs, continued child-table flow, and final bottom borders.
- Special characters: restored square-number and small right-triangle glyphs that could appear as tofu.
- Editing: fixed text/table selection and copy paths inside nested tables.
- Performance: reduced redundant full rerenders for large tables and post-edit updates.
- All processing remains local in WebAssembly. Documents and passwords are not uploaded.

**No new permissions and no new external network endpoints were added.** The v0.8.2 and v0.8.4 extension manifests have identical permission, host-permission, and content-script declarations.

# Security notes

The extension uses bundled WebAssembly generated from Rust. No remote JavaScript is loaded. The CSP contains `wasm-unsafe-eval` only for WebAssembly execution.

`browser_specific_settings.gecko.data_collection_permissions.required` is set to `["none"]`.

Source code: https://github.com/edwardkim/rhwp
