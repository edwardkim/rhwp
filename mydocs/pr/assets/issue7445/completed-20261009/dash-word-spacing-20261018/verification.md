# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## provenance

이전 생성 파일: `provenance.json` · 7558 bytes · SHA-256 `18c24bd96ee99749ef3bbc59e9c1eb1b7e1724fb2f2aa4d50b09ab8ba5154568`.

| 항목 | 당시 기록 |
|---|---|
| `base` | 6ed1682c3267ea786734eb2e75252916a861d1fd |
| `parent` | 0666befe5b8fbac7c5464ac654dbaf672054d016 |
| `scope` | C253 working source, not final committed-head or whole52/PR acceptance. |
| `source` | samples/task2097/75544_pii_bunseok.hwpx |
| `sourceSha256` | f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1 |
| `pdf` | pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf |
| `pdfSha256` | 0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770 |
| `compat` | 2022 |
| `profile` | print |
| `dpi` | 192 |
| `coordinate` | CSS96 |
| `tolerance` | canonical uncorrected silhouette 2px, gate90% |
| `sourceHashes.src/renderer/layout/paragraph_layout.rs` | ef33c2fe63e05d1588c63747abb93e110bee67a35bb60d212a4ccfa033f17c75 |
| `sourceHashes.tests/cases/issue_7445_dash_leader_word_spacing.rs` | 57af3815fd5334e416970dad6182cb4685e974fcaa7381fcc91304142bd7f75b |
| `native.command[0]` | cargo |
| `native.command[1]` | build |
| `native.command[2]` | --locked |
| `native.command[3]` | --profile |
| `native.command[4]` | release-test |
| `native.command[5]` | --features |
| `native.command[6]` | native-skia |
| `native.command[7]` | --bin |
| `native.command[8]` | rhwp |
| `native.command[9]` | -j |
| `native.command[10]` | 8 |
| `native.exitCode` | 0 |
| `native.sha256` | 4394479ed276067533d28233b8bb540d5b80fcabf0ed53b7c9d7f7c9aef0d598 |
| `native.source` | source.json |
| `wasm.command[0]` | scripts/wasm-pack-locked.sh |
| `wasm.command[1]` | --target |
| `wasm.command[2]` | web |
| `wasm.command[3]` | --out-dir |
| `wasm.command[4]` | pkg |
| `wasm.command[5]` | --no-opt |
| `wasm.exitCode` | 0 |
| `wasm.hashes.rhwp.js` | 7ba187379818abd095fe6f21cc48b5466f371739135abbe88b008e880ae43837 |
| `wasm.hashes.rhwp_bg.wasm` | bca79b7240b3d7ba639b52e86bec7126c3d47658f4eff6d6c72d616314988ef4 |
| `wasm.hashes.rhwp.d.ts` | 516dd711bb7744497f04aac485d79ac323465f46f7c8f83012e2fd01b56ed080 |
| `wasm.hashes.rhwp_bg.wasm.d.ts` | e0acb3a0810a8e8f7208c3eabf1ba1eb6df238a118737d57fa34126fe7703761 |
| `wasm.rootAndStudioPublicFilesVerified` | 8 |
| `wasm.scope` | Mac locked web/no-opt local alternative, not Docker optimized; actual runtime/full visual validation pending. |
| `whole[0].backend` | native |
| `whole[0].case` | pii66-declared |
| `whole[0].pages` | 66 |
| `whole[0].minimum` | 94.12098 |
| `whole[1].backend` | native |
| `whole[1].case` | pii66-default |
| `whole[1].pages` | 66 |
| `whole[1].minimum` | 94.12098 |
| `whole[2].backend` | wasm |
| `whole[2].case` | pii66-declared |
| `whole[2].pages` | 66 |
| `whole[2].minimum` | 93.77937 |
| `whole[3].backend` | wasm |
| `whole[3].case` | pii66-default |
| `whole[3].pages` | 66 |
| `whole[3].minimum` | 93.77937 |
| `pairedAudit.pages` | 264 |
| `pairedAudit.below90` | 0 |
| `pairedAudit.develDeclines` | 0 |
| `directReview.scope` | PII original page4 only, current C253. Directly opened both Native/fresh WASM review and overlay. |
| `directReview.page` | 4 |
| `directReview.dpi` | 192 |
| `directReview.coordinate` | CSS96; review raster192 |
| `directReview.nativeRaw` | 100.00000 |
| `directReview.wasmRaw` | 98.89537 |
| `directReview.observations[0]` | Right cell item symbol and dash start now follow independent Print and original cell content ownership. |
| `directReview.observations[1]` | Same rows, ordered original text and source cells retained by formal regression. |
| `directReview.observations[2]` | Thin underline/table frame and glyph antialiasing residuals remain; no pixel identity claim. |
| `directReview.whole52OrPrAcceptance` | False |
| `residual` | Other documents devel declines remain. Underline, table frame and antialiasing residuals remain. Not pixel-identical. |
| `fullRegression.exitCode` | 0 |
| `fullRegression.seconds` | 808.0450995409628 |
| `fullRegression.command[0]` | cargo |
| `fullRegression.command[1]` | nextest |
| `fullRegression.command[2]` | run |
| `fullRegression.command[3]` | --locked |
| `fullRegression.command[4]` | --cargo-profile |
| `fullRegression.command[5]` | release-test |
| `fullRegression.command[6]` | --tests |
| `fullRegression.command[7]` | --build-jobs |
| `fullRegression.command[8]` | 8 |
| `fullRegression.command[9]` | --test-threads |
| `fullRegression.command[10]` | 8 |
| `fullRegression.command[11]` | --no-fail-fast |
| `fullRegression.source` | source.json |
| `fullRegression.passed` | 10938 |
| `fullRegression.failed` | 0 |
| `fullRegression.skipped` | 45 |
| `fullRegression.testRuntimeSeconds` | 405.452 |
| `fullRegression.scope` | Current C253 working-source default-feature full integration suite; not final PR head or whole52 visual acceptance. |
| `files.wasm-declared-before-all-pages.tsv` | ca0f13e6585627cc65febb2e11a863112ef9c8a7e2ae3e670ab3f6952f0d014f |
| `files.native-p004-overlay.png` | 377c2279a57895b8ab8372ef0f43e7a3f782f18248cd4bd3128dcef194c947e3 |
| `files.native-pii66-default-all-pages.tsv` | c9377e35c9dae4b9022d82d64fcb0af1632da9fc4abd3b09cc1ded4a4e752661 |
| `files.native-declared-devel-all-pages.tsv` | 0c5f76de198a19820bab56d39baf9740631e109d299311492c94dbcf7daf43a8 |
| `files.wasm-declared-devel-all-pages.tsv` | 917483cefa91dbac56b31a371b78a5b79714f149a53fc0d3a280d2d48172948e |
| `files.native-pii66-declared-all-pages.tsv` | c9377e35c9dae4b9022d82d64fcb0af1632da9fc4abd3b09cc1ded4a4e752661 |
| `files.wasm-pii66-declared-all-pages.tsv` | cc8ef16bd1680fd07dc2ed98b821856ff6f0aab54b11b2e560fd7afc3e87872f |
| `files.wasm-p004-review.png` | e06bedd86ac2dcd08a755b43ed247bbe1401859fdc5c58243b246a97a505f946 |
| `files.native-declared-before-all-pages.tsv` | d2365a40b7e56115b6cae7dda98b43564273e28115632882ae41763a05267fcc |
| `files.wasm-p004-overlay.png` | 8424c7422eeb4c0c2480f61f1d20d2c90f741b64f94372bba0942cebc9766d16 |
| `files.wasm-pii66-default-all-pages.tsv` | cc8ef16bd1680fd07dc2ed98b821856ff6f0aab54b11b2e560fd7afc3e87872f |
| `files.native-p004-review.png` | 01189cb6662bc6e872f157001314c9bf6d18173d9fb41a07708d4711cb770ab4 |
| `buildSourceLedgerSha256` | 6c2c468f3e3df727a710cb8b597e8e6f118232566f352e0b97b68543af97524a |
| `sourceLedgerLocalPath` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-6ed1682c-20261018/dash-spacing-repair-c253/source.json |
| `coordinateRegression.page` | 4 |
| `coordinateRegression.renderDpi` | 192 |
| `coordinateRegression.coordinate` | CSS96 |
| `coordinateRegression.independentOrigin[0]` | 544.96 |
| `coordinateRegression.independentOrigin[1]` | 193.28 |
| `coordinateRegression.tolerancePerAxis` | 2.0 |
| `coordinateRegression.observedCanvasOrigin[0]` | 545.0888671875 |
| `coordinateRegression.observedCanvasOrigin[1]` | 193.24000549316406 |
| `coordinateRegression.beforeCanvasOrigin[0]` | 539.559998 |
| `coordinateRegression.beforeCanvasOrigin[1]` | 193.240005 |
| `coordinateRegression.beforeRasterWitness` | C249 frozen C240 trace, previously recorded 5.4CSSpx horizontal difference. The original Cargo before-case failed the word-spacing relation, not a separately executed coordinate assertion. |
| `coordinateRegression.defaultFocused.passed` | 1 |
| `coordinateRegression.defaultFocused.failed` | 0 |
| `coordinateRegression.defaultFocused.seconds` | .166 |
| `coordinateRegression.nativeSkiaFocused.passed` | 1 |
| `coordinateRegression.nativeSkiaFocused.failed` | 0 |
| `coordinateRegression.nativeSkiaFocused.seconds` | .168 |
| `coordinateRegression.testSourceSha256` | f48b6c73e02011a2207ad308dc593271e264feb0d89871760774beeef1dd83d5 |
| `coordinateRegression.runtimeSourceUnchangedFromC253` | True |
| `coordinateRegression.fullSuiteStatus` | 10938 PASS/45 skip on C253 before this test-only added assertion; final exact-head full validation pending. |


## 보존한 전체 TSV

- [native-declared-before-all-pages.tsv](native-declared-before-all-pages.tsv)
- [native-declared-devel-all-pages.tsv](native-declared-devel-all-pages.tsv)
- [native-pii66-declared-all-pages.tsv](native-pii66-declared-all-pages.tsv)
- [native-pii66-default-all-pages.tsv](native-pii66-default-all-pages.tsv)
- [wasm-declared-before-all-pages.tsv](wasm-declared-before-all-pages.tsv)
- [wasm-declared-devel-all-pages.tsv](wasm-declared-devel-all-pages.tsv)
- [wasm-pii66-declared-all-pages.tsv](wasm-pii66-declared-all-pages.tsv)
- [wasm-pii66-default-all-pages.tsv](wasm-pii66-default-all-pages.tsv)
