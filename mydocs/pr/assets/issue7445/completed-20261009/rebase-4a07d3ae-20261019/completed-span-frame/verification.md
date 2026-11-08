# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## validation

이전 생성 파일: `validation.json` · 9370 bytes · SHA-256 `e5420f3ec261cbbe975381c5180f888de26c19a337e0441bfd70e9a1cf1b125d`.

| 항목 | 당시 기록 |
|---|---|
| `base` | 4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa |
| `parent` | 979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea |
| `runtimeSource.parent` | 979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea |
| `runtimeSource.files.src/renderer/layout/table_layout.rs` | 64f475828732378c14fa39edc0d5604d457ca0a64b837d19d54bf82548555b15 |
| `runtimeSource.files.src/renderer/layout/table_layout/row_block_cut.rs` | 83546aafdcd85a3d4c278ea7fa22a1d6b10c618bc61417ab20979bfa0e93ecb5 |
| `runtimeSource.files.src/renderer/layout/table_partial.rs` | 981716d6b2d7c4f01888ce4be457dbe89e817a220189575e7b82633bf1052dc7 |
| `runtimeSource.files.src/renderer/typeset/table/scan.rs` | 3ff9e2fb01949a175000c464eb71c149dd19128f73613be0cfa4ebc5ba67cfb7 |
| `runtimeSource.patchSha256` | b13b25576a73b157338bb7f157ab6e2e3c547729c595a6bdf11350cce5b12e42 |
| `runtimeSource.scope` | shared actual block cut and accepted source row-frame alignment candidate; not accepted |
| `nativeArtifacts.cliSha256` | 69c8415332b89002a8673b497428e299719dae9c54f8ffef58fd13a33a326574 |
| `nativeArtifacts.librarySha256` | 1abf354e2d767d2cd4720f79c3beec8d2d3fe13984f9a055e7627ba47ea63169 |
| `wasm.head` | 979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea |
| `wasm.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair486-shared-block-cut-fresh-wasm/source.json |
| `wasm.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `wasm.hashes.rhwp_bg.wasm` | d398095ccd6e711726eb03482df50cbe4ff7595ecd54a835d8cb2b92ec8a4761 |
| `wasm.rootPackageRecompiled` | True |
| `wasm.wrapperInvoked` | True |
| `wasm.changesSincePreviousWasm[0]` | src/renderer/layout/table_layout.rs |
| `wasm.changesSincePreviousWasm[1]` | src/renderer/layout/table_layout/row_block_cut.rs |
| `wasm.changesSincePreviousWasm[2]` | src/renderer/layout/table_partial.rs |
| `wasm.changesSincePreviousWasm[3]` | src/renderer/typeset/table/scan.rs |
| `wasm.rootPkgStudioPublicMatch` | True |
| `wasm.buildExitCode` | 0 |
| `wasm.seconds` | 267.25586891174316 |
| `wasm.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `wasm.actualBrowserInitialization` | pending |
| `sourceInput` | samples/issue1937_rowbreak_footnote_overpagination.hwp |
| `inputSha256` | a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990 |
| `referencePdf` | pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf |
| `pdfSha256` | 7f14bc85eb389a68cc88bd40fcafeb7a638d8454293a30bb2c8fe884a16978eb |
| `canonicalToolSha256` | 69d9d56394b26fcb7a64b41588a378da745475b3297bae89fd6b4d16903cede4 |
| `dpi` | 192 |
| `layoutCoordinateDpi` | 96 |
| `compat` | 2022 |
| `rawTolerancePx` | 2 |
| `threshold` | 90 |
| `rasterizer` | current canonical visual_sweep_pdf.py/PDFium smooth_rgb_v1 |
| `backends.native.pages` | 50 |
| `backends.native.minimum` | 94.90661 |
| `backends.native.selected.9` | 99.88474 |
| `backends.native.selected.13` | 99.99969 |
| `backends.native.selected.37` | 96.16765 |
| `backends.native.selected.38` | 98.55731 |
| `backends.native.selected.43` | 99.99482 |
| `backends.native.previousProducer` | repair469-first-frame-native-whole |
| `backends.native.changed[0].page` | 38 |
| `backends.native.changed[0].before` | 97.13861 |
| `backends.native.changed[0].after` | 98.55731 |
| `backends.native.changed[0].delta` | 1.41870 |
| `backends.native.producer` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair482-shared-block-cut-native/source.json |
| `backends.wasm.pages` | 50 |
| `backends.wasm.minimum` | 94.96082 |
| `backends.wasm.selected.9` | 99.88906 |
| `backends.wasm.selected.13` | 99.99970 |
| `backends.wasm.selected.37` | 96.18486 |
| `backends.wasm.selected.38` | 98.52943 |
| `backends.wasm.selected.43` | 99.99482 |
| `backends.wasm.previousProducer` | repair471-first-frame-wasm-whole |
| `backends.wasm.changed[0].page` | 38 |
| `backends.wasm.changed[0].before` | 97.11528 |
| `backends.wasm.changed[0].after` | 98.52943 |
| `backends.wasm.changed[0].delta` | 1.41415 |
| `backends.wasm.producer` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair482-shared-block-cut-native/source.json |
| `controls.pages` | 508 |
| `controls.renderTreeJsonFiles` | 512 |
| `controls.changed` | 0 |
| `controls.notVisualSweep` | True |
| `status` | focused correction verified; full nextest/remaining original-PR regression repairs and complete PR validation pending |
| `scope` | completed spanning suffix alignment; page37 prefix residual is not claimed fixed; whole #7445 is not claimed solved |
| `officialFocusedGate.exitCode` | 0 |
| `officialFocusedGate.seconds` | 191.53091887501068 |
| `officialFocusedGate.summary[0]` |      Summary [   0.425s] 18 tests run: 18 passed, 1246 skipped |
| `officialFocusedGate.sourceUnchanged` | True |
| `officialFocusedGate.scope` | Native official case plus five control cases named by original PR1976; not full nextest |
| `actualStudioInitialization.head` | 979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea |
| `actualStudioInitialization.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `actualStudioInitialization.browserVersion` | Chrome/155.0.8059.85 |
| `actualStudioInitialization.producer.head` | 979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea |
| `actualStudioInitialization.producer.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair486-shared-block-cut-fresh-wasm/source.json |
| `actualStudioInitialization.producer.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `actualStudioInitialization.producer.hashes.rhwp_bg.wasm` | d398095ccd6e711726eb03482df50cbe4ff7595ecd54a835d8cb2b92ec8a4761 |
| `actualStudioInitialization.producer.rootPackageRecompiled` | True |
| `actualStudioInitialization.producer.wrapperInvoked` | True |
| `actualStudioInitialization.producer.changesSincePreviousWasm[0]` | src/renderer/layout/table_layout.rs |
| `actualStudioInitialization.producer.changesSincePreviousWasm[1]` | src/renderer/layout/table_layout/row_block_cut.rs |
| `actualStudioInitialization.producer.changesSincePreviousWasm[2]` | src/renderer/layout/table_partial.rs |
| `actualStudioInitialization.producer.changesSincePreviousWasm[3]` | src/renderer/typeset/table/scan.rs |
| `actualStudioInitialization.producer.rootPkgStudioPublicMatch` | True |
| `actualStudioInitialization.producer.buildExitCode` | 0 |
| `actualStudioInitialization.producer.seconds` | 267.25586891174316 |
| `actualStudioInitialization.producer.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `actualStudioInitialization.producer.actualBrowserInitialization` | pending |
| `actualStudioInitialization.rows[0].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp.js |
| `actualStudioInitialization.rows[0].httpMethod` | GET |
| `actualStudioInitialization.rows[0].resourceType` | script |
| `actualStudioInitialization.rows[0].status` | 200 |
| `actualStudioInitialization.rows[0].fromCache` | False |
| `actualStudioInitialization.rows[0].bytes` | 2705563 |
| `actualStudioInitialization.rows[0].servedSha256` | f2294e7a48873a5ffe66aa3409d96be9a7d191fce5973dbf6b514faa09cc929b |
| `actualStudioInitialization.rows[0].frozenSha256` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `actualStudioInitialization.rows[0].matchesFrozenBuild` | True |
| `actualStudioInitialization.rows[0].sourceMapContentSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `actualStudioInitialization.rows[0].expectedAssetRewriteSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `actualStudioInitialization.rows[0].viteAssetRewrite.original` | new URL('rhwp_bg.wasm', import.meta.url) |
| `actualStudioInitialization.rows[0].viteAssetRewrite.transformed` | new URL("/@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm", '' + import.meta.url) |
| `actualStudioInitialization.rows[0].viteAssetRewrite.wasmPath` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `actualStudioInitialization.rows[0].viteAssetRewrite.sourceMapSource` | rhwp.js |
| `actualStudioInitialization.rows[0].method` | vite-inline-source-map-exact-asset-url-rewrite |
| `actualStudioInitialization.rows[1].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `actualStudioInitialization.rows[1].httpMethod` | GET |
| `actualStudioInitialization.rows[1].resourceType` | fetch |
| `actualStudioInitialization.rows[1].status` | 200 |
| `actualStudioInitialization.rows[1].fromCache` | False |
| `actualStudioInitialization.rows[1].bytes` | 22645001 |
| `actualStudioInitialization.rows[1].servedSha256` | d398095ccd6e711726eb03482df50cbe4ff7595ecd54a835d8cb2b92ec8a4761 |
| `actualStudioInitialization.rows[1].frozenSha256` | d398095ccd6e711726eb03482df50cbe4ff7595ecd54a835d8cb2b92ec8a4761 |
| `actualStudioInitialization.rows[1].matchesFrozenBuild` | True |
| `actualStudioInitialization.rows[1].method` | raw-byte-identity |
| `actualStudioInitialization.wasmInitialized` | True |
| `actualStudioInitialization.scope` | actual Studio7736 initialization responses only; not document paint, fonts, CanvasKit readiness or actual36 acceptance |
| `directReading.native38` | 완료 꼬리 Center·표 경계·후속 내용 직접 판독 |
| `directReading.freshWasm38` | 같은 영역 직접 판독 |
| `directReading.nativeFreshWasm9and13` | 중요 종료·이어받기 괘선·마지막 행·쪽번호 직접 판독; Native는 신규PNG와 직전 직접 판독한PNG 동일성 확인 |
| `directReading.notClaimed` | 37쪽 앞 조각 기존 잔여 또는 전체 #7445 개선 |
| `nativeSelectedPngIdentity[0].page` | 9 |
| `nativeSelectedPngIdentity[0].previousPngSha256` | 038c24d2b5e9f9b5ca972da45a35dd63bbc83e5d4cc2518471ed5a7fdc915911 |
| `nativeSelectedPngIdentity[0].currentPngSha256` | 038c24d2b5e9f9b5ca972da45a35dd63bbc83e5d4cc2518471ed5a7fdc915911 |
| `nativeSelectedPngIdentity[0].same` | True |
| `nativeSelectedPngIdentity[1].page` | 13 |
| `nativeSelectedPngIdentity[1].previousPngSha256` | 0586e4d969deeb6213eca839108473785f4db0be99a41f4faedabb835fe9b8c5 |
| `nativeSelectedPngIdentity[1].currentPngSha256` | 0586e4d969deeb6213eca839108473785f4db0be99a41f4faedabb835fe9b8c5 |
| `nativeSelectedPngIdentity[1].same` | True |


## 보존한 전체 TSV

- [native.tsv](native.tsv)
- [wasm.tsv](wasm.tsv)
