# REVIEW - PR 2219 HML equation and export

## Summary

Intent: add loss-safe HML equation import/edit/export and an additive embed HML export contract to PR 2219. Verdict: **REQUEST CHANGES**. The supported-schema parser/serializer path is generally cohesive, but the current diff does not prove the required real editor flow and does not fully satisfy the durable unknown-semantics diagnostic contract.

Reviewer approval is not verification. A verifier must rerun the final exact-SHA commands and browser flow after these findings are resolved.

## Critical issues

None found in the inspected supported-schema happy path.

## Major issues

### 1. Required browser edit/visible-equation proof is bypassed

- Files: `rhwp-studio/e2e/hml-equation-embed.test.mjs:41`, `rhwp-studio/e2e/hml-equation-embed.test.mjs:47`, `rhwp-studio/e2e/hml-equation-embed.test.mjs:66`, `rhwp-studio/e2e/hml-equation-embed.test.mjs:108`
- Requirement: REQ-HML-EQ-005 and SPEC 7.3 require selecting an imported equation, editing it through the existing equation editor UI, then proving each equation is visibly rendered after export/reload.
- Found: the E2E reads `contentWindow.__wasm`, calls `getEquationProperties`/`setEquationProperties` directly, and counts render-tree nodes. It never selects the equation on the canvas, opens/submits the equation editor, exercises its snapshot/undo integration, or makes a visual assertion for each SCRIPT. The screenshot is taken, but no assertion is derived from the rendered result.
- Impact: the test can pass when selection, dialog wiring, keyboard/focus behavior, property application, undo, or canvas presentation is broken. It therefore cannot satisfy the GOAL browser/edit criterion or the claimed “existing editor” evidence.
- Fix/test: use Playwright to select the first imported equation through the canvas/hit target, open the existing equation dialog, change SCRIPT, submit, invoke the public parent RPC, reload, and assert the four equation regions are non-empty/non-overlapping. Keep internal API inspection only as a secondary semantic oracle. Add undo/redo assertions before export and confirm the durable save blocker remains unchanged through both operations.

### 2. Unknown child/text values are discarded from the required durable diagnostic

- Files: `src/parser/hml/reader.rs:1087`, `src/parser/hml/reader.rs:1099`, `src/parser/hml/reader.rs:1049`; inadequate coverage at `tests/hml_parser.rs:192`
- Requirement: REQ-HML-EQ-001 requires unsupported equation attributes/children to retain path/name/value in import metadata; NFR-07 requires durable diagnostics. The implementation correctly retains values for unknown attributes directly on `EQUATION` and `SCRIPT`, but not for unsupported child subtrees or direct equation text.
- Found: `warn_if_unsupported()` emits only the child element name (`FUTURE`), then the streaming loop ignores that child's attributes and text. Direct non-whitespace equation text similarly records only `#text`. For `<FUTURE Mode="matrix">secret</FUTURE>`, neither `Mode=matrix` nor `secret` survives in metadata. Existing tests assert path/code only, so this contract loss is hidden.
- Impact: save is safely blocked, but the parent cannot report which unsupported value caused the block, and the diagnostic does not meet the finalized specification. Multiple distinct unknown values collapse to indistinguishable blockers.
- Fix/test: while inside an equation, emit decoded exact-path diagnostics for each unsupported child attribute and non-whitespace text value (with bounded/truncated diagnostic storage if needed), or capture a bounded subtree diagnostic representation. Add parser plus `getHmlSaveState` tests for entity-decoded child attribute/text values and verify they remain attached after edit, snapshot restore, undo, and redo.

### 3. Fixture provenance says “minimized” although the repo fixture is the source file plus one newline

- Files: `tests/fixtures/hml/README.md:5`, `tests/fixtures/hml/exambank_math_equations_min.hml:1`
- Requirement: the fixture contract requires a minimized/anonymized repo-owned fixture and accurate provenance.
- Found: the recorded source checksum is correct, but `cmp` shows the repo fixture is byte-identical to the 4,087-byte source through EOF and merely adds a trailing newline (repo fixture SHA-256 `b51be49cde780d39b92f42cfd1cbd58474900c46c12c4df971f79bd511c7045a`, 4,088 bytes). The provenance calls it “minimized” without explaining that the source was already minimized or recording the derived fixture checksum/transformation.
- Impact: the authorization statement may still be valid, but the auditable data-handling claim is inaccurate and cannot independently prove anonymization/minimization.
- Fix/test: either actually reduce/anonymize the source into the minimal page/style/equation structure, or state explicitly that this is an authorized copy of an already-minimized synthetic source. Record both source and derived fixture SHA-256 values and the exact transformation; add a fixture contract test for four ordered scripts and absence of disallowed identifiers.

## Minor issues

### 4. Run evidence documents disagree after the final edge pass

- Files: `docs/changelog/2026-07/13-pr-2219-hml-equation-export-implementation/BUILD.md:43`, `docs/changelog/2026-07/13-pr-2219-hml-equation-export-implementation/QA.md:15`, `docs/changelog/2026-07/13-pr-2219-hml-equation-export-implementation/run-state.json:29`
- Found: BUILD/QA/run-state retain serializer `28` and frontend `268` counts, while IMPROVE-FULL/EDGE report serializer `29` and frontend `269`; run-state also remains in phase `Build` with the old proof command.
- Fix: regenerate the final evidence summary from the exact reviewed SHA and keep one authoritative count/phase record.

## Ask-user decisions

1. Missing `SCRIPT`: current behavior canonicalizes absent and empty SCRIPT to the same empty model. The specification does not define whether absence must block. Decide after corpus/schema evidence; do not guess.
2. Numeric domains: decide supported ranges for `BaseUnit`, `TextColor`, and editable `BaseLine`. Current `u32`/`i16` conversions allow values outside the evidenced schema, including wrapping an edited `i32` baseline to `i16`. Until defined, document this as unsupported input or add typed blockers based on corpus/schema evidence.
3. Public SDK capability visibility: low-level parents can inspect `rhwp-connected.capabilities`, but `@rhwp/editor` consumes the handshake and exposes no negotiated capability API. Decide whether SDK consumers must be able to gate `exportHml()` without probing failure; if yes, expose a read-only negotiated-capabilities method/property.

## Residual risks

- Hosted deployment SHA equality, hosted canary, rollback signal, and 24-hour observation remain outside this PR-readiness run and must not be claimed complete.
- The fixture covers one evidenced HWPML equation schema only; wider casing/attribute/numeric compatibility remains P1.
- Exact-SHA WASM provenance is not yet established because current browser evidence was built from an uncommitted tree.
- The requested `cargo test issue_1061_equation` filter did execute and pass the four matching equation tests, but the overall command was interrupted afterward while Cargo continued traversing unrelated integration binaries; rerun it cleanly in final verification. `cargo test --test hml_cli` independently passed 12/12 during review.

## Positive feedback

- Unknown equation semantics are typed and remain outside document snapshots, which is the right fail-closed architecture for edits.
- Serializer preflight runs before byte production and the MessageChannel response copies before transfer, preserving atomicity and handler-owned buffers.
- Parser state accepts only the first direct SCRIPT, handles CDATA/entities, and materializes one shared intrinsic size path for import/insert/edit.
- The canonical Rust save-state DTO is adapted consistently to the studio and embed shapes, including `preserved:false`.

## Verdict

**REQUEST CHANGES** — resolve Major 1-3, then rerun the exact final verifier and real UI browser flow.
