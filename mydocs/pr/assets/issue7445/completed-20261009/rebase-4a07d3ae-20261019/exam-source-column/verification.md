# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## fresh-wasm-producer

이전 생성 파일: `fresh-wasm-producer.json` · 771 bytes · SHA-256 `7599de3e67d152cda32fbef8c9f18bf52e7761aa4c34fbf7430c681c25c92ab1`.

| 항목 | 당시 기록 |
|---|---|
| `head` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair509-exam-source-object-origin-fresh-wasm/source.json |
| `hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `hashes.rhwp_bg.wasm` | 077c60d0406d9a774637f5aba38510fd258439baf3a1b7405e7023b33ec2ead3 |
| `rootPackageRecompiled` | True |
| `wrapperInvoked` | True |
| `changesSincePreviousWasm[0]` | src/renderer/typeset/controls/stored_tac.rs |
| `rootPkgStudioPublicMatch` | True |
| `buildExitCode` | 0 |
| `seconds` | 270.75369024276733 |
| `mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `actualBrowserInitialization` | pending |

## native-artifacts

이전 생성 파일: `native-artifacts.json` · 173 bytes · SHA-256 `829b9bd9582c42d730598afef5d5e095f6dfcdf9c6262a18bef80ba41ec42c58`.

| 항목 | 당시 기록 |
|---|---|
| `cliSha256` | 472e108a13fa176a2c9223937f3f382a71420d5840e6c13e81b9b413ce580867 |
| `librarySha256` | f5b5be982378faa282d67ad7dfd18da1ea73bc8db7106d2e500d2a71c9f3a300 |

## native-base-comparison

이전 생성 파일: `native-base-comparison.json` · 6507 bytes · SHA-256 `3563bd3a3b484dfa526f8d664e5b360d5586ad33302868873ae2621054c3f629`.

| 항목 | 당시 기록 |
|---|---|
| `[0].case` | science |
| `[0].page` | 1 |
| `[0].base` | 98.66658 |
| `[0].candidate` | 98.65358 |
| `[0].decline` | 0.01300 |
| `[0].hold` | False |
| `[1].case` | science |
| `[1].page` | 2 |
| `[1].base` | 97.39951 |
| `[1].candidate` | 98.03442 |
| `[1].decline` | -0.63491 |
| `[1].hold` | False |
| `[2].case` | science |
| `[2].page` | 3 |
| `[2].base` | 95.06581 |
| `[2].candidate` | 94.73771 |
| `[2].decline` | 0.32810 |
| `[2].hold` | True |
| `[3].case` | science |
| `[3].page` | 4 |
| `[3].base` | 93.18598 |
| `[3].candidate` | 93.11518 |
| `[3].decline` | 0.07080 |
| `[3].hold` | False |
| `[4].case` | kor-hwp |
| `[4].page` | 1 |
| `[4].base` | 99.77829 |
| `[4].candidate` | 99.81946 |
| `[4].decline` | -0.04117 |
| `[4].hold` | False |
| `[5].case` | kor-hwp |
| `[5].page` | 2 |
| `[5].base` | 99.27365 |
| `[5].candidate` | 99.90736 |
| `[5].decline` | -0.63371 |
| `[5].hold` | False |
| `[6].case` | kor-hwp |
| `[6].page` | 3 |
| `[6].base` | 98.65859 |
| `[6].candidate` | 99.90062 |
| `[6].decline` | -1.24203 |
| `[6].hold` | False |
| `[7].case` | kor-hwp |
| `[7].page` | 4 |
| `[7].base` | 99.94065 |
| `[7].candidate` | 99.97381 |
| `[7].decline` | -0.03316 |
| `[7].hold` | False |
| `[8].case` | kor-hwp |
| `[8].page` | 5 |
| `[8].base` | 99.75031 |
| `[8].candidate` | 99.77386 |
| `[8].decline` | -0.02355 |
| `[8].hold` | False |
| `[9].case` | kor-hwp |
| `[9].page` | 6 |
| `[9].base` | 99.78391 |
| `[9].candidate` | 99.80964 |
| `[9].decline` | -0.02573 |
| `[9].hold` | False |
| `[10].case` | kor-hwp |
| `[10].page` | 7 |
| `[10].base` | 99.89246 |
| `[10].candidate` | 99.90078 |
| `[10].decline` | -0.00832 |
| `[10].hold` | False |
| `[11].case` | kor-hwp |
| `[11].page` | 8 |
| `[11].base` | 97.97284 |
| `[11].candidate` | 98.01670 |
| `[11].decline` | -0.04386 |
| `[11].hold` | False |
| `[12].case` | kor-hwp |
| `[12].page` | 9 |
| `[12].base` | 99.19365 |
| `[12].candidate` | 99.20139 |
| `[12].decline` | -0.00774 |
| `[12].hold` | False |
| `[13].case` | kor-hwp |
| `[13].page` | 10 |
| `[13].base` | 99.51601 |
| `[13].candidate` | 99.60032 |
| `[13].decline` | -0.08431 |
| `[13].hold` | False |
| `[14].case` | kor-hwp |
| `[14].page` | 11 |
| `[14].base` | 99.66234 |
| `[14].candidate` | 99.64118 |
| `[14].decline` | 0.02116 |
| `[14].hold` | False |
| `[15].case` | kor-hwp |
| `[15].page` | 12 |
| `[15].base` | 99.79798 |
| `[15].candidate` | 99.91891 |
| `[15].decline` | -0.12093 |
| `[15].hold` | False |
| `[16].case` | kor-hwp |
| `[16].page` | 13 |
| `[16].base` | 99.52685 |
| `[16].candidate` | 99.55205 |
| `[16].decline` | -0.02520 |
| `[16].hold` | False |
| `[17].case` | kor-hwp |
| `[17].page` | 14 |
| `[17].base` | 99.79879 |
| `[17].candidate` | 99.83022 |
| `[17].decline` | -0.03143 |
| `[17].hold` | False |
| `[18].case` | kor-hwp |
| `[18].page` | 15 |
| `[18].base` | 99.76839 |
| `[18].candidate` | 99.79496 |
| `[18].decline` | -0.02657 |
| `[18].hold` | False |
| `[19].case` | kor-hwp |
| `[19].page` | 16 |
| `[19].base` | 99.81648 |
| `[19].candidate` | 99.84973 |
| `[19].decline` | -0.03325 |
| `[19].hold` | False |
| `[20].case` | kor-hwp |
| `[20].page` | 17 |
| `[20].base` | 92.32711 |
| `[20].candidate` | 96.27225 |
| `[20].decline` | -3.94514 |
| `[20].hold` | False |
| `[21].case` | kor-hwp |
| `[21].page` | 18 |
| `[21].base` | 99.22355 |
| `[21].candidate` | 99.32852 |
| `[21].decline` | -0.10497 |
| `[21].hold` | False |
| `[22].case` | kor-hwp |
| `[22].page` | 19 |
| `[22].base` | 99.55825 |
| `[22].candidate` | 99.57121 |
| `[22].decline` | -0.01296 |
| `[22].hold` | False |
| `[23].case` | kor-hwp |
| `[23].page` | 20 |
| `[23].base` | 99.83405 |
| `[23].candidate` | 99.86436 |
| `[23].decline` | -0.03031 |
| `[23].hold` | False |
| `[24].case` | kor-hwpx |
| `[24].page` | 1 |
| `[24].base` | 99.77829 |
| `[24].candidate` | 99.81946 |
| `[24].decline` | -0.04117 |
| `[24].hold` | False |
| `[25].case` | kor-hwpx |
| `[25].page` | 2 |
| `[25].base` | 99.27365 |
| `[25].candidate` | 99.90736 |
| `[25].decline` | -0.63371 |
| `[25].hold` | False |
| `[26].case` | kor-hwpx |
| `[26].page` | 3 |
| `[26].base` | 98.65859 |
| `[26].candidate` | 99.90062 |
| `[26].decline` | -1.24203 |
| `[26].hold` | False |
| `[27].case` | kor-hwpx |
| `[27].page` | 4 |
| `[27].base` | 99.94065 |
| `[27].candidate` | 99.97381 |
| `[27].decline` | -0.03316 |
| `[27].hold` | False |
| `[28].case` | kor-hwpx |
| `[28].page` | 5 |
| `[28].base` | 99.75031 |
| `[28].candidate` | 99.77386 |
| `[28].decline` | -0.02355 |
| `[28].hold` | False |
| `[29].case` | kor-hwpx |
| `[29].page` | 6 |
| `[29].base` | 99.78391 |
| `[29].candidate` | 99.80964 |
| `[29].decline` | -0.02573 |
| `[29].hold` | False |
| `[30].case` | kor-hwpx |
| `[30].page` | 7 |
| `[30].base` | 99.89246 |
| `[30].candidate` | 99.90078 |
| `[30].decline` | -0.00832 |
| `[30].hold` | False |
| `[31].case` | kor-hwpx |
| `[31].page` | 8 |
| `[31].base` | 97.97284 |
| `[31].candidate` | 98.01670 |
| `[31].decline` | -0.04386 |
| `[31].hold` | False |
| `[32].case` | kor-hwpx |
| `[32].page` | 9 |
| `[32].base` | 99.19365 |
| `[32].candidate` | 99.20139 |
| `[32].decline` | -0.00774 |
| `[32].hold` | False |
| `[33].case` | kor-hwpx |
| `[33].page` | 10 |
| `[33].base` | 99.51601 |
| `[33].candidate` | 99.60032 |
| `[33].decline` | -0.08431 |
| `[33].hold` | False |
| `[34].case` | kor-hwpx |
| `[34].page` | 11 |
| `[34].base` | 99.66234 |
| `[34].candidate` | 99.64118 |
| `[34].decline` | 0.02116 |
| `[34].hold` | False |
| `[35].case` | kor-hwpx |
| `[35].page` | 12 |
| `[35].base` | 99.79798 |
| `[35].candidate` | 99.91891 |
| `[35].decline` | -0.12093 |
| `[35].hold` | False |
| `[36].case` | kor-hwpx |
| `[36].page` | 13 |
| `[36].base` | 99.52685 |
| `[36].candidate` | 99.55205 |
| `[36].decline` | -0.02520 |
| `[36].hold` | False |
| `[37].case` | kor-hwpx |
| `[37].page` | 14 |
| `[37].base` | 99.79879 |
| `[37].candidate` | 99.83022 |
| `[37].decline` | -0.03143 |
| `[37].hold` | False |
| `[38].case` | kor-hwpx |
| `[38].page` | 15 |
| `[38].base` | 99.76839 |
| `[38].candidate` | 99.79496 |
| `[38].decline` | -0.02657 |
| `[38].hold` | False |
| `[39].case` | kor-hwpx |
| `[39].page` | 16 |
| `[39].base` | 99.81648 |
| `[39].candidate` | 99.84973 |
| `[39].decline` | -0.03325 |
| `[39].hold` | False |
| `[40].case` | kor-hwpx |
| `[40].page` | 17 |
| `[40].base` | 92.32711 |
| `[40].candidate` | 96.27225 |
| `[40].decline` | -3.94514 |
| `[40].hold` | False |
| `[41].case` | kor-hwpx |
| `[41].page` | 18 |
| `[41].base` | 99.22355 |
| `[41].candidate` | 99.32852 |
| `[41].decline` | -0.10497 |
| `[41].hold` | False |
| `[42].case` | kor-hwpx |
| `[42].page` | 19 |
| `[42].base` | 99.55825 |
| `[42].candidate` | 99.57121 |
| `[42].decline` | -0.01296 |
| `[42].hold` | False |
| `[43].case` | kor-hwpx |
| `[43].page` | 20 |
| `[43].base` | 99.83405 |
| `[43].candidate` | 99.86436 |
| `[43].decline` | -0.03031 |
| `[43].hold` | False |

## native-source

이전 생성 파일: `native-source.json` · 431 bytes · SHA-256 `03fda78938996348eddefa7451fc278e46445558dc3f5eee8f4ef0c4b637abe4`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `files.src/renderer/typeset/controls/stored_tac.rs` | c0c00a5f3d4f5dd32aab5e2eb3bfe85e0fd927d2eb0db2dd5f1bcc0ee06ff410 |
| `patchSha256` | 81762297432b9566e870ee0f3f954529d125a16a48833eea0573251122971ba0 |
| `scope` | source-authenticated zero column keeps independent stored object origin across slight text row remeasurement; before/after tests pending |

## producer

이전 생성 파일: `producer.json` · 90038 bytes · SHA-256 `861a0fda8bfe5b0c4d8596d5bb94f7888152398cc3b0d5705c2def21fbfdff08`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair507-exam-source-object-origin-native/source.json |
| `fonts[0]` | /Users/tsjang/Library/Fonts |
| `canonicalToolSha256` | 69d9d56394b26fcb7a64b41588a378da745475b3297bae89fd6b4d16903cede4 |
| `dpi` | 96 |
| `compat` | 2022 |
| `wholeDocument` | True |
| `cases[0].key` | science |
| `cases[0].input` | samples/exam_science.hwp |
| `cases[0].inputSha256` | 22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8 |
| `cases[0].pdf` | pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf |
| `cases[0].pdfSha256` | 1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14 |
| `cases[0].pages` | 4 |
| `cases[1].key` | kor-hwp |
| `cases[1].input` | samples/exam_kor.hwp |
| `cases[1].inputSha256` | 0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69 |
| `cases[1].pdf` | pdf/issue7642-exam-print/exam-03-exam_kor-hwp-print-2020.pdf |
| `cases[1].pdfSha256` | 72341eae4b56ca18894129e244ad00170991d91e7262bd3983c00720cd872972 |
| `cases[1].pages` | 20 |
| `cases[2].key` | kor-hwpx |
| `cases[2].input` | samples/hwpx/exam_kor.hwpx |
| `cases[2].inputSha256` | c71a8991d63d084c18db1c90d2e32865b2af69cae5762b20cb554ee53a539b7c |
| `cases[2].pdf` | pdf/issue7642-exam-print/exam-12-exam_kor-hwpx-print-2020.pdf |
| `cases[2].pdfSha256` | 3328a5c8696aa8884ef3f7035e5fc4813773e0f8a49e1496e7bedf2179b2221f |
| `cases[2].pages` | 20 |
| `scope` | Matched originalPR8064 Print96DPI/raw2px; fixed4a07 Native and candidate Native/freshWASM original three docs; not all7445 acceptance |

## validation

이전 생성 파일: `validation.json` · 16161 bytes · SHA-256 `ff0d833bf8e6eeb2ad5a80a64d6fd2ba02dbe346a6b2d7a6664ee657b350a6da`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `base` | 4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa |
| `sourceFile` | src/renderer/typeset/controls/stored_tac.rs |
| `sourceSha256` | c0c00a5f3d4f5dd32aab5e2eb3bfe85e0fd927d2eb0db2dd5f1bcc0ee06ff410 |
| `testFile` | tests/cases/verified_hancom_paragraph_frames.rs |
| `testSha256` | c530cdf2d984a919ecb6c83c6618601da9a2ee8cd09cb78038a7819f3312714c |
| `officialGate.exitCode` | 0 |
| `officialGate.seconds` | 465.7367027499713 |
| `officialGate.summary[0]` |      Summary [ 208.531s] 83 tests run: 83 passed (7 slow), 1917 skipped |
| `officialGate.sourceUnchanged` | True |
| `officialGate.scope` | Native original8064 source equations/page ownership plus chain-aware original baseline contract and stored TAC controls/3930/1937; not full nextest |
| `format` | final fmt/check completed; only if-expression line wrapping changed after official run; no runtime/test logic change |
| `nativeWasmWholePageRows` | 44 |
| `metrics[0].backend` | base-native |
| `metrics[0].key` | science |
| `metrics[0].counts.rhwp` | 4 |
| `metrics[0].counts.pdf` | 4 |
| `metrics[0].minimum` | 93.18598 |
| `metrics[0].worstPage` | 4 |
| `metrics[0].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/base-native/science/silhouette.tsv |
| `metrics[0].directReading` | pending |
| `metrics[1].backend` | base-native |
| `metrics[1].key` | kor-hwp |
| `metrics[1].counts.rhwp` | 20 |
| `metrics[1].counts.pdf` | 20 |
| `metrics[1].minimum` | 92.32711 |
| `metrics[1].worstPage` | 17 |
| `metrics[1].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/base-native/kor-hwp/silhouette.tsv |
| `metrics[1].directReading` | pending |
| `metrics[2].backend` | base-native |
| `metrics[2].key` | kor-hwpx |
| `metrics[2].counts.rhwp` | 20 |
| `metrics[2].counts.pdf` | 20 |
| `metrics[2].minimum` | 92.32711 |
| `metrics[2].worstPage` | 17 |
| `metrics[2].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/base-native/kor-hwpx/silhouette.tsv |
| `metrics[2].directReading` | pending |
| `metrics[3].backend` | native |
| `metrics[3].key` | science |
| `metrics[3].counts.rhwp` | 4 |
| `metrics[3].counts.pdf` | 4 |
| `metrics[3].minimum` | 93.11518 |
| `metrics[3].worstPage` | 4 |
| `metrics[3].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/native/science/silhouette.tsv |
| `metrics[3].directReading` | pending |
| `metrics[4].backend` | native |
| `metrics[4].key` | kor-hwp |
| `metrics[4].counts.rhwp` | 20 |
| `metrics[4].counts.pdf` | 20 |
| `metrics[4].minimum` | 96.27225 |
| `metrics[4].worstPage` | 17 |
| `metrics[4].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/native/kor-hwp/silhouette.tsv |
| `metrics[4].directReading` | pending |
| `metrics[5].backend` | native |
| `metrics[5].key` | kor-hwpx |
| `metrics[5].counts.rhwp` | 20 |
| `metrics[5].counts.pdf` | 20 |
| `metrics[5].minimum` | 96.27225 |
| `metrics[5].worstPage` | 17 |
| `metrics[5].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/native/kor-hwpx/silhouette.tsv |
| `metrics[5].directReading` | pending |
| `metrics[6].backend` | wasm |
| `metrics[6].key` | science |
| `metrics[6].counts.rhwp` | 4 |
| `metrics[6].counts.pdf` | 4 |
| `metrics[6].minimum` | 93.04014 |
| `metrics[6].worstPage` | 4 |
| `metrics[6].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/wasm/science/silhouette.tsv |
| `metrics[6].directReading` | pending |
| `metrics[7].backend` | wasm |
| `metrics[7].key` | kor-hwp |
| `metrics[7].counts.rhwp` | 20 |
| `metrics[7].counts.pdf` | 20 |
| `metrics[7].minimum` | 96.29373 |
| `metrics[7].worstPage` | 17 |
| `metrics[7].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/wasm/kor-hwp/silhouette.tsv |
| `metrics[7].directReading` | pending |
| `metrics[8].backend` | wasm |
| `metrics[8].key` | kor-hwpx |
| `metrics[8].counts.rhwp` | 20 |
| `metrics[8].counts.pdf` | 20 |
| `metrics[8].minimum` | 96.29373 |
| `metrics[8].worstPage` | 17 |
| `metrics[8].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair512-exam-original-matched-dpi/wasm/kor-hwpx/silhouette.tsv |
| `metrics[8].directReading` | pending |
| `korInputFormatsPngIdentity.native` | True |
| `korInputFormatsPngIdentity.wasm` | True |
| `directReading` | Science p2 owned final table and following column10 question restored; p3/p4 residual spacing/ink retained. Kor14 body/punctuation baselines, original first face, passage border and inline header read in Native/freshWASM; format identity uses same20 PNG hashes. |
| `declineHold[0].case` | science |
| `declineHold[0].page` | 3 |
| `declineHold[0].base` | 95.06581 |
| `declineHold[0].candidate` | 94.73771 |
| `declineHold[0].decline` | 0.32810 |
| `declineHold[0].hold` | True |
| `prReady` | False |
| `browserInitialization.head` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `browserInitialization.mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `browserInitialization.browserVersion` | Chrome/155.0.8059.85 |
| `browserInitialization.producer.head` | 455aac1ac07f58eac8b4a553a227cf09c0ca2bf1 |
| `browserInitialization.producer.sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair509-exam-source-object-origin-fresh-wasm/source.json |
| `browserInitialization.producer.hashes.rhwp.js` | 23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591 |
| `browserInitialization.producer.hashes.rhwp_bg.wasm` | 077c60d0406d9a774637f5aba38510fd258439baf3a1b7405e7023b33ec2ead3 |
| `browserInitialization.producer.rootPackageRecompiled` | True |
| `browserInitialization.producer.wrapperInvoked` | True |
| `browserInitialization.producer.changesSincePreviousWasm[0]` | src/renderer/typeset/controls/stored_tac.rs |
| `browserInitialization.producer.rootPkgStudioPublicMatch` | True |
| `browserInitialization.producer.buildExitCode` | 0 |
| `browserInitialization.producer.seconds` | 270.75369024276733 |
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
| `browserInitialization.rows[1].bytes` | 22645059 |
| `browserInitialization.rows[1].servedSha256` | 077c60d0406d9a774637f5aba38510fd258439baf3a1b7405e7023b33ec2ead3 |
| `browserInitialization.rows[1].frozenSha256` | 077c60d0406d9a774637f5aba38510fd258439baf3a1b7405e7023b33ec2ead3 |
| `browserInitialization.rows[1].matchesFrozenBuild` | True |
| `browserInitialization.rows[1].method` | raw-byte-identity |
| `browserInitialization.wasmInitialized` | True |
| `browserInitialization.scope` | actual Studio7736 initialization responses only; not document paint, fonts, CanvasKit readiness or actual36 acceptance |
| `extraDiagnostic192Dpi.path` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair510-exam-original-whole |
| `extraDiagnostic192Dpi.nativeWhole44` | True |
| `extraDiagnostic192Dpi.wasmScience4` | True |
| `extraDiagnostic192Dpi.wasmKorFailed` | ENOSPC |
| `extraDiagnostic192Dpi.sameAsOriginalPrDpi` | False |
| `extraDiagnostic192Dpi.acceptance` | False |
| `cleanup.probe.deleted` | 14 |
| `cleanup.probe.bytes` | 294135220 |
| `cleanup.probe.scope` | closed temporary probe executables and duplicate trace dumps only; primary base/current JSON, source, logs, original HWP/PDF/PNG/TSV and all frozen engine libraries preserved |
| `cleanup.old13.deleted` | 13 |
| `cleanup.old13.bytes` | 1025019464 |
| `cleanup.old13.protected[0]` | current507 |
| `cleanup.old13.protected[1]` | before497 |
| `cleanup.old13.protected[2]` | before482 |
| `cleanup.old13.protected[3]` | before467 |
| `cleanup.old13.protected[4]` | 393 |
| `cleanup.old13.protected[5]` | fixedbase4a07 |
| `cleanup.old13.protected[6]` | all inputs/PDF/PNG/TSV/log/source/producer |
| `cleanup.old13.protected[7]` | shared target |
| `cleanup.old89.deleted` | 89 |
| `cleanup.old89.bytes` | 6894757848 |
| `cleanup.old89.preservedEvidenceChanged` | 0 |
| `finalManifest` | [RustTestSuite] 확인 완료: 1643 sources / 7175 static test attrs / 28 suites + 20 exceptions = 48/48 integration targets / nextest 최소 6288 cases / weight 1470219..1475194 |
| `formatAndManifestRepair` | Initial fmt check exposed if-expression line wrapping; after fmt, harness drift was repaired with prepare then final base-ref manifest check passed. Executed83 case functions are unchanged in logic. |


## 보존한 전체 TSV

- [base-native-kor-hwp.tsv](base-native-kor-hwp.tsv)
- [base-native-kor-hwpx.tsv](base-native-kor-hwpx.tsv)
- [base-native-science.tsv](base-native-science.tsv)
- [native-kor-hwp.tsv](native-kor-hwp.tsv)
- [native-kor-hwpx.tsv](native-kor-hwpx.tsv)
- [native-science.tsv](native-science.tsv)
- [wasm-kor-hwp.tsv](wasm-kor-hwp.tsv)
- [wasm-kor-hwpx.tsv](wasm-kor-hwpx.tsv)
- [wasm-science.tsv](wasm-science.tsv)
