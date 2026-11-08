# 최종 전쪽 Native / fresh WASM 원본 TSV

엔진 source `d7d4d2ea6ed427f2f3bc8b59a936a5f99aca852b`. Native179조건·2,082쪽, fresh WASM179조건·2,082쪽을 전쪽 비교했다. 모든 쪽 raw≥90%, 독립 Print 쪽수 일치, 측정 실패0이다. 동일 문서의 서로 다른 기준 PDF·폰트 조건은 별도 조건으로 세며 고유 문서 수로 바꾸어 보고하지 않는다.

- Native binary SHA-256 `fb3c6d537ad5ae92f57b66d8f77a86f14e53619b91136d3d0a4e932153cf143a`.
- fresh WASM SHA-256 `24ff17cf6906c3779a268fd9b9bd43a8658fd493c5c3148242947d7488aa747c`.
- Print/96 DPI, raw2px. 계측 source `78f2f6bbfdfdfdf479dbc879bf8db73e83439c87`, PDFium153.0.7999.0/pypdfium2 5.13.0.
- base `224a326063a5aeab5a1ca8dcaa92918f55843ffc`와 같은 런타임인 실제 producer `6ed1682c3267ea786734eb2e75252916a861d1fd`를 구분한다. Native/fresh WASM 각각의 devel 대비0.1%p 이상 하락은0이다. 쪽수가 이미 다른 base 조건은 직접 점수 비교에서 분리하고 후보가 독립 PDF와 일치함을 확인했다.
- PrEP 새 MCP Print와 기존 Print의 전140쪽 raster byte identity 증거는 stage4118이다.
- 아래 `n/`·`w/`는 계측기가 쓴 원본 TSV를 바이트 그대로 복사했다. [summary.tsv](summary.tsv)는 별도의 입력/Print SHA·쪽수·최저값 색인이며 원본 TSV를 대신하지 않는다. 실제 font 경로·명령·예외 범위·직접 판독·정식 테스트는 `mydocs/working/task_m100_7642_stage3905.md`와 해당 대표 PNG 증거를 함께 따른다.

| 번호 | 조건 / 원본 | 쪽수 | Native 최저 | WASM 최저 | 원본 TSV |
| --- | --- | ---: | ---: | ---: | --- |
| 001 | `additional-2` / `samples/test-image.hwp` | 5 | 100.00000% | 100.00000% | [Native](n/001.tsv) / [WASM](w/001.tsv) |
| 002 | `additional-3` / `samples/수식-문자처럼취급-아님.hwp` | 1 | 99.55612% | 99.43503% | [Native](n/002.tsv) / [WASM](w/002.tsv) |
| 003 | `additional-prep-current-mcp-print` / `samples/issue2006/1790387_prep_final_report.hwpx` | 140 | 93.97471% | 93.70762% | [Native](n/003.tsv) / [WASM](w/003.tsv) |
| 004 | `additional-prep-original` / `samples/issue2006/1790387_prep_final_report.hwpx` | 140 | 93.97471% | 93.70762% | [Native](n/004.tsv) / [WASM](w/004.tsv) |
| 005 | `all-01` / `samples/21_언어_기출_편집가능본.hwp` | 15 | 95.19794% | 95.24383% | [Native](n/005.tsv) / [WASM](w/005.tsv) |
| 006 | `all-02` / `samples/3-09월_교육_통합_2022.hwp` | 23 | 95.05435% | 95.37371% | [Native](n/006.tsv) / [WASM](w/006.tsv) |
| 007 | `all-03` / `samples/3-09월_교육_통합_2022.hwpx` | 23 | 95.08296% | 95.40193% | [Native](n/007.tsv) / [WASM](w/007.tsv) |
| 008 | `all-04` / `samples/3-09월_교육_통합_2023.hwp` | 20 | 97.67861% | 97.55892% | [Native](n/008.tsv) / [WASM](w/008.tsv) |
| 009 | `all-05` / `samples/3-09월_교육_통합_2023.hwpx` | 20 | 97.72531% | 97.60553% | [Native](n/009.tsv) / [WASM](w/009.tsv) |
| 010 | `all-06` / `samples/3-09월_교육_통합_2024-격자기준종이.hwp` | 23 | 95.08296% | 95.40193% | [Native](n/010.tsv) / [WASM](w/010.tsv) |
| 011 | `all-07` / `samples/3-09월_교육_통합_2024-격자기준쪽.hwp` | 23 | 95.08296% | 95.40193% | [Native](n/011.tsv) / [WASM](w/011.tsv) |
| 012 | `all-08` / `samples/3-09월_교육_통합_2024-구분선아래20.hwp` | 23 | 95.08296% | 95.40193% | [Native](n/012.tsv) / [WASM](w/012.tsv) |
| 013 | `all-09` / `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` | 23 | 95.08823% | 95.40721% | [Native](n/013.tsv) / [WASM](w/013.tsv) |
| 014 | `all-10` / `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` | 23 | 95.08823% | 95.40721% | [Native](n/014.tsv) / [WASM](w/014.tsv) |
| 015 | `all-11` / `samples/3-09월_교육_통합_2024-미주사이20.hwp` | 24 | 96.48033% | 96.74455% | [Native](n/015.tsv) / [WASM](w/015.tsv) |
| 016 | `all-12` / `samples/3-10월_교육_통합_2022.hwp` | 18 | 98.07832% | 98.11724% | [Native](n/016.tsv) / [WASM](w/016.tsv) |
| 017 | `all-13` / `samples/3-10월_교육_통합_2022.hwpx` | 18 | 98.11499% | 98.15247% | [Native](n/017.tsv) / [WASM](w/017.tsv) |
| 018 | `all-14` / `samples/3-11월_실전_통합_2022.hwp` | 21 | 97.56166% | 97.61215% | [Native](n/018.tsv) / [WASM](w/018.tsv) |
| 019 | `all-15` / `samples/3-11월_실전_통합_2022.hwpx` | 21 | 97.61256% | 97.66388% | [Native](n/019.tsv) / [WASM](w/019.tsv) |
| 020 | `all-16` / `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp` | 22 | 97.61256% | 97.66388% | [Native](n/020.tsv) / [WASM](w/020.tsv) |
| 021 | `all-17` / `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwpx` | 22 | 97.61256% | 97.66388% | [Native](n/021.tsv) / [WASM](w/021.tsv) |
| 022 | `all-18` / `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp` | 21 | 96.49787% | 96.57753% | [Native](n/022.tsv) / [WASM](w/022.tsv) |
| 023 | `all-19` / `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwpx` | 21 | 96.49787% | 96.57753% | [Native](n/023.tsv) / [WASM](w/023.tsv) |
| 024 | `all-20` / `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp` | 22 | 97.61256% | 97.66388% | [Native](n/024.tsv) / [WASM](w/024.tsv) |
| 025 | `all-21` / `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 22 | 97.61256% | 97.66388% | [Native](n/025.tsv) / [WASM](w/025.tsv) |
| 026 | `all-22` / `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp` | 21 | 97.61256% | 97.66388% | [Native](n/026.tsv) / [WASM](w/026.tsv) |
| 027 | `all-23` / `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwpx` | 21 | 97.61256% | 97.66388% | [Native](n/027.tsv) / [WASM](w/027.tsv) |
| 028 | `all-24` / `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp` | 21 | 97.61256% | 97.66388% | [Native](n/028.tsv) / [WASM](w/028.tsv) |
| 029 | `all-25` / `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwpx` | 21 | 97.61256% | 97.66388% | [Native](n/029.tsv) / [WASM](w/029.tsv) |
| 030 | `all-26` / `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp` | 21 | 96.49787% | 96.57753% | [Native](n/030.tsv) / [WASM](w/030.tsv) |
| 031 | `all-27` / `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwpx` | 21 | 96.49787% | 96.57753% | [Native](n/031.tsv) / [WASM](w/031.tsv) |
| 032 | `all-28` / `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp` | 21 | 97.61256% | 97.66388% | [Native](n/032.tsv) / [WASM](w/032.tsv) |
| 033 | `all-29` / `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwpx` | 21 | 97.61256% | 97.66388% | [Native](n/033.tsv) / [WASM](w/033.tsv) |
| 034 | `all-30` / `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp` | 21 | 97.64288% | 97.68557% | [Native](n/034.tsv) / [WASM](w/034.tsv) |
| 035 | `all-31` / `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwpx` | 21 | 97.61256% | 97.66388% | [Native](n/035.tsv) / [WASM](w/035.tsv) |
| 036 | `all-32` / `samples/eq-002.hwp` | 1 | 93.52679% | 93.76592% | [Native](n/036.tsv) / [WASM](w/036.tsv) |
| 037 | `all-33` / `samples/exam-kor-1p.hwp` | 1 | 99.76684% | 99.76577% | [Native](n/037.tsv) / [WASM](w/037.tsv) |
| 038 | `all-34` / `samples/exam-kor-2p.hwp` | 2 | 99.36253% | 99.36330% | [Native](n/038.tsv) / [WASM](w/038.tsv) |
| 039 | `all-35` / `samples/exam-kor-3p.hwp` | 3 | 98.84014% | 98.88842% | [Native](n/039.tsv) / [WASM](w/039.tsv) |
| 040 | `all-36` / `samples/exam-kor-4p.hwp` | 4 | 98.77781% | 98.82952% | [Native](n/040.tsv) / [WASM](w/040.tsv) |
| 041 | `all-37` / `samples/exam_eng.hwp` | 8 | 97.10866% | 96.88952% | [Native](n/041.tsv) / [WASM](w/041.tsv) |
| 042 | `all-38` / `samples/exam_kor.hwp` | 20 | 92.51027% | 92.56032% | [Native](n/042.tsv) / [WASM](w/042.tsv) |
| 043 | `all-39` / `samples/exam_math.hwp` | 20 | 92.03354% | 92.21948% | [Native](n/043.tsv) / [WASM](w/043.tsv) |
| 044 | `all-40` / `samples/exam_math_8.hwp` | 1 | 99.57770% | 99.89101% | [Native](n/044.tsv) / [WASM](w/044.tsv) |
| 045 | `all-41` / `samples/exam_math_no.hwp` | 20 | 91.06380% | 91.24697% | [Native](n/045.tsv) / [WASM](w/045.tsv) |
| 046 | `all-42` / `samples/exam_science.hwp` | 4 | 93.70755% | 93.61777% | [Native](n/046.tsv) / [WASM](w/046.tsv) |
| 047 | `all-43` / `samples/exam_social-p1.hwp` | 1 | 98.91779% | 98.99091% | [Native](n/047.tsv) / [WASM](w/047.tsv) |
| 048 | `all-44` / `samples/exam_social.hwp` | 4 | 98.88036% | 98.91383% | [Native](n/048.tsv) / [WASM](w/048.tsv) |
| 049 | `all-45` / `samples/hwpx/eq-002.hwpx` | 1 | 93.52679% | 93.76592% | [Native](n/049.tsv) / [WASM](w/049.tsv) |
| 050 | `all-46` / `samples/hwpx/exam-kor-1p.hwpx` | 1 | 99.76684% | 99.76577% | [Native](n/050.tsv) / [WASM](w/050.tsv) |
| 051 | `all-47` / `samples/hwpx/exam-kor-2p.hwpx` | 2 | 99.36253% | 99.36330% | [Native](n/051.tsv) / [WASM](w/051.tsv) |
| 052 | `all-48` / `samples/hwpx/exam-kor-3p.hwpx` | 3 | 98.84014% | 98.88842% | [Native](n/052.tsv) / [WASM](w/052.tsv) |
| 053 | `all-49` / `samples/hwpx/exam-kor-4p.hwpx` | 4 | 98.77781% | 98.82952% | [Native](n/053.tsv) / [WASM](w/053.tsv) |
| 054 | `all-50` / `samples/hwpx/exam_kor.hwpx` | 20 | 92.51027% | 92.56032% | [Native](n/054.tsv) / [WASM](w/054.tsv) |
| 055 | `all-51` / `samples/hwpx/exam_social-p1.hwpx` | 1 | 98.91779% | 98.99091% | [Native](n/055.tsv) / [WASM](w/055.tsv) |
| 056 | `all-52` / `samples/hwpx/exam_social.hwpx` | 4 | 98.07087% | 98.05641% | [Native](n/056.tsv) / [WASM](w/056.tsv) |
| 057 | `all-two-row-no-split` / `mydocs/pr/assets/issue7642/independent_controls_20261019/all-two-row-no-split.hwpx` | 10 | 93.37061% | 93.35370% | [Native](n/057.tsv) / [WASM](w/057.tsv) |
| 058 | `background-cell-background-layers` / `tests/fixtures/issue7661_backgrounds/cell-background-layers.hwpx` | 1 | 99.42153% | 99.63889% | [Native](n/058.tsv) / [WASM](w/058.tsv) |
| 059 | `background-connected-backgrounds` / `tests/fixtures/issue7661_backgrounds/connected-backgrounds.hwpx` | 1 | 99.47172% | 99.52104% | [Native](n/059.tsv) / [WASM](w/059.tsv) |
| 060 | `background-continued-paragraph` / `tests/fixtures/issue7661_backgrounds/continued-paragraph.hwpx` | 3 | 99.96287% | 99.95863% | [Native](n/060.tsv) / [WASM](w/060.tsv) |
| 061 | `background-flow-band-ownership` / `tests/fixtures/issue7661_backgrounds/flow-band-ownership.hwpx` | 1 | 99.74446% | 98.68802% | [Native](n/061.tsv) / [WASM](w/061.tsv) |
| 062 | `bitmap-crop-palette-print` / `pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls.hwpx` | 1 | 99.98227% | 99.98255% | [Native](n/062.tsv) / [WASM](w/062.tsv) |
| 063 | `bitmap-indexed-palette` / `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-indexed-color-controls.hwpx` | 1 | 99.74919% | 99.74198% | [Native](n/063.tsv) / [WASM](w/063.tsv) |
| 064 | `bitmap-white-palette` / `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-white-palette-controls.hwpx` | 1 | 99.75947% | 99.75526% | [Native](n/064.tsv) / [WASM](w/064.tsv) |
| 065 | `colon-authored` / `pdf/issue7642-colon-authored-20261016/colon-authored.hwp` | 8 | 99.14071% | 99.13103% | [Native](n/065.tsv) / [WASM](w/065.tsv) |
| 066 | `compound-fill` / `tests/fixtures/shapes/hancom-2020-compound-rectangles.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/066.tsv) / [WASM](w/066.tsv) |
| 067 | `compound-wide` / `mydocs/pr/assets/issue7642/independent_controls_20261019/a3-authored.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/067.tsv) / [WASM](w/067.tsv) |
| 068 | `condensed-title` / `samples/issue5821/condensed_ratio_title_box.hwpx` | 1 | 99.98787% | 100.00000% | [Native](n/068.tsv) / [WASM](w/068.tsv) |
| 069 | `crossing-span` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/crossing-span/crossing-span.hwpx` | 2 | 100.00000% | 100.00000% | [Native](n/069.tsv) / [WASM](w/069.tsv) |
| 070 | `crossing-span-24000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/crossing-span-24000/crossing-span-24000.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/070.tsv) / [WASM](w/070.tsv) |
| 071 | `crossing-span-36000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/crossing-span-36000/crossing-span-36000.hwpx` | 10 | 100.00000% | 100.00000% | [Native](n/071.tsv) / [WASM](w/071.tsv) |
| 072 | `diagnostic-char-style-ascii-2020` / `pdf/issue7642-hft-char-style-diagnostic/char-style-ascii-2020.hwp` | 1 | 97.36530% | 98.95378% | [Native](n/072.tsv) / [WASM](w/072.tsv) |
| 073 | `diagnostic-hft-full-ascii-saved-2020` / `mydocs/pr/assets/issue7642/independent_controls_20261019/hft-full-ascii-saved-2020.hwp` | 1 | 99.52785% | 99.79573% | [Native](n/073.tsv) / [WASM](w/073.tsv) |
| 074 | `diagnostic-junggothic-alternative-saved-2020` / `mydocs/pr/assets/issue7642/independent_controls_20261019/junggothic-alternative-saved-2020.hwp` | 1 | 91.05512% | 90.33230% | [Native](n/074.tsv) / [WASM](w/074.tsv) |
| 075 | `diagnostic-junggothic-program-saved-2020` / `mydocs/pr/assets/issue7642/independent_controls_20261019/junggothic-program-saved-2020.hwp` | 1 | 91.05512% | 90.33230% | [Native](n/075.tsv) / [WASM](w/075.tsv) |
| 076 | `distribution-2020` / `samples/task1768/distribution_doc.hwpx` | 3 | 98.39638% | 98.38449% | [Native](n/076.tsv) / [WASM](w/076.tsv) |
| 077 | `equation-lim-print2020` / `samples/equation-lim.hwp` | 1 | 99.88926% | 100.00000% | [Native](n/077.tsv) / [WASM](w/077.tsv) |
| 078 | `explicit-lines-center` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/explicit-lines-center/explicit-lines-center.hwpx` | 2 | 98.20348% | 100.00000% | [Native](n/078.tsv) / [WASM](w/078.tsv) |
| 079 | `explicit-lines-top` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/explicit-lines-top/explicit-lines-top.hwpx` | 2 | 98.20348% | 100.00000% | [Native](n/079.tsv) / [WASM](w/079.tsv) |
| 080 | `explicit-pen-cardinality` / `pdf/issue7642-metafile-palette-visibility-20261017/metafile-cardinality-explicit-pen.hwpx` | 1 | 99.83265% | 99.82777% | [Native](n/080.tsv) / [WASM](w/080.tsv) |
| 081 | `explicit-space-terms` / `pdf/issue7642-explicit-space-term-diagnostic-20261017/explicit-space-terms-saved-2024.hwp` | 6 | 99.99141% | 99.99149% | [Native](n/081.tsv) / [WASM](w/081.tsv) |
| 082 | `failed-emf-7266` / `samples/issue5637/2817919_emfplus_ole_preview.hwpx` | 1 | 92.18967% | 92.02698% | [Native](n/082.tsv) / [WASM](w/082.tsv) |
| 083 | `failed-wmf-6865` / `samples/issue6866/156627451-quantum-science-press-note.hwpx` | 15 | 93.69925% | 93.80427% | [Native](n/083.tsv) / [WASM](w/083.tsv) |
| 084 | `failure1440-hwp` / `samples/[2027] 온새미로 1 본교재.hwp` | 46 | 90.17559% | 90.10782% | [Native](n/084.tsv) / [WASM](w/084.tsv) |
| 085 | `failure1440-hwpx` / `samples/[2027] 온새미로 1 본교재.hwpx` | 46 | 90.16447% | 90.11795% | [Native](n/085.tsv) / [WASM](w/085.tsv) |
| 086 | `failure7095` / `samples/issue2004_cell_image_stack.hwp` | 8 | 98.07378% | 98.06449% | [Native](n/086.tsv) / [WASM](w/086.tsv) |
| 087 | `failure7418` / `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` | 74 | 94.95118% | 94.80658% | [Native](n/087.tsv) / [WASM](w/087.tsv) |
| 088 | `form-hwp-manual-crossformat` / `samples/form-01.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/088.tsv) / [WASM](w/088.tsv) |
| 089 | `form-hwpx` / `samples/hwpx/form-01.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/089.tsv) / [WASM](w/089.tsv) |
| 090 | `hancom-blackwhite-axis-probe-2024` / `tests/fixtures/issue_7751_color_controls/hancom-blackwhite-axis-probe-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/090.tsv) / [WASM](w/090.tsv) |
| 091 | `hancom-blackwhite-zero-2024` / `tests/fixtures/issue_7751_color_controls/hancom-blackwhite-zero-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/091.tsv) / [WASM](w/091.tsv) |
| 092 | `hancom-color-probe-2024` / `tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/092.tsv) / [WASM](w/092.tsv) |
| 093 | `hancom-color-probe-boundary-2024` / `tests/fixtures/issue_7751_color_controls/hancom-color-probe-boundary-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/093.tsv) / [WASM](w/093.tsv) |
| 094 | `hancom-color-probe-cross-2024` / `tests/fixtures/issue_7751_color_controls/hancom-color-probe-cross-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/094.tsv) / [WASM](w/094.tsv) |
| 095 | `headroom-1` / `tests/fixtures/issue7642_complete_plain_row_boundary/headroom-1.hwpx` | 8 | 100.00000% | 100.00000% | [Native](n/095.tsv) / [WASM](w/095.tsv) |
| 096 | `headroom-170` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/headroom-170/headroom-170.hwpx` | 8 | 93.36207% | 100.00000% | [Native](n/096.tsv) / [WASM](w/096.tsv) |
| 097 | `headroom-40` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/headroom-40/headroom-40.hwpx` | 8 | 100.00000% | 100.00000% | [Native](n/097.tsv) / [WASM](w/097.tsv) |
| 098 | `housing` / `tests/fixtures/issue_7047/housing-lease-standard-form.hwp` | 3 | 95.57354% | 95.54413% | [Native](n/098.tsv) / [WASM](w/098.tsv) |
| 099 | `hwpml` / `samples/hwp-3.0-HWPML.hwp` | 122 | 96.68211% | 96.49315% | [Native](n/099.tsv) / [WASM](w/099.tsv) |
| 100 | `independent-cells` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/independent-cells/independent-cells.hwpx` | 2 | 100.00000% | 100.00000% | [Native](n/100.tsv) / [WASM](w/100.tsv) |
| 101 | `independent-cells-24000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/independent-cells-24000/independent-cells-24000.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/101.tsv) / [WASM](w/101.tsv) |
| 102 | `independent-cells-36000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/independent-cells-36000/independent-cells-36000.hwpx` | 10 | 100.00000% | 100.00000% | [Native](n/102.tsv) / [WASM](w/102.tsv) |
| 103 | `inline-shape-line-baseline` / `samples/issue5789/tac_line_shape_baseline.hwpx` | 2 | 99.96389% | 100.00000% | [Native](n/103.tsv) / [WASM](w/103.tsv) |
| 104 | `issue1880_takeplace_host_before` / `samples/issue1880_takeplace_host_before.hwpx` | 10 | 93.37061% | 93.35370% | [Native](n/104.tsv) / [WASM](w/104.tsv) |
| 105 | `issue1950-current-mcp-print` / `samples/issue1950_hwp3_tab_charoffset.hwp` | 1 | 99.99235% | 99.96391% | [Native](n/105.tsv) / [WASM](w/105.tsv) |
| 106 | `issue1950-hwp3-tab-charoffset` / `samples/issue1950_hwp3_tab_charoffset.hwp` | 1 | 99.99235% | 99.96391% | [Native](n/106.tsv) / [WASM](w/106.tsv) |
| 107 | `issue7336-stored_frame_page_larger_rowbreak` / `samples/issue7336/stored_frame_page_larger_rowbreak.hwpx` | 7 | 96.86057% | 96.94182% | [Native](n/107.tsv) / [WASM](w/107.tsv) |
| 108 | `kopub-justify-controls-saved` / `mydocs/pr/assets/issue7642/independent_controls_20261019/kopub-justify-controls-saved-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/108.tsv) / [WASM](w/108.tsv) |
| 109 | `kopub-tracking-controls-saved` / `mydocs/pr/assets/issue7642/independent_controls_20261019/kopub-tracking-controls-saved-2024.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/109.tsv) / [WASM](w/109.tsv) |
| 110 | `leading-tracking-contract` / `tests/fixtures/justified_edge_spaces/leading-tracking-contract.hwpx` | 6 | 98.96842% | 98.85014% | [Native](n/110.tsv) / [WASM](w/110.tsv) |
| 111 | `logical-isolated` / `pdf/issue7642-logical-symbol-diagnostic-20261017/logical-symbol-isolated-fractions-saved-2024.hwp` | 6 | 100.00000% | 100.00000% | [Native](n/111.tsv) / [WASM](w/111.tsv) |
| 112 | `logical-original` / `pdf/issue7642-logical-symbol-diagnostic-20261017/logical-symbol-matrix-saved-2024.hwp` | 6 | 100.00000% | 100.00000% | [Native](n/112.tsv) / [WASM](w/112.tsv) |
| 113 | `long-tail` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/long-tail/long-tail.hwpx` | 8 | 95.87321% | 95.88090% | [Native](n/113.tsv) / [WASM](w/113.tsv) |
| 114 | `lseg-01-basic` / `samples/lseg-01-basic.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/114.tsv) / [WASM](w/114.tsv) |
| 115 | `lseg-02-mixed` / `samples/lseg-02-mixed.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/115.tsv) / [WASM](w/115.tsv) |
| 116 | `minimum-23100` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-23100/minimum-23100.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/116.tsv) / [WASM](w/116.tsv) |
| 117 | `minimum-24000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/independent-cells-24000/independent-cells-24000.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/117.tsv) / [WASM](w/117.tsv) |
| 118 | `minimum-25500` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-25500/minimum-25500.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/118.tsv) / [WASM](w/118.tsv) |
| 119 | `minimum-27000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-27000/minimum-27000.hwpx` | 8 | 100.00000% | 100.00000% | [Native](n/119.tsv) / [WASM](w/119.tsv) |
| 120 | `minimum-28500` / `tests/fixtures/issue7642_complete_plain_row_boundary/minimum-28500.hwpx` | 8 | 100.00000% | 100.00000% | [Native](n/120.tsv) / [WASM](w/120.tsv) |
| 121 | `minimum-30000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-30000/minimum-30000.hwpx` | 8 | 100.00000% | 100.00000% | [Native](n/121.tsv) / [WASM](w/121.tsv) |
| 122 | `minimum-480-bottom` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-480-bottom/minimum-480-bottom.hwpx` | 10 | 100.00000% | 100.00000% | [Native](n/122.tsv) / [WASM](w/122.tsv) |
| 123 | `minimum-480-center` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/minimum-480-center/minimum-480-center.hwpx` | 10 | 100.00000% | 100.00000% | [Native](n/123.tsv) / [WASM](w/123.tsv) |
| 124 | `mixed-stored-36000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/mixed-stored-36000/mixed-stored-36000.hwpx` | 10 | 100.00000% | 100.00000% | [Native](n/124.tsv) / [WASM](w/124.tsv) |
| 125 | `mixed-stored-6000` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/mixed-stored-6000/mixed-stored-6000.hwpx` | 2 | 100.00000% | 100.00000% | [Native](n/125.tsv) / [WASM](w/125.tsv) |
| 126 | `mixed-tail-valid_generated` / `tests/fixtures/pr7518_review_page_budget/valid_generated/nested-mixed-cell.hwp` | 8 | 95.87321% | 95.88090% | [Native](n/126.tsv) / [WASM](w/126.tsv) |
| 127 | `mixed-tail-valid_orientation` / `tests/fixtures/pr7518_review_page_budget/valid_orientation/nested-mixed-cell.hwp` | 8 | 95.87321% | 95.88090% | [Native](n/127.tsv) / [WASM](w/127.tsv) |
| 128 | `mixed-width-controls` / `mydocs/pr/assets/issue7642/independent_controls_20261019/mixed-width-controls-saved-2024.hwp` | 2 | 100.00000% | 100.00000% | [Native](n/128.tsv) / [WASM](w/128.tsv) |
| 129 | `more-room-long-tail` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/more-room-long-tail/more-room-long-tail.hwpx` | 8 | 95.87321% | 95.88090% | [Native](n/129.tsv) / [WASM](w/129.tsv) |
| 130 | `named-page-number-engine2020` / `samples/issue7336/nested_table_fragment_cut.hwp` | 6 | 97.07166% | 97.37778% | [Native](n/130.tsv) / [WASM](w/130.tsv) |
| 131 | `named-page-number-engine2024` / `samples/issue7336/nested_table_fragment_cut.hwp` | 6 | 97.07166% | 97.37778% | [Native](n/131.tsv) / [WASM](w/131.tsv) |
| 132 | `native-ime` / `mydocs/pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/132.tsv) / [WASM](w/132.tsv) |
| 133 | `negative-justify-controls` / `mydocs/pr/assets/issue7642/independent_controls_20261019/negative-justify-controls-saved-2024.hwp` | 4 | 100.00000% | 100.00000% | [Native](n/133.tsv) / [WASM](w/133.tsv) |
| 134 | `nested-host-spacing` / `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | 1 | 94.31466% | 94.11015% | [Native](n/134.tsv) / [WASM](w/134.tsv) |
| 135 | `normal-hancom-structures` / `tests/fixtures/equation-font-metrics/hancom-structure-saved-2024.hwp` | 16 | 99.58671% | 99.59576% | [Native](n/135.tsv) / [WASM](w/135.tsv) |
| 136 | `observed-print-fonts` / `samples/task2146/21761835_jeonjik_exemption_table.hwp` | 6 | 99.95485% | 99.95697% | [Native](n/136.tsv) / [WASM](w/136.tsv) |
| 137 | `operator-script-diagnostic` / `tests/fixtures/equation-font-metrics/operator-script-saved-2024.hwp` | 8 | 98.90385% | 98.89643% | [Native](n/137.tsv) / [WASM](w/137.tsv) |
| 138 | `original-table-sample` / `samples/hwp_table_test-m.hwp` | 3 | 98.09909% | 98.10665% | [Native](n/138.tsv) / [WASM](w/138.tsv) |
| 139 | `page-number-normal-latin-face-engine2020` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/page-number-normal-latin-face-engine2020/page-number-normal-latin-face.hwpx` | 1 | 94.18957% | 94.08539% | [Native](n/139.tsv) / [WASM](w/139.tsv) |
| 140 | `page-number-normal-latin-face-engine2024` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/page-number-normal-latin-face-engine2020/page-number-normal-latin-face.hwpx` | 1 | 94.52682% | 94.32063% | [Native](n/140.tsv) / [WASM](w/140.tsv) |
| 141 | `page-number-normal-size-color-engine2020` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/page-number-normal-size-color-engine2020/page-number-normal-size-color.hwpx` | 1 | 94.18957% | 94.08539% | [Native](n/141.tsv) / [WASM](w/141.tsv) |
| 142 | `page-number-normal-size-color-engine2024` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/page-number-normal-size-color-engine2020/page-number-normal-size-color.hwpx` | 1 | 94.49215% | 94.30052% | [Native](n/142.tsv) / [WASM](w/142.tsv) |
| 143 | `page-number-original-engine2020` / `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | 1 | 94.18957% | 94.08539% | [Native](n/143.tsv) / [WASM](w/143.tsv) |
| 144 | `page-number-original-engine2024` / `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | 1 | 94.52232% | 94.31623% | [Native](n/144.tsv) / [WASM](w/144.tsv) |
| 145 | `paper-a3` / `pdf/issue7642-paper-shape-controls-authored-20261016/a3-authored.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/145.tsv) / [WASM](w/145.tsv) |
| 146 | `paper-a4` / `pdf/issue7642-paper-shape-controls-authored-20261016/a4-authored.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/146.tsv) / [WASM](w/146.tsv) |
| 147 | `paper-letter` / `pdf/issue7642-paper-shape-controls-authored-20261016/letter-authored.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/147.tsv) / [WASM](w/147.tsv) |
| 148 | `paragraphs-center` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/paragraphs-center/paragraphs-center.hwpx` | 2 | 98.20348% | 100.00000% | [Native](n/148.tsv) / [WASM](w/148.tsv) |
| 149 | `paragraphs-top` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/paragraphs-top/paragraphs-top.hwpx` | 2 | 98.20348% | 100.00000% | [Native](n/149.tsv) / [WASM](w/149.tsv) |
| 150 | `pic-crop-01` / `samples/pic-crop-01.hwp` | 1 | 99.99508% | 99.29125% | [Native](n/150.tsv) / [WASM](w/150.tsv) |
| 151 | `pr8003-field-01` / `samples/field-01.hwp` | 3 | 99.99969% | 99.99969% | [Native](n/151.tsv) / [WASM](w/151.tsv) |
| 152 | `readonly-control` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/readonly-control/readonly-control.hwpx` | 8 | 95.87321% | 95.88090% | [Native](n/152.tsv) / [WASM](w/152.tsv) |
| 153 | `sample16-axis-roundtrip` / `mydocs/pr/assets/issue7642/independent_controls_20261019/hwp3-sample16-axis-export.hwpx` | 64 | 94.37681% | 94.59897% | [Native](n/153.tsv) / [WASM](w/153.tsv) |
| 154 | `sample16-hwp3` / `samples/hwp3-sample16.hwp` | 64 | 96.89198% | 96.98804% | [Native](n/154.tsv) / [WASM](w/154.tsv) |
| 155 | `sample16-hwp5` / `samples/hwp3-sample16-hwp5.hwp` | 64 | 97.73925% | 97.69323% | [Native](n/155.tsv) / [WASM](w/155.tsv) |
| 156 | `sample16-hwpx` / `samples/hwp3-sample16-hwp5.hwpx` | 64 | 96.24353% | 96.14596% | [Native](n/156.tsv) / [WASM](w/156.tsv) |
| 157 | `shape-group-02` / `samples/shape-group-02.hwp` | 1 | 100.00000% | 100.00000% | [Native](n/157.tsv) / [WASM](w/157.tsv) |
| 158 | `shortcut` / `samples/basic/shortcut.hwp` | 7 | 96.81517% | 96.90836% | [Native](n/158.tsv) / [WASM](w/158.tsv) |
| 159 | `sliver-24001` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/sliver-24001/sliver-24001.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/159.tsv) / [WASM](w/159.tsv) |
| 160 | `sliver-24600` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/sliver-24600/sliver-24600.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/160.tsv) / [WASM](w/160.tsv) |
| 161 | `sliver-24900` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/sliver-24900/sliver-24900.hwpx` | 7 | 100.00000% | 100.00000% | [Native](n/161.tsv) / [WASM](w/161.tsv) |
| 162 | `space-font-size-controls` / `mydocs/pr/assets/issue7642/independent_controls_20261019/space-font-size-controls-saved-2024.hwp` | 2 | 100.00000% | 100.00000% | [Native](n/162.tsv) / [WASM](w/162.tsv) |
| 163 | `span-order-split` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/span-order-split/span-order-split.hwpx` | 2 | 100.00000% | 99.77161% | [Native](n/163.tsv) / [WASM](w/163.tsv) |
| 164 | `span-order-whole` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/span-order-whole/span-order-whole.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/164.tsv) / [WASM](w/164.tsv) |
| 165 | `split-blank0` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/split-blank0/split-blank0.hwpx` | 2 | 100.00000% | 99.93564% | [Native](n/165.tsv) / [WASM](w/165.tsv) |
| 166 | `split-blank3` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/split-blank3/split-blank3.hwpx` | 2 | 100.00000% | 99.91868% | [Native](n/166.tsv) / [WASM](w/166.tsv) |
| 167 | `startup-contest` / `samples/issue6132/156482639_startup_ir_contest.hwp` | 10 | 98.43179% | 98.47656% | [Native](n/167.tsv) / [WASM](w/167.tsv) |
| 168 | `superscript-5756` / `samples/issue5756/156732409_superscript_advance.hwp` | 3 | 98.69022% | 98.75599% | [Native](n/168.tsv) / [WASM](w/168.tsv) |
| 169 | `target-no-split` / `mydocs/pr/assets/issue7642/independent_controls_20261019/target-no-split.hwpx` | 10 | 93.37061% | 93.35370% | [Native](n/169.tsv) / [WASM](w/169.tsv) |
| 170 | `tracked-visibility-hwpx` / `tests/fixtures/issue5169_tracked_visibility/issue5169_viewtext_changetracking.hwpx` | 6 | 97.05260% | 97.07717% | [Native](n/170.tsv) / [WASM](w/170.tsv) |
| 171 | `trailing-tab-report` / `mydocs/pr/assets/issue7922/trailing-tab-regressions-20261015/1192000-202100017-policy-research-report.hwp` | 1 | 96.79999% | 96.79280% | [Native](n/171.tsv) / [WASM](w/171.tsv) |
| 172 | `unfinished-ending-minimum` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/unfinished-ending-minimum/unfinished-ending-minimum.hwpx` | 3 | 100.00000% | 99.65413% | [Native](n/172.tsv) / [WASM](w/172.tsv) |
| 173 | `unfinished-ending-span` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/unfinished-ending-span/unfinished-ending-span.hwpx` | 2 | 100.00000% | 100.00000% | [Native](n/173.tsv) / [WASM](w/173.tsv) |
| 174 | `vector-operator-gaps` / `pdf/issue7642-vector-operator-gap-diagnostic-20261017/vector-operator-gaps-saved-2024.hwp` | 12 | 95.95556% | 95.96483% | [Native](n/174.tsv) / [WASM](w/174.tsv) |
| 175 | `voff0` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/voff0/voff0.hwpx` | 3 | 99.27437% | 99.23551% | [Native](n/175.tsv) / [WASM](w/175.tsv) |
| 176 | `voff1904` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/voff1904/voff1904.hwpx` | 3 | 99.26301% | 99.23551% | [Native](n/176.tsv) / [WASM](w/176.tsv) |
| 177 | `whole-blank0` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/whole-blank0/whole-blank0.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/177.tsv) / [WASM](w/177.tsv) |
| 178 | `whole-blank3` / `mydocs/pr/assets/issue7642/preservation_controls_20261019/whole-blank3/whole-blank3.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/178.tsv) / [WASM](w/178.tsv) |
| 179 | `wmf-aspect-control` / `pdf/issue7642-metafile-palette-visibility-20261017/metafile-aspect-explicit-pen.hwpx` | 1 | 100.00000% | 100.00000% | [Native](n/179.tsv) / [WASM](w/179.tsv) |

## devel 대조의 범위

- native: 비교 가능한1660쪽,0.1%p 이상 하락0, base 쪽수 불일치18조건. [페이지별 수치](native-devel-page-comparison.tsv). base 쪽수 불일치를 점수 무회귀로 치환하지 않는다.
- wasm: 비교 가능한1660쪽,0.1%p 이상 하락0, base 쪽수 불일치18조건. [페이지별 수치](wasm-devel-page-comparison.tsv). base 쪽수 불일치를 점수 무회귀로 치환하지 않는다.
