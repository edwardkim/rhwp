# #8153: PDF 기준 래스터의 물리 DPI 보존

## 선행 구현과 독립 근거

PR #8064에 포함된 `f703d1e3e4a0f1128e53c6a65cbdcab4b0f3df17`은 PDFium의 색·가는
Type3 획을 보존하도록 공통 PDF 래스터를 전환했다. 기존 stage2473의 균일색·인접 면·
Type3·독립 한컴 그라데이션 검사는 유지한다. 이번 오류는 그 API 호출이 전체 용지의
소수 픽셀 크기를 정수 출력 크기에 맞춰 다시 확대한다는 점이다.

`PdfPage.render(scale=dpi/72)`는 `ceil(page_points * scale)`의 정수 너비·높이를
`FPDF_RenderPageBitmap`의 표시 영역으로 넘긴다. 편람의 555×754pt를 96dpi로 출력하면
740×1006픽셀인데, 실제 표시 배율은 X 96dpi / Y 약 96.06366dpi가 된다. 출력 버퍼의
크기 올림과 물리 배율은 서로 다른 계약이다.

- [PDFium 공개 API](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdfview.h):
  `FPDF_RenderPageBitmap`의 `size_x/size_y`는 표시 영역의 픽셀 크기다.
  `FPDF_RenderPageBitmapWithMatrix`는 변환 행렬과 장치 clip을 별도로 받는다.
- [PDFium 실제 소비 경로](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/fpdfsdk/fpdf_view.cpp):
  페이지의 display matrix 뒤에 호출자가 지정한 matrix를 결합한다.
- 독립 1인치(72pt) 사각형으로 정수·소수 크기 용지, 0/90/180/270도, 72/96/144dpi의
  24개 조합을 실행했다. 일정한 DPI 행렬은 24개 모두 정확한 인치 길이를 보존한다.
  기존 정수 표시 영역 경로는 그중 12개에서 한 축 이상의 길이를 1픽셀 늘렸다.

## 수정·검증 계획

1. 같은 PDFium·흰 RGB·기존 antialias를 유지하고, 물리 배율 `dpi/72`를 양 축에 동일하게
   전달한다. 버퍼 크기만 올림하고 문서·페이지·점수별 변형을 추가하지 않는다.
2. 새 길이 검사를 기존 자동 수집 Python suite에 추가한다. 수정 전 실패와 수정 후 통과,
   기존 색·가는 획·독립 Print 검사를 실제 실행한다. CI 호출을 추가하지 않는다.
3. 래스터 정책·실행 신원을 새 값으로 구분한다. 이전 PDF PNG·TSV·manifest는 보존한다.
   입력/PDF/producer SHA를 확인한 기존 RHWP PNG는 재사용할 수 있으나, base와 후보의
   PDF PNG·TSV·review는 모두 동일한 새 물리 좌표 계약에서 다시 만든다.
4. 편람의 양쪽 형식 전쪽 Native/fresh WASM, 기존 대조 corpus를 같은 조건에서 비교한다.
   하락·미달·내용·쪽수 차이가 있으면 실제 원인을 보정한다. 임계값·2px·baseline을
   완화하지 않는다. 래스터 오류의 보정을 RHWP 조판 자체의 개선으로 보고하지 않는다.

## 현재 진단과 상태

독립 PDF 290쪽을 동일한 96dpi 행렬로 진단 출력해 후보 `b919780c3b`의 Native PNG와
대조했을 때 미대응 1,534픽셀이 0이 됐다. 이는 새 정책의 진단이며 기존 TSV의 최종
성공을 의미하지 않는다. 정식 전쪽 산출과 fresh WASM·회귀·최종 PR 검증은 미완료다.

## 구현·독립 검사 결과

- 공통 경로를 `FPDF_RenderPageBitmapWithMatrix`와 동일한 양 축 배율로 보정하고,
  이전 기본값의 annotation·색·antialias를 보존했다. 실행 신원에 좌표 계약을 추가했다.
- 수정 전 원 코드(`a0bfde8b96`의 helper)를 별도 진단 모듈로 불러 정확히 같은 최종
  인치 길이 검사를 실행했다. 24개 조합 중 12개가 실패했다. 소수 CropBox와 원본
  annotation의 대조 검사는 통과했다. 실패를 없애려고 원 코드를 고쳐 실행하지 않았다.
- 소수 CropBox의 float32 좌표와 픽셀 채움은 별도의 위치 대조로 검사한다. 이를 인치
  길이의 정수 픽셀 수와 혼합한 첫 진단에서는 경계 픽셀 4건이 달랐다. 인치 길이는
  픽셀 정렬된 독립 24개 조합에서 정확히 검사하고, 소수 원점 24개 조합은 독립 PDF
  좌표의 1픽셀 내 소속·위치와 버퍼 크기를 검사한다. 원본 raw 임계값과 반경은 바꾸지 않았다.
- 최종 실제 `test_visual_sweep*.py` 자동 수집: **180 PASS**, 36.707초, 종료 코드 0.
  가는 Type3 획·기존 RGB·인접 면·독립 한컴 그라데이션 검사도 함께 실행했다.
- 전쪽 산출과 base/후보 비교, fresh WASM 및 최종 회귀는 이어서 수행한다.
