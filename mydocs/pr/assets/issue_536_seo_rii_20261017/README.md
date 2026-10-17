# seo-rii 통합 검토의 최종 제출 증거

기준 source `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`, 후보 producer `3535106051394672e93ca8401ff9eae8b449ec20`. 입력13개 각각 전체1쪽,96DPI·print·검증한 한컴 글꼴·raw2px 실루엣이다. 후보 최저90% 이상·쪽수1:1·누락0·동일 backend 기준0.01%p 이상 하락 없음. 실제 명령·한계·직접 판독은 [8단계](../../../working/task_m100_536_integration_review_stage8.md)를 따른다.

최종 TSV52개는 생성 원본의 바이트를 보존한다. Native/fresh WASM과 base/후보를 구분한다. PR 본문에 전체 원문을 넣고 이 파일들의 정확한 head SHA raw URL도 연결한다. 이전32개 TSV의 source751/b7 결과는 [7단계](../../../working/task_m100_536_integration_review_stage7.md)와 당시 commit의 역사적 증거다. 로그·중간JSON을 asset으로 커밋하지 않는다.

폼 HWP는 기존 HWPX 수동 Print와의 대응을 검증한 것이며 새 HWP 직접 Print라고 표시하지 않는다. 수식의 기존 분자·괄호 간격과 폼 bevel은 전체 해결로 표시하지 않는다. 미달 단독 극한 진단은 [별도 기록](diagnostics/README.md)을 따른다. 추가5개 색조 대조군은 [독립 한컴 저장·Print 출처](../../../../pdf/issue7751/README.md)의 합성 색조 계약이며 편람383쪽 전체의 검증을 대신하지 않는다.

| 원본 TSV | producer SHA | bytes | SHA-256 |
| --- | --- | ---: | --- |
| [base/native/lseg-01-basic_silhouette.tsv](base/native/lseg-01-basic_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `254df30578cc655814a3c76e0c1fe040fd6e8bfb9425a464b0dd3e8db4c9359a` |
| [native/lseg-01-basic_silhouette.tsv](native/lseg-01-basic_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `254df30578cc655814a3c76e0c1fe040fd6e8bfb9425a464b0dd3e8db4c9359a` |
| [base/native/lseg-02-mixed_silhouette.tsv](base/native/lseg-02-mixed_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `bbcc43941531cb3b694b9ccbe557f3cdab16acb4b4ff4fcce195de8e8c99ee3a` |
| [native/lseg-02-mixed_silhouette.tsv](native/lseg-02-mixed_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `bbcc43941531cb3b694b9ccbe557f3cdab16acb4b4ff4fcce195de8e8c99ee3a` |
| [base/native/pic-crop-01_silhouette.tsv](base/native/pic-crop-01_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `14f3dfd6d178218a5b707d5b125fe4383994226f2d38efb81f7d1b6930611785` |
| [native/pic-crop-01_silhouette.tsv](native/pic-crop-01_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `14f3dfd6d178218a5b707d5b125fe4383994226f2d38efb81f7d1b6930611785` |
| [base/native/shape-group-02_silhouette.tsv](base/native/shape-group-02_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 137 | `1d1dc181233097557609523a8cdd6a8ede011a960706dc08154a7829391dcb86` |
| [native/shape-group-02_silhouette.tsv](native/shape-group-02_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/equation-lim-print2020_silhouette.tsv](base/native/equation-lim-print2020_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `693028c71033061f511671bf726c87cb537377c0916d87ad733fa3216c3f8b5a` |
| [native/equation-lim-print2020_silhouette.tsv](native/equation-lim-print2020_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `886eabe3adfeafd320e9366221c4ed5f4bef15a081c1753614645842c5c5db24` |
| [base/native/form-hwpx_silhouette.tsv](base/native/form-hwpx_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `a4ae0656fa7a0665e3cab843c6647633a4ac98d32a30e333129b247ffdd5740c` |
| [native/form-hwpx_silhouette.tsv](native/form-hwpx_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `a4ae0656fa7a0665e3cab843c6647633a4ac98d32a30e333129b247ffdd5740c` |
| [base/native/form-hwp-manual-crossformat_silhouette.tsv](base/native/form-hwp-manual-crossformat_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `a4ae0656fa7a0665e3cab843c6647633a4ac98d32a30e333129b247ffdd5740c` |
| [native/form-hwp-manual-crossformat_silhouette.tsv](native/form-hwp-manual-crossformat_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `a4ae0656fa7a0665e3cab843c6647633a4ac98d32a30e333129b247ffdd5740c` |
| [base/native/native-ime_silhouette.tsv](base/native/native-ime_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/native-ime_silhouette.tsv](native/native-ime_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/hancom-color-probe-2024_silhouette.tsv](base/native/hancom-color-probe-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/hancom-color-probe-2024_silhouette.tsv](native/hancom-color-probe-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/hancom-color-probe-cross-2024_silhouette.tsv](base/native/hancom-color-probe-cross-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/hancom-color-probe-cross-2024_silhouette.tsv](native/hancom-color-probe-cross-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/hancom-color-probe-boundary-2024_silhouette.tsv](base/native/hancom-color-probe-boundary-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/hancom-color-probe-boundary-2024_silhouette.tsv](native/hancom-color-probe-boundary-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/hancom-blackwhite-zero-2024_silhouette.tsv](base/native/hancom-blackwhite-zero-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/hancom-blackwhite-zero-2024_silhouette.tsv](native/hancom-blackwhite-zero-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/native/hancom-blackwhite-axis-probe-2024_silhouette.tsv](base/native/hancom-blackwhite-axis-probe-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [native/hancom-blackwhite-axis-probe-2024_silhouette.tsv](native/hancom-blackwhite-axis-probe-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/lseg-01-basic_silhouette.tsv](base/wasm/lseg-01-basic_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `54b306f061953b06b61c34f2c999a62d68bf2293e69e7774baf4d248d6adeb7c` |
| [wasm/lseg-01-basic_silhouette.tsv](wasm/lseg-01-basic_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `54b306f061953b06b61c34f2c999a62d68bf2293e69e7774baf4d248d6adeb7c` |
| [base/wasm/lseg-02-mixed_silhouette.tsv](base/wasm/lseg-02-mixed_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `265c21b915268cf6494554760c4cd116f45d9e949e871a42ee068f006c2de087` |
| [wasm/lseg-02-mixed_silhouette.tsv](wasm/lseg-02-mixed_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `265c21b915268cf6494554760c4cd116f45d9e949e871a42ee068f006c2de087` |
| [base/wasm/pic-crop-01_silhouette.tsv](base/wasm/pic-crop-01_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `7ad26872b92517b9a41a2b38f010e3b005e3f56f57ce3989a1cca93ece6bbe09` |
| [wasm/pic-crop-01_silhouette.tsv](wasm/pic-crop-01_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `7ad26872b92517b9a41a2b38f010e3b005e3f56f57ce3989a1cca93ece6bbe09` |
| [base/wasm/shape-group-02_silhouette.tsv](base/wasm/shape-group-02_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/shape-group-02_silhouette.tsv](wasm/shape-group-02_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/equation-lim-print2020_silhouette.tsv](base/wasm/equation-lim-print2020_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `ce8c9b8c67d2df44d724a629921e2be911bcc64671e314795e98d065d1f031ae` |
| [wasm/equation-lim-print2020_silhouette.tsv](wasm/equation-lim-print2020_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `63061d97515d2a09016bf904184b28e2207401f70a675f56a7db35655f240bc4` |
| [base/wasm/form-hwpx_silhouette.tsv](base/wasm/form-hwpx_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `b62edbd235584ed8a81745d76985a6ac02a52e5477b551231cfd174229f6be58` |
| [wasm/form-hwpx_silhouette.tsv](wasm/form-hwpx_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `b62edbd235584ed8a81745d76985a6ac02a52e5477b551231cfd174229f6be58` |
| [base/wasm/form-hwp-manual-crossformat_silhouette.tsv](base/wasm/form-hwp-manual-crossformat_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 136 | `b62edbd235584ed8a81745d76985a6ac02a52e5477b551231cfd174229f6be58` |
| [wasm/form-hwp-manual-crossformat_silhouette.tsv](wasm/form-hwp-manual-crossformat_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 136 | `b62edbd235584ed8a81745d76985a6ac02a52e5477b551231cfd174229f6be58` |
| [base/wasm/native-ime_silhouette.tsv](base/wasm/native-ime_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/native-ime_silhouette.tsv](wasm/native-ime_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/hancom-color-probe-2024_silhouette.tsv](base/wasm/hancom-color-probe-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/hancom-color-probe-2024_silhouette.tsv](wasm/hancom-color-probe-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/hancom-color-probe-cross-2024_silhouette.tsv](base/wasm/hancom-color-probe-cross-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/hancom-color-probe-cross-2024_silhouette.tsv](wasm/hancom-color-probe-cross-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/hancom-color-probe-boundary-2024_silhouette.tsv](base/wasm/hancom-color-probe-boundary-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/hancom-color-probe-boundary-2024_silhouette.tsv](wasm/hancom-color-probe-boundary-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/hancom-blackwhite-zero-2024_silhouette.tsv](base/wasm/hancom-blackwhite-zero-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/hancom-blackwhite-zero-2024_silhouette.tsv](wasm/hancom-blackwhite-zero-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base/wasm/hancom-blackwhite-axis-probe-2024_silhouette.tsv](base/wasm/hancom-blackwhite-axis-probe-2024_silhouette.tsv) | `9415b920c8dec299e8ada69b01dbd1c3c044f1f1` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [wasm/hancom-blackwhite-axis-probe-2024_silhouette.tsv](wasm/hancom-blackwhite-axis-probe-2024_silhouette.tsv) | `3535106051394672e93ca8401ff9eae8b449ec20` | 138 | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
