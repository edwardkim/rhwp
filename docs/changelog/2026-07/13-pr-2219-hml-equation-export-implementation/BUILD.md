# BUILD - PR 2219 HML equation and export

## Outcome

Builder slice is complete in the isolated worktree at base `3f1db67d`. The patch imports evidenced HML equations into `Control::Equation`, materializes intrinsic geometry, exports canonical HML equations, retains unknown semantics as durable blockers, exposes canonical HML save-state and transferable embed export, and adds repo/browser regression proof.

No commit or push was performed, per executor assignment.

## Decisions

- Fast-forwarded from `9083e9d1` to `origin/pr/2219@3f1db67d` so MessageChannel v1 was reused rather than reconstructed.
- One intrinsic equation size function is used by HML import, equation insertion, and property edits. This prevents zero-width equations from overlapping following text.
- Only the first direct `SCRIPT` is accepted. Duplicate/nested SCRIPT, unknown equation attributes/children, SCRIPT attributes, and non-whitespace direct equation text create non-preserved warnings.
- Unknown equation warning paths map to `HML_UNSUPPORTED_EQUATION_SEMANTICS` and survive later edits through `DocumentCore.hml_metadata`.
- Canonical save-state is `{ hmlSavable, saveBlockers }`; every blocker includes `preserved:false`.
- Existing transferable response ownership remains unchanged: exported bytes are sliced before their buffer is transferred.
- The existing equation editor now applies through the established snapshot operation path, so its normal UI edit participates in undo/redo.
- Unknown equation child attributes and non-whitespace text retain entity-decoded `name=value` diagnostics, bounded to 256 Unicode scalar values.
- Fixture provenance describes the already-minimized source copy exactly, including both checksums and the appended LF transformation.

## Changed areas

- Rust parser/model adapter: `src/parser/hml/reader.rs`, `adapter.rs`
- Intrinsic sizing/edit path: `src/renderer/equation/mod.rs`, `document_core/commands/object_ops/equation.rs`
- HML serializer/preflight/diagnostics: `src/serializer/hml/body.rs`, `preflight.rs`, `document.rs`
- WASM DTO/API: `src/wasm_api.rs`, `src/wasm_api/tests.rs`
- Embed/public SDK: `rhwp-studio/src/embed/*`, bridge/main, `npm/editor/*`
- Tests/evidence: HML Rust suites, frontend contracts, `hml-equation-embed.test.mjs`, repo fixture/provenance, copied SPEC/critic documents

## Test-first record

1. Parser tests failed with zero equations, then passed after reader/adapter support.
2. Serializer tests failed on unsupported equation controls and missing SCRIPT XML validation, then passed after canonical writer/preflight support.
3. Browser export failed on the public editor's packed `CommonObjAttr.attr`; the preflight now accepts only the equivalent packed form while continuing to reject unknown bits.
4. Browser fixture delivery initially hit Vite's filesystem allowlist; the E2E now injects repo fixture bytes from its Node process without widening production server access.

## Proof

- `cargo test --test hml_parser` — 34 passed
- `cargo test --test hml_serializer` — 29 passed
- `cargo test --lib equation` — 159 passed
- `cargo clippy --all-targets --all-features -- -D warnings` — green
- `cargo fmt --all -- --check`, `git diff --check` — green
- `cd rhwp-studio && npm test` — 270 passed
- `wasm-pack build --target web --release` — green, fresh `pkg/`
- generated binding/public embed tests — 3 passed
- `cd rhwp-studio && npm run build` — green
- repository browser E2E — all ten assertions passed, including real canvas selection, context-menu equation dialog, visible menu undo/redo, public parent RPC export/reload, and four non-overlapping canvas-ink regions (`113,102,32,32`)
- playwright-cli — four expected scripts, savable canonical state, one visible canvas, zero console errors

Evidence:

- `rhwp-studio/e2e/screenshots/pr2219-hml-equation-roundtrip.png`
- `rhwp-studio/e2e/screenshots/pr2219-hml-equation-playwright-cli.png`
- `output/e2e/hml-equation-embed-report.html`

## Remaining gates and concerns

- Mandatory adversarial review findings 1-3 are fixed; final independent verifier/commit gate still has to run against this final diff.
- The generated WASM was built from the current uncommitted source tree. The verifier must rebuild from the final committed SHA before treating it as exact-SHA evidence.
- Commit, non-force push, PR head/check verification, hosted deployment, SHA canary, and 24-hour observation are outside this builder assignment.
- P0 intentionally supports only the evidenced HML equation schema; wider HWPML equation compatibility remains follow-up work.
- Missing `SCRIPT` and numeric domains remain documented P1 decisions; this patch does not guess new behavior.
- `@rhwp/editor` negotiated-capability visibility remains a residual API decision because neither the accepted SPEC nor an existing public SDK capability-access pattern grounds a new API.
