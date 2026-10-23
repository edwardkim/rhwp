# #8153: Print 사각형 clip의 최종 장치 영역

## 위반된 계약과 독립 근거

`d3fdfa0c51`에서 그림의 장치 영역을 보정한 실제 288쪽을 확대 판독했다. 셀 경계
x=210, y=307~315의 Native RGB 248 / 독립 Print RGB 255인 세로 이음은 남았다.
원래 사각형 두 개의 채움은 모두 흰색이며, Native의 소수 TableCell clip이 한쪽
흰색 채움을 부분 alpha로 잘라 아래 회색 그림이 비친다. raw의 내용 임계값 밖이라도
실제 보이는 차이를 완료로 취급하지 않는다.

기존 독립 PDF의 사각형 clip 16건을 실제 Native 소비자로 재생했다. 원래 TableCell
clip은 RGB 전체 일치 2건 / 불일치 14건이었다. 공통 외접 장치 영역을 해당 clip에
전달한 진단은 16건 모두 독립 PNG와 RGB 전체가 일치했다. helper 반환값만 비교한
검사가 아니다. PDF·PNG·임계값은 바꾸지 않았으며 이 합성 진단을 실제 한컴 문서의
완료나 정식 회귀 등록으로 보고하지 않는다.

공개 [PDFium AGG의 `SetClip_PathFill`](https://pdfium.googlesource.com/pdfium/+/a697e197f3a729db2a65b02d46b636644491293a/core/fxge/agg/cfx_agg_devicedriver.cpp)은
사각형을 판별한 뒤 `GetOuterRect`와 `IntersectRect`로 장치 포함 영역을 소비한다. 지금은 bitmap 채움
소속 clip만 이 계약을 사용하지만, 같은 사각형 clip인 Body·TableCell·TextBox에서도
같은 계약을 소비해야 한다. clip 소유 종류와 장치 포함 영역을 혼동한 소비 조건을 고친다.

## 생산·소비와 반례

- 원래 clip bbox → 실제 backend 행렬 → 공통 Rust 사각형 외접 영역 → 실제 clip
  순서로 연결한다. 각 소비자가 다른 floor·ceil·오차값을 재구현하지 않는다.
- Print raster의 양의 축 정렬 사각형만 대응한다. Screen·SVG·벡터 PDF와 임의 경로
  clip, 회전·skew·음의 배율은 기존 계약을 유지한다. clipping을 비활성화한 출력도
  기존 계약을 유지한다. 조판 bbox·쪽·내용 소속은 변경하지 않는다.
- Native·Canvas2D·CanvasKit이 같은 공통 결과를 소비한다. 기존 bitmap clip WASM
  바인딩의 외접 연산·인자 계약은 유지하며, 그림의 표본·색을 다시 바꾸지 않는다.
- 실제 288쪽 이음과 기존 경계 10곳, 영향 페이지, Screen 및 clipping 비활성 대조를
  먼저 확인한다. 전쪽·최종 fresh WASM 검증은 새 code SHA에서 다시 판정한다.

## 보존 자료와 완료 조건

ignored 작업 경로의 `image-coverage-d3fdfa0c51/cell-clip-consumer-diagnostic/`에
16건의 원본·대응 영역 재생 PNG와 source/helper SHA를 보존했다. 실제 원본의 이음은
같은 후보의 `remaining-cell-clip-seam.json`과 `boundary-original-native-review.png`에 있다.
현재 `d3fdfa0c51` Native 전쪽 검증은 독립 실행 중이며 고정 producer와 결과를 보존한다.
진행 중인 결과를 새 clip 후보의 검증으로 혼용하지 않는다. 전쪽 raw 100%·실제 미대응
픽셀 0·직접 판독·최종 회귀와 PR 절차가 끝나야 전체 목표를 완료한다.
