# 제품·원본·측정 신원

- 제품 source: `86709ac16711820e61baa3b87e2c4ddaac23573e`.
- 빌드 Git HEAD: `1da15804ebb5d6fc8f63b7ccb8410018bd42c29d`.
- Native binary SHA-256: `e52882bc5acda6cec799e1cf784861bfda514a867f55daf3b5e1f720b35109e1`.
- fresh WASM SHA-256: `4755dbfeda5bba99d72191a54e90342ae381ec17f0dd840926bbf1fdd75d1f30`.
- JS glue SHA-256: `2a099009a9582a9b34cccb3d61f41a1e4d21085eefd85b0ad8df466b18766138`.
- 같은 제품 tree에서 테스트·주석을 보완한 검토 코드 HEAD: `5172f2d55c73b2ed33d4d3657d81e769bc9004c2`.
- 공통 측정: 2026-11-06 KST, 96 DPI, Print, PDFium 5.13.0/153, 2px raw 실루엣.
- Native·fresh WASM 각각 128입력·301쪽. 독립 PDF 전체 쪽수 일치, 측정 누락·90% 미만 0쪽.
- Native 최저 91.81401%, WASM 최저 91.17272%. 같은 새 base의 301쪽씩 대조에서 하락 자체가 0쪽.

아래 파일 링크는 실제 검증에 사용한 bytes와 SHA-256이 같다. 기존 정식 경로의 파일을 중복 복사하지 않았다.
Mac 12.30 독립 Print와 MCP 2020/2024 Print를 구분하고 PDF 출처·제품을 임의로 바꾸지 않는다.
전체 페이지의 원래 TSV는 `tsv`, 고정 base `170b886f167b705b6e630c5b34bb638d92fae0a9`의 TSV는 `base-tsv`다.

## 입력별 결과

### 8527-levels-hwp-2022

- 원본: [tests/fixtures/numbering_levels/levels.hwp](../../../../../tests/fixtures/numbering_levels/levels.hwp), 36864 bytes, SHA-256 `55746c0211f05ef27ef293ad352e94d946d99b2f0251594faeb60a5e4cf31866`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8527/levels-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8527/levels-hwp-2020.pdf), 14578 bytes, SHA-256 `26e7f539286cb882eaef7b35ede404eeafff309188ecd33ad236cb5d0384bd1e`.
- 원 주장/용도: HWP 8~10수준 번호와 빈 서식. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8527-levels-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8527-levels-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8527-levels-hwpx-2022

- 원본: [tests/fixtures/numbering_levels/levels.hwpx](../../../../../tests/fixtures/numbering_levels/levels.hwpx), 37386 bytes, SHA-256 `0b3852a67e59ae28c100ca210c3c7d3e8ea812d877747da33cd1a46b81e912de`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8527/levels-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8527/levels-hwpx-2020.pdf), 14578 bytes, SHA-256 `67f050917e0aec31cf82d849e6f98c034b1ceec3b75a5fc87c5e2da5732bab0e`.
- 원 주장/용도: HWPX 8~10수준 번호와 빈 서식. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8527-levels-hwpx-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8527-levels-hwpx-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8528-follow-end-20pt-hwp-2022

- 원본: [tests/fixtures/numbering_char_shape/follow-end-20pt.hwp](../../../../../tests/fixtures/numbering_char_shape/follow-end-20pt.hwp), 24576 bytes, SHA-256 `01e60730a99a9e0a42db787dc8f9a0143d168af6a51f788796c6ac82b5c1ceaa`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8528/follow-end-20pt-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8528/follow-end-20pt-hwp-2020.pdf), 12783 bytes, SHA-256 `f181b12cb9934c9573a8f5c162e704ca62780cce75f2776c1d3c3267145553c3`.
- 원 주장/용도: 첫 글자가 아닌 문단 끝 크기를 따르는 번호. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-end-20pt-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-end-20pt-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8528-follow-two-sizes-hwp-2022

- 원본: [tests/fixtures/numbering_char_shape/follow-two-sizes.hwp](../../../../../tests/fixtures/numbering_char_shape/follow-two-sizes.hwp), 26624 bytes, SHA-256 `94306aa8dd8cdac456bfa666b5832fe8857491497720c85c8f5c8d43fc7df6f2`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8528/follow-two-sizes-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8528/follow-two-sizes-hwp-2020.pdf), 10067 bytes, SHA-256 `78dbee13ba462acac71b7c015713b4605505ede3e70cf4d04abbead017bd2860`.
- 원 주장/용도: 문단 끝 20pt·10pt를 따르는 번호 글자 크기. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-two-sizes-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-two-sizes-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8528-follow-two-sizes-hwpx-2022

- 원본: [tests/fixtures/numbering_char_shape/follow-two-sizes.hwpx](../../../../../tests/fixtures/numbering_char_shape/follow-two-sizes.hwpx), 27167 bytes, SHA-256 `837a78c8656fdcd3d6b81c8d0e27c7ea60b0ad6a4e33f1142bbdf18957cdb7fd`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8528/follow-two-sizes-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8528/follow-two-sizes-hwpx-2020.pdf), 10067 bytes, SHA-256 `89330cc6aec08d91c50eb90b8bd8257d128a4485702def3e42d5a86a0405fe4b`.
- 원 주장/용도: HWPX 번호 글자 모양. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-two-sizes-hwpx-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8528-follow-two-sizes-hwpx-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8536-note-formats-hwp-2022

- 원본: [tests/fixtures/number_shapes/note-formats.hwp](../../../../../tests/fixtures/number_shapes/note-formats.hwp), 44032 bytes, SHA-256 `c0150356a4f44e15a8100549f0f1e15172f0f95836a6988392703f7e4febdb77`.
- 독립 Print PDF: [pdf/number_shapes/note-formats-hancom-mac-12.30.pdf](../../../../../pdf/number_shapes/note-formats-hancom-mac-12.30.pdf), 60440 bytes, SHA-256 `433cf379642dbbca3de6a2e299ef36d096c2505f96a9424d6f5d8de37d11b892`.
- 원 주장/용도: HWP 각주·쪽 번호 모양. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 19쪽, 최저 99.98307%, [TSV](tsv/8536-note-formats-hwp-2022-native.tsv), SHA-256 `316f6f70e3ffebcf6fcae26b9d8d8b14caf6b6a8fc8684e4b25c8b648ce39807`, 500 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 19쪽, 최저 99.98279%, [TSV](tsv/8536-note-formats-hwp-2022-wasm.tsv), SHA-256 `316f6f70e3ffebcf6fcae26b9d8d8b14caf6b6a8fc8684e4b25c8b648ce39807`, 500 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8536-note-formats-hwpx-2022

- 원본: [tests/fixtures/number_shapes/note-formats.hwpx](../../../../../tests/fixtures/number_shapes/note-formats.hwpx), 63390 bytes, SHA-256 `59c121888f845d78ef5f2a202c4a136ae15079b67a3638af8915a55c3ce725eb`.
- 독립 Print PDF: [pdf/number_shapes/note-formats-hancom-mac-12.30.pdf](../../../../../pdf/number_shapes/note-formats-hancom-mac-12.30.pdf), 60440 bytes, SHA-256 `433cf379642dbbca3de6a2e299ef36d096c2505f96a9424d6f5d8de37d11b892`.
- 원 주장/용도: HWPX 각주·쪽 번호 모양. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 19쪽, 최저 99.98307%, [TSV](tsv/8536-note-formats-hwpx-2022-native.tsv), SHA-256 `316f6f70e3ffebcf6fcae26b9d8d8b14caf6b6a8fc8684e4b25c8b648ce39807`, 500 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 19쪽, 최저 99.98279%, [TSV](tsv/8536-note-formats-hwpx-2022-wasm.tsv), SHA-256 `316f6f70e3ffebcf6fcae26b9d8d8b14caf6b6a8fc8684e4b25c8b648ce39807`, 500 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8536-numbering-hwp-2022

- 원본: [tests/fixtures/number_shapes/numbering.hwp](../../../../../tests/fixtures/number_shapes/numbering.hwp), 154624 bytes, SHA-256 `c3f2bda6f7fa30f9bee85dfd6e2c46b281ba8a6628db0fe36596fc34c83bae88`.
- 독립 Print PDF: [pdf/number_shapes/numbering-hancom-mac-12.30.pdf](../../../../../pdf/number_shapes/numbering-hancom-mac-12.30.pdf), 61424 bytes, SHA-256 `23ca260403c7034e4e6e4d3993ebba5aca10502a2c04f01e34453d7da733505e`.
- 원 주장/용도: 문단 번호 모양 전체. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 6쪽, 최저 100.0%, [TSV](tsv/8536-numbering-hwp-2022-native.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 6쪽, 최저 100.0%, [TSV](tsv/8536-numbering-hwp-2022-wasm.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8536-numbering-hwpx-2022

- 원본: [tests/fixtures/number_shapes/numbering.hwpx](../../../../../tests/fixtures/number_shapes/numbering.hwpx), 152141 bytes, SHA-256 `06c16431c27c3c541bba842bcb4879adbaf4c71b941383af6f40ce6dece41aaf`.
- 독립 Print PDF: [pdf/number_shapes/numbering-hancom-mac-12.30.pdf](../../../../../pdf/number_shapes/numbering-hancom-mac-12.30.pdf), 61424 bytes, SHA-256 `23ca260403c7034e4e6e4d3993ebba5aca10502a2c04f01e34453d7da733505e`.
- 원 주장/용도: HWPX 문단 번호 모양 전체. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 6쪽, 최저 100.0%, [TSV](tsv/8536-numbering-hwpx-2022-native.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 6쪽, 최저 100.0%, [TSV](tsv/8536-numbering-hwpx-2022-wasm.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8536-numbering-wrap-hwp-2022

- 원본: [tests/fixtures/number_shapes/numbering-wrap.hwp](../../../../../tests/fixtures/number_shapes/numbering-wrap.hwp), 136704 bytes, SHA-256 `3fe8595974604a50d0df844a6c30d82e7f39d4ef56d90b9fdbc425c59f062340`.
- 독립 Print PDF: [pdf/number_shapes/numbering-wrap-hancom-mac-12.30.pdf](../../../../../pdf/number_shapes/numbering-wrap-hancom-mac-12.30.pdf), 28504 bytes, SHA-256 `04ed9f388dfd4b332203bd36f02e66c1ab0ca3313a1c84c660597bedae8d50e0`.
- 원 주장/용도: 모양별 차례 초과 번호. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/8536-numbering-wrap-hwp-2022-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/8536-numbering-wrap-hwp-2022-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8538-shape-hwp-2022

- 원본: [tests/fixtures/footnote_drawing/shape.hwp](../../../../../tests/fixtures/footnote_drawing/shape.hwp), 62464 bytes, SHA-256 `2ebcb057709da85d59f5f5eb29a997763e19324ab47eef3fbf30c6792d27bb91`.
- 독립 Print PDF: [pdf/footnote_drawing/shape-hancom-mac-12.30.pdf](../../../../../pdf/footnote_drawing/shape-hancom-mac-12.30.pdf), 35290 bytes, SHA-256 `2dd3906646849b23b50e57ab55659e9cc223bc776d754dba968a144a4ce08aae`.
- 원 주장/용도: 각주 본문 아래·작게·구분선 종류. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 92.35136%, [TSV](tsv/8538-shape-hwp-2022-native.tsv), SHA-256 `893e5e9a9fba86c4bbf5ce859577d3ed5e19a3dfc2c527b1f8e17c00720f2063`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 92.4024%, [TSV](tsv/8538-shape-hwp-2022-wasm.tsv), SHA-256 `db92151b9b71d0fff1847d373163e4e5af8a8dcffda86e8b24f767fee1e81275`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8538-shape-hwpx-2022

- 원본: [tests/fixtures/footnote_drawing/shape.hwpx](../../../../../tests/fixtures/footnote_drawing/shape.hwpx), 68146 bytes, SHA-256 `fd55f56c0e938e491c6fcef6df451bf40db9b6a640e51c3fd45e934088c9f733`.
- 독립 Print PDF: [pdf/footnote_drawing/shape-hancom-mac-12.30.pdf](../../../../../pdf/footnote_drawing/shape-hancom-mac-12.30.pdf), 35290 bytes, SHA-256 `2dd3906646849b23b50e57ab55659e9cc223bc776d754dba968a144a4ce08aae`.
- 원 주장/용도: HWPX 각주 배치·작은 번호·구분선. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 92.35136%, [TSV](tsv/8538-shape-hwpx-2022-native.tsv), SHA-256 `43ff85e5457068e847815adf50374ce1314640223af4d977f2e0d1344e917b05`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 92.4024%, [TSV](tsv/8538-shape-hwpx-2022-wasm.tsv), SHA-256 `b7bcf1bffacb980c191fbae2d7a6ce4be466e423dedeed1e58ed7edae26d24ec`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8541-hancom-two-sections-hwp-2022

- 원본: [tests/fixtures/section_split/hancom-two-sections.hwp](../../../../../tests/fixtures/section_split/hancom-two-sections.hwp), 26112 bytes, SHA-256 `e604c47d11a788340b8117c3d1e727130b2cf56fa54f16dae770c198b7306424`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/hancom-two-sections-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/hancom-two-sections-hwp-2020.pdf), 12799 bytes, SHA-256 `b27247993ae78e223627737cf2c371008e78ed31d3f617f1211bd2dd7a1ba05a`.
- 원 주장/용도: 2구역 원본 읽기; split/merge 편집 API와 구분. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/8541-hancom-two-sections-hwp-2022-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/8541-hancom-two-sections-hwp-2022-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8541-hancom-two-sections-hwpx-2022

- 원본: [tests/fixtures/section_split/hancom-two-sections.hwpx](../../../../../tests/fixtures/section_split/hancom-two-sections.hwpx), 28164 bytes, SHA-256 `156a4eea4276641acd545b6e757b85672d1018418251c57146f8cbaa30c67c19`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/hancom-two-sections-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/hancom-two-sections-hwpx-2020.pdf), 12799 bytes, SHA-256 `b7544ccf178e32f1c1142b3166a89facd9461282dcde0a80e7d459880b9c013f`.
- 원 주장/용도: HWPX 2구역 원본 읽기. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/8541-hancom-two-sections-hwpx-2022-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/8541-hancom-two-sections-hwpx-2022-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8549-rhwp-after-hwp-2022

- 원본: [mydocs/pr/assets/line-seg-start-outside-control/rhwp-after.hwp](../../line-seg-start-outside-control/rhwp-after.hwp), 15872 bytes, SHA-256 `ed1df84780c47a24d45cfa463ea9c74de59da5d8781cd33192e4b4ef0e1c1013`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8549/rhwp-after-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8549/rhwp-after-hwp-2020.pdf), 32704 bytes, SHA-256 `4355a8010f2ef1b052e005a89c8bb4fe9b6871fe93ef0301b37e4608688dc8bc`.
- 원 주장/용도: 제출된 정상 저장본 읽기; 현재 엔진 편집 저장 생성과 구분. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.91686%, [TSV](tsv/8549-rhwp-after-hwp-2022-native.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.91794%, [TSV](tsv/8549-rhwp-after-hwp-2022-wasm.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022

- 원본: [tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-before.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-before.hwp), 15872 bytes, SHA-256 `9a2edc8583033f54037722111827ffffb2496047b9c9a168086f61f029d09a7b`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8564/date-path-context-before-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8564/date-path-context-before-2020.pdf), 14003 bytes, SHA-256 `3ffbc8328d91a0a3121efaf324e63cb7cd2e9dcf3b9b4a4c78b40d380239db17`.
- 원 주장/용도: 한컴이 갱신한 날짜·파일명 코드 표시 상태, 원본 경계 입력 검사의 보충 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 91.81401%, [TSV](tsv/8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022-native.tsv), SHA-256 `08236633b9b2056904a573e9ee220675cab517e81e39b241b0cfedebc803aad1`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 91.17272%, [TSV](tsv/8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022-wasm.tsv), SHA-256 `b8de5e704d648249237538626050eb3c6f3cb9dedfbe7ffcb96cdd6169ebf967`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8572-fix-typed-hwp-2022

- 원본: [mydocs/pr/assets/tac-host-row-cell-growth/fix-typed.hwp](../../tac-host-row-cell-growth/fix-typed.hwp), 15872 bytes, SHA-256 `bc5a8bcdc54ed7e28e5f57e669cb022be80dfac10ea5da20627bd20c81d70004`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8572/fix-typed-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8572/fix-typed-hwp-2020.pdf), 116403 bytes, SHA-256 `93409e6622bf8895b92049a8380ed7b95a94eef6d689af6620ceb5e821b0de13`.
- 원 주장/용도: 제출된 표 셀 편집 정상 저장본과 후속 문단 배치. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 100.0%, [TSV](tsv/8572-fix-typed-hwp-2022-native.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 99.99572%, [TSV](tsv/8572-fix-typed-hwp-2022-wasm.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F16, F20, F22, F23, F24, F25, F26, F27, F28, F31, F32, F35, F36, F37, F38, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8572-original-footnote-hwp

- 원본: [samples/footnote-01.hwp](../../../../../samples/footnote-01.hwp), 32768 bytes, SHA-256 `5bbc8a8fd23415aad59dd91d1eb261050946d9c64635933fcd49609ec2cc94e5`.
- 독립 Print PDF: [pdf/footnote-01-hwp-2020.pdf](../../../../../pdf/footnote-01-hwp-2020.pdf), 114877 bytes, SHA-256 `2b47aa31b3d135f1621ba478189efcbfcacc77dfbc6c9d9082239f7885bb546e`.
- 원 주장/용도: 원 PR 제목 셀 편집 전 동일 원본·표 위치의 전쪽 정상 대조군. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 6쪽, 최저 100.0%, [TSV](tsv/8572-original-footnote-hwp-native.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 6쪽, 최저 99.99572%, [TSV](tsv/8572-original-footnote-hwp-wasm.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F16, F20, F22, F23, F24, F25, F26, F27, F28, F31, F32, F35, F36, F37, F38, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8577-hancom-page-number-top-hwpx-2022

- 원본: [tests/fixtures/page_number_position/hancom-page-number-top.hwpx](../../../../../tests/fixtures/page_number_position/hancom-page-number-top.hwpx), 26745 bytes, SHA-256 `4411fc1d14dea62152f72801a26aa1849966adb0acf635d26d7195c7d0e77bc0`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8577/hancom-page-number-top-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8577/hancom-page-number-top-hwpx-2020.pdf), 14275 bytes, SHA-256 `6fe2ac36d39f82b7d09da6c10740d599aa2ad6620270e0735e3fb18ec3a5ae33`.
- 원 주장/용도: 위쪽 쪽 번호 2쪽의 기준선. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/8577-hancom-page-number-top-hwpx-2022-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/8577-hancom-page-number-top-hwpx-2022-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8586-hancom-page-number-mid-hwpx-2022

- 원본: [tests/fixtures/page_number_position/hancom-page-number-mid.hwpx](../../../../../tests/fixtures/page_number_position/hancom-page-number-mid.hwpx), 24949 bytes, SHA-256 `d49cfee4fffd6107bd51ea0f307fae17f63b8ef7f37840a04f79f43c38c7b3cf`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8586/hancom-page-number-mid-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8586/hancom-page-number-mid-hwpx-2020.pdf), 17865 bytes, SHA-256 `970707d4004ad1d1cf208b5613297aa2c97af3da9f693fb5784785ffcffa4e4c`.
- 원 주장/용도: 문단 중간 pgnp 원본 읽기; 위치 변경 API와 구분. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 6쪽, 최저 100.0%, [TSV](tsv/8586-hancom-page-number-mid-hwpx-2022-native.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 6쪽, 최저 100.0%, [TSV](tsv/8586-hancom-page-number-mid-hwpx-2022-wasm.tsv), SHA-256 `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f`, 232 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8586-hancom-page-number-sections-hwpx-2022

- 원본: [tests/fixtures/page_number_position/hancom-page-number-sections.hwpx](../../../../../tests/fixtures/page_number_position/hancom-page-number-sections.hwpx), 26679 bytes, SHA-256 `79b8a99e5da76863402a9f2500cb24d50bec8db79186a4ec50fd2eb588ebd430`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8586/hancom-page-number-sections-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8586/hancom-page-number-sections-hwpx-2020.pdf), 19059 bytes, SHA-256 `0451615391b214b1d08d3b1f6292275279aeabb18561e4f3f2ea35b5ea125f92`.
- 원 주장/용도: 구역 간 쪽 번호 위치 이어받기 원본 읽기. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 100.0%, [TSV](tsv/8586-hancom-page-number-sections-hwpx-2022-native.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 100.0%, [TSV](tsv/8586-hancom-page-number-sections-hwpx-2022-wasm.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8602-rhwp-after-hwp-2022

- 원본: [mydocs/pr/assets/emptied-paragraph-para-text/rhwp-after.hwp](../../emptied-paragraph-para-text/rhwp-after.hwp), 12800 bytes, SHA-256 `a2ec7d4fe6dcb4520a113693dd615ac654fc7a8ff47c63c1d3d7bd9cc37c3cf5`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8602/rhwp-after-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8602/rhwp-after-hwp-2020.pdf), 9919 bytes, SHA-256 `9b1aea6ce4e44f11f834f678ce51ba36b85293555b22e80f4673dd6c5abb4f79`.
- 원 주장/용도: 제출된 정상 빈 문단 저장본 읽기; 현재 삭제 저장 생성과 구분. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8602-rhwp-after-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8602-rhwp-after-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-existing-caret-false-hwp

- 원본: [tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwp](../../../../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwp), 13312 bytes, SHA-256 `159b91e4500e8916594c93fcde8cda6ad7efbdb1ea13fea7a17a37a08c580d1c`.
- 독립 Print PDF: [pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwp-2020.pdf](../../../../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwp-2020.pdf), 16325 bytes, SHA-256 `d8cfa09f468ecca56497d4f9c212c9abefce4d9ab116cc29e95faf7c54de3c03`.
- 원 주장/용도: 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-false-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-false-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F47, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-existing-caret-false-hwpx

- 원본: [tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwpx](../../../../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwpx), 7606 bytes, SHA-256 `838192690296ba0a56eace4b289345617a5e2c0fa9909db4e587b6aa85b0225e`.
- 독립 Print PDF: [pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwpx-2020.pdf](../../../../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwpx-2020.pdf), 19851 bytes, SHA-256 `9bfa5d1bd00cda96bab48f5c3860fd2232a42d88026e5aa43c506456be9cf106`.
- 원 주장/용도: 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-false-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-false-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F47, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-existing-caret-true-hwp

- 원본: [tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwp](../../../../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwp), 13312 bytes, SHA-256 `a7aafc8b6e37141e10b34077ce592410f7115c67186ab72db828509aebb16e87`.
- 독립 Print PDF: [pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwp-2020.pdf](../../../../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwp-2020.pdf), 16336 bytes, SHA-256 `0d72792090d0aa869fa11ff8ca3712b2532c0c12073e302790ec5fe68c170bdb`.
- 원 주장/용도: 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-true-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-true-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F47, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-existing-caret-true-hwpx

- 원본: [tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwpx](../../../../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwpx), 7621 bytes, SHA-256 `f88d04df5e1c26a0b2c9744045a1464a1bdfaf0168aee84803dc184e6d9cc162`.
- 독립 Print PDF: [pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwpx-2020.pdf](../../../../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwpx-2020.pdf), 19855 bytes, SHA-256 `89a6e008e6772071b4c3e94b2bdae27ff939c821a30c1bb6d810e574b018aff7`.
- 원 주장/용도: 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-true-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-existing-caret-true-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F47, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-dutmal-hwp-2022

- 원본: [samples/dutmal/hancom-mac-dutmal.hwp](../../../../../samples/dutmal/hancom-mac-dutmal.hwp), 33280 bytes, SHA-256 `b5c6eb1fc1edde6b69fb6a6bc7d6bdfb9edfb829158d8844a0eceb93b27fa063`.
- 독립 Print PDF: [samples/dutmal/hancom-mac-dutmal.pdf](../../../../../samples/dutmal/hancom-mac-dutmal.pdf), 17469 bytes, SHA-256 `3fa2e668e84836118efea8f13ca5e7f59154aeba8657e2f6a39b1eb0e1c77bb7`.
- 원 주장/용도: 덧말 본말·덧말과 기준선. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-dutmal-hwp-2024

- 원본: [samples/dutmal/hancom-mac-dutmal.hwp](../../../../../samples/dutmal/hancom-mac-dutmal.hwp), 33280 bytes, SHA-256 `b5c6eb1fc1edde6b69fb6a6bc7d6bdfb9edfb829158d8844a0eceb93b27fa063`.
- 독립 Print PDF: [samples/dutmal/hancom-mac-dutmal.pdf](../../../../../samples/dutmal/hancom-mac-dutmal.pdf), 17469 bytes, SHA-256 `3fa2e668e84836118efea8f13ca5e7f59154aeba8657e2f6a39b1eb0e1c77bb7`.
- 원 주장/용도: 덧말 본말·덧말과 기준선. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwp-2024-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwp-2024-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-dutmal-hwpx-2022

- 원본: [samples/dutmal/hancom-mac-dutmal.hwpx](../../../../../samples/dutmal/hancom-mac-dutmal.hwpx), 33637 bytes, SHA-256 `64fcc6780e5a5bd0b53456b3fc6f067a3608d19c7ba03f1eb6d324987b95b509`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8616/hancom-mac-dutmal-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8616/hancom-mac-dutmal-hwpx-2020.pdf), 18137 bytes, SHA-256 `d0e0e31cfa7a8e0708f7f9a6c34c91e0423990d43dd9793cd4ed08d97ea1bc8e`.
- 원 주장/용도: HWPX 덧말 본말·덧말. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwpx-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-dutmal-hwpx-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-shapes-hwp-2022

- 원본: [samples/char_overlap/hancom-mac-shapes.hwp](../../../../../samples/char_overlap/hancom-mac-shapes.hwp), 33280 bytes, SHA-256 `b1c1a9c04bf63b8093e163d419a600be69a7306242fa4237c182160643ac9432`.
- 독립 Print PDF: [samples/char_overlap/hancom-mac-shapes.pdf](../../../../../samples/char_overlap/hancom-mac-shapes.pdf), 12260 bytes, SHA-256 `348a77671bcd52413800112988bcd3fdf5345d9fc67b9b316de71c4cebb84a9a`.
- 원 주장/용도: 글자 겹치기 14모양·PUA 숫자. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.2128%, [TSV](tsv/8616-hancom-mac-shapes-hwp-2022-native.tsv), SHA-256 `56f0aadd3403c32f4b602e0f39f0dbe9568d80b28ab5c7a69f3224c6e1f25373`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.22558%, [TSV](tsv/8616-hancom-mac-shapes-hwp-2022-wasm.tsv), SHA-256 `0ee3b66de185c9062d4ad18a1cd07c8b1657b96172c2b25204fa8f178d630743`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-shapes-hwp-2024

- 원본: [samples/char_overlap/hancom-mac-shapes.hwp](../../../../../samples/char_overlap/hancom-mac-shapes.hwp), 33280 bytes, SHA-256 `b1c1a9c04bf63b8093e163d419a600be69a7306242fa4237c182160643ac9432`.
- 독립 Print PDF: [samples/char_overlap/hancom-mac-shapes.pdf](../../../../../samples/char_overlap/hancom-mac-shapes.pdf), 12260 bytes, SHA-256 `348a77671bcd52413800112988bcd3fdf5345d9fc67b9b316de71c4cebb84a9a`.
- 원 주장/용도: 글자 겹치기 14모양·PUA 숫자. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.27308%, [TSV](tsv/8616-hancom-mac-shapes-hwp-2024-native.tsv), SHA-256 `d62328bd636b836e5a832b8059d9c7d5de96d3a62de6407f9fc48d604302eb16`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.25168%, [TSV](tsv/8616-hancom-mac-shapes-hwp-2024-wasm.tsv), SHA-256 `46ccdd2e0e2dfdec7bc1366e2418de12c83faf6bf5b872b9be8ee71fe24a7f46`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-shapes-hwpx-2022

- 원본: [samples/char_overlap/hancom-mac-shapes.hwpx](../../../../../samples/char_overlap/hancom-mac-shapes.hwpx), 34184 bytes, SHA-256 `b85d469c5ef101712261e07dc4686ea28360301327e06f459b401f70eb69889f`.
- 독립 Print PDF: [samples/char_overlap/hancom-mac-shapes.pdf](../../../../../samples/char_overlap/hancom-mac-shapes.pdf), 12260 bytes, SHA-256 `348a77671bcd52413800112988bcd3fdf5345d9fc67b9b316de71c4cebb84a9a`.
- 원 주장/용도: HWPX 글자 겹치기 14모양. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-shapes-hwpx-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-shapes-hwpx-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-hancom-mac-shapes-hwpx-2024

- 원본: [samples/char_overlap/hancom-mac-shapes.hwpx](../../../../../samples/char_overlap/hancom-mac-shapes.hwpx), 34184 bytes, SHA-256 `b85d469c5ef101712261e07dc4686ea28360301327e06f459b401f70eb69889f`.
- 독립 Print PDF: [samples/char_overlap/hancom-mac-shapes.pdf](../../../../../samples/char_overlap/hancom-mac-shapes.pdf), 12260 bytes, SHA-256 `348a77671bcd52413800112988bcd3fdf5345d9fc67b9b316de71c4cebb84a9a`.
- 원 주장/용도: HWPX 글자 겹치기 14모양. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-shapes-hwpx-2024-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8616-hancom-mac-shapes-hwpx-2024-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8616-opengov-ruby

- 원본: [samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx](../../../../../samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx), 42208 bytes, SHA-256 `ed3d589d80c59f1c029c275d4bcc03457f413db801246c36e1644c441990922e`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8616/opengov-36389301-ruby-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8616/opengov-36389301-ruby-2020.pdf), 63340 bytes, SHA-256 `96dcf0e213c32f92cc21b4be9fb124bc5aee07f144208338b252b9bdfec372d5`.
- 원 주장/용도: 원 PR의 결재 문서 덧말 본말·덧말 표시. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/8616-opengov-ruby-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/8616-opengov-ruby-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F2, F20, F22, F23, F28, F29, F30, F31, F32, F33, F34, F43, F44, F45, F46, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F61, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8629-hancom-mac-200-b5-hwp-2022

- 원본: [tests/fixtures/wonggoji/hancom-mac-200-b5.hwp](../../../../../tests/fixtures/wonggoji/hancom-mac-200-b5.hwp), 63488 bytes, SHA-256 `aee3057cbc9571400ca9c741ad5210ce2320faa0c2056da9b9773f0d7abb814c`.
- 독립 Print PDF: [tests/fixtures/wonggoji/hancom-mac-200-b5.pdf](../../../../../tests/fixtures/wonggoji/hancom-mac-200-b5.pdf), 22758 bytes, SHA-256 `8a640b6011721f3507c34798a3d196567a5106dcbbeeeafb4b71315147393456`.
- 원 주장/용도: 원고지 칸·줄 배치와 글 소속. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8629-hancom-mac-200-b5-hwp-2022-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8629-hancom-mac-200-b5-hwp-2022-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### 8629-hancom-mac-200-b5-hwp-2024

- 원본: [tests/fixtures/wonggoji/hancom-mac-200-b5.hwp](../../../../../tests/fixtures/wonggoji/hancom-mac-200-b5.hwp), 63488 bytes, SHA-256 `aee3057cbc9571400ca9c741ad5210ce2320faa0c2056da9b9773f0d7abb814c`.
- 독립 Print PDF: [tests/fixtures/wonggoji/hancom-mac-200-b5.pdf](../../../../../tests/fixtures/wonggoji/hancom-mac-200-b5.pdf), 22758 bytes, SHA-256 `8a640b6011721f3507c34798a3d196567a5106dcbbeeeafb4b71315147393456`.
- 원 주장/용도: 원고지 칸·줄 배치와 글 소속. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/8629-hancom-mac-200-b5-hwp-2024-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/8629-hancom-mac-200-b5-hwp-2024-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8526-inserted-style-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwp), 12800 bytes, SHA-256 `4d5a670618405940c4eba1c122ca7bd079bbbdbe3cfb2f04c7a02ffc6c8555b9`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8526/inserted-style-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8526/inserted-style-hwp-2020.pdf), 10663 bytes, SHA-256 `a8e1c0cd34b51f99dc584cf34c79ff4af926ae6b8dc464085e7ac1eb16e6085e`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8526-inserted-style-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8526-inserted-style-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8526-inserted-style-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwpx), 7387 bytes, SHA-256 `904c5c6c78ea156bd447f63cb983c11ae46709534050a8464f965241037b33c4`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8526/inserted-style-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8526/inserted-style-hwpx-2020.pdf), 10663 bytes, SHA-256 `92323cbdf54adee2f433f4304c4d51d4824d86cdd271420e2f741949dcab7591`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8526-inserted-style-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8526-inserted-style-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8541-merged-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8541/actual-api-merged.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8541/actual-api-merged.hwp), 12800 bytes, SHA-256 `b3baa4cbdc0dad80bab76e2c57b3dc5613a3d060fcdccb2abeeaf8c7b730a262`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/actual-wasm-merged-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/actual-wasm-merged-hwp-2020.pdf), 9533 bytes, SHA-256 `a37de9ac8d9ae16740b934645803a1e8414d5be224529fea8f0b712f8083b9c9`.
- 원 주장/용도: 실제 fresh WASM merged API 저장본의 쪽·문단·내용 보존. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8541-merged-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8541-merged-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8541-merged-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8541/actual-api-merged.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8541/actual-api-merged.hwpx), 7411 bytes, SHA-256 `a015ff435ed39f80608cdb786490db237e76da1e35e433afdb9ee9ab2862a06d`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/actual-wasm-merged-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/actual-wasm-merged-hwpx-2020.pdf), 9533 bytes, SHA-256 `49c00170c2996de4c371e3f9381268e7f81a10895848e47b8b12a906137d97e2`.
- 원 주장/용도: 실제 fresh WASM merged API 저장본의 쪽·문단·내용 보존. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8541-merged-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8541-merged-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8541-split-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8541/actual-api-split.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8541/actual-api-split.hwp), 13312 bytes, SHA-256 `48dee01478e8fc5a53ff668f1ab44f2cc7c613bc7cda8a362c00d7bd489c2bce`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/actual-wasm-split-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/actual-wasm-split-hwp-2020.pdf), 10047 bytes, SHA-256 `43d4110818a468fad5bd380b5cec1fe0ca1dd5d5e5705fbe9b156fb348a68b3e`.
- 원 주장/용도: 실제 fresh WASM split API 저장본의 쪽·문단·내용 보존. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/api-8541-split-hwp-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/api-8541-split-hwp-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8541-split-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8541/actual-api-split.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8541/actual-api-split.hwpx), 8658 bytes, SHA-256 `04f6270e56e8751dbb8d071e96248fab7048ab32ad866829d97d4bdbfa7761fc`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8541/actual-wasm-split-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8541/actual-wasm-split-hwpx-2020.pdf), 10047 bytes, SHA-256 `7d86dbea75027af420f6f1b8c2d60589fc277b054fef872517f1ef2843ac60cb`.
- 원 주장/용도: 실제 fresh WASM split API 저장본의 쪽·문단·내용 보존. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/api-8541-split-hwpx-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/api-8541-split-hwpx-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8549-moved-inline-picture-hwp

- 원본: [mydocs/pr/assets/line-seg-start-outside-control/rhwp-after.hwp](../../line-seg-start-outside-control/rhwp-after.hwp), 15872 bytes, SHA-256 `ed1df84780c47a24d45cfa463ea9c74de59da5d8781cd33192e4b4ef0e1c1013`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwp-2020.pdf), 32716 bytes, SHA-256 `acac52dbeb789e507b19b3f831bbed86694249f32e6bac895a13b8f547110274`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.91686%, [TSV](tsv/api-8549-moved-inline-picture-hwp-native.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.91794%, [TSV](tsv/api-8549-moved-inline-picture-hwp-wasm.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8549-moved-inline-picture-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8549/moved-inline-picture.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-inline-picture.hwpx), 10172 bytes, SHA-256 `dfd653562049adba9ed9434f6e2cf3d114b32de457310e9b0563fe2453808674`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwpx-2020.pdf), 32713 bytes, SHA-256 `08293c2b9647a99f00c8c79beeb15c2fe526a9fdab42bf0f281cd8212441a585`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.91678%, [TSV](tsv/api-8549-moved-inline-picture-hwpx-native.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.91787%, [TSV](tsv/api-8549-moved-inline-picture-hwpx-wasm.tsv), SHA-256 `b7e476f3e508c85f96cb0e3916e5da069f20f5bc1a668fe289947ab1f89a19c7`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8549-moved-terminal-tab-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwp), 14848 bytes, SHA-256 `4db4db7ffa5c79590ebd3f0887a640ef1f806e1c9fbea91994c75aea4ea2194b`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwp-2020.pdf), 27250 bytes, SHA-256 `f96fa2ff38656b5efefbf704a1138c507c5112a6732883f7d8de96f9ff146e52`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.92003%, [TSV](tsv/api-8549-moved-terminal-tab-hwp-native.tsv), SHA-256 `10652908888fa0c7849fbcd19439430a2dc305e58036663807aee29aacc60370`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.92149%, [TSV](tsv/api-8549-moved-terminal-tab-hwp-wasm.tsv), SHA-256 `10652908888fa0c7849fbcd19439430a2dc305e58036663807aee29aacc60370`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8549-moved-terminal-tab-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwpx), 9084 bytes, SHA-256 `05edb4bb53c27f2530a7a72bc2559c9db98d887061aeeb9102c65513d9e53c9a`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwpx-2020.pdf), 27260 bytes, SHA-256 `78dc0116b2adef5e98d2ca7c4ee5c4bf24654d48c55808fcbcc867c431d03e5f`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.91994%, [TSV](tsv/api-8549-moved-terminal-tab-hwpx-native.tsv), SHA-256 `10652908888fa0c7849fbcd19439430a2dc305e58036663807aee29aacc60370`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.92141%, [TSV](tsv/api-8549-moved-terminal-tab-hwpx-wasm.tsv), SHA-256 `10652908888fa0c7849fbcd19439430a2dc305e58036663807aee29aacc60370`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-actual-r3-character-bullet-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/actual-api-character-bullet.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/actual-api-character-bullet.hwp), 12800 bytes, SHA-256 `0c3f81df01c542b62a47e7668400cf3fccffeb5bda7aac1a9dbc0e899f13319b`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/actual-r3-character-bullet-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/actual-r3-character-bullet-hwp-2020.pdf), 9814 bytes, SHA-256 `c8434e0f21b278fd19683bb1e8722369ad416a20f4f0481c88f7d7d6048d919d`.
- 원 주장/용도: R3 실제 공개 API 문자·그림 글머리표 생성·저장. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-character-bullet-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-character-bullet-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-actual-r3-character-bullet-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/actual-api-character-bullet.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/actual-api-character-bullet.hwpx), 7548 bytes, SHA-256 `dce9e159cf925edc4da4cacb3dca0031425fd9348a47d9d8cd936aee038e32c9`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/actual-r3-character-bullet-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/actual-r3-character-bullet-hwpx-2020.pdf), 9816 bytes, SHA-256 `764c92b5f9c94fec56191ebf399046d887ecfbb1adedf19ac21054c332825be0`.
- 원 주장/용도: R3 실제 공개 API 문자·그림 글머리표 생성·저장. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-character-bullet-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-character-bullet-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-actual-r3-picture-bullet-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/actual-api-picture-bullet.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/actual-api-picture-bullet.hwp), 13312 bytes, SHA-256 `d65f295d44f3782a8b122c28459b65214f496382216538e065efc3a897365902`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/actual-r3-picture-bullet-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/actual-r3-picture-bullet-hwp-2020.pdf), 10044 bytes, SHA-256 `94efbe0f31f61b7cfd8650b8ec41f8b4d16c09ad4540143bd6ae79277d526701`.
- 원 주장/용도: R3 실제 공개 API 문자·그림 글머리표 생성·저장. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-picture-bullet-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-picture-bullet-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-actual-r3-picture-bullet-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/actual-api-picture-bullet.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/actual-api-picture-bullet.hwpx), 7802 bytes, SHA-256 `a86774f168ebaf8ce0807217d04bda6e208051c4e682ec6e4c90128cd275ee6f`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/actual-r3-picture-bullet-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/actual-r3-picture-bullet-hwpx-2020.pdf), 10044 bytes, SHA-256 `7ea8547a3be06a98b608f28eede8ed993571212f39e0c03aa9698a0f63d072b6`.
- 원 주장/용도: R3 실제 공개 API 문자·그림 글머리표 생성·저장. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-picture-bullet-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-actual-r3-picture-bullet-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-character-bullet-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/character-bullet.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/character-bullet.hwp), 12800 bytes, SHA-256 `25f031581fe43f65791b7d126c9b2873bc570cb356e3b77e9b5c49dc03560141`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/current-809b-character-bullet-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/current-809b-character-bullet-hwp-2020.pdf), 9814 bytes, SHA-256 `04496c286faa98af4c7f01df83683e15e07c698e68aa4f94328b91872a488c95`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-character-bullet-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-character-bullet-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-character-bullet-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/character-bullet.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/character-bullet.hwpx), 7471 bytes, SHA-256 `86c6114e7182b13f558301faaad93a82203fc8adf112e25976d2aa4664a52c42`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/current-809b-character-bullet-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/current-809b-character-bullet-hwpx-2020.pdf), 9836 bytes, SHA-256 `7d58918e6b8b730e5074a0f56caea4f558bff01369b3a814a1a483664914a521`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-character-bullet-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-character-bullet-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-picture-bullet-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/picture-bullet.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/picture-bullet.hwp), 13312 bytes, SHA-256 `a4f30d5d20ebdb4f6ed0182703daebf23860d616ce1b4e96d7e01a0c7ed3e795`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/current-809b-picture-bullet-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/current-809b-picture-bullet-hwp-2020.pdf), 10044 bytes, SHA-256 `a66df0e2483ef6f938dd8af694a289f12dc5d4568d6da9ddf6c08bc7ed812871`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-picture-bullet-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-picture-bullet-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8558-picture-bullet-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8558/picture-bullet.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8558/picture-bullet.hwpx), 7725 bytes, SHA-256 `5602185ac1b5296afdf2d946cf6e0aa1308d7b47973e554a2b4ca224c19a4ef2`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8558/current-809b-picture-bullet-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8558/current-809b-picture-bullet-hwpx-2020.pdf), 10050 bytes, SHA-256 `b4be78bcbdc2720f29c32a2c7ccd07a4de07a1de93f3455b7777e33c7b39bc83`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-picture-bullet-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8558-picture-bullet-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8564-context-after

- 원본: [tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-edited.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-edited.hwp), 14848 bytes, SHA-256 `122d77da16b50d15906833806871f5046541d89c4a2a00fe7d4941cc569227a4`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8564/date-path-context-after-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8564/date-path-context-after-2020.pdf), 14713 bytes, SHA-256 `a25f0c071774fcdd6b4f7d04772d3fb76220c88eb57404450555ae487e312608`.
- 원 주장/용도: 코드 밖 경계 입력 후 독립 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 92.01038%, [TSV](tsv/api-8564-context-after-native.tsv), SHA-256 `c2c3125d0abeb087dc1fd77155c1367bd255bd39410c53cf946ed55899083324`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 92.28612%, [TSV](tsv/api-8564-context-after-wasm.tsv), SHA-256 `035bcdd9d74329b013f2fdc52d8a5d8b717be1cb5f4718b59c39ff1687e043b6`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8570-non-inline-cell-table-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8570/non-inline-cell-table.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8570/non-inline-cell-table.hwp), 13312 bytes, SHA-256 `83b5439fe48b5c7667bdddd0893ddd3dbdc66c3eb70a234cf16455659d64e093`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8570/current-d3e6-non-inline-cell-table-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8570/current-d3e6-non-inline-cell-table-hwp-2020.pdf), 10367 bytes, SHA-256 `2d927c1221da8fe294b10144b46ef22aec6d75740652b71705530a588bb536f4`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8570-non-inline-cell-table-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8570-non-inline-cell-table-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8570-non-inline-cell-table-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8570/non-inline-cell-table.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8570/non-inline-cell-table.hwpx), 8116 bytes, SHA-256 `7cb18664b9a752375cd8531a065195abb94adf404415fec9c4dffe654b7c8870`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8570/current-d3e6-non-inline-cell-table-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8570/current-d3e6-non-inline-cell-table-hwpx-2020.pdf), 10367 bytes, SHA-256 `df330f5d985b897a7ed69e5fe6aa79e53779ff4864fcc0b7faca775d70d911f3`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8570-non-inline-cell-table-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8570-non-inline-cell-table-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-header-table-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwp), 13312 bytes, SHA-256 `7dfe60e9fea1d24f1b9f2ca5919b3d341581a593bf7e2f28c902f77ba9f8c6a7`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwp-2020.pdf), 9595 bytes, SHA-256 `2ac3b352297f522863216427cb54bfa57998eabb817e7077282d70786be52a58`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-header-table-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-header-table-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-header-table-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwpx), 7941 bytes, SHA-256 `aeb1f8b82a5ddb2e921ec0d9421c94f3c1cd2631bdc13fa9ebf25876a527ab62`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwpx-2020.pdf), 9595 bytes, SHA-256 `6f863438f05205ece20dafa7270c1fca09d42483a02a6113526a8e29b9699921`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-header-table-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-header-table-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-footer-grown

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-grown.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-grown.hwpx), 7819 bytes, SHA-256 `3fd043d6eba048a663d434e2e1b8fff9862d8066275ce0e75b2a89be71f63356`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/footer-grown-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-grown-2020.pdf), 9201 bytes, SHA-256 `9b5177b0a1745567fca733edb3f94b4f30fed9fe3ee0b0e7edb05818ba2558a6`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-grown-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-grown-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-footer-near-complete

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-near-complete.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-near-complete.hwpx), 7794 bytes, SHA-256 `af683aa4251759545714659a02bf344bb2d589d4dc303697531ce6321b4f8ea2`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/footer-near-complete-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-near-complete-2020.pdf), 9176 bytes, SHA-256 `e9b823d9e5a309e2673f502e736bfae378a78d80eea2efe86313b2be88b730b2`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-near-complete-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-near-complete-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-footer-stale

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-stale.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-stale.hwpx), 7797 bytes, SHA-256 `05bd9f96230dde25141de727b470729c8a67f8936884c549e0d40ec15773c149`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/footer-stale-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-stale-2020.pdf), 9174 bytes, SHA-256 `fa2ee749e2df08a36b886b1bff42a74b19175e34a57593a42e92c0767f463f91`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-stale-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-footer-stale-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-header-grown

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-grown.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-grown.hwpx), 7821 bytes, SHA-256 `8d16a1051eedf25404f48e36a1a43722474a090b02412042d4816d5d51033b6e`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/header-grown-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/header-grown-2020.pdf), 9195 bytes, SHA-256 `333a8225bc56b47777b67bee5182073155426e72759b6b5d523ed01354ec337c`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-grown-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-grown-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-header-near

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-near.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-near.hwpx), 7793 bytes, SHA-256 `d0d289b6f03d503cfdd7920fee9df42e9124d381725fb47b7922ce7b1a94504f`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/header-near-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/header-near-2020.pdf), 9174 bytes, SHA-256 `fe52150b32a2faf91d0ff2de6293b2b2a3b0de989cd4f3650ddc2a12822ea8f1`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-near-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-near-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-height-contract-header-stale

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-stale.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-stale.hwpx), 7796 bytes, SHA-256 `887ca3b78b2fbceb82e1f0309c275dc0136e344dd1019b2f7b49caf8b71d9660`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/height-contract/header-stale-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/height-contract/header-stale-2020.pdf), 9174 bytes, SHA-256 `7454c033dc314e9274d312be9bd392c7456af09a007e5f275972f81caec1ac6e`.
- 원 주장/용도: 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-stale-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-height-contract-header-stale-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-non-inline-header-table-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwp), 13312 bytes, SHA-256 `60376ce785be80c1aafbc10235a2f7d4dbd0376fc8d2b8a887666bc9fe4c80fc`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwp-2020.pdf), 10445 bytes, SHA-256 `d98816895fee60fb0f0f6e847ee883c73af0af8cf33c0880117450d27f987588`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-non-inline-header-table-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-non-inline-header-table-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8571-non-inline-header-table-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwpx), 7985 bytes, SHA-256 `3beb2f5b9880d1ee71efc8e4dcdaf9ceca5c9a0091fd761fad0275f61c74296c`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwpx-2020.pdf), 10445 bytes, SHA-256 `dfd9b17c766a230861d8e7c129e5b6ee98f67d1cbec42e3988d10179d3f13edb`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-non-inline-header-table-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8571-non-inline-header-table-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8572-cell-typed-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8572/actual-api-cell-typed.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8572/actual-api-cell-typed.hwpx), 13191 bytes, SHA-256 `97557471411c5a967654e4b053708ea974f1557cafde27c43ab8e376904fef52`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8572/actual-cell-typed-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8572/actual-cell-typed-hwpx-2020.pdf), 116403 bytes, SHA-256 `48334664e6eb73cda96cd4d3d804ae9e3d64a6f8915a6776e65b31f31466c5bc`.
- 원 주장/용도: 실제 fresh WASM 제목 셀 편집 후 HWPX 저장·다시 열기와 독립 Print의 전체 페이지. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 100.0%, [TSV](tsv/api-8572-cell-typed-hwpx-native.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 100.0%, [TSV](tsv/api-8572-cell-typed-hwpx-wasm.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F16, F17, F20, F21, F22, F23, F24, F25, F26, F27, F28, F31, F32, F35, F36, F37, F38, F39, F40, F41, F42, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F59, F6, F60, F61, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8602-empty-deleted-hwp

- 원본: [mydocs/pr/assets/emptied-paragraph-para-text/rhwp-after.hwp](../../emptied-paragraph-para-text/rhwp-after.hwp), 12800 bytes, SHA-256 `a2ec7d4fe6dcb4520a113693dd615ac654fc7a8ff47c63c1d3d7bd9cc37c3cf5`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8602/empty-deleted-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8602/empty-deleted-hwp-2020.pdf), 9919 bytes, SHA-256 `100f5db4c3fa79aa3d4bbb600b8e6688b92fa766b591ee5369375a6acacaf789`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8602-empty-deleted-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8602-empty-deleted-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### api-8602-empty-deleted-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8602/empty-deleted.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8602/empty-deleted.hwpx), 7450 bytes, SHA-256 `38e402cc148c6d5050067b4139dcc5d66fcbe2b6f6b6d247e1c43354d414f48f`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8602/empty-deleted-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8602/empty-deleted-hwpx-2020.pdf), 9933 bytes, SHA-256 `0733d769985adc6c82cf45b307387e5eb47376fbc54bfe0cb965ca6db5b25e0a`.
- 원 주장/용도: 새 후보의 실제 공개 API 생성·저장 후 한컴 Print. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8602-empty-deleted-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/api-8602-empty-deleted-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### control-scaffold-a

- 원본: [mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_a.hwpx](../../scaffold_outline_numbering/scaffold_outline_a.hwpx), 6288 bytes, SHA-256 `7dc0e7f41aec968c6510bf2c430dd50cee67590c15bd5f3cc0628a02c883f751`.
- 독립 Print PDF: [pdf/issue8540/scaffold_outline_a-2024.pdf](../../../../../pdf/issue8540/scaffold_outline_a-2024.pdf), 17764 bytes, SHA-256 `20536ee5ac4bdd6dc52cc5f0caf246d07713664e6ca1e46be945358e2c92e12a`.
- 원 주장/용도: 10수준 확장 후 기존 개요 번호 정상 대조군. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/control-scaffold-a-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/control-scaffold-a-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### control-scaffold-b

- 원본: [mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_b.hwpx](../../scaffold_outline_numbering/scaffold_outline_b.hwpx), 6300 bytes, SHA-256 `bdd247054116d4fb181f86a843376ccc9318914a2207bc8fd9802392b00d5eba`.
- 독립 Print PDF: [pdf/issue8540/scaffold_outline_b-2024.pdf](../../../../../pdf/issue8540/scaffold_outline_b-2024.pdf), 18097 bytes, SHA-256 `8f4a28a633bcdac9d7dc99edd68cf0705c2e7eadecf9547541763b7f99440325`.
- 원 주장/용도: 10수준 확장 후 기존 개요 번호 정상 대조군. 조판 compat `2024`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/control-scaffold-b-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/control-scaffold-b-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### header-tac-explicit-break

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-tac-explicit-break.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-explicit-break.hwpx), 7830 bytes, SHA-256 `8a742b6d40016468975e71bb9626b41f12da2967f47ec0b8ee473d9ca76065fe`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/header-tac-explicit-break-boundary-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/header-tac-explicit-break-boundary-2020.pdf), 10510 bytes, SHA-256 `638b67dfce09f96effd398af1043d2ee37a99d184f13dfb37cda2ecacc25ef82`.
- 원 주장/용도: 합성 경계: header-tac-explicit-break. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-explicit-break-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-explicit-break-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### header-tac-mixed-text

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-tac-mixed-text.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-mixed-text.hwpx), 7799 bytes, SHA-256 `fcb28b4b38aad065a24c65310202d15cf72dc3abe3d3972d469e522b449db85a`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/header-tac-mixed-text-boundary-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/header-tac-mixed-text-boundary-2020.pdf), 10391 bytes, SHA-256 `cc55c861ed294b6b139d2011cb26faa07d477dabe8c9a35187fd8ffe7df5f645`.
- 원 주장/용도: 합성 경계: header-tac-mixed-text. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-mixed-text-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-mixed-text-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### header-tac-pair-fit

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-fit.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-fit.hwpx), 7821 bytes, SHA-256 `0a5ebab658b2143deb9b81654c2ef9d061d618f112601e4b15737e093a221893`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/header-tac-pair-fit-boundary-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/header-tac-pair-fit-boundary-2020.pdf), 10513 bytes, SHA-256 `d0ee2efb192e984c577a75bcd7248584b624dc6cc1850b8be1014977c1cf3e3d`.
- 원 주장/용도: 합성 경계: header-tac-pair-fit. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-pair-fit-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-pair-fit-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### header-tac-pair-wrap

- 원본: [tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-wrap.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-wrap.hwpx), 7824 bytes, SHA-256 `c8784e0b26082b32715c50d810b3d31bbc473068c052158b4582afddba812cdc`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8571/header-tac-pair-wrap-boundary-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8571/header-tac-pair-wrap-boundary-2020.pdf), 10511 bytes, SHA-256 `7a2cc523e465292f4c7e86b859bc606bb82e055c60933569346317eb66dd2ea9`.
- 원 주장/용도: 합성 경계: header-tac-pair-wrap. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-pair-wrap-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/header-tac-pair-wrap-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8542-nhis-dependent-requirements

- 원본: [samples/issue8539/nhis-dependent-requirements.hwp](../../../../../samples/issue8539/nhis-dependent-requirements.hwp), 17920 bytes, SHA-256 `53d7763a0e5c9860e62c8dc3b20cfb515aac6f6eceeeef01ff0fb1abbdfe8fef`.
- 독립 Print PDF: [samples/issue8539/nhis-dependent-requirements-2020.pdf](../../../../../samples/issue8539/nhis-dependent-requirements-2020.pdf), 52093 bytes, SHA-256 `a54a96205d1916b44d98a71caf6114d47ddb34ebfa9f34263228f875e90e5da6`.
- 원 주장/용도: 새 base PR #8542의 3쪽 전체·두 쪽 경계·마지막 표 내용 보존. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 3쪽, 최저 99.54448%, [TSV](tsv/inherited8542-nhis-dependent-requirements-native.tsv), SHA-256 `5f479955feb213ff4235027a2cc16dd3b16f73d538de9348314eb40ce2b6c14a`, 166 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/4f9947e29b5f3f39d2704710333daef164f5512d3a5deda576e3b74e7d7af30b.json', 'sha256': '4f9947e29b5f3f39d2704710333daef164f5512d3a5deda576e3b74e7d7af30b'}.
- fresh WASM: 전 3쪽, 최저 99.45768%, [TSV](tsv/inherited8542-nhis-dependent-requirements-wasm.tsv), SHA-256 `839c0f231ae2cc8ab45be7d91a3723fcd2581f44841684463f18cc620aea4f46`, 166 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-borders

- 원본: [tests/fixtures/border_line_types/borders.hwp](../../../../../tests/fixtures/border_line_types/borders.hwp), 15872 bytes, SHA-256 `49566c073210994953b02a0d1e178636ba038c3e7e432f436266f9762407052f`.
- 독립 Print PDF: [pdf/border_line_types/borders-hancom-mac-12.30.pdf](../../../../../pdf/border_line_types/borders-hancom-mac-12.30.pdf), 44777 bytes, SHA-256 `110ef964ad956d961093dadbf725d93c62492ac1feee78f24daae5fb407a02bf`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 97.78663%, [TSV](tsv/inherited8640-borders-native.tsv), SHA-256 `6448d8e9a1413fa56526e349a83ca8d0de4e63f65cb442620fe282c3948249ae`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 97.7264%, [TSV](tsv/inherited8640-borders-wasm.tsv), SHA-256 `7c581e7c82efeb27ea25fb35d58c12980b6440c96169fb0e51ee584459785876`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-borders-windows

- 원본: [tests/fixtures/border_line_types/borders.hwp](../../../../../tests/fixtures/border_line_types/borders.hwp), 15872 bytes, SHA-256 `49566c073210994953b02a0d1e178636ba038c3e7e432f436266f9762407052f`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/borders-windows-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/borders-windows-2020.pdf), 45908 bytes, SHA-256 `faaee897ccaf741c84bef1a32d4db575fd0334dabd51c97f572040118cafa732`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 98.03878%, [TSV](tsv/inherited8640-borders-windows-native.tsv), SHA-256 `d324064af8261601ace4679c84e3d96f24083ce11290f20905240234ddd26260`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 97.97828%, [TSV](tsv/inherited8640-borders-windows-wasm.tsv), SHA-256 `79c924ca1b7610213b9ee00977b38a74f97af88c259dba27df1d8f98f93d1a76`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-drop-cap

- 원본: [tests/fixtures/drop_cap/double-line.hwp](../../../../../tests/fixtures/drop_cap/double-line.hwp), 27648 bytes, SHA-256 `44455394b5ffe21518352d148616c81f97dc0cbc52a0a938b2bb4def19a08a08`.
- 독립 Print PDF: [pdf/drop_cap/double-line-hancom-mac-12.30.pdf](../../../../../pdf/drop_cap/double-line-hancom-mac-12.30.pdf), 9994 bytes, SHA-256 `c8681d0aaf3fd16c53519fc5de521482fc3f40cf4c53d7e9fc38b32f4391767d`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-drop-cap-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-drop-cap-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-continue

- 원본: [tests/fixtures/endnote_numbering/continue.hwpx](../../../../../tests/fixtures/endnote_numbering/continue.hwpx), 33791 bytes, SHA-256 `467ae3882535aaddc4a4ced4c05ad0f154f27d8cb4bc6addb0cf69c9af76a2d8`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-continue-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-continue-2020.pdf), 21108 bytes, SHA-256 `2db2fb60d386267efcf357df85824416b788ee14ee444ac3f2935e2b24f5518b`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continue-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continue-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-continued-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/endnote-continued.hwp](../../../../../tests/fixtures/semanticist21_next10_review/endnote-continued.hwp), 30208 bytes, SHA-256 `89b61448d7ddef3af0d77d6f7b84a9c5700080ecc5d5dbc3cadf770112ef4f7d`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-continued-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-continued-hwp-2020.pdf), 21108 bytes, SHA-256 `0d78ed977fa36f44128a2b1a591a2b7ca67820b6b92e2d75f437f610c0b51079`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continued-hwp-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continued-hwp-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-continued-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/endnote-continued.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/endnote-continued.hwpx), 17850 bytes, SHA-256 `efb8a8b396001862bac488342c699d24aaec230d0cb6b0ac18346c8e4ecdaac9`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-continued-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-continued-hwpx-2020.pdf), 21108 bytes, SHA-256 `bd65a1e5509780be6407412a6651da3d0a1b24056ed806f59eefa89468c8e0de`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continued-hwpx-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-continued-hwpx-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-deleted-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/endnote-deleted.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/endnote-deleted.hwpx), 17835 bytes, SHA-256 `1e4628c195fb13b6bbf8e203ef5032ae389f07e7bacd85e70faf046079bd0748`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-deleted-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-deleted-hwpx-2020.pdf), 20958 bytes, SHA-256 `62d9d98c36199331b5b0bf3252d1a33a6ad3fe6b48dfe72035bc3f49b56897bd`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-deleted-hwpx-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-deleted-hwpx-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-inserted-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/endnote-inserted.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/endnote-inserted.hwpx), 17880 bytes, SHA-256 `e9dc2dd384df631dbcd01735041e8eba77011d0bb211053db0e4549ee260554e`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-inserted-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-inserted-hwpx-2020.pdf), 21407 bytes, SHA-256 `6bc9965ad99d34feb495611a0d2f92173d27bd21217616000893b85df22cb035`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-inserted-hwpx-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-inserted-hwpx-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-new20

- 원본: [tests/fixtures/endnote_numbering/new-number-20.hwpx](../../../../../tests/fixtures/endnote_numbering/new-number-20.hwpx), 33539 bytes, SHA-256 `437ef87552f9c637d4f328ec7bb665deac3bd0ebd544be621d1473804fd4a0c4`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-new20-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-new20-2020.pdf), 20850 bytes, SHA-256 `5d586112655432b70407a8f8633d5ec9958ad9b1681ed164bcb0e0c726ac31f3`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-new20-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-new20-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-endnote-restart5

- 원본: [tests/fixtures/endnote_numbering/restart-5.hwpx](../../../../../tests/fixtures/endnote_numbering/restart-5.hwpx), 33803 bytes, SHA-256 `e88a8288407f7401cad44b7d403646e11cc2a6e4667b8c05e79b6a8cde20c3b9`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/endnote-restart5-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/endnote-restart5-2020.pdf), 21220 bytes, SHA-256 `3624a60d1926c7fc8e674856b050b3aa8beaaa1face0b008567746743936b3cf`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-restart5-native.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 4쪽, 최저 100.0%, [TSV](tsv/inherited8640-endnote-restart5-wasm.tsv), SHA-256 `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1`, 192 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F20, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F50, F51, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-height

- 원본: [tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp](../../../../../tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp), 151552 bytes, SHA-256 `a0b4e465ba880cd0be611f84a3db69fa38b64b31cfc9a807f9a2fc86a998228d`.
- 독립 Print PDF: [pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf](../../../../../pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf), 156321 bytes, SHA-256 `a4c213bb53b86411340b432e8d347c5a6386807ceb606f001c043288bb115755`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.60162%, [TSV](tsv/inherited8640-equal-height-native.tsv), SHA-256 `f37399a830f50dabcc7d36d6494f62ad35f890dd8514ba8739591d155d037012`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/f0a8b94b2aed3c4e084d8ee9f06704b9547dc4701395376db32b1bb3feb9a7b7.json', 'sha256': 'f0a8b94b2aed3c4e084d8ee9f06704b9547dc4701395376db32b1bb3feb9a7b7'}.
- fresh WASM: 전 7쪽, 최저 99.98625%, [TSV](tsv/inherited8640-equal-height-wasm.tsv), SHA-256 `7dad7348242054c774b06a52a72c95fffb0aa26748434c515efba94d60914d54`, 246 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-height-edited-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/equal-height-edited.hwp](../../../../../tests/fixtures/semanticist21_next10_review/equal-height-edited.hwp), 62464 bytes, SHA-256 `7fa333d0c716dff9570386bc6bfaf4160c06dcfaa99608b1251c5b8f0000605e`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/equal-height-edited-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/equal-height-edited-hwp-2020.pdf), 168972 bytes, SHA-256 `c12877e4938327dc539ead1df58ab46bd2c7ae210ba25865174c18d71a6a3675`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.09353%, [TSV](tsv/inherited8640-equal-height-edited-hwp-native.tsv), SHA-256 `8c72ba2b7001a812f9678e6f49405c47c1cca95270cd57a95d29853ca0627c8f`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a.json', 'sha256': '19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a'}.
- fresh WASM: 전 7쪽, 최저 99.67561%, [TSV](tsv/inherited8640-equal-height-edited-hwp-wasm.tsv), SHA-256 `2bf178656b9b80c87e15ad3a0172f68a74b15c6d86e90b83f42b8dd800086e79`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-height-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/equal-height-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/equal-height-edited.hwpx), 22242 bytes, SHA-256 `ac37e38d7f5b380cfc7183231b1d614d82519bd253ec31089f4a3549b9e8304b`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/equal-height-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/equal-height-edited-hwpx-2020.pdf), 168974 bytes, SHA-256 `cb233225593dd3d85848d01e70f8e7234a2e7963860bcd28d3604b3a272dad1f`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.09353%, [TSV](tsv/inherited8640-equal-height-edited-hwpx-native.tsv), SHA-256 `8c72ba2b7001a812f9678e6f49405c47c1cca95270cd57a95d29853ca0627c8f`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a.json', 'sha256': '19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a'}.
- fresh WASM: 전 7쪽, 최저 99.67555%, [TSV](tsv/inherited8640-equal-height-edited-hwpx-wasm.tsv), SHA-256 `2bf178656b9b80c87e15ad3a0172f68a74b15c6d86e90b83f42b8dd800086e79`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F19, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F45, F46, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F61, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-width

- 원본: [tests/fixtures/table_equal_cell_sizes/equal-width-partial-block.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/equal-width-partial-block.hwp), 151552 bytes, SHA-256 `efbdec7676c70fcd5334860b870a441bbc949fe39f2595487b99f422be4fa341`.
- 독립 Print PDF: [pdf/table_equal_cell_sizes/equal-width-partial-block-hancom-mac-12.30.pdf](../../../../../pdf/table_equal_cell_sizes/equal-width-partial-block-hancom-mac-12.30.pdf), 156162 bytes, SHA-256 `8988f4748bdea72a997f632bba5ca5b5025a6291ac2eb762c22ef15af04d4c01`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.59852%, [TSV](tsv/inherited8640-equal-width-native.tsv), SHA-256 `f37399a830f50dabcc7d36d6494f62ad35f890dd8514ba8739591d155d037012`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/f0a8b94b2aed3c4e084d8ee9f06704b9547dc4701395376db32b1bb3feb9a7b7.json', 'sha256': 'f0a8b94b2aed3c4e084d8ee9f06704b9547dc4701395376db32b1bb3feb9a7b7'}.
- fresh WASM: 전 7쪽, 최저 99.98625%, [TSV](tsv/inherited8640-equal-width-wasm.tsv), SHA-256 `7dad7348242054c774b06a52a72c95fffb0aa26748434c515efba94d60914d54`, 246 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-width-edited-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/equal-width-edited.hwp](../../../../../tests/fixtures/semanticist21_next10_review/equal-width-edited.hwp), 62464 bytes, SHA-256 `064a7d51dccaddf1c3136679bfcf40b0f51a71b031ffad5397e953b9d5e92c05`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/equal-width-edited-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/equal-width-edited-hwp-2020.pdf), 168789 bytes, SHA-256 `e50ccf8ce05db966b10ed3063062995bd4a79e33f0ff0b6b314e7a2d4ad18458`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.09353%, [TSV](tsv/inherited8640-equal-width-edited-hwp-native.tsv), SHA-256 `edc55d62ad623c04dacb9a09dace7cfd4d3d879090e2d46755a104423bbaa566`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a.json', 'sha256': '19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a'}.
- fresh WASM: 전 7쪽, 최저 99.67561%, [TSV](tsv/inherited8640-equal-width-edited-hwp-wasm.tsv), SHA-256 `2bf178656b9b80c87e15ad3a0172f68a74b15c6d86e90b83f42b8dd800086e79`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equal-width-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/equal-width-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/equal-width-edited.hwpx), 22177 bytes, SHA-256 `f697cee620b06e271da8b57ba469465f62f346352254924da17fc3e4d53db893`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/equal-width-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/equal-width-edited-hwpx-2020.pdf), 168789 bytes, SHA-256 `35bd25a2d14e2bbd149097437dac9b0cfd08f51ca32a01002be5c1f7ceb3f769`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.09353%, [TSV](tsv/inherited8640-equal-width-edited-hwpx-native.tsv), SHA-256 `edc55d62ad623c04dacb9a09dace7cfd4d3d879090e2d46755a104423bbaa566`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a.json', 'sha256': '19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a'}.
- fresh WASM: 전 7쪽, 최저 99.67555%, [TSV](tsv/inherited8640-equal-width-edited-hwpx-wasm.tsv), SHA-256 `2bf178656b9b80c87e15ad3a0172f68a74b15c6d86e90b83f42b8dd800086e79`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F19, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F45, F46, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F61, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-equalize-original

- 원본: [tests/fixtures/table_equal_cell_sizes/original.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/original.hwp), 131584 bytes, SHA-256 `e3ee5a9615b329e7cd5817e09e905b9973538d3eeceaf5a2f1338f4218766a31`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/equalize-original-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/equalize-original-2020.pdf), 168741 bytes, SHA-256 `2443783fe29d72e60643731aa071e1c86def2eca21026f5c6df2d03eea32b4b5`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 99.09353%, [TSV](tsv/inherited8640-equalize-original-native.tsv), SHA-256 `edc55d62ad623c04dacb9a09dace7cfd4d3d879090e2d46755a104423bbaa566`, 240 bytes. 글꼴 설정 {'path': '/home/tsjang/rhwp/output/pr-review/native-webfont-cache/environments/19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a.json', 'sha256': '19e01ede025d41f4856071be18caf1e114a0a234a2d2b9835b768580412aa81a'}.
- fresh WASM: 전 7쪽, 최저 99.67561%, [TSV](tsv/inherited8640-equalize-original-wasm.tsv), SHA-256 `2bf178656b9b80c87e15ad3a0172f68a74b15c6d86e90b83f42b8dd800086e79`, 244 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F20, F22, F23, F28, F3, F31, F32, F4, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-footnote-shape

- 원본: [tests/fixtures/footnote_shape/restart-page.hwp](../../../../../tests/fixtures/footnote_shape/restart-page.hwp), 27136 bytes, SHA-256 `e6483160868d2f38b9deddfad47f5e2a5a233f0dccc2965c3e3bd8bd2d7bc67d`.
- 독립 Print PDF: [pdf/footnote_shape/restart-page-hancom-mac-12.30.pdf](../../../../../pdf/footnote_shape/restart-page-hancom-mac-12.30.pdf), 12587 bytes, SHA-256 `4c7d8f92a66975d862b1a709758b1d149ece2d14ae72351a0adae79c2e93760b`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-footnote-shape-applied-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/footnote-shape-applied.hwp](../../../../../tests/fixtures/semanticist21_next10_review/footnote-shape-applied.hwp), 27648 bytes, SHA-256 `c176bd86461e51faed330cdb71bc19c3374fbbdfbd35d806c1604a35ae9b83ea`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/footnote-shape-applied-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/footnote-shape-applied-hwp-2020.pdf), 15331 bytes, SHA-256 `b9f21d92481a124d7320b8726b1867a09928e8670f8063967f83466217eba509`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-applied-hwp-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-applied-hwp-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-footnote-shape-applied-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/footnote-shape-applied.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/footnote-shape-applied.hwpx), 7625 bytes, SHA-256 `f9ba9b3ce981e49507a6b307d36f7f2d1592b398ddd365dc889240a05060b705`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/footnote-shape-applied-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/footnote-shape-applied-hwpx-2020.pdf), 14796 bytes, SHA-256 `9de9b556c95fa949cb91c3a86a098b5febc8a4aca9ff9b951c30481040281d51`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-applied-hwpx-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-applied-hwpx-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-footnote-shape-configured-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/footnote-shape-configured.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/footnote-shape-configured.hwpx), 7571 bytes, SHA-256 `7444fb2a0a42f907c67b29f858347fe754f8f866b5413c5770357df008840796`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/footnote-shape-configured-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/footnote-shape-configured-hwpx-2020.pdf), 14500 bytes, SHA-256 `b6bbe9db15978c484cce138b13b89e1bc4cd18b73cdfa1bf867ef39997c0863a`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 99.92019%, [TSV](tsv/inherited8640-footnote-shape-configured-hwpx-native.tsv), SHA-256 `35a75fcdaff4881a2248b1996fb51f1626ee66e7f1b4dd9f3d19e6b40a5129e0`, 150 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 99.92218%, [TSV](tsv/inherited8640-footnote-shape-configured-hwpx-wasm.tsv), SHA-256 `35a75fcdaff4881a2248b1996fb51f1626ee66e7f1b4dd9f3d19e6b40a5129e0`, 150 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-footnote-shape-corrected-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/footnote-shape-corrected.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/footnote-shape-corrected.hwpx), 7629 bytes, SHA-256 `feb37b65b8ef714bbfbc45940403551f4e5681d0ab0aed3b8380ecb71cc76673`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/footnote-shape-corrected-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/footnote-shape-corrected-hwpx-2020.pdf), 15331 bytes, SHA-256 `10666be5c2f124ea9a5f42cc64b8854b222e96e3313eb532a07f440e5ea88050`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-corrected-hwpx-native.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 2쪽, 최저 100.0%, [TSV](tsv/inherited8640-footnote-shape-corrected-hwpx-wasm.tsv), SHA-256 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`, 152 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-outline

- 원본: [tests/fixtures/char_outline_types/outline.hwp](../../../../../tests/fixtures/char_outline_types/outline.hwp), 13824 bytes, SHA-256 `e6317596e9eb1f7c5182e8c7aa821a3c036f00a2b9829b02956d988af4d4fb85`.
- 독립 Print PDF: [pdf/char_outline_types/outline-hancom-mac-12.30.pdf](../../../../../pdf/char_outline_types/outline-hancom-mac-12.30.pdf), 17464 bytes, SHA-256 `9917eb42efb7dfb0ffd34f009f10cb9d1044c089613228229d0fc1471af57006`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-outline-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-outline-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-outline-id2-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/outline-id2.hwp](../../../../../tests/fixtures/semanticist21_next10_review/outline-id2.hwp), 12800 bytes, SHA-256 `d0ee102c8e0734b3b3094b1e48590d517b64d5aa934262e4388978f770276919`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/outline-id2-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/outline-id2-hwp-2020.pdf), 10522 bytes, SHA-256 `286f1d304c5fa9ed48220123224dbfd9f30f4b85316f1b3add4da3b89d61ee4a`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 96.49123%, [TSV](tsv/inherited8640-outline-id2-hwp-native.tsv), SHA-256 `e376e7c4aa812df20f31400cb3fe1592fe1478cfbfed12dac19cfbb4d8a53cd0`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 97.10145%, [TSV](tsv/inherited8640-outline-id2-hwp-wasm.tsv), SHA-256 `14ece59cdde4483ddef51613316d474c866a03110f4ab73f1b1eb550526d1020`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-outline-id2-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/outline-id2.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/outline-id2.hwpx), 7336 bytes, SHA-256 `b3cd3c7fb63c68a8b3e7a0978566886bf9b51e0a736cfd1b0c88cc00404a1ead`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/outline-id2-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/outline-id2-hwpx-2020.pdf), 10522 bytes, SHA-256 `72dd310b8d5f40a6e2bd88aaf2bebb36c06a04a9cc323b0001392000c8ae554d`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 96.49123%, [TSV](tsv/inherited8640-outline-id2-hwpx-native.tsv), SHA-256 `e376e7c4aa812df20f31400cb3fe1592fe1478cfbfed12dac19cfbb4d8a53cd0`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 97.10145%, [TSV](tsv/inherited8640-outline-id2-hwpx-wasm.tsv), SHA-256 `14ece59cdde4483ddef51613316d474c866a03110f4ab73f1b1eb550526d1020`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-paragraph-head-edited-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwp](../../../../../tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwp), 86528 bytes, SHA-256 `2728d6263b06ffbc0e714d7d1e120cc8b09605e9c7660d45c0b20cc51b59ae0b`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/paragraph-head-edited-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/paragraph-head-edited-hwp-2020.pdf), 22705 bytes, SHA-256 `73a8218740c074d520475a5430c0b657f16b452b0a14c6fb340971aa519eaede`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.71287%, [TSV](tsv/inherited8640-paragraph-head-edited-hwp-native.tsv), SHA-256 `f6e99f783ad447b18bd54fefdbdbf0f28ff3b6c782c5df5f4d735d4fcfc09822`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.69142%, [TSV](tsv/inherited8640-paragraph-head-edited-hwp-wasm.tsv), SHA-256 `ba5f24809ecae2a1bc6986bf2cce7f3cc9647a970d72e8e76b131918d2968bba`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-paragraph-head-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwpx), 62515 bytes, SHA-256 `6af6c35a61063be766838a6a1b174ecfc4f233b08b70b4434d94f15e051b7198`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/paragraph-head-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/paragraph-head-edited-hwpx-2020.pdf), 24601 bytes, SHA-256 `5cf97a8ac4441e631424379d42bc34b543beec6418fe70ac383c95e6c6b17701`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-head-edited-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-head-edited-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-paragraph-head-off-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/paragraph-head-off.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/paragraph-head-off.hwpx), 62535 bytes, SHA-256 `e1375eb9b97a38ccf9f76c4c6cd99cfc463db35e0ad7baffba4d52e97623adb0`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/paragraph-head-off-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/paragraph-head-off-hwpx-2020.pdf), 24560 bytes, SHA-256 `985ddea8e1345aeb1e7a87908534f47144efd08257e5a44ac4823fa61b04723f`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-head-off-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-head-off-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-paragraph-heads

- 원본: [tests/fixtures/paragraph_head/hancom-heads.hwpx](../../../../../tests/fixtures/paragraph_head/hancom-heads.hwpx), 77216 bytes, SHA-256 `d73699eb4e0b536496d6d2d6d3e0e0f9f68816e38cb8891457c61c239baff808`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/paragraph-heads-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/paragraph-heads-2020.pdf), 24547 bytes, SHA-256 `d2f391d16174ee582bc80f45c04fa5966a984819cc58d0a8234fc945f64cf012`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-heads-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-paragraph-heads-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-pdf-image-patterns

- 원본: [pdf/issue8403/image-patterns.pdf](../../../../../pdf/issue8403/image-patterns.pdf), 3578 bytes, SHA-256 `f121e0f0ca222f397ea80d5b16dba45b1e2d768cc339e1e495ade9e3f512869f`.
- 독립 Print PDF: [pdf/issue8403/image-patterns.pdf](../../../../../pdf/issue8403/image-patterns.pdf), 3578 bytes, SHA-256 `f121e0f0ca222f397ea80d5b16dba45b1e2d768cc339e1e495ade9e3f512869f`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 5쪽, 최저 99.87421%, [TSV](tsv/inherited8640-pdf-image-patterns-native.tsv), SHA-256 `794b7551657be217bf2c7c36d4ac06249f4bf5614620cf41e8342761bfc9ebcb`, 210 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 5쪽, 최저 99.87421%, [TSV](tsv/inherited8640-pdf-image-patterns-wasm.tsv), SHA-256 `794b7551657be217bf2c7c36d4ac06249f4bf5614620cf41e8342761bfc9ebcb`, 210 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.

### inherited8640-pdf-text-paint

- 원본: [pdf/issue8403/text-stroke.pdf](../../../../../pdf/issue8403/text-stroke.pdf), 8781 bytes, SHA-256 `034386aced59c69c9901ac4b747e24ff3c1d1b9a3cd5f86e44e930aedc252e84`.
- 독립 Print PDF: [pdf/issue8403/text-stroke.pdf](../../../../../pdf/issue8403/text-stroke.pdf), 8781 bytes, SHA-256 `034386aced59c69c9901ac4b747e24ff3c1d1b9a3cd5f86e44e930aedc252e84`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 7쪽, 최저 100.0%, [TSV](tsv/inherited8640-pdf-text-paint-native.tsv), SHA-256 `04a7e280637cf842b190f8c7978f149e30d3cc0625014f90650234041e3a1502`, 252 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 7쪽, 최저 98.17721%, [TSV](tsv/inherited8640-pdf-text-paint-wasm.tsv), SHA-256 `9a77b8b8ae0867ca51ea80ed7f7c15a810346c7716e04fc17a3cc4218676d6f2`, 238 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F22, F23, F28, F31, F32, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F6, F7, F8, F9.

### inherited8640-table-formula-cycles

- 원본: [tests/fixtures/table_formula/cycles.hwpx](../../../../../tests/fixtures/table_formula/cycles.hwpx), 16793 bytes, SHA-256 `737c454944b3551d9cddcb759b2337093dd1d879eac525d7a218fa3e75704e65`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-cycles-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-cycles-2020.pdf), 9782 bytes, SHA-256 `a87e4ded81122fd791696765ae2f19c7e7726877780a9d9f6050432b1e75d951`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-cycles-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-cycles-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-easy

- 원본: [tests/fixtures/table_formula/easy.hwpx](../../../../../tests/fixtures/table_formula/easy.hwpx), 15701 bytes, SHA-256 `b46636826c13cc09045a0097dd079580914244ddd8db4ce66f507a770583aed0`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-easy-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-easy-2020.pdf), 11680 bytes, SHA-256 `d46e01eebbf388dabd4346ad6513ab62fd3db5a2c24c3609eb75cc5b68bb5533`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-easy-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-easy-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-easy-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/table-formula-easy-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/table-formula-easy-edited.hwpx), 15951 bytes, SHA-256 `646db28aeb61d828dcc66a7eea54fcb256a2e336e696793208d276db8250eb72`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-easy-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-easy-edited-hwpx-2020.pdf), 11688 bytes, SHA-256 `c6df311a25761442ede5cb8db67295850adf35b0eca9217c23eaaa97a41b5ec5`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-easy-edited-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-easy-edited-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-hancom-result

- 원본: [tests/fixtures/table_formula/hancom.hwp](../../../../../tests/fixtures/table_formula/hancom.hwp), 43008 bytes, SHA-256 `d8ac4dd6db6d4c73714c2c7519898f1398552bb840325f9782f59e10a94db903`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-hancom-result-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-hancom-result-2020.pdf), 14308 bytes, SHA-256 `f6b36077ebbfe982544a151bb49592707b8f589cc31db863e2acf0f9bb1111d4`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-hancom-result-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-hancom-result-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-recalculated-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/table-formula-recalculated.hwp](../../../../../tests/fixtures/semanticist21_next10_review/table-formula-recalculated.hwp), 52224 bytes, SHA-256 `410ae0f30937a83449a4c8e813b19029c8fd84000aa9c14b8dbb0b3f88b7fe59`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-recalculated-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-recalculated-hwp-2020.pdf), 14288 bytes, SHA-256 `749976d98cc9d0dc9bfe86da6585fbd838b8a7306a95a466f48e29d4bf7676d7`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-recalculated-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-recalculated-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-recalculated-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/table-formula-recalculated.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/table-formula-recalculated.hwpx), 19067 bytes, SHA-256 `9ced780bb37467cfaa7137c866c437f6ad710a45c3879caf69e84e102221cc41`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-recalculated-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-recalculated-hwpx-2020.pdf), 14288 bytes, SHA-256 `12c9c727279a552d2c82ca6c017c94ae1acbffabe416c2352d70b733497dfc37`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-recalculated-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-recalculated-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-table-formula-stale

- 원본: [tests/fixtures/table_formula/stale.hwpx](../../../../../tests/fixtures/table_formula/stale.hwpx), 18245 bytes, SHA-256 `e11b4ee37a4ab016cbd7056e725ccf89e2a3448e68ba92da18c39c4442ad8177`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/table-formula-stale-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/table-formula-stale-2020.pdf), 12047 bytes, SHA-256 `a767229bca103363f79e0e0f8d80110401df0f98d868a2cd59f6b278780f796e`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-stale-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-table-formula-stale-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-height

- 원본: [tests/fixtures/table_equal_cell_sizes/uneven-height-hancom.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/uneven-height-hancom.hwp), 25088 bytes, SHA-256 `dd3606bb1786c0c64e870dcd0bc47a82f6a3328bb25d28d1b1096961382a5342`.
- 독립 Print PDF: [pdf/table_equal_cell_sizes/uneven-height-hancom-mac-12.30.pdf](../../../../../pdf/table_equal_cell_sizes/uneven-height-hancom-mac-12.30.pdf), 8412 bytes, SHA-256 `6d36a414ee573158df39ea8996954009c99534d3b85b06b2325ceb5dd73966ee`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-height-edited-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/uneven-height-edited.hwp](../../../../../tests/fixtures/semanticist21_next10_review/uneven-height-edited.hwp), 13312 bytes, SHA-256 `edeb687bdddd19021a3fd8e446a0b225b5c3c917fc9154329610254085527fe9`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-height-edited-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-height-edited-hwp-2020.pdf), 9973 bytes, SHA-256 `79c3458295a4cb2622322632ac26e25d2bd0a539888d720e7785d86cdb7bc89b`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-edited-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-edited-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-height-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/uneven-height-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/uneven-height-edited.hwpx), 7908 bytes, SHA-256 `5c6416c2f4a1514e5aa32999603f2cdfcbab9466f1f30eb815546c9c56055b43`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-height-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-height-edited-hwpx-2020.pdf), 9973 bytes, SHA-256 `8954bb2f9ffa710a298594d321df3e98db91ffdf63fba7e5eb99a251bebc8dc3`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-edited-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-edited-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-height-original

- 원본: [tests/fixtures/table_equal_cell_sizes/uneven-height.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/uneven-height.hwp), 13312 bytes, SHA-256 `003f6a7f48cc87e60b7d1b023330ebaf958f6ae94fc0c9a3fadc7fc727277b4d`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-height-original-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-height-original-2020.pdf), 9953 bytes, SHA-256 `2edd6f096081683f1c6412d5381f25578a9f70c3edef07327393b95eb35a3038`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-original-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-height-original-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-width

- 원본: [tests/fixtures/table_equal_cell_sizes/uneven-width-hancom.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/uneven-width-hancom.hwp), 25088 bytes, SHA-256 `09b190de71f57fce92921d56c1e8731f4bc769911cb59bbd8f2b59dac3d5c3b1`.
- 독립 Print PDF: [pdf/table_equal_cell_sizes/uneven-width-hancom-mac-12.30.pdf](../../../../../pdf/table_equal_cell_sizes/uneven-width-hancom-mac-12.30.pdf), 6341 bytes, SHA-256 `b866117e35154fa4ff3c5f4915dab9e3121533bea02dd743f15ff9d5ccfe4321`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-width-edited-hwp

- 원본: [tests/fixtures/semanticist21_next10_review/uneven-width-edited.hwp](../../../../../tests/fixtures/semanticist21_next10_review/uneven-width-edited.hwp), 13312 bytes, SHA-256 `23c0c4941e0f6d21f3cbba4122be6ac9df2ad422c348a4154f1758e2a6797f03`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-width-edited-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-width-edited-hwp-2020.pdf), 9961 bytes, SHA-256 `838ee6da65a8dfb196e6653e198306ab2cf468af5f065cf445ac9c07a583c20a`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-edited-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-edited-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-width-edited-hwpx

- 원본: [tests/fixtures/semanticist21_next10_review/uneven-width-edited.hwpx](../../../../../tests/fixtures/semanticist21_next10_review/uneven-width-edited.hwpx), 7919 bytes, SHA-256 `6c39965155be3b1dc987dc8454c0e9093d9322390d99613b68aae742946478fd`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-width-edited-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-width-edited-hwpx-2020.pdf), 9961 bytes, SHA-256 `3f66399bc0d9881115d5c4e7c0c18acf4bd5c0c5e5a02d5732b5d8779ce82698`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-edited-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-edited-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### inherited8640-uneven-width-original

- 원본: [tests/fixtures/table_equal_cell_sizes/uneven-width.hwp](../../../../../tests/fixtures/table_equal_cell_sizes/uneven-width.hwp), 13312 bytes, SHA-256 `3b480c147060bf93582bcf1e3fa2a1af71f22ee4632207dc9db8996b345e697f`.
- 독립 Print PDF: [pdf/semanticist21-20261105-next10/uneven-width-original-2020.pdf](../../../../../pdf/semanticist21-20261105-next10/uneven-width-original-2020.pdf), 9949 bytes, SHA-256 `64e16d4ff2ffea5a43b8547e9463b0a73f16cf8e7edc11925be06b3adcfaa72c`.
- 원 주장/용도: 새 devel PR #8640의 기존 실제 입력·독립 Print·전쪽 보존 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-original-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/inherited8640-uneven-width-original-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F20, F28, F32, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### issue7344-broken-cell-ladder

- 원본: [samples/issue7344/36312980_broken_cell_ladder_center.hwpx](../../../../../samples/issue7344/36312980_broken_cell_ladder_center.hwpx), 14132 bytes, SHA-256 `57e3acec05d3c856e931b5c4146fae6ecd3f96c6356fc7d6d99c501a83a34e21`.
- 독립 Print PDF: [pdf/issue7344/36312980-2020.pdf](../../../../../pdf/issue7344/36312980-2020.pdf), 38491 bytes, SHA-256 `ef35e530d654dcbfc9c4f008e89b2280d7a297b0e7bbcb2f39efef344ea4532b`.
- 원 주장/용도: 기존 #7344 저장 줄·탭·셀 정렬의 원본 대조군. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 99.86918%, [TSV](tsv/issue7344-broken-cell-ladder-native.tsv), SHA-256 `04d5aaced6735bab5829864e58d39818e1a60046989fae5ecf994affa242236f`, 130 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/issue7344-broken-cell-ladder-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F12, F13, F18, F19, F2, F20, F22, F23, F28, F29, F30, F31, F32, F33, F34, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F57, F58, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### issue7344-first-cell-no-ls-before

- 원본: [samples/issue7344/36455985_no_lineseg_cell_spacing_before.hwpx](../../../../../samples/issue7344/36455985_no_lineseg_cell_spacing_before.hwpx), 14728 bytes, SHA-256 `9518b5ee4ca2c625b9b6ac8504def1915472bd7be60023eec9b79a4f38f045a2`.
- 독립 Print PDF: [pdf/issue7344/36455985-2020.pdf](../../../../../pdf/issue7344/36455985-2020.pdf), 38803 bytes, SHA-256 `6b6dc687b810ab6a1f6def894334fcf9e2bde1084d1a7ad6bf38b1416fc2b8a7`.
- 원 주장/용도: R1 합성 줄 소유 보정 뒤 기존 #7344 첫 셀 문단 앞 간격 보존 검사의 독립 Print 대조. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/issue7344-first-cell-no-ls-before-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 99.99788%, [TSV](tsv/issue7344-first-cell-no-ls-before-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F1, F10, F11, F14, F15, F2, F20, F22, F23, F28, F29, F30, F31, F32, F33, F34, F43, F44, F48, F49, F5, F52, F53, F54, F55, F56, F59, F6, F60, F61, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### issue7344-stored-fields-tabs

- 원본: [samples/issue7344/36494836_field_marker_tac_lines.hwpx](../../../../../samples/issue7344/36494836_field_marker_tac_lines.hwpx), 14338 bytes, SHA-256 `4103c9e87b43bd783043c9d757d83179363e9ca5e50cc6c50bcea7220ac86358`.
- 독립 Print PDF: [pdf/issue7344/36494836-2020.pdf](../../../../../pdf/issue7344/36494836-2020.pdf), 36394 bytes, SHA-256 `485e04ee4d309235cdbb0bcb345432c879391d850e1b4831a3d22136a2dbabde`.
- 원 주장/용도: 기존 #7344 저장 줄·탭·셀 정렬의 원본 대조군. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/issue7344-stored-fields-tabs-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/issue7344-stored-fields-tabs-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음. 실제 로드 face F2, F20, F21, F28, F29, F30, F32, F33, F34, F5, F52, F53, F54, F55, F56, F6, F62, F63, F64, F65, F66, F67, F68, F7, F8, F9.

### rebase-empty-nested-hwp

- 원본: [tests/fixtures/semanticist21_integration_review/pr8262/nested-empty-current.hwp](../../../../../tests/fixtures/semanticist21_integration_review/pr8262/nested-empty-current.hwp), 13312 bytes, SHA-256 `e1fc37322b616ef261b23550fa46dbebe1f37413199c2d2f21e977fc8f9393a5`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8262/rebase-empty-nested-hwp-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8262/rebase-empty-nested-hwp-2020.pdf), 6255 bytes, SHA-256 `4a3bf6fdceddb9bbc549d4246e7d6e2a5006cf80dc71a16f65a644d083470938`.
- 원 주장/용도: 원 PR #8300의 빈 셀 TAC 표 생성 순서와 실제 원본 여백 관계. 새 API 저장본 자체의 MCP 2020 Print.. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/rebase-empty-nested-hwp-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/rebase-empty-nested-hwp-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.

### rebase-empty-nested-hwpx

- 원본: [tests/fixtures/semanticist21_integration_review/pr8262/nested-empty-current.hwpx](../../../../../tests/fixtures/semanticist21_integration_review/pr8262/nested-empty-current.hwpx), 8075 bytes, SHA-256 `4def85aecd0d2bc9954ff75387484176c8dfe6f3ffb3f9b927378145adac8c8d`.
- 독립 Print PDF: [pdf/semanticist21-20261105/pr8262/rebase-empty-nested-hwpx-2020.pdf](../../../../../pdf/semanticist21-20261105/pr8262/rebase-empty-nested-hwpx-2020.pdf), 6255 bytes, SHA-256 `10a5b08862b562b8aa8efd87507d4777cb25b9b9110aa4cc7e74afde97eaa389`.
- 원 주장/용도: 원 PR #8300의 빈 셀 TAC 표 생성 순서와 실제 원본 여백 관계. 새 API 저장본 자체의 MCP 2020 Print.. 조판 compat `2022`는 PDF 실제 출력 버전의 이름과 별도다.
- Native: 전 1쪽, 최저 100.0%, [TSV](tsv/rebase-empty-nested-hwpx-native.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
- fresh WASM: 전 1쪽, 최저 100.0%, [TSV](tsv/rebase-empty-nested-hwpx-wasm.tsv), SHA-256 `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466`, 132 bytes. 글꼴 설정 자동 시스템·웹 공급; 별도 대체 설정 없음.
