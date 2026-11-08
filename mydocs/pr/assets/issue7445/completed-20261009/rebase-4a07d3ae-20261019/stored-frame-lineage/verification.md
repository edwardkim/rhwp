# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## all-pages-logical-comparison

이전 생성 파일: `all-pages-logical-comparison.json` · 966 bytes · SHA-256 `8e588690a8f0f6ead5e5615ada0302fd81ec436a8364d1447c9a23bb44742fdc`.

| 항목 | 당시 기록 |
|---|---|
| `pages` | 383 |
| `savedRuntimeChanged[0]` | 294 |
| `beforeSourceSavedDifferences[0]` | 3 |
| `beforeSourceSavedDifferences[1]` | 4 |
| `beforeSourceSavedDifferences[2]` | 6 |
| `beforeSourceSavedDifferences[3]` | 7 |
| `beforeSourceSavedDifferences[4]` | 8 |
| `beforeSourceSavedDifferences[5]` | 11 |
| `beforeSourceSavedDifferences[6]` | 13 |
| `beforeSourceSavedDifferences[7]` | 15 |
| `beforeSourceSavedDifferences[8]` | 17 |
| `beforeSourceSavedDifferences[9]` | 19 |
| `beforeSourceSavedDifferences[10]` | 21 |
| `beforeSourceSavedDifferences[11]` | 23 |
| `beforeSourceSavedDifferences[12]` | 25 |
| `beforeSourceSavedDifferences[13]` | 29 |
| `beforeSourceSavedDifferences[14]` | 92 |
| `beforeSourceSavedDifferences[15]` | 94 |
| `beforeSourceSavedDifferences[16]` | 95 |
| `beforeSourceSavedDifferences[17]` | 128 |
| `beforeSourceSavedDifferences[18]` | 131 |
| `beforeSourceSavedDifferences[19]` | 140 |
| `beforeSourceSavedDifferences[20]` | 146 |
| `beforeSourceSavedDifferences[21]` | 147 |
| `beforeSourceSavedDifferences[22]` | 161 |
| `beforeSourceSavedDifferences[23]` | 162 |
| `beforeSourceSavedDifferences[24]` | 163 |
| `beforeSourceSavedDifferences[25]` | 166 |
| `beforeSourceSavedDifferences[26]` | 185 |
| `beforeSourceSavedDifferences[27]` | 193 |
| `beforeSourceSavedDifferences[28]` | 245 |
| `beforeSourceSavedDifferences[29]` | 248 |
| `beforeSourceSavedDifferences[30]` | 273 |
| `beforeSourceSavedDifferences[31]` | 274 |
| `beforeSourceSavedDifferences[32]` | 294 |
| `beforeSourceSavedDifferences[33]` | 309 |
| `afterSourceSavedDifferences[0]` | 3 |
| `afterSourceSavedDifferences[1]` | 4 |
| `afterSourceSavedDifferences[2]` | 6 |
| `afterSourceSavedDifferences[3]` | 7 |
| `afterSourceSavedDifferences[4]` | 8 |
| `afterSourceSavedDifferences[5]` | 11 |
| `afterSourceSavedDifferences[6]` | 13 |
| `afterSourceSavedDifferences[7]` | 15 |
| `afterSourceSavedDifferences[8]` | 17 |
| `afterSourceSavedDifferences[9]` | 19 |
| `afterSourceSavedDifferences[10]` | 21 |
| `afterSourceSavedDifferences[11]` | 23 |
| `afterSourceSavedDifferences[12]` | 25 |
| `afterSourceSavedDifferences[13]` | 29 |
| `afterSourceSavedDifferences[14]` | 92 |
| `afterSourceSavedDifferences[15]` | 94 |
| `afterSourceSavedDifferences[16]` | 95 |
| `afterSourceSavedDifferences[17]` | 128 |
| `afterSourceSavedDifferences[18]` | 131 |
| `afterSourceSavedDifferences[19]` | 140 |
| `afterSourceSavedDifferences[20]` | 146 |
| `afterSourceSavedDifferences[21]` | 147 |
| `afterSourceSavedDifferences[22]` | 161 |
| `afterSourceSavedDifferences[23]` | 162 |
| `afterSourceSavedDifferences[24]` | 163 |
| `afterSourceSavedDifferences[25]` | 166 |
| `afterSourceSavedDifferences[26]` | 185 |
| `afterSourceSavedDifferences[27]` | 193 |
| `afterSourceSavedDifferences[28]` | 245 |
| `afterSourceSavedDifferences[29]` | 248 |
| `afterSourceSavedDifferences[30]` | 273 |
| `afterSourceSavedDifferences[31]` | 274 |
| `afterSourceSavedDifferences[32]` | 309 |
| `sameSavedInputSha256` | True |
| `savedInputSha256` | 16b9510359b23c3be3ce25ae6190448101446b3ecbb4ed0c54e29b0c8320fabb |
| `scope` | logical rounded render-tree JSON across all383 pages; not Visual Sweep or independent Print acceptance |

## fresh-wasm-producer

이전 생성 파일: `fresh-wasm-producer.json` · 767 bytes · SHA-256 `22db54118e793740cd7c5e963c314c2a3157e140d5b654623155886d2362c73d`.

| 항목 | 당시 기록 |
|---|---|
| `head` | bd21c14458d138eeb613b5a53710fcb056894a19 |
| `sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair499-3930-stored-lineage-fresh-wasm/source.json |
| `hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `hashes.rhwp_bg.wasm` | 148d57807c079abbc263eba4bbf79561ce163e42d2b5ea385973404873ed134c |
| `rootPackageRecompiled` | True |
| `wrapperInvoked` | True |
| `changesSincePreviousWasm[0]` | src/renderer/typeset/table/block/whole_fit.rs |
| `rootPkgStudioPublicMatch` | True |
| `buildExitCode` | 0 |
| `seconds` | 224.66906213760376 |
| `mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `actualBrowserInitialization` | pending |

## native-artifacts

이전 생성 파일: `native-artifacts.json` · 173 bytes · SHA-256 `2887a955fcdc0e20698bed4be81aa5c72c1ab37af2bfeebff19c6755a5e8f8c4`.

| 항목 | 당시 기록 |
|---|---|
| `cliSha256` | 54c0e59a008403d0232350d7bb99d97ea1a3a5afbb2e73580832fd15249ca577 |
| `librarySha256` | a51645961be8273a1b80bd1b6e0400b1bf5d10a6f4907f44a7a60c85167985af |

## native-source

이전 생성 파일: `native-source.json` · 406 bytes · SHA-256 `7f528c289d4223098df36ffdfe3f7c7b82f17d9b0baf5ab52a94ec3b1d0be222`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | bd21c14458d138eeb613b5a53710fcb056894a19 |
| `files.src/renderer/typeset/table/block/whole_fit.rs` | 4efffe2d75e61ccd41fb97034c0e5288dd77d9b067859e82b2bb8b301b916f65 |
| `patchSha256` | 90e7429110c2269ab91258600d2605b26b7d1e49abf74cbc6fbd61bdcb629cf8 |
| `scope` | whole stored RowBreak frame follows layout lineage across HWPX-to-HWP; diagnostic trace removed; not accepted |

## validation

이전 생성 파일: `validation.json` · 9231 bytes · SHA-256 `502a574831627ad4593842e35295188400a59ea37345575aa706680f2ba8650b`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | bd21c14458d138eeb613b5a53710fcb056894a19 |
| `base` | 4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa |
| `sourceFile` | src/renderer/typeset/table/block/whole_fit.rs |
| `sourceSha256` | 4efffe2d75e61ccd41fb97034c0e5288dd77d9b067859e82b2bb8b301b916f65 |
| `input.source` | samples/2025 행정업무운영 편람(최종).hwpx |
| `input.sourceSha256` | c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6 |
| `input.savedSha256` | 16b9510359b23c3be3ce25ae6190448101446b3ecbb4ed0c54e29b0c8320fabb |
| `input.savedReproduction` | existing issue3930 test route: HwpDocument::from_bytes, render source, export_hwp_with_adapter, reload; ignored saved binary is unchanged before/after |
| `pdf.path` | pdf/issue3930-stored-lineage-saved-20261020-2020.pdf |
| `pdf.sha256` | 56183a15a3cf8f3c5adfb0976afefaf94ee3258a9ce697fdafd1056a6d882076 |
| `pdf.pages` | 383 |
| `pdf.engine` | 2020 |
| `pdf.version` | 11.0.0.9136 |
| `pdf.mode` | hancom2020_pdf_driver_one_up |
| `pdf.printMethod` | 0 |
| `pdf.timeoutSeconds` | 1800 |
| `pdf.jobId` | 4543a382-fad5-4801-8a4e-386c8d517d57 |
| `visual.page` | 294 |
| `visual.dpi` | 192 |
| `visual.compat` | 2022 |
| `visual.pdfRenderer` | PDFium smooth_rgb_v1 |
| `visual.rawTolerancePx` | 2 |
| `visual.minimum` | 90 |
| `visual.counts[0]` | 383 |
| `visual.counts[1]` | 383 |
| `visual.wholeDocumentAcceptance` | False |
| `visual.nativeBefore` | 92.38356 |
| `visual.nativeAfter` | 96.74475 |
| `visual.wasmBefore` | 92.99938 |
| `visual.wasmAfter` | 98.88533 |
| `visual.newPixelPinEligible` | False |
| `visual.directReading` | Q26 table and body origin restored; unchanged Q24/Q25 and page ownership; Native typeface weight/line spacing residuals remain |
| `gates.nativeOfficial.exitCode` | 0 |
| `gates.nativeOfficial.seconds` | 265.2554615000263 |
| `gates.nativeOfficial.summary[0]` |      Summary [   5.196s] 35 tests run: 35 passed, 1730 skipped |
| `gates.nativeOfficial.sourceUnchanged` | True |
| `gates.nativeOfficial.scope` | Native original3930 unchanged tests plus3820 band/7336 page-larger and original1937 controls; not full nextest |
| `gates.manifest` | [RustTestSuite] 확인 완료: 1643 sources / 7175 static test attrs / 28 suites + 20 exceptions = 48/48 integration targets / nextest 최소 6288 cases / weight 1470216..1475190 |
| `gates.fmt` | check completed without diagnostic |
| `browserInitialization.head` | bd21c14458d138eeb613b5a53710fcb056894a19 |
| `browserInitialization.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `browserInitialization.browserVersion` | Chrome/155.0.8059.85 |
| `browserInitialization.producer.head` | bd21c14458d138eeb613b5a53710fcb056894a19 |
| `browserInitialization.producer.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair499-3930-stored-lineage-fresh-wasm/source.json |
| `browserInitialization.producer.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `browserInitialization.producer.hashes.rhwp_bg.wasm` | 148d57807c079abbc263eba4bbf79561ce163e42d2b5ea385973404873ed134c |
| `browserInitialization.producer.rootPackageRecompiled` | True |
| `browserInitialization.producer.wrapperInvoked` | True |
| `browserInitialization.producer.changesSincePreviousWasm[0]` | src/renderer/typeset/table/block/whole_fit.rs |
| `browserInitialization.producer.rootPkgStudioPublicMatch` | True |
| `browserInitialization.producer.buildExitCode` | 0 |
| `browserInitialization.producer.seconds` | 224.66906213760376 |
| `browserInitialization.producer.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `browserInitialization.producer.actualBrowserInitialization` | pending |
| `browserInitialization.rows[0].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp.js |
| `browserInitialization.rows[0].httpMethod` | GET |
| `browserInitialization.rows[0].resourceType` | script |
| `browserInitialization.rows[0].status` | 200 |
| `browserInitialization.rows[0].fromCache` | False |
| `browserInitialization.rows[0].bytes` | 2705563 |
| `browserInitialization.rows[0].servedSha256` | f2294e7a48873a5ffe66aa3409d96be9a7d191fce5973dbf6b514faa09cc929b |
| `browserInitialization.rows[0].frozenSha256` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `browserInitialization.rows[0].matchesFrozenBuild` | True |
| `browserInitialization.rows[0].sourceMapContentSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `browserInitialization.rows[0].expectedAssetRewriteSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `browserInitialization.rows[0].viteAssetRewrite.original` | new URL('rhwp_bg.wasm', import.meta.url) |
| `browserInitialization.rows[0].viteAssetRewrite.transformed` | new URL("/@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm", '' + import.meta.url) |
| `browserInitialization.rows[0].viteAssetRewrite.wasmPath` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `browserInitialization.rows[0].viteAssetRewrite.sourceMapSource` | rhwp.js |
| `browserInitialization.rows[0].method` | vite-inline-source-map-exact-asset-url-rewrite |
| `browserInitialization.rows[1].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `browserInitialization.rows[1].httpMethod` | GET |
| `browserInitialization.rows[1].resourceType` | fetch |
| `browserInitialization.rows[1].status` | 200 |
| `browserInitialization.rows[1].fromCache` | False |
| `browserInitialization.rows[1].bytes` | 22644998 |
| `browserInitialization.rows[1].servedSha256` | 148d57807c079abbc263eba4bbf79561ce163e42d2b5ea385973404873ed134c |
| `browserInitialization.rows[1].frozenSha256` | 148d57807c079abbc263eba4bbf79561ce163e42d2b5ea385973404873ed134c |
| `browserInitialization.rows[1].matchesFrozenBuild` | True |
| `browserInitialization.rows[1].method` | raw-byte-identity |
| `browserInitialization.wasmInitialized` | True |
| `browserInitialization.scope` | actual Studio7736 initialization responses only; not document paint, fonts, CanvasKit readiness or actual36 acceptance |
| `scope` | Restored original3930 source/saved stored-frame contract. Not whole383 visual, full regression or actual36 acceptance. |


## 보존한 전체 TSV

- [after-native-p294-silhouette.tsv](after-native-p294-silhouette.tsv)
- [after-wasm-p294-silhouette.tsv](after-wasm-p294-silhouette.tsv)
- [before-native-p294-silhouette.tsv](before-native-p294-silhouette.tsv)
- [before-wasm-p294-silhouette.tsv](before-wasm-p294-silhouette.tsv)
