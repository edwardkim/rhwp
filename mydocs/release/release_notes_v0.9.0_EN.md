# rhwp v0.9.0 — release notes

## [0.9.0]

This release introduces PDF import into editable documents, Rust/CLI consumption contracts, and accumulated editing, layout and serialization improvements. See the [GitHub Release](https://github.com/edwardkim/rhwp/releases/tag/v0.9.0) for publication time and distribution artifacts.

### PDF and developer onboarding

- Import PDF into Document IR and save HWP/HWPX via CLI, Rust, WASM or Studio. Select source pages and choose `fidelity` or `editable` layout; inspect table-inference diagnostics ([#7913](https://github.com/edwardkim/rhwp/pull/7913), [#8120](https://github.com/edwardkim/rhwp/pull/8120)).
- Default JBIG2, JPEG 2000, ICC conversion and bounded xref recovery, request policies, Linux CLI isolation and browser Worker cancellation/recovery ([#8181](https://github.com/edwardkim/rhwp/pull/8181), [#8190](https://github.com/edwardkim/rhwp/pull/8190), [#8360](https://github.com/edwardkim/rhwp/pull/8360)).
- Pattern images and text outlines, supported curved clipping, rotated text and masks; Standard Security R2–R6 password authentication and source-page/total-page progress ([#8410](https://github.com/edwardkim/rhwp/pull/8410), [#8420](https://github.com/edwardkim/rhwp/pull/8420)).
- A usable import dialog: selection no longer retains the busy cursor; advanced JSON no longer dominates the default screen ([#8401](https://github.com/edwardkim/rhwp/pull/8401), [#8412](https://github.com/edwardkim/rhwp/pull/8412)).
- **[PDF developer quick start](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_developer_onboarding_EN.md)** covers builds, first conversion, passwords, Rust/WASM, Worker hosting, progress and error recovery.

### Editing, layout and round trips

- Corrections to nested/RowBreak tables, stored lines, padding, inline objects, headers/footers, equations and font fallback, integrated with original contributor history and verification evidence.
- Pictures in table cells and relocation across pages with Undo/Redo, continuous highlighting, nested paragraph formatting, logical editing and multi-column reflow ([#8725](https://github.com/edwardkim/rhwp/pull/8725), [#8379](https://github.com/edwardkim/rhwp/pull/8379), [#8340](https://github.com/edwardkim/rhwp/pull/8340), [#8187](https://github.com/edwardkim/rhwp/pull/8187)).
- Better preservation of text-box vertical alignment, captions, definition slots, BinData references, pictures and solid fills after saving/reopening ([#8309](https://github.com/edwardkim/rhwp/pull/8309), [#8711](https://github.com/edwardkim/rhwp/pull/8711), [#8738](https://github.com/edwardkim/rhwp/pull/8738)).
- Preserved container lines, text around inline tables, and header/footer definition slots after HWP/HWPX saving and reopening ([#8741](https://github.com/edwardkim/rhwp/pull/8741)).
- Reduced large-table lookup and equation/JPEG rendering costs, plus handling for slow/unresponsive web fonts. Performance benefits depend on input and environment.

- Includes cell-shape movement, equation font/original-frame preservation and continued-table corrections merged during release preparation ([#8754](https://github.com/edwardkim/rhwp/pull/8754)).
- Memo read/insert/update/delete APIs, HWP/HWPX round trips, author/creation-time preservation, and Undo/Redo for anchor-text deletion in Studio. A new memo UI remains separate work ([#8269](https://github.com/edwardkim/rhwp/pull/8269)).

### CLI, packaging and Native hosts

- Sixel and Kitty graphics with document-font guidance ([#7829](https://github.com/edwardkim/rhwp/pull/7829), [#8154](https://github.com/edwardkim/rhwp/pull/8154)). Official CLI archives use `native-skia` and execute a PNG packaging smoke test, addressing its absence from v0.8.7 official archives ([#7818](https://github.com/edwardkim/rhwp/pull/7818)).
- **crates.io distribution begins with 0.9.0.** Rust developers can add `rhwp = "0.9.0"`; CLI users can install with `cargo install rhwp --version 0.9.0 --locked --bin rhwp` once published. Initial publication and consumption from the public registry are release completion requirements for the 14-package graph, using Rust 1.93.1. Installation becomes available when version 0.9.0 is published to the relevant registry.
- Native snapshots, tiles, caches and Qt/Flutter host foundations. macOS Metal remains disabled-by-default experimental infrastructure awaiting Apple SDK validation; this is not a claim of a complete cross-platform GPU editor ([#8739](https://github.com/edwardkim/rhwp/pull/8739)).
- Image insertion in exam ingest and agent/MCP guidance ([#8499](https://github.com/edwardkim/rhwp/pull/8499)).
- Separated password authentication from diagnostic flows and redacted Debug output without disabling CodeQL. A protected-branch full security scan remains a separate follow-up ([#8750](https://github.com/edwardkim/rhwp/pull/8750)).

### Compatibility and limits

PDF import does not reconstruct every authoring structure or OCR scans. Unsupported clipping, masks, text paint and encryption combinations remain explicit errors. Custom limits only lower profile ceilings. Complete parser heap/RSS bounds are not proven; synchronous APIs require host cancellation/isolation. Imported documents do not inherit the source PDF password.

Other editors may render RHWP text-paint extensions or transparent PNGs differently. Supply actual fonts and verify target-application output. Pin JS/WASM, Workers and hosted Studio together. VS Code retains HWP/HWPX file associations.

### Contributors

The audit covers 4,896 commits after v0.8.7 through the release finalization baseline `41e08ae19e360a32179d851ca2534c1195a8dced`, including authors, co-authors, original PRs and cherry-pick origins. It identifies 33 people and one separately credited bot (Dependabot), with zero unresolved human identities or reference dispositions. Nine unavailable cherry-pick source objects are disclosed; integrated commit authors remain included. Verify that release-record changes after this baseline introduce no uncredited authors.

[Contributor ledger](https://github.com/edwardkim/rhwp/blob/devel/mydocs/tech/investigations/release-v0.9.0/contributor-ledger.json) · [Identity evidence](https://github.com/edwardkim/rhwp/blob/devel/mydocs/tech/investigations/release-v0.9.0/contributor-overrides.json)

<!-- release-contributors:v0.9.0:start -->
- @9ood2s
- @ashcastle
- @bellkjtt
- @chrisryugj
- @davindev
- @donggyun112
- @edwardkim
- @emptinessform
- @greenoceanplanet
- @h1-andro2d
- @hccgahy1-hub
- @hexisteme
- @hyunmyungJaneLee
- @jangster77
- @jeong-sik
- @johndoekim
- @Kang-T
- @ldj-diquest
- @lidge-jun
- @mytrashcan
- @oksure
- @pblsketch
- @Phantomn
- @physwkim
- @planet6897
- @postmelee
- @sacru2red
- @scari
- @semanticist21
- @seo-rii
- @Steveyang8282
- @studyreadbook4ever
- @wook8170
<!-- release-contributors:v0.9.0:end -->

[Full change range](https://github.com/edwardkim/rhwp/compare/v0.8.7...41e08ae19e360a32179d851ca2534c1195a8dced)
