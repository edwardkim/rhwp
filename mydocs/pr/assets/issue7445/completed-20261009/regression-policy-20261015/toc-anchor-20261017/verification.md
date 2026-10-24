# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## cause

이전 생성 파일: `cause.json` · 954 bytes · SHA-256 `fb325b78252c7c41b93ea6d811963f57799dd8831b72feb13da3705597b100cf`.

| 항목 | 당시 기록 |
|---|---|
| `parentHead` | acd4c6e31645db3ff14b0e257fe79c3294916130 |
| `independentPdf` | pdf/hwp3-sample16-hwp5-lossless-2024.pdf |
| `independentNumericRightCss` | 641.4053955078125 |
| `currentRightCss[0]` | 641.4933 |
| `currentRightCss[1]` | 637.1599666666666 |
| `cause` | Cross-run uses right stop; intra-run unspecified inline tab still subtracts quarter-em leader gap from the number anchor in both exact measurement and character position walks. |
| `contract` | The right tab aligns the numeric block end to the stop; quarter-em leader spacing changes the leader end only. |
| `productionChain[0]` | right_leader_tab_target_rel -> EmbeddedTextMeasurer::estimate_text_width_exact -> paragraph width |
| `productionChain[1]` | right_leader_tab_target_rel -> EmbeddedTextMeasurer::compute_char_positions -> emitted TextRun layout_positions |
| `controls` | Preserve explicit left/decimal and non-numeric inline-tab paths; do not change TAB_LEADER_HEAD_GAP_EM or leader-tail spacing. |

## independent-numeric-edges

이전 생성 파일: `independent-numeric-edges.json` · 3653 bytes · SHA-256 `26db03bb0290915ebaf161a43e593086ca9802414450bca7bf0df8747000d927`.

| 항목 | 당시 기록 |
|---|---|
| `[0].label` | manual |
| `[0].pdf` | /Users/tsjang/rhwp-7445-completed-20261009/pdf/hwp3-sample16-hwp5-lossless-2024.pdf |
| `[0].rows[0].y` | 167.7 |
| `[0].rows[0].text` | 1 |
| `[0].rows[0].leftCss` | 632.7999674479166 |
| `[0].rows[0].rightCss` | 641.4053955078125 |
| `[0].rows[1].y` | 190.2 |
| `[0].rows[1].text` | 1 |
| `[0].rows[1].leftCss` | 632.7999674479166 |
| `[0].rows[1].rightCss` | 641.4053955078125 |
| `[0].rows[2].y` | 212.8 |
| `[0].rows[2].text` | 1 |
| `[0].rows[2].leftCss` | 632.7999674479166 |
| `[0].rows[2].rightCss` | 641.4053955078125 |
| `[0].rows[3].y` | 271.7 |
| `[0].rows[3].text` | 3 |
| `[0].rows[3].leftCss` | 632.7999674479166 |
| `[0].rows[3].rightCss` | 641.4053955078125 |
| `[0].rows[4].y` | 294.2 |
| `[0].rows[4].text` | 5 |
| `[0].rows[4].leftCss` | 632.7999674479166 |
| `[0].rows[4].rightCss` | 641.4053955078125 |
| `[0].rows[5].y` | 316.8 |
| `[0].rows[5].text` | 6 |
| `[0].rows[5].leftCss` | 632.7999674479166 |
| `[0].rows[5].rightCss` | 641.4053955078125 |
| `[0].rows[6].y` | 375.8 |
| `[0].rows[6].text` | 8 |
| `[0].rows[6].leftCss` | 632.7999674479166 |
| `[0].rows[6].rightCss` | 641.4053955078125 |
| `[0].rows[7].y` | 398.4 |
| `[0].rows[7].text` | 9 |
| `[0].rows[7].leftCss` | 632.7999674479166 |
| `[0].rows[7].rightCss` | 641.4053955078125 |
| `[0].rows[8].y` | 421.0 |
| `[0].rows[8].text` | 9 |
| `[0].rows[8].leftCss` | 632.7999674479166 |
| `[0].rows[8].rightCss` | 641.4053955078125 |
| `[0].rows[9].y` | 443.5 |
| `[0].rows[9].text` | 12 |
| `[0].rows[9].leftCss` | 624.1599934895834 |
| `[0].rows[9].rightCss` | 641.4053955078125 |
| `[0].rows[10].y` | 466.1 |
| `[0].rows[10].text` | 13 |
| `[0].rows[10].leftCss` | 624.1599934895834 |
| `[0].rows[10].rightCss` | 641.4053955078125 |
| `[0].rows[11].y` | 488.6 |
| `[0].rows[11].text` | 14 |
| `[0].rows[11].leftCss` | 624.1599934895834 |
| `[0].rows[11].rightCss` | 641.4053955078125 |
| `[0].rows[12].y` | 549.9 |
| `[0].rows[12].text` | 15 |
| `[0].rows[12].leftCss` | 624.1599934895834 |
| `[0].rows[12].rightCss` | 641.4053955078125 |
| `[0].rows[13].y` | 576.0 |
| `[0].rows[13].text` | 16 |
| `[0].rows[13].leftCss` | 624.1599934895834 |
| `[0].rows[13].rightCss` | 641.4053955078125 |
| `[0].rows[14].y` | 601.9 |
| `[0].rows[14].text` | 17 |
| `[0].rows[14].leftCss` | 624.1599934895834 |
| `[0].rows[14].rightCss` | 641.4053955078125 |
| `[0].rows[15].y` | 628.0 |
| `[0].rows[15].text` | 20 |
| `[0].rows[15].leftCss` | 624.1599934895834 |
| `[0].rows[15].rightCss` | 641.4053955078125 |
| `[0].rows[16].y` | 654.1 |
| `[0].rows[16].text` | 23 |
| `[0].rows[16].leftCss` | 624.1599934895834 |
| `[0].rows[16].rightCss` | 641.4053955078125 |
| `[0].rows[17].y` | 714.2 |
| `[0].rows[17].text` | 24 |
| `[0].rows[17].leftCss` | 624.1599934895834 |
| `[0].rows[17].rightCss` | 641.4053955078125 |
| `[0].rows[18].y` | 736.8 |
| `[0].rows[18].text` | 25 |
| `[0].rows[18].leftCss` | 624.1599934895834 |
| `[0].rows[18].rightCss` | 641.4053955078125 |
| `[0].rows[19].y` | 795.7 |
| `[0].rows[19].text` | 33 |
| `[0].rows[19].leftCss` | 624.1599934895834 |
| `[0].rows[19].rightCss` | 641.4053955078125 |
| `[0].rows[20].y` | 818.2 |
| `[0].rows[20].text` | 34 |
| `[0].rows[20].leftCss` | 624.1599934895834 |
| `[0].rows[20].rightCss` | 641.4053955078125 |
| `[0].rows[21].y` | 840.8 |
| `[0].rows[21].text` | 35 |
| `[0].rows[21].leftCss` | 624.1599934895834 |
| `[0].rows[21].rightCss` | 641.4053955078125 |
| `[0].rows[22].y` | 863.4 |
| `[0].rows[22].text` | 36 |
| `[0].rows[22].leftCss` | 624.1599934895834 |
| `[0].rows[22].rightCss` | 641.4053955078125 |
| `[0].rows[23].y` | 932.8 |
| `[0].rows[23].text` | 37 |
| `[0].rows[23].leftCss` | 624.1599934895834 |
| `[0].rows[23].rightCss` | 641.4053955078125 |
| `[0].rows[24].y` | 955.4 |
| `[0].rows[24].text` | 37 |
| `[0].rows[24].leftCss` | 624.1599934895834 |
| `[0].rows[24].rightCss` | 641.4053955078125 |
| `[0].rows[25].y` | 977.9 |
| `[0].rows[25].text` | 38 |
| `[0].rows[25].leftCss` | 624.1599934895834 |
| `[0].rows[25].rightCss` | 641.4053955078125 |

## validation

이전 생성 파일: `validation.json` · 2383 bytes · SHA-256 `d8937ba6befbb466f03fa2b2f334073272ec1d3e4df8032999f8804adb6600b2`.

| 항목 | 당시 기록 |
|---|---|
| `productionCodeHead` | 4541014e3ede18099a747d6209156ef74c5a68c8 |
| `base` | 9415b920c8dec299e8ada69b01dbd1c3c044f1f1 |
| `buildParentHead` | acd4c6e31645db3ff14b0e257fe79c3294916130 |
| `buildDiffSha256` | 4818a68feaadab4753b063fde16053d839c20383fd90db5b9c7ee82da9ace9d6 |
| `productionFilesVerifiedEqual` | 2760 |
| `input` | samples/hwp3-sample16-hwp5.hwp |
| `inputSha256` | 6a3cdf2c148bf39f40ab06e847767e7f722543fce59e6ee4328ef44b835f3f45 |
| `pdf` | pdf/hwp3-sample16-hwp5-lossless-2024.pdf |
| `pdfSha256` | 9d0a504f5344f39b770d84eba613567feb7910eded9fbb125ad4e7b0bab6b589 |
| `portable.dpi` | 192 |
| `portable.compat` | 2024 |
| `portable.captureWorkers` | 8 |
| `portable.records[0].backend` | native |
| `portable.records[0].pages` | 64 |
| `portable.records[0].minimumRaw` | 93.07399 |
| `portable.records[0].page2Before` | 97.57913 |
| `portable.records[0].page2After` | 97.97410 |
| `portable.records[0].page19Raw` | 99.43242 |
| `portable.records[0].page19PngByteIdentical` | True |
| `portable.records[0].comparisonArithmetic` | Decimal on original TSV values |
| `portable.records[1].backend` | wasm |
| `portable.records[1].pages` | 64 |
| `portable.records[1].minimumRaw` | 95.87967 |
| `portable.records[1].page2Before` | 98.33185 |
| `portable.records[1].page2After` | 98.71502 |
| `portable.records[1].page19Raw` | 99.61477 |
| `portable.records[1].page19PngByteIdentical` | True |
| `portable.records[1].comparisonArithmetic` | Decimal on original TSV values |
| `actualStudio.dpi` | 96 |
| `actualStudio.pdfDpi` | 96 |
| `actualStudio.compat` | product default2022 |
| `actualStudio.backend` | Canvas2D Print |
| `actualStudio.page` | 2 |
| `actualStudio.raw` | 99.33039 |
| `actualStudio.initialBuildResponsesVerified` | 2 |
| `actualStudio.directReview` | True |
| `scope` | Source correction and manual64 qualification; focused13/16/17 PASS; relational13 PASS. Final global nextest/gates/48/base comparison/actual68 pending. Previous-candidate comparison is not an upstream comparison. No px pin added on page2: Native raw remains below98. |
| `relationalLeaderRegression.case` | issue_1116 |
| `relationalLeaderRegression.test` | sample16_hwp5_toc_leaders_stop_before_page_numbers |
| `relationalLeaderRegression.testsRun` | 13 |
| `relationalLeaderRegression.passed` | 13 |
| `relationalLeaderRegression.failed` | 0 |
| `relationalLeaderRegression.seconds` | 0.354 |
| `relationalLeaderRegression.preparedTarget` | regression_suite_015 |
| `relationalLeaderRegression.precondition` | Native/fresh WASM current64-page raw90 qualification completed before replacing x650 pin |
| `relationalLeaderRegression.sourceUnchangedByTestRefactor` | True |


## 보존한 전체 TSV

- [whole64-native.tsv](whole64-native.tsv)
- [whole64-wasm.tsv](whole64-wasm.tsv)
