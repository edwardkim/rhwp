# #8153: Print bitmap 채움 그림의 최종 장치 영역

## 근거와 반례

원본 288·307쪽의 PNG 채움은 실제 Print frame과 clip을 갖지만, Native의 최종
`draw_image_rect`는 소수 목적 영역의 끝 픽셀을 부분 alpha로 남긴다. 독립 Print는 그
위치에 불투명 회색 226을 기록한다. 같은 tree에서 캐시·흰 사각형을 각각 제거해도
달라지지 않았고, 그림 자체의 최종 장치 영역을 외접 정수 영역으로 재생한 진단에서
해당 5곳이 모두 226이 됐다. 조판 좌표를 바꿔 해결할 문제가 아니다.

공개 [PDFium의 CPDF_ImageRenderer](https://pdfium.googlesource.com/pdfium/+/a697e197f3a729db2a65b02d46b636644491293a/core/fpdfapi/render/cpdf_imagerenderer.cpp)의
`GetUnitRect`는 최종 image matrix의 단위 사각형에서 `GetOuterRect`를 구한다.
clip이나 단색 사각형 채움과 다른 소유 정보이며, 그림의 표본 방식과도 별개다.
독립 합성 PDF 5개 × 72/96/144/192dpi의 20건에서 실제 포함 영역과 불투명 끝 픽셀을
확인했다. float32 행렬 합성을 포함하며 원본 PDF나 점수 임계값을 수정하지 않는다.
이 합성 자료는 원본 한컴 문서의 시각 검증을 대신하지 않는다.

## 공통 생산·소비와 범위

- 2024 Print bitmap carrier의 영역 맞춤·전체 채움, 효과 없는 불투명 원본을 대상으로
  공통 Rust에서 판정한다. crop·내부 inset·회전·뒤집기·비축 정렬은 기존 경로를 유지한다.
- 최종 그림 frame → 실제 backend 장치 행렬 → 공통 외접 영역 → 같은 좌표계의 목적
  사각형 → 기존 PNG/JPEG 표본 재생 순서다. 그림과 clip을 각각 대응하며 조판 bbox와
  문단·셀 소속을 바꾸지 않는다. 색·필터·표본 방식을 경계 보정과 혼합하지 않는다.
- Native, Canvas2D와 CanvasKit은 같은 공통 영역을 소비한다. JSON은 대상 표식과 원래
  정밀도의 bbox를 전달한다. Screen·SVG·벡터 PDF는 기존 계약을 유지한다.
- 실제 원본 경계 10곳, 기존 영향 36쪽과 정상 대조군을 먼저 다시 출력한다. 새 후보가
  검증되기 전에는 fresh WASM 전쪽이나 전체 회귀 성공으로 원본 결함을 대신하지 않는다.

## 검증 자료

작업 자료는 ignored `output/pr-review/issue8153-handbook-raw100-20261022/`의
`rectangle-consumer-fca79de4f0/`, `pdf-exact-dpi-efd9d52168/image-coverage-contract/`,
`diagnose_pdf_image_coverage.py`와 관련 로그에 보존한다. 원본·독립 Print는 재출력하지 않는다.
제품 보정 후에는 같은 source SHA의 Native/fresh WASM 원본 출력·전체 TSV·확대 판독을
확보한다. 완료 목표인 전쪽 raw 100%와 실제 미대응 픽셀 0은 아직 달성하지 않았다.
