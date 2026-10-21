# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## validation

이전 생성 파일: `validation.json` · 10594 bytes · SHA-256 `5c30a29ad86e5fe3e2a8776d40bbe0f3b9b02052d32662162d706cec7f9316bd`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | d3cd374b84f5b9f8c46bf51f59d444ec06555424 |
| `source.src/renderer/layout/text_measurement.rs` | c8312296fc1d01f18e92b798a10423d433b8d9a68a1aa6e8a611be86f802beb1 |
| `testSha256` | 0aad780a7a1153ddcbdab9e0681868bb7a2139376e4bad757f1f1bf337761229 |
| `input.path` | samples/hwpx/issue1948_cross_para_fieldend.hwpx |
| `input.sha256` | 38244a9ce101e1b24f012bf856bdb5bdc2e006632542b66813a615488cbf0066 |
| `pdf.path` | pdf/hwpx/issue1948_cross_para_fieldend-hwpx-2020.pdf |
| `pdf.sha256` | e300ed55e58565dd2af6755562b8bec00e5b188c638132e57c073b14fbc676d8 |
| `pdf.creator` | Hwp2022 |
| `pdf.scope` | existing valid independent Hancom Print; no PDF replaced |
| `nativeBuild.command[0]` | cargo |
| `nativeBuild.command[1]` | build |
| `nativeBuild.command[2]` | --locked |
| `nativeBuild.command[3]` | --profile |
| `nativeBuild.command[4]` | release-test |
| `nativeBuild.command[5]` | --features |
| `nativeBuild.command[6]` | native-skia |
| `nativeBuild.command[7]` | --bin |
| `nativeBuild.command[8]` | rhwp |
| `nativeBuild.command[9]` | --lib |
| `nativeBuild.exitCode` | 0 |
| `nativeBuild.seconds` | 139.65586749999784 |
| `nativeBuild.sourceUnchanged` | True |
| `freshWasm.head` | d3cd374b84f5b9f8c46bf51f59d444ec06555424 |
| `freshWasm.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair554-zero-size-reflow-space-fresh-wasm/source.json |
| `freshWasm.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `freshWasm.hashes.rhwp_bg.wasm` | 41fe6753ca2f1486839d67d08a2d984266658d42af98a5dd835db2dc77e69324 |
| `freshWasm.rootPackageRecompiled` | True |
| `freshWasm.wrapperInvoked` | True |
| `freshWasm.changesSincePreviousWasm[0]` | src/renderer/layout/text_measurement.rs |
| `freshWasm.rootPkgStudioPublicMatch` | True |
| `freshWasm.buildExitCode` | 0 |
| `freshWasm.seconds` | 215.51824688911438 |
| `freshWasm.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `freshWasm.actualBrowserInitialization` | pending |
| `focused.exitCode` | 100 |
| `focused.seconds` | 323.05600754194893 |
| `focused.summary[0]` |      Summary [  46.791s] 32 tests run: 31 passed, 1 failed, 1697 skipped |
| `focused.sourceUnchanged` | True |
| `focused.scope` | Zero-size source notice and public measurement controls, previous caption/font/equation/footer/source controls, two known full-regression failures. Native-only focused; not full nextest. |
| `finalPixelFocused.exitCode` | 0 |
| `finalPixelFocused.seconds` | 9.606437499984168 |
| `finalPixelFocused.summary[0]` |      Summary [   0.032s] 2 tests run: 2 passed, 258 skipped |
| `finalPixelFocused.sourceUnchanged` | True |
| `finalPixelFocused.scope` | Zero-size source notice ownership plus completed p1 independent Print ±2 CSSpx pin, explicit0/unresolved/positive superscript controls. Current prepared Native suite only; not full nextest. |
| `beforeAfter.before` | 2FAIL |
| `beforeAfter.after` | 2PASS |
| `beforeAfter.beforeLog` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair560-final-zero-size-before-after/before.log |
| `beforeAfter.afterLog` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair560-final-zero-size-before-after/after.log |
| `offCanvas.fixedBase` | 0 |
| `offCanvas.before` | 1 |
| `offCanvas.after` | 0 |
| `sourceZeroNotice.paragraph` | 10 |
| `sourceZeroNotice.charShape` | 14 |
| `sourceZeroNotice.baseSize` | 0 |
| `sourceZeroNotice.textPreserved` | True |
| `sourceZeroNotice.afterWidth` | 0 |
| `criticalPixel.source` | paragraph10/char7 digit2 |
| `criticalPixel.xCss` | 131.1289333333 |
| `criticalPixel.baselineCss` | 679.417492 |
| `criticalPixel.toleranceCss` | 2.0 |
| `criticalPixel.independentObservation` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair549-fieldend-offcanvas-diagnostic/independent-attachment-origins.json |
| `browser.version` | Chrome/155.0.8059.85 |
| `browser.rows[0].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp.js |
| `browser.rows[0].httpMethod` | GET |
| `browser.rows[0].resourceType` | script |
| `browser.rows[0].status` | 200 |
| `browser.rows[0].fromCache` | False |
| `browser.rows[0].bytes` | 2705563 |
| `browser.rows[0].servedSha256` | f2294e7a48873a5ffe66aa3409d96be9a7d191fce5973dbf6b514faa09cc929b |
| `browser.rows[0].frozenSha256` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `browser.rows[0].matchesFrozenBuild` | True |
| `browser.rows[0].sourceMapContentSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `browser.rows[0].expectedAssetRewriteSha256` | 319f1364be37ceb61752deeda2a8da9c1ed11505eb080c5f506edabc08356f5e |
| `browser.rows[0].viteAssetRewrite.original` | new URL('rhwp_bg.wasm', import.meta.url) |
| `browser.rows[0].viteAssetRewrite.transformed` | new URL("/@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm", '' + import.meta.url) |
| `browser.rows[0].viteAssetRewrite.wasmPath` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `browser.rows[0].viteAssetRewrite.sourceMapSource` | rhwp.js |
| `browser.rows[0].method` | vite-inline-source-map-exact-asset-url-rewrite |
| `browser.rows[1].pathname` | /@fs/Users/tsjang/rhwp-7445-completed-20261009/pkg/rhwp_bg.wasm |
| `browser.rows[1].httpMethod` | GET |
| `browser.rows[1].resourceType` | fetch |
| `browser.rows[1].status` | 200 |
| `browser.rows[1].fromCache` | False |
| `browser.rows[1].bytes` | 22646522 |
| `browser.rows[1].servedSha256` | 41fe6753ca2f1486839d67d08a2d984266658d42af98a5dd835db2dc77e69324 |
| `browser.rows[1].frozenSha256` | 41fe6753ca2f1486839d67d08a2d984266658d42af98a5dd835db2dc77e69324 |
| `browser.rows[1].matchesFrozenBuild` | True |
| `browser.rows[1].method` | raw-byte-identity |
| `browser.wasmInitialized` | True |
| `browser.scope` | actual Studio7736 JS/WASM initialization only, not actual36 paint/font readiness |
| `manualReading.pages[0]` | 1 |
| `manualReading.beforeAndAfterNative` | True |
| `manualReading.afterWasm` | True |
| `manualReading.findings` | attachment2/body positions restored, hidden source notice keeps zero advance; main paragraph spacing and emblem residuals remain |
| `remaining[0]` | issue1105 p23/24 actual split-word mismatch against valid Hancom Print |
| `remaining[1]` | saved issue265 caption number differs from independent Print |
| `remaining[2]` | Science p3 fixed-base decline0.32810pp |
| `remaining[3]` | Final full regression/gates/whole corpus/actual36 |
| `fullPrAcceptance` | False |


## 보존한 전체 TSV

- [after-native-all-pages.tsv](after-native-all-pages.tsv)
- [after-wasm-all-pages.tsv](after-wasm-all-pages.tsv)
- [base-native-all-pages.tsv](base-native-all-pages.tsv)
- [before-native-all-pages.tsv](before-native-all-pages.tsv)
- [before-wasm-all-pages.tsv](before-wasm-all-pages.tsv)
