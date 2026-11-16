# 보호 대상 그림의 Canvas 가장자리 보정

## 원 해결과 독립 근거

PR #8389의 `c417310cab`은 Print 그림의 장치 영역을 `device_image`와
`web_canvas/image_coverage`에서 공유한다. 이 규칙은 잘리지 않은 2024 bitmap 채움에
적용되며, `pic-crop-01.hwp`의 실제 crop 창에는 적용되지 않는다. 원본과 기존 독립
`pdf/pic-crop-01-2022.pdf`는 변경하지 않는다. Native는 전 1쪽 100.00%지만 동일 입력의
최적화 WASM은 94.65%이며, 두 번째 그림 위 테두리가 사라진다.

실제 Canvas 호출을 추적하면 테두리 뒤에 소수 좌표의 `drawImage`가 실행된다. Native는
`skia/image_conv.rs`에서 그림 paint의 anti-alias를 사용한다. Canvas의 축에 평행한
`drawImage`는 그림 가장자리의 부분 덮임을 보존하지 않아 아래 테두리를 덮는다.
글꼴이 없는 그림 입력이며, 최적화 유무에 따른 출력도 같아 두 원인 가설은 제외했다.
좌표·source 창·표본·색을 유지하고 실제 목적 사각형의 Canvas clip 덮임만 공급한 진단은
raw 99.99756%, 공식 표시값 100.00%다. 이는 아직 제품 수정의 완료 증거가 아니다.

## 구현과 소비 경로

원본 그림의 배치·crop → 기존 공통 bitmap 준비 → 최종 Canvas `drawImage` 목적 영역 →
그 목적 영역의 부분 픽셀 덮임을 소비하게 한다. 기존 장치 프레임·축소 재구성의 조기 반환은
그대로 두며, 최종 일반 bitmap 그리기에만 적용한다. Print의 축에 평행한 그림 경계에서
Canvas clip의 표준 덮임을 공급하고 Screen·회전/기울임은 기존 브라우저 처리를 유지한다.
목적 좌표를 반올림하거나 이동하지 않으며 원본 source 창과 보간 선언도 바꾸지 않는다.
전체 그림과 crop의 canvas/image 캐시 분기가 같은 scope를 사용하고 종료 시 context를 복원한다.

## 검증 순서

1. 실제 WASM의 원본 두 그림을 그려 독립 PDF에 있는 각 위 테두리의 연속 잉크가 보존되는지
   관계형 브라우저 검사로 확인한다. 문서별 절대 좌표·기존 오답 PNG·사설 글꼴은 사용하지 않는다.
2. 기존 최적화 package에서 새 검사의 FAIL을 확인하고, 제품 보정 후 최적화 fresh WASM에서
   PASS를 확인한다. 기존 JPEG 장치 프레임 두 검사도 함께 통과해야 한다.
3. 같은 독립 PDF와 제품 Native/fresh WASM 전 1쪽 TSV·review/overlay를 재산출한다.
   진단 PNG를 제품 증거로 대신하지 않는다. 보호 대상 전체·본 PR의 원본·편람 gate와
   필수 로컬 검증은 최종 코드에서 완료한다. 기준을 낮추거나 미완료 검증을 통과로 기록하지 않는다.

진단 기록은 `output/pr-review/pr9075/picture-canvas-coverage-isolation.json`과
`picture-canvas-trace-2/canvas-trace-001.json`에 있으며, 실제 제품 결과는 별도로 고정한다.

제품의 최적화 fresh WASM·새 관계형 검사 FAIL→PASS·기존 browser aggregate 20건 통과와
그림 전 1쪽 100.00%는 [보정 단계 결과](canvas-image-stage/README.md)에 고정했다.
주 입력 전 14쪽의 실제 fresh WASM 재출력은 [별도 결과](main-fresh-wasm-stage/README.md)에 있다.
보호 기준의 잔여 하락과 전체 필수 검증은 계속 진행한다.
