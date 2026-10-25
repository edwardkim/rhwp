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

## 연결된 그림 표본의 이중 장치 변환

앞선 `14940666168501838d3812e40c2385459e8caf29`의 공통 면적 표본은 호출자가
실제 장치 행렬을 적용한 뒤에도 `ceil(page * scale) / (page * scale)`을 다시 곱했다.
이는 당시 기준 PDF 래스터의 정수 display viewport를 따라간 처리였으나, 물리 DPI와
버퍼 크기를 분리하면 이미 변환된 좌표를 다시 확대하는 오류가 된다.

실제 고정된 `b919780c3b` WASM의 공개 `rasterizePrintImageRGBA`를 직접 호출했다.
48×48px의 불투명 그림은 정수 픽셀 용지에서 48×48px지만, 편람의 소수 픽셀 높이
용지에서는 48×49px가 됐다. 2배 출력도 96×97px였으며, 소수 너비 용지에서는 두 축이
늘어났다. 최초 진단 8건은 2 PASS / 6 FAIL이고, 같은 소수 높이 입력 2건이 중복됨을
기록한다. 별도의 4개 용지 크기·96/192dpi 독립 PDF 8건은 모두 실제 물리 크기를 유지했다.

공통 `rasterize`에서 최종 장치 사각형의 floor/ceil 외접 경계만 소비하도록 보정한다.
16.16 면적 가중치·축별 절삭·원본 RGB·clip·불투명 원본 축소의 적용 조건과 public ABI는
바꾸지 않는다. Native/Canvas2D/CanvasKit은 이미 같은 함수를 소비하므로 중복 계산을
추가하지 않는다. 이 보정과 clip 보정을 함께 새 Native 후보에서 검증한다.

실제 진단은 ignored output의 `print-image-physical-grid-before.json`과
`pdf-exact-dpi-efd9d52168/image-physical-grid/result.json`에 있다. 원본 288·307쪽의
경계 픽셀 진단은 이전 후보에서 0 PASS / 10 FAIL이었고, 의미 render tree는 유지됐다.
그림 clip 조상도 실제 layer tree에서 본문 clip과 채움 자체 clip으로 추적했다.
새 후보의 개선과 정상 대조군·전체 출력을 실제로 확인하기 전에는 완료로 판정하지 않는다.

## 후보의 실제 실행과 다음 소비 지점

`afd8d95856`의 Native 빌드는 10분 39초·종료 코드 0이었다. 실제 공통 Rust 코드를
독립 PDF의 사각형 clip 16건·그림 크기 8건·affine 비적용 조건과 대조한 진단 3개도
통과했다. 이 결과를 원본 문서의 시각 통과로 대신하지 않았다.

Native 영향 페이지 36건의 실제 출력에서 1쪽 표지는 양쪽 형식 모두 raw 100%로
개선됐으나, 295·299쪽 등에는 최대 0.2771%p 하락이 남았다. 288·307쪽의 경계 색상
10건은 여전히 독립 Print의 회색 226과 다르고, render tree는 앞 후보와 같았다.
실제 layer tree에서 그림 clip 이후의 흰색 셀 사각형 채움이 같은 픽셀에 부분 alpha를
덮어쓰는 경로를 확인했다. clip helper의 통과만으로 완료 처리하지 않고, 사각형 채움의
공통 장치 규칙을 다음 단계에서 보정한다. 이 후보의 최종 PR gate는 통과하지 않았다.

실제 자료는 ignored 작업 경로의 `bitmap-clip-afd8d95856/device-contract-result.json`,
`native-focus-result.json`, `bitmap-clip-boundary-after-afd8d95856.json`과 페이지별 PNG에 있다.

## 2026-10-25: 본문 단의 원점과 채움 끝점의 공통 변환

`3509cafe88`에서 표 배경의 남은 가는 경계를 독립 Print 원시 연산과 대조했다.
283쪽은 후보의 회색 행이 추가되고, 291·297·304쪽은 Print에 있는 회색 행이 빠진다.
298쪽도 후보에만 회색 행이 있다. 이들은 실제 선 개체가 아니라 배경 bitmap과 흰 셀
채움의 경계다. 숨김·clip 축소·페이지별 오프셋으로 처리하지 않는다.

원인은 절대 좌표를 한 번에 장치 정수로 반올림한 것과, 본문 단 원점과 상대 좌표를
각각 변환한 것의 차이다. source의 단 원점 A와 끝점 P에서 장치 좌표는
`round(A × 600/96) + round((P-A) × 600/96)`다. 물리 용지 단위는 이미 검증한
`PrintPaper.device_units`를 사용한다. 길이만 따로 반올림하지 않고 시작·끝을 각각
같은 원점으로 옮긴 뒤 차이를 취한다. 그림과 셀 채움이 같은 결과를 소비해야 한다.

| 독립 관측 | 기존 최대 좌표 오차 | 단 원점·상대 끝점 변환 |
| --- | ---: | ---: |
| 283·291·297·298·304쪽의 표 bitmap clip 14개 | 약 0.16062px | 최대 0.00066px |
| 같은 쪽 본문 흰 셀 채움 104개 | 약 0.16067px | PDF 0.001pt 표기 정밀도 안 |

위 계산은 원본·Print와 고정 Native library의 관측을 대조한 진단이다. 제품 수정이나
새 head의 시각 검증 완료를 뜻하지 않는다. 원시 좌표와 모든 쌍은 ignored
`handbook-fill-owner-diagnostic-3509/source-predicted-column-{clips,white-rectangles}.json`에
보존한다. 흰 사각형 진단의 나머지 두 사례는 본문 단 밖의 바탕쪽 가림띠다. 이 별도
좌표계를 본문 단으로 바꾸지 않으며, 진단의 최대 오차만 보고 같은 변환을 전역 적용하지 않는다.

### 생산·소비 경로와 적용 범위

1. 원본 `Table.common`의 `treat_as_char`, `horz_rel_to`, `vert_rel_to`에서 각 축이 본문
   단 좌표계를 쓰는지 명시한다. 기존 `publish_table_paint_layer`의 공통 생산 지점에서
   이 원본 계약을 전달한다. z-order·페이지·문단·모델 좌표는 바꾸지 않는다.
2. `PagePaintContext`는 실제 Column 원점과 표의 축별 참조를 유지한다. 표/셀 배경을
   그리는 직접 자식에서만 이를 사용하고, 셀 본문·중첩 개체·바탕쪽·머리말·꼬리말·각주·
   별도 affine 좌표계를 본문 단으로 추정하지 않는다. PAPER/PAGE 축은 기존 경로를 유지한다.
3. `bitmap_fill_frame`은 같은 참조로 clip 양 끝과 carrier의 요구 길이를 확정한다.
   `solid_fill_frame`도 같은 참조로 흰 셀의 양 끝을 확정한다. Native·Canvas2D·CanvasKit은
   공통 paint 결과를 소비하며 backend마다 오프셋을 재계산하지 않는다.
4. 위 5쪽과 원본 282·288·289·307쪽, 기존 PAPER 도형·그림/팔레트/종이 대조군을 먼저
   실행한다. 원시 clip/채움/그림 matrix가 독립 Print와 일치하는지 확인하고 직접 확대
   판독한다. 그 뒤 Native/fresh WASM 전쪽과 회귀 검증을 수행한다. 실패한 기존 검사의
   기대값을 먼저 바꾸거나 raw 기준·허용 반경을 완화하지 않는다.

### 공통 구현 후보

전체·분할 표의 기존 생산 지점에서 축별 원본 참조를 전달한다. 공통 paint의
`TableFillOrigin`은 실제 Column과 직접 표/셀 배경의 소유를 보존하며, 중첩 표·셀 본문·
바탕쪽·머리말·꼬리말·각주·별도 affine에서는 바깥 단 참조를 끊는다.
`PagePaintContext::fill_device_coordinate`가 단색 채움 양 끝과 bitmap clip/carrier의
양 끝을 함께 확정한다. 화면·2020 Print·PAPER/PAGE 축과 기존 bitmap 재생 산식은
유지한다. 이 기록 시점에는 새 후보의 실제 Native/fresh WASM 검증은 아직 실행 전이다.

후속 소비 경로 점검에서 셀의 직접 자식도 독립 도형일 수 있음을 확인했다.
`render_cell_background`가 실제 BorderFill 배경에 `table_background_paint`를 발행하고,
같은 공통 채움 변환은 이 명시적 소유를 확인한다. 셀 안에 있다는 사실이나 도형의
색·stroke 조합으로 배경을 추정하지 않는다. 첫 후보 `7f219ae638`의 Native 빌드는
6분 43초·exit 0이며, 집중 검증은 보존하고 보완 전 전쪽/WASM 예약만 중지했다.
이후 보완 후보를 실제로 다시 빌드·검증한다.

### 반 장치 픽셀에서의 원본 단위 보존

첫 집중 검증은 283·297·298·304쪽을 raw 100%로 개선했지만 정상 대조 307쪽은
527픽셀 후퇴했다. 이 결과를 통과로 보지 않는다. source의 단 원점과 상대 끝점을
독립 Print와 다시 대조했다. 307쪽 끝점의 상대 위치는 정확히 반 장치 픽셀이지만,
CSS 좌표의 뺄셈으로 `3766.5`가 `3766.4999999999995`가 되어 아래 정수로 바뀌었다.
실제 제품 layer의 흰 셀 끝은 749.847px이고 독립 Print의 끝은 750.006667px다.
후퇴는 회색 행이 추가된 것이 아니라 Print에 있는 750행이 사라진 것이다.

단 원점·상대 좌표의 변환에서만 원본 HU 정수를 부동소수점 연산 오차 범위 안에서
복원한다. 오차 한계는 HU/CSS 변환 두 연산과 차·역변환 두 연산에서 정하며,
시각 허용 오차·문서 ID·쪽·좌표 오프셋을 추가하지 않는다. 실제 소수 HU와 별도
PAPER/PAGE·화면·2020 경로는 유지한다. 보완한 실제 후보의 정상 대조·전쪽 검증은 남는다.
