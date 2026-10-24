# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## evidence

이전 생성 파일: `evidence.json` · 96002 bytes · SHA-256 `6223078dc308e6bf831a9476ade854467f6252dba949c241b3c201eb71e33a37`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | 16abb05616 |
| `sourceChanged` | False |
| `testSha256` | 8e8fde37543695072dfb7ac4c712dae7f32ae5ab7900becd1507719d54851340 |
| `sourceProducer.head` | 16abb05616 |
| `sourceProducer.sourceLedger.src/renderer/typeset/paragraph/content.rs` | 92ea61b01402615951ebcbbedef3ecbb8bd52a3647cfcdf8542276313b73c9fd |
| `sourceProducer.nativeSha256` | 40cc14568b1fd71704522b7547102a03df3251790f3b456e9d67c872f13a80d2 |
| `sourceProducer.wasmProducer.head` | f437aa89fc783a96673afc90ec423c3b817854b6 |
| `sourceProducer.wasmProducer.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair791-junction-fresh-wasm/source.json |
| `sourceProducer.wasmProducer.hashes.rhwp.js` | 63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524 |
| `sourceProducer.wasmProducer.hashes.rhwp_bg.wasm` | 92b78ea0f0b30da37181daeb9981a26863fa6645d8a4189cfcb091ed756ff5c1 |
| `sourceProducer.wasmProducer.rootPackageRecompiled` | True |
| `sourceProducer.wasmProducer.wrapperInvoked` | True |
| `sourceProducer.wasmProducer.rootPkgStudioPublicMatch` | True |
| `sourceProducer.wasmProducer.buildExitCode` | 0 |
| `sourceProducer.wasmProducer.seconds` | 198.01042995799799 |
| `sourceProducer.wasmProducer.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `sourceProducer.wasmProducer.actualBrowserInitialization` | pending |
| `sourceProducer.case.key` | pii66-declared |
| `sourceProducer.case.source` | samples/task2097/75544_pii_bunseok.hwpx |
| `sourceProducer.case.pdf` | pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf |
| `sourceProducer.case.pages` | 66 |
| `sourceProducer.case.review[0]` | 59 |
| `sourceProducer.case.review[1]` | 60 |
| `sourceProducer.case.review[2]` | 66 |
| `sourceProducer.case.environment` | mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json |
| `sourceProducer.case.compat` | 2022 |
| `sourceProducer.case.dpi` | 192 |
| `sourceProducer.case.sourceSha256` | f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1 |
| `sourceProducer.case.pdfSha256` | 0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770 |
| `sourceProducer.case.kind` | original |
| `sourceProducer.case.latestCandidateValidation` | pending |
| `sourceProducer.case.latestHeadValidation` | pending |
| `sourceProducer.case.environmentSha256` | 21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544 |
| `sourceProducer.case.fontProgramScope.includeOriginalHftSymbolOverlay` | False |
| `sourceProducer.case.fontProgramScope.sourceDeclarationEvidence` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4296fa04-20261018/font-scope-diagnostic-c163/source-fonts.json |
| `sourceProducer.case.fontProgramScope.reason` | No matching declared HFT source face; glyph-only HFT review resource must not advertise a complete TTF primary family |
| `sourceProducer.toolSha256` | 69d9d56394b26fcb7a64b41588a378da745475b3297bae89fd6b4d16903cede4 |
| `sourceProducer.pdfRasterizer.program` | pypdfium2 |
| `sourceProducer.pdfRasterizer.backend` | pdfium |
| `sourceProducer.pdfRasterizer.policy` | pdfium_smooth_rgb_v1 |
| `sourceProducer.pdfRasterizer.antialias.text` | True |
| `sourceProducer.pdfRasterizer.antialias.image` | True |
| `sourceProducer.pdfRasterizer.antialias.path` | True |
| `sourceProducer.pdfRasterizer.script_sha256` | eda5cb9561ea417f55efdf2a36510a5db2563d4382adf2b5ef564389f6736faf |
| `sourceProducer.pdfRasterizer.available` | True |
| `sourceProducer.pdfRasterizer.executable` | /Users/tsjang/.local/share/uv/python/cpython-3.12.11-macos-aarch64-none/bin/python3.12 |
| `sourceProducer.pdfRasterizer.executable_sha256` | dcebc9a450a63fcc1801995f64c59705dd5eaddb26bcd817b7bfcee5d9b7300e |
| `sourceProducer.pdfRasterizer.library` | /Users/tsjang/.cache/uv/archive-v0/ccqc8baXFndV2x3B1p1yk/lib/python3.12/site-packages/pypdfium2_raw/libpdfium.dylib |
| `sourceProducer.pdfRasterizer.library_sha256` | 33c98063af28c0b7cbf8227f4422bf5c15942df2455cf7f0a5dce3dc601d52b0 |
| `sourceProducer.pdfRasterizer.version` | 153.0.7999.0 |
| `sourceProducer.pdfRasterizer.wrapper_version` | 5.13.0 |
| `sourceProducer.pdfRasterizer.version_exit_code` | 0 |
| `sourceProducer.scope` | Stage21 current source Native787/fresh WASM791 whole66; declared original font environment. Required qualification before changing shared-border paint identity test. No font/metric guards bypassed. |
| `wholeResults[0].backend` | native |
| `wholeResults[0].counts.rhwp` | 66 |
| `wholeResults[0].counts.pdf` | 66 |
| `wholeResults[0].minimum` | 94.49334 |
| `wholeResults[0].worstPage` | 32 |
| `wholeResults[0].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair800-pii-current-whole/native/pii66-declared/silhouette.tsv |
| `wholeResults[0].directReading` | pending |
| `wholeResults[1].backend` | wasm |
| `wholeResults[1].counts.rhwp` | 66 |
| `wholeResults[1].counts.pdf` | 66 |
| `wholeResults[1].minimum` | 94.53407 |
| `wholeResults[1].worstPage` | 32 |
| `wholeResults[1].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair800-pii-current-whole/wasm/pii66-declared/silhouette.tsv |
| `wholeResults[1].directReading` | pending |
| `before.pid` | 62950 |
| `before.complete` | True |
| `before.exitCode` | 101 |
| `after.exitCode` | 0 |
| `after.testSha256` | 8e8fde37543695072dfb7ac4c712dae7f32ae5ab7900becd1507719d54851340 |
| `after.summary[0]` | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s |
| `official.pid` | 70153 |
| `official.complete` | True |
| `official.exitCode` | 0 |
| `official.seconds` | 12.79623031616211 |
| `official.testSha256` | 8e8fde37543695072dfb7ac4c712dae7f32ae5ab7900becd1507719d54851340 |
| `directReading.nativePages[0]` | 28 |
| `directReading.wasmPages[0]` | 28 |
| `directReading.types[0]` | review |
| `directReading.types[1]` | overlay |
| `directReading.findings` | Short gray boundary and adjacent black boundary preserved, lower-cell white overpaint retained; text/edge residuals remain. |
| `fullPrAcceptance` | False |


## 보존한 전체 TSV

- [native-whole66.tsv](native-whole66.tsv)
- [wasm-whole66.tsv](wasm-whole66.tsv)
