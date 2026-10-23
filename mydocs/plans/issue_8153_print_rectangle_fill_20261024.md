# #8153: Print의 단색 사각형 채움과 장치 픽셀

## 위반된 계약과 독립 근거

`afd8d95856`은 bitmap 채움 clip과 그림의 이중 장치 변환을 보정했으나, 원본 288·307쪽의
회색 경계는 흰색 셀 사각형이 뒤에서 다시 덮었다. 실제 layer tree의 해당 도형은 모서리
반경 0·불투명 단색·stroke 없음·gradient/pattern/shadow 없음이다. Native는 이를
`draw_rect`의 부분 alpha로 채워 226이어야 할 픽셀을 241~249로 바꾼다. 일부 영향 쪽의
raw가 0.1%p 이상 하락했으므로 이 후보의 완료·PR 제출은 보류했다.

공개 [PDFium chromium/7999 CFX_RenderDevice::DrawPath](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/cfx_renderdevice.cpp)의 사각형 fast path는
stroke가 없는 사각형의 정수 외접 영역에서 목표 길이 `ceil(원래 길이)`, 최소 1px를 구한다.
양쪽 경계에 걸쳐 목표보다 1px 넓으면 덮는 비율이 작은 쪽을 제거한다. 같은 경우 뒤쪽을
제거한다. 무조건 round·floor하거나 clip처럼 외접 영역 전체를 채우는 규칙과 다르다.
원본 clip은 바깥 정수 경계이고, 그 안에 그리는 도형의 채움은 별도 계약이다.

독립 PDF의 단색 사각형 5개와 72/96/144/192dpi 조합 20건을 실제 래스터했다.
공개 구현과 같은 float32 좌표·연산을 사용했을 때 20건의 포함 영역·끝 픽셀 RGB가
일치했다. 최초 double 계산은 같은 비율로 걸친 끝점의 마지막 비트에서 다른 쪽을 골랐다.
그 실패도 보존했으며 임계값·2px 반경·RGB를 바꿔 통과시키지 않았다.

원본 HWP/HWPX와 기존 독립 Print는 바꾸거나 재출력하지 않았다. 합성 PDF는 좌표 계약의
별도 진단이며 한컴 문서의 출력 증거가 아니다. 자료는 ignored 작업 경로의
`pdf-exact-dpi-efd9d52168/rect-fill-contract/`와 `diagnose_pdf_rect_fill*.py`에 보존한다.

## 생산·소비 경로와 구현 범위

- 실제 RectangleNode의 단색·불투명·stroke/효과/둥근 모서리 없음 여부는 공통 Rust에서
  판정한다. 문서·색상·위치·셀 번호로 선택하지 않는다.
- 최종 도형 frame → backend의 실제 장치 행렬 → 공통 Rust의 정수 채움 영역 →
  같은 좌표계로 돌린 결과 → Native·Canvas2D·CanvasKit의 실제 채움으로 연결한다.
  이후 bbox를 다시 선택하거나 다른 반올림을 적용하지 않는다.
- Print의 raster에만 적용한다. 벡터 PDF·SVG와 screen, stroke·gradient·pattern·shadow·
  둥근 모서리·회전/skew/음의 배율은 기존 계약을 유지한다. 조판 bbox와 문단/셀 소유는
  바꾸지 않는다. clip의 정수화와 채움의 정수화를 혼합하지 않는다.
- JSON에는 공통 판정 결과를 전달하고 이 도형의 좌표는 원래 정밀도를 보존한다.
  CanvasKit이 스타일 조건이나 경계 선택을 따로 추측하지 않게 한다.

## 실행 순서와 완료 판정

1. 앞 후보의 실제 Native 36건·경계 색상 10 FAIL·render tree 보존·하락 쪽을 보존한다.
2. 새 helper를 독립 PDF 관측 20건과 비교하고, 같은 경계·표지·정상 그림 대조군에서
   실제 Native 출력을 먼저 확인한다. helper 단위 통과를 원본 출력의 통과로 대신하지 않는다.
3. 같은 새 후보의 fresh WASM Canvas2D와 CanvasKit 실제 소비 경로를 확인한다. 새 캡처·
   입력/PDF/producer SHA와 색·미세 간격 판독을 기록한다. 하락·새 결함이 있으면 계속 보정한다.
4. 최종 전쪽 Native/fresh WASM, 대조 corpus, 정식 회귀·lint·Skia·Studio를 완료한 뒤
   PR 절차를 진행한다. 현재 전쪽 100%와 최종 검증은 미완료다.

## 구현과 선행 진단

공통 `paint/device_fill.rs`에서 대상 판정·float32 장치 끝점·정수 채움 영역을 계산한다.
Native와 Canvas2D는 실제 행렬을 전달하고, CanvasKit은 Rust가 직렬화한 대상 표식과
같은 WASM 바인딩의 결과를 소비한다. Print 단색 사각형의 JSON bbox는 원래 f64 값을
보존하며 schema minor 31로 구분한다. 별도의 backend 반올림은 추가하지 않았다.

실제 helper를 독립 PDF에서 관측한 20개 픽셀 영역에 대조한 Rust 진단은 20건 모두
정확히 일치했다. 해당 진단은 이전 Native library의 공개 자료형을 사용해 현재 helper만
실행했으며, 새 제품 전체의 빌드·출력 결과가 아니다. helper SHA와 명령·종료 코드는
ignored `rectangle-fill-contract-diagnostic/result.json`에 구분했다.
Studio `tsc --noEmit`도 종료 코드 0이다.

## 실제 Native 결과와 원인 재분리

`fca79de4f0` Native 빌드는 성공했고 영향 36쪽을 다시 출력했다. 의미 render tree는
보존됐지만 288·307쪽의 경계 10곳은 독립 Print의 RGB 226과 여전히 달랐다. 따라서 위의
흰 사각형 덮기 설명만으로 이 경계의 원인을 확정한 판단은 잘못됐다. helper 20건의
통과는 원본 문서의 경계 해결을 입증하지 않는다. 현재 후보는 완료하지 않는다.

실제 최종 소비자를 같은 library에서 재생해 캐시 비활성화·흰 사각형 제거·generic clip
제거를 각각 비교했다. 두 쪽의 문제 픽셀 5곳은 모두 그대로였다. 반면 bitmap 채움 그림의
최종 목적 영역을 정수 장치 외접 영역으로 대응한 진단에서는 5곳 모두 RGB 226이 됐다.
이는 원인 분리용 수정 tree의 결과이며 정상 제품 수정·전쪽 통과로 보고하지 않는다.

현재 Print 채움 그림은 `fill_mode=Total`인 PNG다. JPEG용 면적 표본 경로는 이 입력에
적용되지 않으며, `draw_image_rect`가 소수 끝점의 부분 alpha를 남긴다. clip의 외접 경계를
고쳐도 그림 자체의 끝점은 그대로다. 다음 보정은 기존 PNG 재구성 방식과 색을 유지하면서
그림의 실제 장치 영역을 공통 결과로 만들어 모든 raster 소비자가 사용하도록 한다.

증거는 ignored 작업 경로의 `rectangle-fill-fca79de4f0/native-focus-result.json`,
`rectangle-fill-boundary-result.json`, `rectangle-consumer-fca79de4f0/`와
`logs/rectangle-consumer-run.log`에 보존했다. Screen 88쪽의 수정 전후 실제 RGBA는
동일했으며 `rectangle-fill-screen-control/result.json`에 producer·입력·PNG SHA를 기록했다.
