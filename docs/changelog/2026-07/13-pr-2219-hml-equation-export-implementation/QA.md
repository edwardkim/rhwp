# QA - PR 2219 HML equation and export

- Verdict: PASS

Backward-trace: clean

## Results

- [x] `cargo test --test hml_parser` - 34 passed; four ordered fixture equations, inline offsets, color/base unit, first-direct SCRIPT, malformed XML, unknown exact-path values, fixture contract.
- [x] `cargo test --test hml_serializer` - 29 passed; canonical equation export/reparse, changed and untouched scripts, invalid XML/offset aggregation, durable unknown-semantics blockers.
- [x] `cargo test --lib equation` - 160 passed; shared equation parser/layout/render/HWPX/WASM regression slice.
- [x] Canonical save-state edge tests - unknown child attribute/text values survive edit/undo/redo; HML/non-HML/unknown wire DTO remains exact.
- [x] `cd rhwp-studio && npm test` - 270 passed; shipped equation editor snapshot history and additive embed `exportHml`/`getHmlSaveState`/capability behavior.
- [x] Generated/public binding tests - 3 passed.
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `git diff --check` - exit 0.
- [x] Fresh `wasm-pack build --target web --release` - exit 0 with wasm-opt; `pkg/` rebuilt from the verified tree. The initial sandboxed attempt hit EPERM while installing wasm-bindgen, so it was discarded and rerun with required execution permission.
- [x] `cd rhwp-studio && npm run build` after the successful fresh WASM build - exit 0; TypeScript, Vite, and PWA output generated.
- [x] Git/GitHub linkage - `9083e9d1` is an ancestor of `3f1db67d`; fork push remote is `cskwork/rhwp`, target branch is `feature/hml-document-open`, and open upstream PR is `edwardkim/rhwp#2219` with that fork head.
- [x] playwright-cli public embed/UI round-trip - initial scripts `x^2 +1|x^2 +1|3|3`; canvas selection, context equation dialog, submit `x^3 +2`, visible Edit menu undo/redo, public `exportHml`, public reload; final scripts `x^3 +2|x^2 +1|3|3`.
- [x] Visual/a11y/runtime proof - four positive non-overlapping regions, canvas ink `112,102,32,32`, exported length 5,837 bytes, five canonical save states all HML-savable with zero blockers, `lang=ko`, title present, zero runtime errors.

## Commands

| Command | Source | Result |
|---|---|---|
| `cargo test --test hml_parser` | frozen_repo | PASS 34/34 |
| `cargo test --test hml_serializer` | frozen_repo | PASS 29/29 |
| `cd rhwp-studio && npm test` | frozen_repo | PASS 270/270 |
| `cd rhwp-studio && npm run build` | frozen_repo | PASS after fresh WASM |
| `cargo fmt --all -- --check` | frozen_repo | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | frozen_repo | PASS |
| `cargo test --lib equation` | agent_detected | PASS 160/160 |
| `cargo test --lib test_unknown_equation_values_survive_edit_undo_redo_and_save_state` | agent_detected | PASS 1/1 |
| `cargo test --lib test_hml_save_state_is_one_canonical_dto_for_hml_non_hml_and_unknown_equation` | agent_detected | PASS 1/1 |
| `node --test ../scripts/frontend-wasm-bindings.test.mjs ../scripts/frontend-editor-embed.test.mjs` | agent_detected | PASS 3/3 |
| `wasm-pack build --target web --release` | agent_detected | PASS, fresh optimized pkg |
| `git diff --check` | agent_detected | PASS |
| `playwright-cli -s=pr2219-verify run-code --filename=.../qa/hml-equation-flow.js` | agent_detected | PASS, action_count 16/35 |

## QA

Tool: playwright-cli 0.1.14

- Served editor: `http://127.0.0.1:7700/e2e/embed-harness.html` from the exact worktree; fixture served read-only at `http://127.0.0.1:7701/tests/fixtures/hml/exambank_math_equations_min.hml` and loaded through the public parent SDK.
- Comparison: before-after plus hosted baseline.
- Repeatable flow: `qa/hml-equation-flow.js`.
- Action count: 16 / action cap 35.
- Initial local visible state: `qa/to-be-hml-equations.png` (SHA-256 `2fc6935534ec4ac0a6ab593a460e773892cedb14550b4d91aa07d057efbbb28d`).
- Round-trip visible state: `qa/to-be-hml-roundtrip.png` (SHA-256 `fbdf1ef09f975f468283de53a266973d95e9173f83a589f37b4276de62e623ed`).
- Hosted as-is baseline: `qa/as-is-hml-equations.png` (SHA-256 `a46687a32628ed6dcd7f2ea39bab55364578f87f8d2f9522bb0339303f7bddb8`). Provenance: copied byte-for-byte from the original checkout's previously captured `docs/changelog/2026-07/13-hml-web-poc/exambank-real-math-hml-opened.png`, which records `https://hwp-editor.agentic-worker.store/` dropping the same fixture's equations and showing EQUATION warnings.
- Accessibility snapshot: `qa/to-be-hml-roundtrip-a11y.yml`; interactive editor controls are exposed, document language is Korean, and the editor title is present.
- Runtime: 0 errors. One non-failing Canvas2D `willReadFrequently` performance warning was caused by the verifier's four-region `getImageData` ink measurement.
- Network: both initial and reload editors fetched the freshly generated `pkg/rhwp_bg.wasm` with HTTP 200.
- Teardown: playwright-cli and both local read-only servers are stopped after gate execution.

## Reproduction Fidelity

- Fidelity level: prod-snapshot
- Data source: authorized repo-owned copy of the already-minimized synthetic ExamBank fixture; four ordered equations and checksums are fixed by the fixture contract test.
- Residual risk from data gap: the fixture covers the evidenced HWPML 2.9 equation schema, not every HWPML casing, numeric domain, or unsupported control.
- Post-deploy confirmation plan: after merge/deploy, verify hosted commit SHA, rerun the same fixture load/edit/export/reload canary, then observe host error telemetry for 24 hours.

## Residual Risk

- Hosted deployment SHA equality, canary, rollback signal, and 24-hour observation are post-merge/deploy responsibilities and are not claimed by this PR-readiness verification.
- Missing `SCRIPT` semantics, wider numeric domains, and optional public SDK negotiated-capability visibility remain named P1 decisions; this patch does not guess them.
