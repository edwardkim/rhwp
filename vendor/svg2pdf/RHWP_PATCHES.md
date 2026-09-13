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

- #6936: `src/render/text.rs`의 단색 fill+stroke text를 PDF `Tr 2`로 한 번만 기록한다.
  기존 fill·stroke의 불투명도, 색 공간, 선 폭·join·cap·dash는 `path.rs`의 공통 helper로 보존한다.
  역순 paint는 동일한 불투명 색일 때만 결합하고, 그 밖의 역순·gradient·pattern은 기존 경로를 유지한다.
  기본 SVG→PDF 경로의 합성 굵게가 텍스트 검색·추출을 중복시키지 않게 하는 패치다.
  제품 경로 회귀는 `tests/cases/issue_6936_pdf_synthetic_bold.rs`, 한컴 및 음성 대조군 증적은
  `samples/issue6936/README.md`와 `mydocs/pr/assets/issue_6936/README.md`에 있다.
