# #8153: 그림 장치 영역의 원본 창 자격 보존

## 독립 실패와 원인 경로

사각형 보정 source `934aae310d`의 표준 fresh WASM 빌드는 22분 21초에 성공했다.
최초 정상 대조군 `bitmap-crop-palette-controls.hwpx`는 이전 producer와 같은
독립 Print를 정확한 물리 DPI 정책으로 비교했을 때, 이전 미대응 0에서 현재
37픽셀로 바뀌었다(raw 100% → 99.98976%). 현재 Native도 같은 37픽셀이다.
Native/WASM 차이나 PDF 재래스터화 때문에 생긴 차이로 분류하지 않는다.

실패는 원본이 조금 잘린 그림의 왼쪽 경계에 있다. source crop은 1 HU의 차이도
유효한 창이며, 기존 출력 장치 payload는 그 창을 바이트에 반영한다.
`image_for_resolved_window`는 중복 자르기를 막기 위해 이 payload를 재생할 복사본의
crop·원본 크기를 비운다. 이 정규화 자체는 필요한 소비 계약이다.

`d3fdfa0c51`에서 추가한 `device_image::eligible`는 crop이 없는 원본만 대상으로
선언했으나, Native·Canvas2D·JSON 모두 창 정규화 **뒤**의 복사본에 그 판정을
적용했다. 바이트에 crop이 이미 반영된 그림이 잘리지 않은 원본으로 오인되어
다른 목적 영역을 소비할 수 있다. 재생용 창과 원본 자격을 혼동한 공통 경로다.

## 보정과 범위

원본 PaintOp의 ImageNode로 장치 영역 자격을 판단한다. 바이트 재생은 기존대로
정규화한 창을 소비하며 crop을 다시 적용하지 않는다. Native·Canvas2D·JSON이
같은 `device_image::eligible`의 원본 판정을 사용하고, CanvasKit에는 JSON이 그
결과를 전달한다. 바이트·색·필터·원래 clip·줄/개체 소속은 수정하지 않는다.

문서 ID·그림 ID·37픽셀 위치·회색 값으로 예외를 만들지 않는다. 원래 선언한
무 crop 계약을 실제 원본 창에서 지키도록 고치며, 기준 PDF·임계값·반경과 기존
100% 대조군의 기대값을 완화하지 않는다.

## 재검증

- 같은 crop 대조군의 실제 Native/fresh WASM에서 미대응 0을 복구한다.
- 원본 PNG 채움 288·307쪽과 정상 무 crop 영역의 장치 경계 보정을 유지한다.
- 영향 페이지와 원본 전쪽을 새 source로 재출력하고, 바뀐 대조군의 PNG를 직접 판독한다.
- 현재 source의 전체 WASM 출력은 첫 대조군 실패 후 시작하지 않았다. 실패를
  무시하거나 전체 766쪽의 통과로 대신하지 않는다.

자료는 ignored 작업 루트의 `rectangle-overlap-934aae310d/` 아래
`wasm-visible-control-current-reference.json`, `native-visible-control/exact-audit.json`,
`wasm-visible-control-boundary-review.png`와 actual PNG/입력/PDF 해시에 보존했다.
이 계획 단계에서 원인 보정·미대응 0 복구·최종 제출 완료를 판정하지 않는다.
