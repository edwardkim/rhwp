# #8153: crop 대조군 실패의 원본 창 가설과 실제 원인 추적

> 아래 원본 창 가설은 실제 소비자 추적으로 기각했다. 그 가설에 따른 코드 변경은
> 적용하지 않았다. 최신 판정은 마지막 절을 따른다.

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

## 실제 소비자에서 기각한 원본 창 가설

고정 producer `b919780c3b`, `afd8d95856`, `d3fdfa0c51`, `e8ec8c9845`,
`934aae310d`로 같은 입력을 다시 실행했다. B의 PNG는 이전 PNG와 바이트까지 같고,
B는 raw 100%, afd 이후 후보는 99.98976%였다. 실패는 device image 보정 이전부터
있었으므로 `d3fdfa0c51`의 자격 판정이 원인이라는 앞선 결론을 철회한다.

실제 B/D의 layer tree는 schema minor를 제외하고 같았다. 그림 5개는 모두
`fill_mode=None`으로 `device_image::eligible`의 대상이 아니다. 이 경로에는
`PaintOp::image`의 공통 창 정규화도 이미 적용된다. 재생 창을 원본으로 오인하는
일반 가능성과 이 실패의 실제 원인을 구분하며, 근거 없이 새 필드·예외를 넣지 않는다.

직접 실행한 변경점은 `afd8d95856`의 물리 DPI 표본 계약이다. 그 변경을 되돌리면
소수 종이에서 48px 그림을 49px로 늘리는 이미 입증한 이중 변환을 복구하게 되므로
37픽셀만 보고 되돌리지 않는다. 전체 가로·세로 프레임 투영만 바꾼 진단도 같은
37픽셀을 남겼다. crop 대상의 면적 표본을 끈 진단은 32픽셀을 남겼으므로 그것도
해결로 채택하지 않는다.

독립 PDF의 세 번째 그림은 원본에 없는 1bit `/Mask`를 함께 가진다. 원본 BMP는
모든 픽셀 alpha=255지만 PDF mask는 첫 열의 3, 7, 11…507행에 비트를 가진다.
JPEG에도 그 가장자리 차이가 있다. 이는 원본 투명도·정상 불투명 그림 축소와
구분해야 하는 실제 관측이며, 원인을 확정하기 전 같은 원본의 1쪽 Print 재현성을
확인한다. 기존 독립 PDF를 덮어쓰거나 기준·반경·임계값을 바꾸지 않는다.

진단·원본·후보 PNG와 layer는 ignored 작업 루트의 `visible-layer-*-diagnostic/`,
`visible-y-projection-diagnostic/`, 각 producer의 `native-visible-control/`,
`visible-control-independent-image-placement.json`에 보존했다. 실제 한컴 재출력은
`visible-control-repeat-print-20261024/`에만 보존한다. 이 단계는 미대응 0 복구나
최종 검증 완료가 아니다.

### 같은 원본의 독립 Print 반복 확인

job `d1653a0a-9ead-4e9b-a651-a625f2b11ef2`는 실제 한컴 2024
13.0.0.3901의 `PrintMethod=0`, `frame_print_to_pdf_ex_one_up`으로 1쪽·17,234바이트를
출력했다. 입력 전처리는 없다. 반복 PDF의 SHA-256은
`53b04c50b5e887cddf02b45061343a67b116fa53b199a0085840434028a34cbf`다.
기존 PDF와 여섯 이미지·mask의 압축/디코딩 스트림이 모두 같았다. 특히 세 번째
그림의 1bit mask도 그대로 재현됐다. 따라서 일시적 기준 손상으로 분류하거나
새 PDF로 기준을 교체하지 않는다. 투명도·JPEG 전달과 실제 source window의 관계를
계속 추적해야 한다. 반복 PDF와 상태·비교 해시는 앞 절의 ignored 경로에만 보존했다.
