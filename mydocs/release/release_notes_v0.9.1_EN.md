# rhwp v0.9.1 — Layout, round trips, editing and delivery

## [0.9.1] — Release preparation

This candidate improves table, paragraph, font and round-trip fidelity, clipboard handling, small-screen dialogs, Native host foundations and release validation. Publication remains channel-specific.

### Layout, editing and round trips

- Correct first-line, continuation and terminal fragments of nested tables; preserve cell pictures/shapes and the content that follows tables ([#8984](https://github.com/edwardkim/rhwp/pull/8984), [#9066](https://github.com/edwardkim/rhwp/pull/9066), [#9069](https://github.com/edwardkim/rhwp/pull/9069), [#9106](https://github.com/edwardkim/rhwp/pull/9106)).
- Improve saved lines, empty paragraphs, headers/footers, master pages, equation symbols and actual font-program selection. Existing document differences remain; this is not a claim of complete typesetting fidelity ([#9074](https://github.com/edwardkim/rhwp/pull/9074), [#9083](https://github.com/edwardkim/rhwp/pull/9083), [#9096](https://github.com/edwardkim/rhwp/pull/9096), [#9106](https://github.com/edwardkim/rhwp/pull/9106)).
- Preserve HTML table borders, vertical alignment and repeated saves; retain numbering/bullet character styles ([#9017](https://github.com/edwardkim/rhwp/pull/9017), [#9140](https://github.com/edwardkim/rhwp/pull/9140)).
- Keep About and shared dialogs usable in small viewports and at higher zoom ([#9092](https://github.com/edwardkim/rhwp/pull/9092), [#9136](https://github.com/edwardkim/rhwp/pull/9136)).

- Recollect host font requests only for the edited Canvas2D page when document, font and pagination state remain valid; preserve full discovery after other changes ([#9168](https://github.com/edwardkim/rhwp/pull/9168)).

- Report automatic HWP download opening as complete after the initial page is painted, including font-preparation races ([#9170](https://github.com/edwardkim/rhwp/pull/9170)).

- Preserve HWP3 picture ownership across saved page breaks and recover native font programs while preserving the original TTF/TTC preference ([#9157](https://github.com/edwardkim/rhwp/pull/9157)). Release preparation completed the full 18-document Native/fresh WASM protected verification.
- Handle Chrome settings-channel closure during worker suspension without enabling badges or preview before settings are available.

- Track pending document opens through completion, failure and replacement so startup blank/recovery work cannot supersede an immediate SDK load. Preserve first-page completion and cancellation behavior.

### PDF, CLI, Native hosts and release tooling

- Parse PDF ToUnicode range tokens and comments correctly. Existing PDF onboarding covers passwords, page selection, resource policies and Worker ownership ([#8922](https://github.com/edwardkim/rhwp/pull/8922)).
- Extend picture/shape CLI editing and Native font diagnostics ([#9052](https://github.com/edwardkim/rhwp/pull/9052), [#9147](https://github.com/edwardkim/rhwp/pull/9147)).
- Extend Qt/Flutter Native preview, tiles, multiple pages, cache and lifecycle foundations. Reference and experimental support, including platform validation limits, remains documented in the Native guides; a complete GPU editor on every platform is not promised ([#8809](https://github.com/edwardkim/rhwp/pull/8809), [#9132](https://github.com/edwardkim/rhwp/pull/9132)).
- Remove PDF body debug output and correct qcms test pointers while retaining security detection rules ([#8884](https://github.com/edwardkim/rhwp/pull/8884)).
- Pin the Ubuntu 24.04 baseline, check 26.04 compatibility, and separate Fuzz build/run with split code generation. The earlier runner shutdown was not proven to be OOM ([#9155](https://github.com/edwardkim/rhwp/pull/9155)).
- Replace 13 deprecated Atomic calls while keeping Rust 1.93.1, resource caps, identifier exhaustion and memory ordering.

### Packages and compatibility

Product, npm, VS Code and browser-extension candidates use 0.9.1. Eight changed or dependency-updated crates receive new versions; six unchanged published crates are reused. Published versions remain immutable. The Rust MSRV remains 1.93.1. Publication and independent consumer results are verified in the delivery stage.

### Contributors

The audit covers **1,238 commits** from v0.9.0 through feature source `197ce19e99438c2cba452c654ae1af504dab85ec`, including Git authors, co-authors and original/integration PR evidence. Thank you to **19 people** and Dependabot. The local Atomic cleanup and release metadata are follow-ups by the maintainer already included below.

<!-- release-contributors:v0.9.1:start -->
- @Bongseop-Kim
- @chrisryugj
- @donggyun112
- @edwardkim
- @gitprogam
- @hyunmyungJaneLee
- @jangster77
- @jeong-sik
- @JinHeeWond
- @JinHo-von-Choi
- @johndoekim
- @pblsketch
- @planet6897
- @postmelee
- @puckqook
- @sacru2red
- @semanticist21
- @seo-rii
- @signspark
<!-- release-contributors:v0.9.1:end -->

Bot: @dependabot[bot]. See the [public ledger](../tech/investigations/release-v0.9.1/contributor-ledger.json) for credit evidence and identity decisions.
