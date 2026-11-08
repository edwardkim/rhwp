# PDFium 면적 표본의 공개 근거

`src/renderer/print_image_sampling.rs`의 고정소수점 면적 계산은 공개 PDFium
`chromium/7999`의 CStretchEngine 계약을 Rust로 적용한다. 그림의 원본과 실제 배율은
호출자가 공급하며, PDF나 외부 프로그램의 정답 픽셀을 제품 자원으로 사용하지 않는다.

- [소스](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/dib/cstretchengine.cpp)
- [고정소수점 정의](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/dib/cstretchengine.h)
- [원래 LICENSE](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/LICENSE)

원래 저작권·라이선스 고지는 [LICENSE](LICENSE)에 보존한다. 이 구현은 불투명한 전체
원본의 축소와 실제 장치 외접 경계를 공통으로 계산한다. alpha·부분 자르기·회전은 이
계산의 적용 범위가 아니며, 일반 렌더러의 기존 처리를 유지한다.
