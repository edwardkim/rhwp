# #8153: Print bitmap 채움의 장치 사각형 clip

## 원본과 독립 근거

편람 HWP/HWPX 288·307쪽의 남은 회색 띠 차이는 bitmap 채움 경계에 몰린다.
기존 후보 `b919780c3b`의 실제 Native/fresh WASM PNG와 원본 독립 Print의 PDF 연산을
같은 물리 96dpi에서 대조했다. 그림의 물리 프레임은 유지되고, 소수 픽셀의 clip 경계에서
현재 Canvas/Skia가 부분 투명도를 적용하는 반면 독립 PDF의 사각형 clip은 장치 픽셀을 포함한다.

예를 들어 288쪽 첫 clip의 아래 224.498pt는 96dpi에서 299.330667px이다.
PDF의 포함 영역은 300px 전까지이며, 현재 출력의 299번째 줄은 부분 alpha가 적용돼
회색 226 대신 약 252가 된다. 두 번째 clip의 위 230.138pt도 306.850667px에 해당하며
독립 PDF는 306번째 줄부터 원래 회색을 보존한다. 글자 위치·문서 식별자로 보정하지 않는다.

공개 PDFium chromium/7999의 실제 소비 경로를 확인했다.

- [CPDF_RenderStatus::ProcessClipPath](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fpdfapi/render/cpdf_renderstatus.cpp)는 원본 clip을 장치 행렬과 함께 전달한다.
- [CFX_AggDeviceDriver::SetClip_PathFill](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/agg/cfx_agg_devicedriver.cpp)는 축 정렬 사각형을 바깥 정수 경계로 변환해 교차한다.
- [CFX_SkiaDeviceDriver::SetClip_PathFill](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/skia/fx_skia_device.cpp)도 같은 바깥 정수 경계를 사용한다. 일반 곡선·회전 경로는 별도다.

독립 PDF 명령으로 만든 소수·정수 사각형 4개와 72/96/144/192dpi의 16개 조합에서
실제 raster의 포함 경계와 불투명한 원래 회색을 확인했다. 16 PASS였다. 이 합성 PDF는
한컴 원본의 대체 자료가 아니며, 원본 Print는 재출력하지 않았다. 스크립트·PDF·PNG·관측값은
ignored `output/pr-review/issue8153-handbook-raw100-20261022/pdf-exact-dpi-efd9d52168/rect-clip-contract/`에 둔다.

## 공통 처리와 소비 경로

2024 Print bitmap 채움의 원본 물리 clip → 의미가 분명한 clip 종류 → 실제 backend의
장치 행렬 → 공통 Rust의 바깥 정수 경계 → 같은 Canvas/Skia clip 소비로 연결한다.
원본 좌표와 조판 bbox는 유지한다. SVG·PDF 같은 벡터 기록에는 물리 clip을 그대로 전달하며,
실제 raster 출력에서만 장치 픽셀의 포함 경계를 적용한다. 일반 본문·셀·텍스트 상자 clip이나
회전·skew·음의 배율에는 이 사각형 전용 처리를 적용하지 않는다.

## 검증 순서

1. 원본 288·307쪽의 이전 출력·독립 clip·경계 줄을 보존한다.
2. 실제 장치 행렬을 받는 공통 helper와 Native/Canvas2D/CanvasKit 소비 경로를 구현한다.
   helper 반환 후 backend가 다시 자체 반올림하거나 다른 원점을 선택하지 않게 한다.
3. 앞 후보의 실행 중인 공유 빌드를 보존한 뒤, 새 Native 경계·정상 대조군부터 실행한다.
   그림이 잘리거나 밖으로 넘지 않는지, 앞뒤 본문·회색 영역·기존 제어 그림을 직접 확인한다.
4. 새 후보의 fresh WASM 실제 PNG와 같은 영역을 대조한다. 관계·좌표 회귀는 시각 선행
   조건을 확인한 뒤 등록한다. 이후 편람 전쪽·대조 corpus·필수 회귀와 lint를 마친다.

이 문서는 독립 근거와 구현 계획이다. 새 후보의 실제 출력·전쪽 100%·최종 검증은 미완료다.

## 후보 구현 상태

`ClipKind::PrintBitmapFill`은 기존 2024 Print bitmap 채움에만 붙인다. Native raster와
Canvas2D, CanvasKit은 실제 장치 행렬을 같은 Rust helper에 전달하며 독자적인 반올림을
추가하지 않는다. CanvasKit용 WASM 바인딩은 같은 helper의 결과를 전달한다.
Native의 PDF·그림 기록 canvas 및 SVG는 원본 물리 clip을 보존한다. 회전·skew·음의
배율은 기존 경로로 남긴다. JSON의 새 clip 종류를 구분하도록 schema minor를 30으로
갱신하고 기존 schema 계약의 기대 버전도 함께 갱신했다.

Studio의 실제 `node_modules/.bin/tsc --noEmit`은 종료 코드 0이었다.
이전 `bea7004f55`의 최적화 WASM 빌드가 실행 중이므로 같은 target에서 새 Rust 빌드를
겹쳐 실행하지 않았다. 새 후보의 실제 Native·fresh WASM 경계 검증은 다음 필수 단계다.
