# 보정23 최종 검증 요약

[구현 보고서](../../../../../working/task_m100_7445_rebase_repair_stage23.md).
이 문서는 Stage23의 실제 결과를 보존한 것으로 Stage24 이후 전체 branch 통과 증거가 아니다.
생성 run/분석/source/build JSON은 output의repair921에 원래 SHA 그대로 보존했다.
원본 입력·PDF·최종 PNG·전체 TSV는 변경하지 않았다.

## 전쪽 지표

각 backend215쪽(PrEP140/대조색상1/법령74), 누락·중복·90% 미달0이다.
96DPI/Print/compat는 입력별2024·2022, canonical PDFium smooth_rgb_v1/raw2px90%다.

| 출력 | 입력 | 쪽수 | 최저raw | 최저쪽 | 전체TSV |
|---|---|---:|---:|---:|---|
| native | prep | 140 | 94.75150% | 27 | [TSV](native-prep.tsv) |
| native | palette | 1 | 99.97513% | 1 | [TSV](native-palette.tsv) |
| native | legal-control | 74 | 93.69360% | 36 | [TSV](native-legal-control.tsv) |
| wasm | prep | 140 | 94.22354% | 27 | [TSV](wasm-prep.tsv) |
| wasm | palette | 1 | 99.97512% | 1 | [TSV](wasm-palette.tsv) |
| wasm | legal-control | 74 | 93.46860% | 36 | [TSV](wasm-legal-control.tsv) |

PrEP15쪽은 회색 밴드·캡션·뒤 본문,72쪽은 격자의 내용/줄 소유를 Native/fresh WASM
review와overlay에서 직접 읽었다. 얇은 선의 위상·진하기와 미세 가장자리 차이는 남으며,
raw100% 또는 gate passed를 엄격RGB 일치나 전체 소비 경로 통과로 해석하지 않는다.
정식 Native 출력4검사와 PrEP2검사가 통과했고 fmt/diff/manifest 확인도 통과했다.
상세 명령과 수정 전FAIL/수정 후PASS는 구현 보고서에 연결한다.

## 기존 Native 품질 하한: 보류

0.1%p 이상 하락은 보류한다. 아래4쪽은90% 이상이어도 면제하지 않았다.

| 조건 | 쪽 | 기준raw | 현재raw | 하락%p |
|---|---:|---:|---:|---:|
| prep | 1 | 99.70907 | 99.08608 | 0.62299 |
| legal-control | 2 | 99.35778 | 99.20447 | 0.15331 |
| legal-control | 13 | 97.05721 | 96.87649 | 0.18072 |
| legal-control | 36 | 95.28632 | 93.69360 | 1.59272 |

## 입력과 독립 Print

| 조건 | 입력 | 입력SHA256 | 독립PDF | PDF SHA256 |
|---|---|---|---|---|
| prep | `samples/issue2006/1790387_prep_final_report.hwpx` | `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f` | `pdf/issue2006/1790387_prep_final_report-current-print-2024.pdf` | `9580bb332813b70f27a367e79ad735b238bcd716ff51b849e716f7ddd1e60af0` |
| palette | `pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls.hwpx` | `d60a1a891c4235330ed0950d63ca3dac921c06892bb8b19b3951bbbadb64ba51` | `pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls-2024.pdf` | `2480817b897f7e8bbcd0db5e14de0668ddcf0d71786fc93b89d97442a202469e` |
| legal-control | `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` | `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617` | `pdf/issue7642-legal-table-origin/78494-virtual-convergence-industry-decree-2020.pdf` | `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2` |

## 실행 신원

- 부모HEAD: `37aff1d19e5a724b502c2f4e0a8e6b389c05c892` / 고정base: `2e3b4f880bd11d35398e6ea0100846611f13a72d`
- Native CLI SHA256: `2a30fe92ad9df66200a203454fa6b525a9c5e36a45399010a4d0f4d0947b47c1`
- Native library SHA256: `7e0dffb202711ef28f0eb4f13b4e04a958108856d2f6977d31512e47751948b1`
- fresh `rhwp.js` SHA256: `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`
- fresh `rhwp_bg.wasm` SHA256: `13f70758c2ee635c7716968d90342d9b69c851af78f31f3e204ee2e2073f9a71`
- fresh wrapper: Mac web/pkg/no-opt 로컬 대체, exit0/236.971초. Docker 최적화가 아니다.
- 이 당시 루트pkg/Studio public hash는 같았고 실제 app 초기화는 pending이었다.
- canonical visual_sweep.py SHA256: `69d9d56394b26fcb7a64b41588a378da745475b3297bae89fd6b4d16903cede4`
- 키 정렬·compact JSON 방식의 전체 공급 font-file map SHA256: `bb335dbb6c811ef16c52e4e3d9b97059e3d32b4876626f3d81163601a5736256`. 상세 공급 목록은 로컬 원장에 보존한다.

| runtime source | SHA256 |
|---|---|
| `src/paint/builder.rs` | `a0e2af305bb771ebd49d67a83361b8b8886bac62cb4928b0837c73db913278ff` |
| `src/paint/json.rs` | `b3982463e30576989c459be72f57e5836188456c4718a9783fca6c767b18bbc6` |
| `src/paint/paint_op.rs` | `d671e5319967d153f0f4fa4421e35398ac81599ad1b8f67dafbd59d3ddd10427` |
| `src/paint/paper_viewport.rs` | `973498b8de5bdc5fa8072667a9c9bf8f69330e33c3d72a339d610f4acbb7c532` |
| `src/renderer/bitmap_raster.rs` | `07dddf7a46e6dae79e810f8cc50b0d596d5fccc7258dc72f168ab134fea1d09b` |
| `src/renderer/canvaskit_policy.rs` | `0205e92d7eb549eb68644d3737089fb6a198bd3f7b05e1fe79881a212ee72db8` |
| `src/renderer/image_resolver.rs` | `93ba90d6c53387d715a5d1978acdde87d92481906c05e682586317cf0ad70b45` |
| `src/renderer/image_resolver/print_crop.rs` | `25bfdef2bf4d9064d28fb919be7c611103341986fef19120e6ef1d646904d9fb` |

| 정식 검사 source | SHA256 |
|---|---|
| `tests/cases/native_output_device_surface.rs` | `fd45f9cff0611ddafc851f75082b43bc317ec627d3616c2e03e95c9bf5bbdf6b` |
| `tests/cases/verified_prep_bitmap_grid_flow.rs` | `97bf6b0a9f58c47a861fee0d57865c60cbae480bc277e9e3dce6cc709c7d9478` |

## 로컬로 옮긴 원시 생성 기록

| 기존 파일 | 보존SHA256 |
|---|---|
| `evidence.json` | `556a6ef2d04c94a4943781db1ff1de1d5b6280cad1a7edb65d8541a0ad0517b0` |
| `native-quality.json` | `c6f646493796cb38b7dd8b15f9229129221392bfc86cf7119b8ea8e74d3af6ef` |
| `native-source.json` | `b8e73998111b7779fbc5805446d943a992c29283434b878f8e501915e17b4962` |
| `wasm-producer.json` | `5803adf6b57b4b10b70b6848e3702b0a990c04840c626a68f5cce2842c530aa9` |
| `whole-producer.json` | `807a9c6c6b64a2864a06b90184c15117f61dd3aac57c1da68ad8da87a1e0b0cf` |

로그·MCP 응답·생성 JSON을 최종 PR asset으로 제출하지 않는다. 전체 회귀·actual36·공유 그림 실제 소비 경로는 별도 잔여다.
