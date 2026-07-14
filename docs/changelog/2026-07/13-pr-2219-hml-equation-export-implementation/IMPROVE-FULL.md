# IMPROVE FULL SPEC - PR 2219 HML equation and export

## Outcome

The full-spec pass found and fixed three grounded gaps without expanding P0 scope:

1. `getHmlSaveState` now returns the required exact wire DTO: `{ sourceFormat, hmlSavable, blockers }`. The studio metadata path still adapts the same canonical Rust result to its existing `{ hmlSavable, saveBlockers }` UI contract.
2. Unknown, duplicate, or nested equation semantics now carry `HmlWarningCode::UnsupportedEquationSemantics`; save preflight maps that typed diagnostic directly to `HML_UNSUPPORTED_EQUATION_SEMANTICS` instead of inferring meaning from an XML path substring.
3. Asymmetric `TextColor`, `BaseUnit`, and optional `Font` now have render and export/reparse proof. The fixture provenance records the requesting source owner, and Korean/English README claims match supported equation import and loss-safe HML save behavior.

No commit or push was performed.

## Requirement trace

- REQ-HML-EQ-001/003/007, NFR-07: typed equation warning is durable through import metadata and becomes the typed save blocker.
- VG-02/VG-03, T-EQ-10/T-EQ-11: `BaseUnit=1200` produces a non-zero rendered font size; `TextColor=1122867` renders as model `0x00112233` / SVG `#332211` and exports unchanged with `Font="Hancom"`.
- REQ-EMBED-HML-004, PLAN Amendment 5: Rust owns one canonical save-state DTO; exact deep equality covers HML, non-HML, and unknown-equation states including `preserved:false`.
- Fixture contract and NFR-01 documentation: provenance authorization is explicit; README and README_EN no longer claim equations/HML save are always unsupported.

## RED to GREEN

- RED: parser tests could not compile because `UnsupportedEquationSemantics` did not exist.
- RED: canonical WASM DTO test received `{hmlSavable,saveBlockers}` instead of required `{sourceFormat,hmlSavable,blockers}`.
- GREEN: `cargo test --test hml_parser --test hml_serializer` - 32 parser and 29 serializer tests passed.
- GREEN: `cargo test --lib test_hml_save_state_is_one_canonical_dto_for_hml_non_hml_and_unknown_equation` - 1 passed.
- GREEN: `cd rhwp-studio && npm test` - 268 passed.
- GREEN: `node --test scripts/frontend-editor-embed.test.mjs` - 2 passed.
- GREEN: fresh `wasm-pack build --target web --release`, then `cd rhwp-studio && npm run build`.
- GREEN: with local Vite server, `cd rhwp-studio && node e2e/hml-equation-embed.test.mjs --mode=headless` - all 7 assertions passed, including exact save-state shape and zero browser errors.
- GREEN: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `git diff --check`.

## Decision gates and residual risk

- Resolved automatically: the public DTO field names were explicitly fixed by SPEC section REQ-EMBED-HML-004; no product decision was required.
- Environment-only hurdle: sandboxed `wasm-pack` helper installation and Chromium launch needed approved escalation; both exact commands then passed.
- Delegation was attempted per repository rules, but the collaboration thread limit was already full; the isolated executor completed the audit locally.
- Remaining gates are outside this phase: independent edge-case/adversarial review, verifier promotion of evidence, commit/push/PR SHA checks, and post-deploy canary/24-hour observation.
