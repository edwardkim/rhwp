# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## evidence

이전 생성 파일: `evidence.json` · 9602 bytes · SHA-256 `dc63ccdc486adc8361e10aa831942533b21ea10e993690882df41abceada71cc`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | f437aa89fc783a96673afc90ec423c3b817854b6 |
| `base` | 2e3b4f880bd11d35398e6ea0100846611f13a72d |
| `files.src/renderer/typeset/paragraph/content.rs` | 92ea61b01402615951ebcbbedef3ecbb8bd52a3647cfcdf8542276313b73c9fd |
| `files.tests/cases/issue_7445_stored_multicolumn_flow.rs` | c5339682c3fd6a84abcea35dc4c3bb173ba8b6af704c6a1c46b4cd1ec26cfa2a |
| `nativeLedger.parent` | f437aa89fc783a96673afc90ec423c3b817854b6 |
| `nativeLedger.files.src/renderer/typeset/paragraph/content.rs` | 92ea61b01402615951ebcbbedef3ecbb8bd52a3647cfcdf8542276313b73c9fd |
| `nativeLedger.patchSha256` | 23c35e4b354433078e1e6d3e171ba4b5fa7406ff0b0e2294d674974e8d175655 |
| `nativeLedger.scope` | Stage21 repair787: certify the adjoining original TAC successor when transferring stored physical content end; preserve ordinary multicolumn fit advancement. Correct actual shortcut regression; independent prior whole7 min91.78141 in786. New tests and fresh outputs pending. |
| `wasmProducer.head` | f437aa89fc783a96673afc90ec423c3b817854b6 |
| `wasmProducer.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair791-junction-fresh-wasm/source.json |
| `wasmProducer.hashes.rhwp.js` | 63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524 |
| `wasmProducer.hashes.rhwp_bg.wasm` | 92b78ea0f0b30da37181daeb9981a26863fa6645d8a4189cfcb091ed756ff5c1 |
| `wasmProducer.rootPackageRecompiled` | True |
| `wasmProducer.wrapperInvoked` | True |
| `wasmProducer.rootPkgStudioPublicMatch` | True |
| `wasmProducer.buildExitCode` | 0 |
| `wasmProducer.seconds` | 198.01042995799799 |
| `wasmProducer.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `wasmProducer.actualBrowserInitialization` | pending |
| `scope[0].label` | shortcut-native |
| `scope[0].origin` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair790-junction-native-shortcut-whole/native/junction |
| `scope[0].tsvSha256` | 8eda872dd60236344063633408c44c6fb6e8d67dfa31125c59597c12db0de96c |
| `scope[0].rows` | 7 |
| `scope[1].label` | shortcut-wasm |
| `scope[1].origin` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair793-junction-wasm-shortcut-whole/wasm/junction |
| `scope[1].tsvSha256` | d482114d99b2f5caff6919026a5363b156ae616a4df0b64acbf54ec9dfa7b152 |
| `scope[1].rows` | 7 |
| `scope[2].label` | science-native |
| `scope[2].origin` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair789-junction-native-science-whole/native/science |
| `scope[2].tsvSha256` | 2ae182628d4b1fbbc29ff5f1c7a8f7eabc415fc77e5aad1d2862a43fdba7f476 |
| `scope[2].rows` | 4 |
| `scope[3].label` | science-wasm |
| `scope[3].origin` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair792-junction-wasm-science-whole/wasm/science |
| `scope[3].tsvSha256` | 62d9be842e43e065e895dee0c30060b16d17b297924fedfc8f4dc1d9db90c81e |
| `scope[3].rows` | 4 |
| `sciencePngComparisons[0].backend` | native |
| `sciencePngComparisons[0].page` | 1 |
| `sciencePngComparisons[0].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair789-junction-native-science-whole/native/science/rhwp_png/rhwp_001.png |
| `sciencePngComparisons[0].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair772-native-science-stored-flow-whole/native/science/rhwp_png/rhwp_001.png |
| `sciencePngComparisons[0].sha256` | af039a45c7a9cbd6ea2ad26ca3dd811a9162eae5b403ba2dfa18414e30e43d33 |
| `sciencePngComparisons[0].previousSha256` | af039a45c7a9cbd6ea2ad26ca3dd811a9162eae5b403ba2dfa18414e30e43d33 |
| `sciencePngComparisons[0].byteIdentical` | True |
| `sciencePngComparisons[1].backend` | native |
| `sciencePngComparisons[1].page` | 2 |
| `sciencePngComparisons[1].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair789-junction-native-science-whole/native/science/rhwp_png/rhwp_002.png |
| `sciencePngComparisons[1].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair772-native-science-stored-flow-whole/native/science/rhwp_png/rhwp_002.png |
| `sciencePngComparisons[1].sha256` | 600339b7af05845ff938b4de3a5d5ddafa0210164c5d852c217133fc93b7809d |
| `sciencePngComparisons[1].previousSha256` | 600339b7af05845ff938b4de3a5d5ddafa0210164c5d852c217133fc93b7809d |
| `sciencePngComparisons[1].byteIdentical` | True |
| `sciencePngComparisons[2].backend` | native |
| `sciencePngComparisons[2].page` | 3 |
| `sciencePngComparisons[2].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair789-junction-native-science-whole/native/science/rhwp_png/rhwp_003.png |
| `sciencePngComparisons[2].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair772-native-science-stored-flow-whole/native/science/rhwp_png/rhwp_003.png |
| `sciencePngComparisons[2].sha256` | 09f7fb5d759f600a5b8b1e2ac372c70419dead55c6826483bd9d096e1a304c98 |
| `sciencePngComparisons[2].previousSha256` | 09f7fb5d759f600a5b8b1e2ac372c70419dead55c6826483bd9d096e1a304c98 |
| `sciencePngComparisons[2].byteIdentical` | True |
| `sciencePngComparisons[3].backend` | native |
| `sciencePngComparisons[3].page` | 4 |
| `sciencePngComparisons[3].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair789-junction-native-science-whole/native/science/rhwp_png/rhwp_004.png |
| `sciencePngComparisons[3].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair772-native-science-stored-flow-whole/native/science/rhwp_png/rhwp_004.png |
| `sciencePngComparisons[3].sha256` | 616da855d1c711556e2080edb488e5aa959351bb46f278bb0ef7708f74954632 |
| `sciencePngComparisons[3].previousSha256` | 616da855d1c711556e2080edb488e5aa959351bb46f278bb0ef7708f74954632 |
| `sciencePngComparisons[3].byteIdentical` | True |
| `sciencePngComparisons[4].backend` | wasm |
| `sciencePngComparisons[4].page` | 1 |
| `sciencePngComparisons[4].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair792-junction-wasm-science-whole/wasm/science/rhwp_png/rhwp_001.png |
| `sciencePngComparisons[4].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair775-wasm-science-stored-flow-whole/wasm/science/rhwp_png/rhwp_001.png |
| `sciencePngComparisons[4].sha256` | f2807755dd15514f19394bcaca114dc191f4614162d3ebf3aab04424e0a05fee |
| `sciencePngComparisons[4].previousSha256` | f2807755dd15514f19394bcaca114dc191f4614162d3ebf3aab04424e0a05fee |
| `sciencePngComparisons[4].byteIdentical` | True |
| `sciencePngComparisons[5].backend` | wasm |
| `sciencePngComparisons[5].page` | 2 |
| `sciencePngComparisons[5].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair792-junction-wasm-science-whole/wasm/science/rhwp_png/rhwp_002.png |
| `sciencePngComparisons[5].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair775-wasm-science-stored-flow-whole/wasm/science/rhwp_png/rhwp_002.png |
| `sciencePngComparisons[5].sha256` | 393f5a9b7e074438026ca282f8145c0f86fd4b624221d4cf29fbb9c1daae7233 |
| `sciencePngComparisons[5].previousSha256` | 393f5a9b7e074438026ca282f8145c0f86fd4b624221d4cf29fbb9c1daae7233 |
| `sciencePngComparisons[5].byteIdentical` | True |
| `sciencePngComparisons[6].backend` | wasm |
| `sciencePngComparisons[6].page` | 3 |
| `sciencePngComparisons[6].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair792-junction-wasm-science-whole/wasm/science/rhwp_png/rhwp_003.png |
| `sciencePngComparisons[6].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair775-wasm-science-stored-flow-whole/wasm/science/rhwp_png/rhwp_003.png |
| `sciencePngComparisons[6].sha256` | 6b8fd6cbfd6e66064205d7a0ff27af3c79c4ac50c6c0cc8554630824d455aa0d |
| `sciencePngComparisons[6].previousSha256` | 6b8fd6cbfd6e66064205d7a0ff27af3c79c4ac50c6c0cc8554630824d455aa0d |
| `sciencePngComparisons[6].byteIdentical` | True |
| `sciencePngComparisons[7].backend` | wasm |
| `sciencePngComparisons[7].page` | 4 |
| `sciencePngComparisons[7].current` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair792-junction-wasm-science-whole/wasm/science/rhwp_png/rhwp_004.png |
| `sciencePngComparisons[7].previous` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair775-wasm-science-stored-flow-whole/wasm/science/rhwp_png/rhwp_004.png |
| `sciencePngComparisons[7].sha256` | 6439933b4781aed60429345d10a5e45b7925ec0782088f68522a27d17497c92a |
| `sciencePngComparisons[7].previousSha256` | 6439933b4781aed60429345d10a5e45b7925ec0782088f68522a27d17497c92a |
| `sciencePngComparisons[7].byteIdentical` | True |
| `shortcutDirectReading.nativePages[0]` | 1 |
| `shortcutDirectReading.nativePages[1]` | 7 |
| `shortcutDirectReading.wasmPages[0]` | 7 |
| `shortcutDirectReading.kinds[0]` | review |
| `shortcutDirectReading.kinds[1]` | overlay |
| `shortcutDirectReading.findings` | Both column tails, shortcuts, band, decorative page7 retained. Small dot/ink edge residuals remain; no whole pixel equality claimed. |
| `fullPrAcceptance` | False |
| `beforeAfter.testSha256` | c5339682c3fd6a84abcea35dc4c3bb173ba8b6af704c6a1c46b4cd1ec26cfa2a |
| `beforeAfter.results[0].label` | final-before |
| `beforeAfter.results[0].exitCode` | 101 |
| `beforeAfter.results[0].summary[0]` | test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s |
| `beforeAfter.results[1].label` | final-after |
| `beforeAfter.results[1].exitCode` | 0 |
| `beforeAfter.results[1].summary[0]` | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s |
| `officialNextest.pid` | 13559 |
| `officialNextest.complete` | True |
| `officialNextest.exitCode` | 0 |
| `officialNextest.seconds` | 123.51205801963806 |
| `officialNextest.testsRun` | 5 |
| `officialNextest.passed` | 5 |
| `officialNextest.failed` | 0 |
| `officialNextest.filteredOut` | 243 |
| `officialNextest.executionSeconds` | 0.126 |
| `relatedShortcutPartitions[0].case` | overlap::text_overlaps_do_not_grow_partition_11 |
| `relatedShortcutPartitions[0].exitCode` | 0 |
| `relatedShortcutPartitions[0].summary[0]` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.59s |
| `relatedShortcutPartitions[1].case` | oracle::page_counts_do_not_drift_from_hancom_oracle_partition_2 |
| `relatedShortcutPartitions[1].exitCode` | 0 |
| `relatedShortcutPartitions[1].summary[0]` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.27s |

## independent-shortcut-page7

이전 생성 파일: `independent-shortcut-page7.json` · 501 bytes · SHA-256 `4d793bc821cd9f5ff27eb77be32ade0cda3a7bb855ed28af484356196e3c65db`.

| 항목 | 당시 기록 |
|---|---|
| `pdf` | pdf/basic/shortcut-2022.pdf |
| `pdfSha256` | 9c877ad7aafe78d4734536f0a7b0e985a431534cc9d01d0737a90c106e2f2eb5 |
| `mediaBox[0]` | 0.0 |
| `mediaBox[1]` | 0.0 |
| `mediaBox[2]` | 841.0 |
| `mediaBox[3]` | 595.0 |
| `page` | 7 |
| `coordinateSource` | Original PDF text matrix transformed from PDF points to CSS96, not candidate bbox |
| `anchors[0].text` | 글 앞으로 |
| `anchors[0].x` | 121.28 |
| `anchors[0].baseline` | 461.76 |
| `anchors[1].text` | 개체 풀기 |
| `anchors[1].x` | 589.92 |
| `anchors[1].baseline` | 461.76 |


## 보존한 전체 TSV

- [science-native-whole.tsv](science-native-whole.tsv)
- [science-wasm-whole.tsv](science-wasm-whole.tsv)
- [shortcut-native-whole.tsv](shortcut-native-whole.tsv)
- [shortcut-wasm-whole.tsv](shortcut-wasm-whole.tsv)
