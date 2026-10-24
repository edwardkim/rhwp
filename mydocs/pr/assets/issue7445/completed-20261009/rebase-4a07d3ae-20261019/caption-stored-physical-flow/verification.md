# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## before-after-pages

이전 생성 파일: `before-after-pages.json` · 3993 bytes · SHA-256 `557e7a8aff891b01d44ac2429904608be4272c475fa7754c81be5b682917a497`.

| 항목 | 당시 기록 |
|---|---|
| `[0].backend` | native |
| `[0].page` | 1 |
| `[0].before` | 99.02719 |
| `[0].after` | 99.02719 |
| `[0].decline` | 0.00000 |
| `[1].backend` | native |
| `[1].page` | 2 |
| `[1].before` | 99.89730 |
| `[1].after` | 99.89730 |
| `[1].decline` | 0.00000 |
| `[2].backend` | native |
| `[2].page` | 3 |
| `[2].before` | 78.94771 |
| `[2].after` | 99.94409 |
| `[2].decline` | -20.99638 |
| `[3].backend` | native |
| `[3].page` | 4 |
| `[3].before` | 99.86161 |
| `[3].after` | 99.86161 |
| `[3].decline` | 0.00000 |
| `[4].backend` | native |
| `[4].page` | 5 |
| `[4].before` | 99.87986 |
| `[4].after` | 99.87986 |
| `[4].decline` | 0.00000 |
| `[5].backend` | native |
| `[5].page` | 6 |
| `[5].before` | 99.90852 |
| `[5].after` | 99.90852 |
| `[5].decline` | 0.00000 |
| `[6].backend` | native |
| `[6].page` | 7 |
| `[6].before` | 99.90771 |
| `[6].after` | 99.90771 |
| `[6].decline` | 0.00000 |
| `[7].backend` | native |
| `[7].page` | 8 |
| `[7].before` | 38.04003 |
| `[7].after` | 99.38620 |
| `[7].decline` | -61.34617 |
| `[8].backend` | native |
| `[8].page` | 9 |
| `[8].before` | 99.93300 |
| `[8].after` | 99.93300 |
| `[8].decline` | 0.00000 |
| `[9].backend` | native |
| `[9].page` | 10 |
| `[9].before` | 99.89409 |
| `[9].after` | 99.89409 |
| `[9].decline` | 0.00000 |
| `[10].backend` | native |
| `[10].page` | 11 |
| `[10].before` | 99.91055 |
| `[10].after` | 99.91055 |
| `[10].decline` | 0.00000 |
| `[11].backend` | native |
| `[11].page` | 12 |
| `[11].before` | 99.90452 |
| `[11].after` | 99.90452 |
| `[11].decline` | 0.00000 |
| `[12].backend` | native |
| `[12].page` | 13 |
| `[12].before` | 99.92704 |
| `[12].after` | 99.92704 |
| `[12].decline` | 0.00000 |
| `[13].backend` | native |
| `[13].page` | 14 |
| `[13].before` | 99.78295 |
| `[13].after` | 99.78295 |
| `[13].decline` | 0.00000 |
| `[14].backend` | native |
| `[14].page` | 15 |
| `[14].before` | 99.84094 |
| `[14].after` | 99.84094 |
| `[14].decline` | 0.00000 |
| `[15].backend` | native |
| `[15].page` | 16 |
| `[15].before` | 99.94268 |
| `[15].after` | 99.94268 |
| `[15].decline` | 0.00000 |
| `[16].backend` | wasm |
| `[16].page` | 1 |
| `[16].before` | 98.92066 |
| `[16].after` | 98.92066 |
| `[16].decline` | 0.00000 |
| `[17].backend` | wasm |
| `[17].page` | 2 |
| `[17].before` | 99.89913 |
| `[17].after` | 99.89913 |
| `[17].decline` | 0.00000 |
| `[18].backend` | wasm |
| `[18].page` | 3 |
| `[18].before` | 78.97684 |
| `[18].after` | 99.94480 |
| `[18].decline` | -20.96796 |
| `[19].backend` | wasm |
| `[19].page` | 4 |
| `[19].before` | 99.86849 |
| `[19].after` | 99.86849 |
| `[19].decline` | 0.00000 |
| `[20].backend` | wasm |
| `[20].page` | 5 |
| `[20].before` | 99.88252 |
| `[20].after` | 99.88252 |
| `[20].decline` | 0.00000 |
| `[21].backend` | wasm |
| `[21].page` | 6 |
| `[21].before` | 99.91030 |
| `[21].after` | 99.91030 |
| `[21].decline` | 0.00000 |
| `[22].backend` | wasm |
| `[22].page` | 7 |
| `[22].before` | 99.90827 |
| `[22].after` | 99.90827 |
| `[22].decline` | 0.00000 |
| `[23].backend` | wasm |
| `[23].page` | 8 |
| `[23].before` | 38.13085 |
| `[23].after` | 99.91881 |
| `[23].decline` | -61.78796 |
| `[24].backend` | wasm |
| `[24].page` | 9 |
| `[24].before` | 99.93429 |
| `[24].after` | 99.93429 |
| `[24].decline` | 0.00000 |
| `[25].backend` | wasm |
| `[25].page` | 10 |
| `[25].before` | 99.89859 |
| `[25].after` | 99.89859 |
| `[25].decline` | 0.00000 |
| `[26].backend` | wasm |
| `[26].page` | 11 |
| `[26].before` | 99.91087 |
| `[26].after` | 99.91087 |
| `[26].decline` | 0.00000 |
| `[27].backend` | wasm |
| `[27].page` | 12 |
| `[27].before` | 99.90544 |
| `[27].after` | 99.90544 |
| `[27].decline` | 0.00000 |
| `[28].backend` | wasm |
| `[28].page` | 13 |
| `[28].before` | 99.92553 |
| `[28].after` | 99.92553 |
| `[28].decline` | 0.00000 |
| `[29].backend` | wasm |
| `[29].page` | 14 |
| `[29].before` | 99.86077 |
| `[29].after` | 99.86077 |
| `[29].decline` | 0.00000 |
| `[30].backend` | wasm |
| `[30].page` | 15 |
| `[30].before` | 99.86918 |
| `[30].after` | 99.86918 |
| `[30].decline` | 0.00000 |
| `[31].backend` | wasm |
| `[31].page` | 16 |
| `[31].before` | 99.94179 |
| `[31].after` | 99.94179 |
| `[31].decline` | 0.00000 |

## original-before-after

이전 생성 파일: `original-before-after.json` · 113 bytes · SHA-256 `dfbdfa69a56ae5611015c6946af2f0644a1fd507462cf9ec70e97c2fd0c56289`.

| 항목 | 당시 기록 |
|---|---|
| `[0].backend` | native |
| `[0].rowsEqual` | True |
| `[1].backend` | wasm |
| `[1].rowsEqual` | True |

## title-origin-summary

이전 생성 파일: `title-origin-summary.json` · 1657 bytes · SHA-256 `bd44bc61771faf349a585cb7c6543d54e00dcbe11dd820b7bd3b454adbb90e6a`.

| 항목 | 당시 기록 |
|---|---|
| `[0].format` | hwp |
| `[0].page` | 12 |
| `[0].text` | 4. |
| `[0].xCss` | 113.37855333333333 |
| `[0].baselineCss` | 152.63319333333342 |
| `[1].format` | hwp |
| `[1].page` | 12 |
| `[1].text` | 작업 |
| `[1].xCss` | 145.68104666666667 |
| `[1].baselineCss` | 152.63319333333342 |
| `[2].format` | hwp |
| `[2].page` | 12 |
| `[2].text` | 할당 |
| `[2].xCss` | 208.04724666666667 |
| `[2].baselineCss` | 152.63319333333342 |
| `[3].format` | hwp |
| `[3].page` | 12 |
| `[3].text` | 방식 |
| `[3].xCss` | 270.41344666666663 |
| `[3].baselineCss` | 152.63319333333342 |
| `[4].format` | hwp |
| `[4].page` | 12 |
| `[4].text` | (Scheduling |
| `[4].xCss` | 320.94606 |
| `[4].baselineCss` | 152.63319333333342 |
| `[5].format` | hwp |
| `[5].page` | 12 |
| `[5].text` | Algorithms) |
| `[5].xCss` | 475.1025133333333 |
| `[5].baselineCss` | 152.63319333333342 |
| `[6].format` | hwpx |
| `[6].page` | 12 |
| `[6].text` | 4. |
| `[6].xCss` | 113.37855333333333 |
| `[6].baselineCss` | 152.63319333333342 |
| `[7].format` | hwpx |
| `[7].page` | 12 |
| `[7].text` | 작업 |
| `[7].xCss` | 145.68104666666667 |
| `[7].baselineCss` | 152.63319333333342 |
| `[8].format` | hwpx |
| `[8].page` | 12 |
| `[8].text` | 할당 |
| `[8].xCss` | 208.04724666666667 |
| `[8].baselineCss` | 152.63319333333342 |
| `[9].format` | hwpx |
| `[9].page` | 12 |
| `[9].text` | 방식 |
| `[9].xCss` | 270.41344666666663 |
| `[9].baselineCss` | 152.63319333333342 |
| `[10].format` | hwpx |
| `[10].page` | 12 |
| `[10].text` | (Scheduling |
| `[10].xCss` | 320.94606 |
| `[10].baselineCss` | 152.63319333333342 |
| `[11].format` | hwpx |
| `[11].page` | 12 |
| `[11].text` | Algorithms) |
| `[11].xCss` | 475.1025133333333 |
| `[11].baselineCss` | 152.63319333333342 |

## validation

이전 생성 파일: `validation.json` · 21887 bytes · SHA-256 `62ed2c484b740d4098c7312807c489798bbdd15ab7c7a8ca0819d4b3ce46d598`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | b5424f50c718954ea9bb183aef9944e1d4abba56 |
| `sourceFiles.src/renderer/hwp3_caption_host.rs` | 3c43947e747d6af08fdecfc4cdf0e75537184bf8b730849cffcce37e9031eba0 |
| `sourceFiles.src/renderer/layout/shape_item.inc.rs` | acd5a02cf6f2c9bd339bfe5db751395793a6d56d00824bf64a819884bcf33d51 |
| `sourceFiles.src/renderer/typeset/section/controls.rs` | a049aeb58d31cf64d914e1b576a334d5e617bc0e9fd5464660c411f35943bdbd |
| `testSha256` | dacfc5b1e63c15bc7d568e7e5dc27943b77dcdc7d26b95eea6fa71cc91e9c689 |
| `native.cliSha256` | 46fe072293852341616922cd05e963214b581d45d5b4204d6b598dbd6b94525e |
| `native.librarySha256` | 2fc5dec3d9e0bbc869bdb3a921c36f07e18749357934aefaffa028487adaf32b |
| `wasm.head` | b5424f50c718954ea9bb183aef9944e1d4abba56 |
| `wasm.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair541-caption-physical-flow-final-fresh-wasm/source.json |
| `wasm.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `wasm.hashes.rhwp_bg.wasm` | 3810a9dffc2f7908d4cb32082ada23e67d196e1881e6b67074b677d23380a24b |
| `wasm.rootPackageRecompiled` | True |
| `wasm.wrapperInvoked` | True |
| `wasm.changesSincePreviousWasm[0]` | src/renderer/hwp3_caption_host.rs |
| `wasm.changesSincePreviousWasm[1]` | src/renderer/layout/shape_item.inc.rs |
| `wasm.changesSincePreviousWasm[2]` | src/renderer/typeset/section/controls.rs |
| `wasm.rootPkgStudioPublicMatch` | True |
| `wasm.buildExitCode` | 0 |
| `wasm.seconds` | 330.87651109695435 |
| `wasm.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `wasm.actualBrowserInitialization` | pending |
| `whole[0].backend` | native |
| `whole[0].key` | issue265-original |
| `whole[0].counts.rhwp` | 16 |
| `whole[0].counts.pdf` | 16 |
| `whole[0].minimum` | 95.34478 |
| `whole[0].worstPage` | 6 |
| `whole[0].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/native/issue265-original/silhouette.tsv |
| `whole[0].directReading.representativePagesRead[0]` | 3 |
| `whole[0].directReading.representativePagesRead[1]` | 6 |
| `whole[0].directReading.residualsRecorded` | True |
| `whole[0].directReading.allPagesManuallyRead` | False |
| `whole[1].backend` | native |
| `whole[1].key` | issue265-hwp-save |
| `whole[1].counts.rhwp` | 16 |
| `whole[1].counts.pdf` | 16 |
| `whole[1].minimum` | 99.02719 |
| `whole[1].worstPage` | 1 |
| `whole[1].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/native/issue265-hwp-save/silhouette.tsv |
| `whole[1].directReading.representativePagesRead[0]` | 1 |
| `whole[1].directReading.representativePagesRead[1]` | 3 |
| `whole[1].directReading.representativePagesRead[2]` | 8 |
| `whole[1].directReading.representativePagesRead[3]` | 12 |
| `whole[1].directReading.residualsRecorded` | True |
| `whole[1].directReading.allPagesManuallyRead` | False |
| `whole[2].backend` | native |
| `whole[2].key` | issue265-hwpx-save |
| `whole[2].counts.rhwp` | 16 |
| `whole[2].counts.pdf` | 16 |
| `whole[2].minimum` | 99.03102 |
| `whole[2].worstPage` | 1 |
| `whole[2].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/native/issue265-hwpx-save/silhouette.tsv |
| `whole[2].directReading.representativePagesRead[0]` | 8 |
| `whole[2].directReading.residualsRecorded` | True |
| `whole[2].directReading.allPagesManuallyRead` | False |
| `whole[3].backend` | wasm |
| `whole[3].key` | issue265-original |
| `whole[3].counts.rhwp` | 16 |
| `whole[3].counts.pdf` | 16 |
| `whole[3].minimum` | 95.33483 |
| `whole[3].worstPage` | 4 |
| `whole[3].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/wasm/issue265-original/silhouette.tsv |
| `whole[3].directReading.representativePagesRead[0]` | 4 |
| `whole[3].directReading.residualsRecorded` | True |
| `whole[3].directReading.allPagesManuallyRead` | False |
| `whole[4].backend` | wasm |
| `whole[4].key` | issue265-hwp-save |
| `whole[4].counts.rhwp` | 16 |
| `whole[4].counts.pdf` | 16 |
| `whole[4].minimum` | 98.92066 |
| `whole[4].worstPage` | 1 |
| `whole[4].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/wasm/issue265-hwp-save/silhouette.tsv |
| `whole[4].directReading.representativePagesRead[0]` | 1 |
| `whole[4].directReading.representativePagesRead[1]` | 8 |
| `whole[4].directReading.residualsRecorded` | True |
| `whole[4].directReading.allPagesManuallyRead` | False |
| `whole[5].backend` | wasm |
| `whole[5].key` | issue265-hwpx-save |
| `whole[5].counts.rhwp` | 16 |
| `whole[5].counts.pdf` | 16 |
| `whole[5].minimum` | 98.92493 |
| `whole[5].worstPage` | 1 |
| `whole[5].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair542-caption-physical-flow-final-whole/wasm/issue265-hwpx-save/silhouette.tsv |
| `whole[5].directReading.representativePagesRead[0]` | 3 |
| `whole[5].directReading.representativePagesRead[1]` | 8 |
| `whole[5].directReading.representativePagesRead[2]` | 12 |
| `whole[5].directReading.residualsRecorded` | True |
| `whole[5].directReading.allPagesManuallyRead` | False |
| `focused.exitCode` | 100 |
| `focused.seconds` | 297.149123875075 |
| `focused.summary[0]` |      Summary [  48.180s] 30 tests run: 28 passed, 2 failed, 1702 skipped |
| `focused.sourceUnchanged` | True |
| `focused.scope` | Final physical caption flow candidate: current prepared Native focused suite including new p8 ownership/Print regression, original/save font contracts, source equations/footer controls and two remaining full-regression failures. Not full nextest. |
| `regression.name` | caption_host_followed_by_text_keeps_the_table_and_its_following_body |
| `regression.before` | 1FAIL |
| `regression.after` | 1PASS |
| `regression.logs` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair545-caption-chain-regression |
| `browserInitialization.browser` | Chrome/155.0.8059.85 |
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
| `browserInitialization.rows[1].bytes` | 22646511 |
| `browserInitialization.rows[1].servedSha256` | 3810a9dffc2f7908d4cb32082ada23e67d196e1881e6b67074b677d23380a24b |
| `browserInitialization.rows[1].frozenSha256` | 3810a9dffc2f7908d4cb32082ada23e67d196e1881e6b67074b677d23380a24b |
| `browserInitialization.rows[1].matchesFrozenBuild` | True |
| `browserInitialization.rows[1].method` | raw-byte-identity |
| `browserInitialization.wasmInitialized` | True |
| `browserInitialization.scope` | initialization only; no document paint or actual36 acceptance |
| `directReading.native[0]` | original p3/6 |
| `directReading.native[1]` | hwp save p1/3/8/12 |
| `directReading.native[2]` | hwpx save p8 |
| `directReading.wasm[0]` | original p4 |
| `directReading.wasm[1]` | hwp save p1/8 |
| `directReading.wasm[2]` | hwpx save p3/8/12 |
| `directReading.residuals[0]` | saved p8 caption number3 vs PDF1 remains |
| `directReading.residuals[1]` | original glyph/spacing and footer-logo outline differences |
| `fullPrAcceptance` | False |
| `remaining[0]` | Saved AutoNumber source/serializer mismatch |
| `remaining[1]` | issue1105 p23/24 boundary |
| `remaining[2]` | off_canvas partition15 issue1948 cross-para fieldend +1 |
| `remaining[3]` | Science p3 fixed-base decline0.32810pp |
| `remaining[4]` | Final full nextest/gates/corpus/actual36 |


## 보존한 전체 TSV

- [native-issue265-hwp-save-all-pages.tsv](native-issue265-hwp-save-all-pages.tsv)
- [native-issue265-hwpx-save-all-pages.tsv](native-issue265-hwpx-save-all-pages.tsv)
- [native-issue265-original-all-pages.tsv](native-issue265-original-all-pages.tsv)
- [wasm-issue265-hwp-save-all-pages.tsv](wasm-issue265-hwp-save-all-pages.tsv)
- [wasm-issue265-hwpx-save-all-pages.tsv](wasm-issue265-hwpx-save-all-pages.tsv)
- [wasm-issue265-original-all-pages.tsv](wasm-issue265-original-all-pages.tsv)
