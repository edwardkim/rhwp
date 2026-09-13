# rhwp PDF 호환성 패치

- 출처: https://github.com/edwardkim/svg2pdf/tree/2caeb0a038f9128b79833d803b94c2667565c4da
- 기존 `determinism-0.13`의 정확한 Cargo.lock revision에서 라이브러리 src·ICC·라이선스·NOTICE를 복사했다.
- Cargo workspace에서 미포함 CLI/테스트 members만 제외했다. 기존 결정화 수정은 그대로 유지한다.
- #7077/#7078: `src/render/gradient.rs`에서 256개를 넘는 PDF Type 3 함수의 자식만 계층화한다.
  모든 원래 Type 2 구간·색·불투명도를 보존하고, 부모 Encode와 자식 Domain을 같은 원좌표로 유지한다.
  공통 HWP 그러데이션 step, SVG, Canvas, Skia 및 기존 step=100 계약을 바꾸지 않는다.
- 근거: [PDF 32000-1 §7.10.4](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf),
  [MuPDF MAX_STITCHING](https://github.com/ArtifexSoftware/mupdf/blob/master/source/pdf/pdf-function.c).
- 제품 경로 회귀는 `tests/cases/issue_7077_pdf_gradient_functions.rs`에서 확인한다.

- 현재 Rust lint에 맞춰 `Name` 반환형 두 곳에 생략된 lifetime `'_`를 명시했다. 동작 변경은 없다.
