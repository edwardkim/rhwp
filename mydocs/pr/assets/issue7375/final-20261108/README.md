# 최종 전쪽 검증 증적

실제 조판 producer는 `5011277356ad04aac8b07df7e56cc695ba297943`이며, 고정 devel은 `5077b216cbc51d2046da07350de562f6f521a7c1`, 리베이스 직전은 `e944fff2f919f540c19e21caf4902b92f5a6b6da`이다. 최종 asset commit과 실행 producer를 구분한다.

완료 범위는 CBTA를 제외한 20조건·backend당 562쪽이다. 모든 쪽이 98% 이상이며 고정 devel/리베이스 전 대비 0.1%p 초과 하락·누락·쪽수 차이는 0이다. RGB 전체 동일을 주장하지 않는다.

Native 최저 98.31507%, fresh WASM 최저 98.32714%.

## 입력·독립 Print·전쪽 TSV

| 조건·입력 | 독립 Print PDF | 쪽수 | Native 최저 / TSV | fresh WASM 최저 / TSV |
| --- | --- | ---: | --- | --- |
| delegation7700: [10857_delegation_rules.hwp](<../../../../../samples/issue5679/10857_delegation_rules.hwp>) | [Print](<../../../../../pdf/semanticist21-20261008/issue5679-10857_delegation_rules-de43607f0507-2020.pdf>) | 103 | 99.21307% · [TSV](candidate/native/delegation7700.tsv) | 99.21221% · [TSV](candidate/wasm/delegation7700.tsv) |
| prime-width: [equation-prime-width-diagnostic-2020.hwp](<../../../../../tests/fixtures/equation-font-metrics/equation-prime-width-diagnostic-2020.hwp>) | [Print](<../../../../../pdf/equation-font-metrics/equation-prime-width-diagnostic-print-2020.pdf>) | 3 | 99.95925% · [TSV](candidate/native/prime-width.tsv) | 99.99209% · [TSV](candidate/wasm/prime-width.tsv) |
| rowbreak8539: [nhis-dependent-requirements.hwp](<../../../../../samples/issue8539/nhis-dependent-requirements.hwp>) | [Print](<../../../../../samples/issue8539/nhis-dependent-requirements-2020.pdf>) | 3 | 100.00000% · [TSV](candidate/native/rowbreak8539.tsv) | 100.00000% · [TSV](candidate/wasm/rowbreak8539.tsv) |
| science: [exam_science.hwp](<../../../../../samples/exam_science.hwp>) | [Print](<../../../../../pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf>) | 4 | 99.19496% · [TSV](candidate/native/science.tsv) | 99.22469% · [TSV](candidate/wasm/science.tsv) |
| nov-resaved: [nov-original-resaved-2020.hwp](<../../../../../tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp>) | [Print](<../../../../../pdf/pr7600-equation-font-frame-diagnostic/nov-original-resaved-print-2020.pdf>) | 21 | 99.03786% · [TSV](candidate/native/nov-resaved.tsv) | 99.02442% · [TSV](candidate/wasm/nov-resaved.tsv) |
| nov-original: [3-11월_실전_통합_2022.hwp](<../../../../../samples/3-11월_실전_통합_2022.hwp>) | [Print](<../../pr_7600_control_print_20261006/input-09-3-11월_실전_통합_2022-2020.pdf>) | 21 | 98.78575% · [TSV](candidate/native/nov-original.tsv) | 98.77721% · [TSV](candidate/wasm/nov-original.tsv) |
| rectangles: [aaaaaa.hwp](<../../../../../samples/issue7333/aaaaaa.hwp>) | [Print](<../../../../../pdf/issue7333/aaaaaa-2020.pdf>) | 50 | 98.31507% · [TSV](candidate/native/rectangles.tsv) | 98.32714% · [TSV](candidate/wasm/rectangles.tsv) |
| staggered7: [equal-height-partial-block.hwp](<../../../../../tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp>) | [Print](<../../../../../pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf>) | 7 | 99.61307% · [TSV](candidate/native/staggered7.tsv) | 99.61347% · [TSV](candidate/wasm/staggered7.tsv) |
| electrical7728: [70833-electrical-safety-rule-regulatory-analysis.hwp](<../../../../../samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp>) | [Print](<../../../../../pdf/issue7728/70833-electrical-safety-rule-hancom2020-print-20261106.pdf>) | 18 | 98.88416% · [TSV](candidate/native/electrical7728.tsv) | 98.81333% · [TSV](candidate/wasm/electrical7728.tsv) |
| giant4138: [saved-split-1x2.hwp](<../../issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp>) | [Print](<../../../../../pdf/issue4138-ascii-split-20261016-2024.pdf>) | 197 | 98.47416% · [TSV](candidate/native/giant4138.tsv) | 98.50155% · [TSV](candidate/wasm/giant4138.tsv) |
| pii5846: [75544_pii_bunseok.hwpx](<../../../../../samples/task2097/75544_pii_bunseok.hwpx>) | [Print](<../../../../../pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf>) | 66 | 99.21550% · [TSV](candidate/native/pii5846.tsv) | 99.21467% · [TSV](candidate/wasm/pii5846.tsv) |
| lpg71352: [71352_lpg_regulatory_impact.hwp](<../../../../../samples/issue8060/71352_lpg_regulatory_impact.hwp>) | [Print](<../../../../../pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf>) | 48 | 99.24608% · [TSV](candidate/native/lpg71352.tsv) | 99.26513% · [TSV](candidate/wasm/lpg71352.tsv) |
| clip5587: [issue1994_behindtext_table_20200830.hwp](<../../../../../samples/basic/issue1994_behindtext_table_20200830.hwp>) | [Print](<../../../../../pdf/semanticist21-20261008/basic-issue1994_behindtext_table_20200830-8e7a95cf5919-2020.pdf>) | 4 | 98.75606% · [TSV](candidate/native/clip5587.tsv) | 98.62409% · [TSV](candidate/wasm/clip5587.tsv) |
| textbox-control: [header-footer-paper-frame.hwpx](<../../../../../samples/header-footer-paper-frame.hwpx>) | [Print](<../../../../../pdf/pblsketch_review_20261030/header-footer-paper-frame-2024.pdf>) | 1 | 100.00000% · [TSV](candidate/native/textbox-control.tsv) | 100.00000% · [TSV](candidate/wasm/textbox-control.tsv) |
| textbox-repro: [font_list_name_fallback.hwpx](<../../../../../pdf/issue8687/font_list_name_fallback.hwpx>) | [Print](<../../../../../pdf/issue8687/font_list_name_fallback-2024.pdf>) | 1 | 100.00000% · [TSV](candidate/native/textbox-repro.tsv) | 100.00000% · [TSV](candidate/wasm/textbox-repro.tsv) |
| textbox-used: [font_list_name_used_fallback.hwpx](<../../../../../pdf/issue8687/font_list_name_used_fallback.hwpx>) | [Print](<../../../../../pdf/issue8687/font_list_name_used_fallback-2024.pdf>) | 1 | 100.00000% · [TSV](candidate/native/textbox-used.tsv) | 100.00000% · [TSV](candidate/wasm/textbox-used.tsv) |
| incoming-8555: [21868765_cell_text_edit.hwp](<../../../../../samples/issue8555/21868765_cell_text_edit.hwp>) | [Print](<../../../../../pdf/issue8555/21868765_cell_text_edit-hwp-mac-12.30.pdf>) | 4 | 98.74480% · [TSV](candidate/native/incoming-8555.tsv) | 98.74711% · [TSV](candidate/wasm/incoming-8555.tsv) |
| incoming-8556: [21868765_cell_picture_insert.hwp](<../../../../../samples/issue8556/21868765_cell_picture_insert.hwp>) | [Print](<../../../../../pdf/issue8556/21868765_cell_picture_insert-hwp-mac-12.30.pdf>) | 5 | 98.93199% · [TSV](candidate/native/incoming-8556.tsv) | 98.92990% · [TSV](candidate/wasm/incoming-8556.tsv) |
| incoming-8557: [21868765_relocated_picture.hwp](<../../../../../samples/issue8557/21868765_relocated_picture.hwp>) | [Print](<../../../../../pdf/issue8557/21868765_relocated_picture-hwp-mac-12.30.pdf>) | 4 | 98.84129% · [TSV](candidate/native/incoming-8557.tsv) | 98.84344% · [TSV](candidate/wasm/incoming-8557.tsv) |
| leading-original: [report-implicit-column.hwpx](<../../leading-definition-slots/input/report-implicit-column.hwpx>) | [Print](<../../../../../pdf/leading-definition-slots/report-implicit-column-hancom-print.pdf>) | 1 | 100.00000% · [TSV](candidate/native/leading-original.tsv) | 100.00000% · [TSV](candidate/wasm/leading-original.tsv) |

## 환경과 범위 구분

- 정본 visual_sweep.py·색상 경계 보정 전 2px raw·현재 PDF rasterizer를 사용했다. 각 입력의 compat/DPI와 글꼴 공급은 아래 실행 원장에 기록한다. 분할 저장본은 192DPI, 일반 문서는 원래 검증 조건을 유지했다.
- macOS Native는 실제 설치 글꼴/명시적 Print 환경을 공급했다. 편람383쪽과 Rust 의미 검사는 별도 macOS sandbox에서 사용자/추가 시스템 글꼴 및 fontserver 접근을 차단했다. 개인 글꼴을 비활성화·삭제하지 않았다.
- Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 통과가 아니다.
- `candidate/`, `fixed-base/`, `before-rebase/`의 TSV는 생성 원문을 바이트 그대로 복사했다. [전쪽 하락 대조](pagewise-preservation.tsv)와 [파일별 SHA-256·producer](assets.tsv)로 검증한다.
- 뒤배경 표 원본의 Print 환경은 사용자가 승인한 Haansoft Batang 선택과 한겨레결체 원본 공급을 사용한다. 설치 글꼴 바이너리는 제출하지 않는다.
- 편람 Native 383쪽은 별도 보존 검사이며 [TSV](handbook-native.tsv)에 원값을 남긴다. 표시 반올림 100%와 원값 100%를 구분한다.
- incoming8741은 새 devel의32조건·88쪽을 양 backend·base/후보로 대조한128조건이다. 원래20조건의562쪽에 합산하지 않는다.
- source-frames는 기존 #7765의 25~29쪽 보존 대조이며 새 전쪽 완료 주장이 아니다. Native 최저 96.98466%, WASM 최저 98.05371%이며 28/29쪽은 양쪽 98% 이상이다. base Native 29.81354%와 구분하고 새 하락은 없다.
- CBTA#7727은 사용자 지정 제외 범위다. 기존 부분 보정·기록·검사는 유지하고 해결/종료 대상으로 주장하지 않는다. #7445 전체 완료도 주장하지 않는다.

## 실행 재현 정보

| 조건 | compat | DPI | 입력 SHA-256 | PDF SHA-256 |
| --- | --- | ---: | --- | --- |
| delegation7700 | 2022 | 96 | `de43607f05072840097b19a294b4455905210402cc42823545125e03567df881` | `90eda86b2b4dfeabc601cd9b583408c6528a9efabf4e626fd6f2e28e3bcb2b97` |
| prime-width | 2022 | 96 | `03f38b458201832b4bab3cb2c040b6bb2640e125740c78277ed8be276afc1c44` | `12057cf4571d018ea32db8546189712a2dce933fdbf3f4a7eaea92243b08d0d0` |
| rowbreak8539 | 2022 | 96 | `53d7763a0e5c9860e62c8dc3b20cfb515aac6f6eceeeef01ff0fb1abbdfe8fef` | `a54a96205d1916b44d98a71caf6114d47ddb34ebfa9f34263228f875e90e5da6` |
| science | 2022 | 96 | `22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8` | `1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14` |
| nov-resaved | 2022 | 96 | `1392d5dab6350d232f39a06aa813f1797aec399e41a9e632ffd93e421916f72b` | `9c737bf4a8700c5e31d0eb05bba16ea8e6bad4a3bfed7c3bf88334053abcbd79` |
| nov-original | 2022 | 96 | `bc8bccbb954c337d813d1af96f4e3047242124c2f2939163e282634eb721accd` | `b5dc3580c0c3716963c9686ec62fd779cedfea97f50d8f49eb98a2ec8fae369a` |
| rectangles | 2022 | 96 | `ab88150db2eb4652c945e6773ed40f61806782b6ca85694eb5f8c8ec390f96c5` | `fbc0ff34b909e8824e775872ff47ac7d6c904d2cb74504fbe9e7a8411b2c597b` |
| staggered7 | 2022 | 96 | `a0b4e465ba880cd0be611f84a3db69fa38b64b31cfc9a807f9a2fc86a998228d` | `a4c213bb53b86411340b432e8d347c5a6386807ceb606f001c043288bb115755` |
| electrical7728 | 2022 | 96 | `a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb` | `6cec821ff5897714669602c8707d66f669cfcb1187362c69101a0255c8d17956` |
| giant4138 | 2024 | 192 | `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9` | `5a82290de1571c692c7246fe6b57ae136602ae52c2068f2568fffc6c91bfd571` |
| pii5846 | 2022 | 96 | `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1` | `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770` |
| lpg71352 | 2024 | 96 | `4e9b240082ba91b99669325292cae2b62e88765854d1e7bbbc6d44146dc14d55` | `c2e1b4beea546d02436e85f9b491ab64093c6327bf6ef227f02f4f075608f501` |
| clip5587 | 2022 | 96 | `8e7a95cf591944bff56050879fa90251921ec57e28eac66d40c6fb8ad103016f` | `901d842b033e30be1e9b4a7da8109be9e165af96af61fc4f175809ffbecc88a8` |
| textbox-control | 2024 | 96 | `8589147dc4a70140d9ccd0c83b899932432541c9db9e714cc59cfdd4c16c67b6` | `6eef9e9a4f0b5345468c3be6c29ed4998613f981dea96cc2225cc0930795ddfc` |
| textbox-repro | 2024 | 96 | `b84f9c8815b9093135a1e3e630dd8b55aac62916275edac0dc0e8fc61520120e` | `38656e23c496930d4b80405eb71c044ffd8056a930eae82549bd7e2af8723d96` |
| textbox-used | 2024 | 96 | `cafee169b69eec6dc8595d5f98e5f03ca724ddf169c333b022b3a8874ed7c930` | `79f03ae09ae889f7f39443b00b8991c991d100087c31c1d16bbd4c00e73df4f8` |
| incoming-8555 | 2022 | 96 | `bbf1f72951b6ff5de258f806c5d2569214538b76a867ecebce10f9da8718ff9e` | `aa6a2bf0fb12525a5108fab7a812e7ba81f8b879e9a50651b5128dc36f722c93` |
| incoming-8556 | 2022 | 96 | `507362e6c73d6db4db73fe4b42ae8110c945747ec4b7564e9b83044cb9ceb074` | `bf19f16e555477bdbe63862bb196a0ad35e65942b77a3d11d41e65a1e3b1a57a` |
| incoming-8557 | 2022 | 96 | `0d7cfc1df9f86da46cb12ebe5aa1eb0fbd9e660d4302f7a6facee8296ee09001` | `568d2037f8b132c25b70c4a26b3bca0f13168b92d755ad1a13891ddc2e929107` |
| leading-original | 2022 | 96 | `28768b01550ba2596c5e2bd182454a39a6700c8789fbe95e198d83c3898c4a40` | `8e7af1fcc369dbc45405bd0073850d085122a9d3f9f637678f119b324831d0d7` |

실행 로그·중간 JSON·진단 스크립트는 ignored output에만 생성했다. 최종 결과·실제 판독·남은 차이와 현재 source 검증은 작업 보고서에서 기록한다.

[최종 구현·검증·직접 판독 및 잔여](validation.md)

실제 Native Skia PNG·Studio Canvas2D·지원 CanvasKit의 프로필별 원본은 [실제 소비 원장](actual.tsv)에 연결한다. SVG raster 비교와 구분한다. CanvasKit 항목은 지원 대조군만 포함하며 미지원 문서 fallback을 통과로 세지 않는다.

[새 devel 보존 128조건의 입력·Print·실제 producer·manifest checkout·전쪽 TSV 원장](incoming-inputs.tsv). 고정 base를 출력할 때 사용한 checkout의 HEAD와 실행한 frozen producer source를 별도 열로 구분한다.

## HFT 굵기 전진의 마지막 보존 보정

#8418의 같은 입력·독립 Print 2쪽이다. Native 초기 후보63c39702의99.89494%에서50112773의99.99715%로 복원됐고 고정devel99.99715%와 같다. fresh WASM 후보도99.99715%다. 같은 새 출력 PNG를 사용해 직접 별지·제목·날짜·하단기관명과 닫는 괄호 간격을 대조했다. 현재 SVG/raster를 새로 실행한 결과로 바꾸어 쓰지 않는다.

| producer | raw | review | overlay |
| --- | --- | --- | --- |
| before63c39702 Native | [raw](hft8418/before-native-p002-raw.png) | [review](hft8418/before-native-p002-review.png) | [overlay](hft8418/before-native-p002-overlay.png) |
| current50112773 Native | [raw](hft8418/native-p002-raw.png) | [review](hft8418/native-p002-review.png) | [overlay](hft8418/native-p002-overlay.png) |
| current50112773 fresh WASM | [raw](hft8418/wasm-p002-raw.png) | [review](hft8418/wasm-p002-review.png) | [overlay](hft8418/wasm-p002-overlay.png) |
