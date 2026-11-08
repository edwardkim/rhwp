# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## evidence

이전 생성 파일: `evidence.json` · 13839 bytes · SHA-256 `6aac0817e3da5557c1431b186b0d871c0287fb5bd66d629584ed0d632906d8bb`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | c07d208e2b996b254d5484df88d741849928e3da |
| `productionSource.src/paint/builder.rs` | bf28944bba3d8b9d3b3dd866d44bc7791a8dd9f1ff2883e51f8376894d0a5703 |
| `productionSource.src/paint/paper_viewport.rs` | c5cf7728bd85558fd859a1c76a9985a5b725ba5adc93151d442959b822f79e3b |
| `productionSource.src/renderer/float_placement.rs` | 23e02f4212f2ff69b513034d5ac20f113658a639626b791ee6571cca65c65a3f |
| `productionSource.src/renderer/height_measurer.rs` | e6e6d43ff029b0661ebb452bacf657ea590a0338f9eec313b44b4830c851dcbe |
| `productionSource.src/renderer/layout/paragraph_layout.rs` | 9c9295e53df30a54e58c579e8d968d357de4f03145e2caf4f4d64a4f6ab222e2 |
| `productionSource.src/renderer/layout/table_layout/stored_plain_cell_minimum.rs` | bc2db4d7d39c18a87dfce93d1af34b87a59a5ffff341b91be793ef12262c74cd |
| `productionSource.src/renderer/skia/text_replay.rs` | 078e8efe34ce086f454971394766fdd594a340108c8b06f058d2a16b0757d1a5 |
| `productionSource.src/renderer/typeset/paragraph/content.rs` | b712143afc634de386a905a719b09464400780b5e856760e818fc37626a25fd9 |
| `scope` | Stage20 checkpoint. Science source771 whole4 Native772/freshWASM775. Sample11 whole151 uses earlier765 Native and751 WASM; native glyph width fix proof, not claimed latest771/global qualification. Complete branch acceptance/actual36/full regression pending. |
| `readings.science` | Native2/3/4 review+overlay and WASM2/3/4 review read; no loss/reordering in reviewed region. Previous p119 drift fixed. Residual text baseline differences on3/4 remain; no px pins there. |
| `readings.sample11` | Native/WASM2/50 reviewed. Native widths now align Canvas. Independent Print gray/brown regions under diagram remain visible in overlay; not masked. This artifact is not claimed fully identical. |

## native-source-771

이전 생성 파일: `native-source-771.json` · 1333 bytes · SHA-256 `c4265e0e9ccfc9c0fd18c94c3aeb81e23a7e2b333bd572906f5751709f64f09c`.

| 항목 | 당시 기록 |
|---|---|
| `parent` | c07d208e2b996b254d5484df88d741849928e3da |
| `files.src/paint/builder.rs` | bf28944bba3d8b9d3b3dd866d44bc7791a8dd9f1ff2883e51f8376894d0a5703 |
| `files.src/paint/paper_viewport.rs` | c5cf7728bd85558fd859a1c76a9985a5b725ba5adc93151d442959b822f79e3b |
| `files.src/renderer/float_placement.rs` | 23e02f4212f2ff69b513034d5ac20f113658a639626b791ee6571cca65c65a3f |
| `files.src/renderer/height_measurer.rs` | e6e6d43ff029b0661ebb452bacf657ea590a0338f9eec313b44b4830c851dcbe |
| `files.src/renderer/layout/paragraph_layout.rs` | 9c9295e53df30a54e58c579e8d968d357de4f03145e2caf4f4d64a4f6ab222e2 |
| `files.src/renderer/layout/table_layout/stored_plain_cell_minimum.rs` | bc2db4d7d39c18a87dfce93d1af34b87a59a5ffff341b91be793ef12262c74cd |
| `files.src/renderer/skia/text_replay.rs` | 078e8efe34ce086f454971394766fdd594a340108c8b06f058d2a16b0757d1a5 |
| `files.src/renderer/typeset/paragraph/content.rs` | b712143afc634de386a905a719b09464400780b5e856760e818fc37626a25fd9 |
| `patchSha256` | ca99325c471b7e5247ad44ab929fa047622fe6bd0c96ff8ca48b7691a6efd6a7 |
| `scope` | Stage20 stored multicolumn paragraph flow consumes certified source content bottom, trailing spacing and paragraph after spacing; no document/page conditions. Independent Hancom source and before typeset/paint trace in repair770 and cause.md. Full validation pending. |

## science-page2-independent-vectors

이전 생성 파일: `science-page2-independent-vectors.json` · 2514 bytes · SHA-256 `d0c0fd0cb7e1ab9a3fe7d26cd469d9c579b31fb106c13c6943a5df83018d2486`.

| 항목 | 당시 기록 |
|---|---|
| `pdf` | /Users/tsjang/rhwp-7445-completed-20261009/pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf |
| `pdfSha256` | 1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14 |
| `page` | 2 |
| `dpi` | 96 |
| `mediaBox[0]` | 0.0 |
| `mediaBox[1]` | 0.0 |
| `mediaBox[2]` | 771.0 |
| `mediaBox[3]` | 1116.0 |
| `coordinateSpace` | PDF source vector operators transformed to CSS96; no raster measurement or candidate coordinate copied |
| `vectors[0].start[0]` | 423.0 |
| `vectors[0].start[1]` | 263.442 |
| `vectors[0].end[0]` | 570.48 |
| `vectors[0].end[1]` | 263.442 |
| `vectors[0].cssStart[0]` | 564.0 |
| `vectors[0].cssStart[1]` | 1136.744 |
| `vectors[0].cssEnd[0]` | 760.64 |
| `vectors[0].cssEnd[1]` | 1136.744 |
| `vectors[1].start[0]` | 570.48 |
| `vectors[1].start[1]` | 263.442 |
| `vectors[1].end[0]` | 570.48 |
| `vectors[1].end[1]` | 249.172 |
| `vectors[1].cssStart[0]` | 760.64 |
| `vectors[1].cssStart[1]` | 1136.744 |
| `vectors[1].cssEnd[0]` | 760.64 |
| `vectors[1].cssEnd[1]` | 1155.7706666666666 |
| `vectors[2].start[0]` | 570.48 |
| `vectors[2].start[1]` | 249.172 |
| `vectors[2].end[0]` | 423.0 |
| `vectors[2].end[1]` | 249.172 |
| `vectors[2].cssStart[0]` | 760.64 |
| `vectors[2].cssStart[1]` | 1155.7706666666666 |
| `vectors[2].cssEnd[0]` | 564.0 |
| `vectors[2].cssEnd[1]` | 1155.7706666666666 |
| `vectors[3].start[0]` | 423.0 |
| `vectors[3].start[1]` | 249.172 |
| `vectors[3].end[0]` | 570.48 |
| `vectors[3].end[1]` | 249.172 |
| `vectors[3].cssStart[0]` | 564.0 |
| `vectors[3].cssStart[1]` | 1155.7706666666666 |
| `vectors[3].cssEnd[0]` | 760.64 |
| `vectors[3].cssEnd[1]` | 1155.7706666666666 |
| `vectors[4].start[0]` | 570.48 |
| `vectors[4].start[1]` | 249.172 |
| `vectors[4].end[0]` | 570.48 |
| `vectors[4].end[1]` | 234.903 |
| `vectors[4].cssStart[0]` | 760.64 |
| `vectors[4].cssStart[1]` | 1155.7706666666666 |
| `vectors[4].cssEnd[0]` | 760.64 |
| `vectors[4].cssEnd[1]` | 1174.796 |
| `vectors[5].start[0]` | 570.48 |
| `vectors[5].start[1]` | 234.903 |
| `vectors[5].end[0]` | 423.0 |
| `vectors[5].end[1]` | 234.903 |
| `vectors[5].cssStart[0]` | 760.64 |
| `vectors[5].cssStart[1]` | 1174.796 |
| `vectors[5].cssEnd[0]` | 564.0 |
| `vectors[5].cssEnd[1]` | 1174.796 |
| `vectors[6].start[0]` | 422.88 |
| `vectors[6].start[1]` | 249.172 |
| `vectors[6].end[0]` | 570.72 |
| `vectors[6].end[1]` | 249.172 |
| `vectors[6].cssStart[0]` | 563.84 |
| `vectors[6].cssStart[1]` | 1155.7706666666666 |
| `vectors[6].cssEnd[0]` | 760.96 |
| `vectors[6].cssEnd[1]` | 1155.7706666666666 |
| `vectors[7].start[0]` | 422.88 |
| `vectors[7].start[1]` | 249.172 |
| `vectors[7].end[0]` | 570.72 |
| `vectors[7].end[1]` | 249.172 |
| `vectors[7].cssStart[0]` | 563.84 |
| `vectors[7].cssStart[1]` | 1155.7706666666666 |
| `vectors[7].cssEnd[0]` | 760.96 |
| `vectors[7].cssEnd[1]` | 1155.7706666666666 |

## wasm-producer-774

이전 생성 파일: `wasm-producer-774.json` · 681 bytes · SHA-256 `57416e2c1df60f88672f4df6626b19692fb949b6ef6c146a3b0a4376103d0053`.

| 항목 | 당시 기록 |
|---|---|
| `head` | c07d208e2b996b254d5484df88d741849928e3da |
| `sourceLedger` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair774-multicolumn-stored-flow-fresh-wasm/source.json |
| `hashes.rhwp.js` | 63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524 |
| `hashes.rhwp_bg.wasm` | efacc584d529451970d9ac1337927a05928de0543cadf3b471875498e6d8e6a9 |
| `rootPackageRecompiled` | True |
| `wrapperInvoked` | True |
| `rootPkgStudioPublicMatch` | True |
| `buildExitCode` | 0 |
| `seconds` | 211.1597949579591 |
| `mode` | Mac locked web/pkg/no-opt local alternative; not Docker optimized |
| `actualBrowserInitialization` | pending |


## 보존한 전체 TSV

- [sample11-native-whole-765-751.tsv](sample11-native-whole-765-751.tsv)
- [sample11-wasm-whole-765-751.tsv](sample11-wasm-whole-765-751.tsv)
- [science-native-whole.tsv](science-native-whole.tsv)
- [science-wasm-whole.tsv](science-wasm-whole.tsv)
