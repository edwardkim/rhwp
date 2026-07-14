# PLAN - PR 2219 HML equation and export

Frozen plan. Builder reads this file as the whole brief.

## Approval

- Status: auto-approved
- Record: 2026-07-13, user explicitly requested subagent code modification and inclusion in both user fork and upstream PR 2219.

## Intent

- Goal: extend current PR 2219 head with loss-safe HML equation import/edit/export and embed HML export.
- Constraints: work only in this worktree; test-first; reuse existing `Equation`, HWPX parser concepts, HML metadata/preflight, WASM exporter, and MessageChannel transfer path. Preserve existing public APIs and HWP/HWPX behavior. Do not add direct storage/network/credentials.
- Tradeoffs: P0 supports the evidenced equation schema only; unknown attrs/children become durable blockers. Full HWPML compatibility inventory remains future work.
- Rejected: hiding warnings, preserving success while dropping equations, exporting HWPX instead of HML, direct object-storage upload, parsing RPC error strings.
- Completion promise: equation fixture load/edit/export/reload plus typed save-state and embed RPC are proven by Rust/frontend/browser tests; all regressions and review findings cleared; commit gate green; max_iterations 8.
- Stop condition: all GOAL criteria checked or a grounded user decision/permission blocker is recorded.

## Steps

1. Copy the finalized spec from `/Users/chaeseong-gug/Documents/PARA/Resource/rhwp/docs/changelog/2026-07/13-pr-2219-hml-equation-export-spec/` into the same relative path in this worktree using apply_patch. Add a minimized repo-owned fixture at `tests/fixtures/hml/exambank_math_equations_min.hml` and provenance README with source SHA `66998b57e70d38175e68facc3bf2fb2b7e6e0839c41c012acb47209d3071c538`.
2. Write RED parser tests first: four equations/scripts; inline 8-unit placement; BaseLine/BaseUnit/Version/Font; asymmetric TextColor; entity/CDATA; malformed XML; unknown attr/child durable warning. Validate assumptions before final mappings.
3. Extend `src/parser/hml/reader.rs` with HmlEquation state/event handling and exact warning paths; extend `src/parser/hml/adapter.rs` to `Control::Equation`. Reuse existing insertion/HWPX defaults only where validation tests prove equivalence.
4. Write RED serializer/preflight tests: canonical equation output; escape/reparse; edited vs untouched scripts; invalid char/offset; unknown semantics blocker aggregation that survives edits via `HmlImportMetadata` held by `DocumentCore`.
5. Extend `src/serializer/hml/body.rs`, `preflight.rs`, warning/blocker mapping only as required. Never allow silent unknown-semantics export.
6. Write RED frontend/embed tests, then add `exportHml()`, required `getHmlSaveState()`, and additive `hml-export` capability in protocol/router/main/bridge. Preserve existing `postPortResponse` slice-and-transfer ownership.
7. Add/extend browser embed E2E: as-is screenshot; local to-be load with visible equations; edit first equation; export/reload; visible changed and unchanged scripts; error/a11y checks.
8. Run improve-full-spec and edge-case passes, then mandatory adversarial review. Fix grounded findings only.
9. Exact verify: Rust targeted tests, relevant full suites, fmt/clippy, rhwp-studio test/build, browser QA gate. Update GOAL/QA/run-state and write Z marker only when all green.
10. Run commit gate, commit, push the verified commit to `origin/feature/hml-document-open`, verify `edwardkim/rhwp#2219` head SHA and checks. Do not force-push.

## Tools & Skills

- Builder/improvers: `/Users/chaeseong-gug/Documents/PARA/Resource/supergoal-skill/agents/executor.md`
- Reviewer: `/Users/chaeseong-gug/Documents/PARA/Resource/supergoal-skill/agents/code-reviewer.md`
- Browser QA: qa-tester persona, `reference/qa.md`, `reference/playwright-cli.md`; driver must be playwright-cli.
- Code discovery: codebase-memory-mcp first; rg/sed only for exact strings/non-code.
- Tests: Cargo repo tests, rhwp-studio npm scripts, existing embed E2E.

## Verification strategy

- Before proof: hosted screenshot has blank equation positions and EQUATION warnings; source reader explicitly marks EQUATION unsupported; router returns Unknown method.
- Step 1-3 -> criteria 1-3.
- Step 4-5 -> criterion 4.
- Step 6 -> criterion 5.
- Step 7 -> QA cases and criterion 7.
- Step 8-9 -> criteria 6, 8.
- Step 10 -> criterion 9.
- Trusted commands: `cargo test --test hml_parser` (frozen_repo), `cargo test --test hml_serializer` (frozen_repo), `cd rhwp-studio && npm test` (frozen_repo), `cd rhwp-studio && npm run build` (frozen_repo), `cargo fmt -- --check` (frozen_repo), `cargo clippy --all-targets --all-features -- -D warnings` (frozen_repo).

## Grounding ledger

- Actual PR head -> `cskwork:feature/hml-document-open@9083e9d1` from GitHub PR metadata -> worktree source fixed.
- Equation model -> existing `src/model/control.rs::Equation` and HWPX parser -> reuse, no parallel domain model.
- Durable diagnostics -> `parse_document_with_metadata -> HmlImportMetadata -> DocumentCore.hml_metadata -> hml_export_preflight` -> extend existing path.
- Transfer semantics -> existing runtime slices Uint8Array then transfers copy buffer -> keep additive method.
- Source ownership -> user requested fork and upstream PR inclusion -> push verified non-force update to fork PR head.

## Amendment — 2026-07-13 plan attack

Grounded blockers in `PLAN-ATTACK.md` change execution details without changing the requested outcome.

1. Before source edits, fast-forward the run branch to `origin/pr/2219@3f1db67d`. That commit contains the actual PR head `9083e9d1` plus `devel@e750e02f`, including MessageChannel v1. Verify `9083e9d1` is an ancestor and do not reconstruct the transport manually.
2. Browser/frontend proof must run `wasm-pack build --target web --release` from the exact tested Rust SHA before `npm run build`/E2E. Stale `pkg/` artifacts are not evidence.
3. VG-04 is a hard precondition: zero `common.width/height` may overlap following text because layout measurement reads those fields. Add an interleaved CHAR+EQUATION+CHAR bbox/line/hit test and implement one shared intrinsic-size path or materialize proven dimensions.
4. Add explicit CDATA and duplicate/nested SCRIPT RED cases. Accept only the first direct SCRIPT; unknown/duplicate subtrees produce durable exact-path blockers.
5. Define one canonical Rust/WASM save-state DTO. Adapt both current studio metadata and embed `getHmlSaveState` from it; exact deep-equality tests include `preserved:false` for HML, non-HML, and unknown-equation cases.
6. This run's completion boundary is verified PR readiness and push to `cskwork/feature/hml-document-open`, making the commit visible in `edwardkim/rhwp#2219`. Hosted deployment/SHA canary/24-hour observation remains a required post-merge/deploy phase, because this run has no deploy-operator authority or wait contract. Do not claim hosted completion.

## Amendment — 2026-07-13 delivery gate ordering

The original ninth success criterion combined pre-commit verification with the post-gate push. Because the commit gate requires every success criterion to be checked before a commit can exist, that wording created a circular dependency. The criterion is reclassified without weakening product scope: Exact Verify proves the branch ancestry, fork target, and upstream PR linkage; after the commit gate passes, step 10 performs the real non-force push and verifies the upstream PR head SHA. Final delivery evidence is reported from Git/GitHub and is not claimed by the pre-commit verifier.
