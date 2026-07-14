# REVIEW-FIX - PR 2219 HML equation and export

## Outcome

Mandatory review Major 1-3 and the evidence-count drift are resolved with scoped changes. No commit or push was performed.

## Major 1 - real editor and visible browser proof

- Root cause: the E2E called `__wasm.setEquationProperties()` directly, and the shipped equation editor also bypassed snapshot history.
- Fix: `EquationEditorDialog` now receives existing `CommandServices` and applies through `executeOperation({ kind: 'snapshot', operationType: 'equationEdit' })`.
- E2E: a neutral parent harness uses the public `@rhwp/editor` client. It clicks the first equation on the real canvas, opens the existing context-menu equation editor, submits `x^2 + 2`, invokes visible Edit-menu undo and redo, exports through parent `exportHml()`, and reloads through parent `loadFile()`.
- Primary visual assertion: four equation regions are non-empty, pairwise non-overlapping, and contain canvas ink counts `113,102,32,32`. Internal `__wasm` properties are used only as the secondary SCRIPT semantic oracle and hit-coordinate locator.

## Major 2 - durable unknown child values

- Unsupported equation child attributes now emit exact attribute paths with entity-decoded `name=value`.
- Unsupported non-whitespace text chunks, including entity/reference splits, are combined by exact `.../#text` path and retain entity-decoded `#text=value`.
- Diagnostic semantics are bounded to 256 Unicode scalar values and truncate on a valid character boundary.
- Rust tests prove child attribute/text values survive equation edit and document snapshot restore in both undo and redo directions and remain present in canonical `getHmlSaveState()` blockers with `preserved:false`.

## Major 3 - fixture contract and provenance

- Provenance now states that the source was already a minimized synthetic fixture; no new anonymization/minimization is claimed.
- Recorded source SHA-256: `66998b57e70d38175e68facc3bf2fb2b7e6e0839c41c012acb47209d3071c538`.
- Recorded derived SHA-256: `b51be49cde780d39b92f42cfd1cbd58474900c46c12c4df971f79bd511c7045a`.
- Exact transformation: copy 4,087 source bytes unchanged and append one LF, yielding 4,088 bytes.
- Contract test fixes the four ordered SCRIPT values and rejects source paths, URLs, email markers, and source repository identifiers.

## Verification

- `cargo test --test hml_parser --test hml_serializer` - 34/34 and 29/29 passed.
- `cargo test --lib test_unknown_equation_values_survive_edit_undo_redo_and_save_state` - 1/1 passed.
- `cargo test issue_1061_equation -- --nocapture` - exited cleanly after the full filtered-binary traversal; 4/4 matching equation tests passed.
- `cd rhwp-studio && npm test` - 270/270 passed.
- `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs` - 3/3 passed.
- `wasm-pack build --target web --release` - passed after the sandboxed helper-install attempt was rerun with required execution permission.
- `cd rhwp-studio && npm run build` - passed against the fresh WASM package.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `git diff --check` - passed.
- `VITE_URL=http://127.0.0.1:7700 node e2e/hml-equation-embed.test.mjs --mode=headless` - 10/10 assertions passed with zero runtime browser errors.

## Remaining gates and residuals

- Missing `SCRIPT` semantics and numeric domains remain P1 decisions. No behavior was guessed.
- Public SDK negotiated-capability visibility remains a residual API decision. The accepted SPEC grounds `exportHml()` and `getHmlSaveState()`, but no existing public SDK capability-access pattern grounds an additional API in this patch.
- Final independent verifier, commit gate, commit/non-force push, PR head SHA/checks, hosted deployment canary, and observation remain outside this executor assignment.
