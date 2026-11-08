# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## independent-wmf-cause

이전 생성 파일: `independent-wmf-cause.json` · 1371 bytes · SHA-256 `2b2196ca2bc6182efb314bcc73949c1b5b751a5d959a0b3a712d9414fba04fe5`.

| 항목 | 당시 기록 |
|---|---|
| `sourceSha256` | 1927d7c64d345e4467f5936b3e9a28cfe1b395bf5a2301f5fef6fb1ff829cb5d |
| `independentWindowsRenderer` | Windows System.Drawing GDI+ Metafile.DrawImage; original source bytes verified |
| `sourceRecords.PATINVERT[0]` | 157 |
| `sourceRecords.PATINVERT[1]` | 166 |
| `sourceRecords.DPA` | 162 |
| `sourceRecords.brushCreate` | 155 |
| `sourceRecords.rgb` | #FFCCFF |
| `sourceRecords.maskCreate` | 159 |
| `sourceRecords.maskRowsHex` | aa0000005d000000aa00000077000000aa000000dd000000aa00000077000000 |
| `convertedSvg.outerRect[0]` | 3822 |
| `convertedSvg.outerRect[1]` | 387 |
| `convertedSvg.outerRect[2]` | 2432 |
| `convertedSvg.outerRect[3]` | 1161 |
| `convertedSvg.maskRect[0]` | 3823 |
| `convertedSvg.maskRect[1]` | 387 |
| `convertedSvg.maskRect[2]` | 2431 |
| `convertedSvg.maskRect[3]` | 1161 |
| `convertedSvg.maskContained` | True |
| `convertedSvg.pattern` | rop_pat2 |
| `convertedSvg.maskAttached` | False |
| `convertedSvg.visibleColorUnderPattern` | #FFCCFF |
| `cause` | Non-checker 1-bit black/white mask lies wholly inside both identical outer XOR rectangles; mask_rect_matches rejects it because it is not the hardcoded alternating halftone. The unmasked black/white pattern covers the pink brush paint. |
| `independentAlgebra` | Inside inner rectangle: (D XOR P) AND M XOR P => black mask selects P, white selects D. Outside inner but inside outer: D XOR P XOR P = D. Therefore contained mask can transfer exactly without edge approximation or halftone-content assumption. |
| `status` | diagnosis only; production patch and new upstream interaction pending; no full WMF support claim |

## previous-candidate-comparison

이전 생성 파일: `previous-candidate-comparison.json` · 687 bytes · SHA-256 `82ac8cefc9b0528bab460c9de484c0f844092afcba7df02151723251d3fa020c`.

| 항목 | 당시 기록 |
|---|---|
| `[0].backend` | native |
| `[0].pages` | 64 |
| `[0].previousCandidate` | C85 before rebase onto9415 |
| `[0].newCandidate` | ac5d062c plus contained-mask patch |
| `[0].changed[0].page` | 18 |
| `[0].changed[0].before` | 98.00111 |
| `[0].changed[0].after` | 98.01466 |
| `[0].notAnUpstreamBaseComparison` | True |
| `[1].backend` | wasm |
| `[1].pages` | 64 |
| `[1].previousCandidate` | C85 before rebase onto9415 |
| `[1].newCandidate` | ac5d062c plus contained-mask patch |
| `[1].changed[0].page` | 18 |
| `[1].changed[0].before` | 97.95139 |
| `[1].changed[0].after` | 97.96086 |
| `[1].notAnUpstreamBaseComparison` | True |

## rebase-result

이전 생성 파일: `rebase-result.json` · 616 bytes · SHA-256 `725ae7e3cac4b7f58abd05dc937556d98bd10372c0d6425df174a2c1e110f403`.

| 항목 | 당시 기록 |
|---|---|
| `newHead` | ac5d062c8d7e0194c3254cf58ee925bd055b94a7 |
| `base` | 9415b920c8dec299e8ada69b01dbd1c3c044f1f1 |
| `preservedFiles` | 42 |
| `backupBranch` | backup/7445-before-9415b920-20261017 |
| `preservedStash` | f4eea7a90560c339cdfa4d36c9b63d2523e64941 |
| `conflictsResolved[0]` | image_resolver: preserve upstream picture-adjustment modules and branch crop/dimension helpers |
| `conflictsResolved[1]` | paragraph_layout: upstream shared source-line indent supersedes old effective_margin_left_line clamp |
| `conflictsResolved[2]` | 7751 working report: keep upstream expanded report; raw-gate cherry patch already present |

## validation

이전 생성 파일: `validation.json` · 1704 bytes · SHA-256 `13c3e24cdaea49d3b544885ad88ccd0bab390eb3cd5c5f376a2c7e98e2d531f4`.

| 항목 | 당시 기록 |
|---|---|
| `parentHead` | ac5d062c8d7e0194c3254cf58ee925bd055b94a7 |
| `base` | 9415b920c8dec299e8ada69b01dbd1c3c044f1f1 |
| `candidateDiff` | src/wmf/converter/svg/ternary_raster_operator.rs contained monochrome AND mask rule |
| `results.native.pages` | 64 |
| `results.native.minimumRaw` | 93.07399 |
| `results.native.below90` | 0 |
| `results.native.pages18and19.18` | 98.01466 |
| `results.native.pages18and19.19` | 99.43242 |
| `results.wasm.pages` | 64 |
| `results.wasm.minimumRaw` | 95.87967 |
| `results.wasm.below90` | 0 |
| `results.wasm.pages18and19.18` | 97.96086 |
| `results.wasm.pages18and19.19` | 99.61477 |
| `directReview[0]` | Native18 review and overlay |
| `directReview[1]` | WASM18 review and overlay |
| `directReview[2]` | Native19 review and overlay |
| `directReview[3]` | WASM19 review and overlay |
| `formalRegressions.before.run` | 17 |
| `formalRegressions.before.passed` | 15 |
| `formalRegressions.before.failed` | 2 |
| `formalRegressions.before.seconds` | 0.137 |
| `formalRegressions.after.run` | 17 |
| `formalRegressions.after.passed` | 17 |
| `formalRegressions.after.failed` | 0 |
| `formalRegressions.after.seconds` | 0.097 |
| `formalRegressions.rebasedFontOwner.run` | 16 |
| `formalRegressions.rebasedFontOwner.passed` | 16 |
| `formalRegressions.rebasedFontOwner.failed` | 0 |
| `formalRegressions.intendedBeforeFailuresFound` | True |
| `formalRegressions.sourceRestored` | True |
| `upstreamComparison` | Native/WASM base sample16 is65 pages versus independent Print64; canonical comparison failed without truncation. Native giant80 base94.40109 vs candidate96.82269; WASM base97.54273 vs candidate97.50476. |
| `remaining[0]` | new fixed-base complete per-page comparisons: manual reference mismatch65vs64 retained |
| `remaining[1]` | latest source complete regression and gates |
| `remaining[2]` | all48 documents and actual68 consumers |
| `remaining[3]` | giant80 default WASM decline0.03797pp confirmed versus9415b920 |
| `frozenProducerProductionSourceHashesMatch` | True |


## 보존한 전체 TSV

- [native-all64.tsv](native-all64.tsv)
- [wasm-all64.tsv](wasm-all64.tsv)
