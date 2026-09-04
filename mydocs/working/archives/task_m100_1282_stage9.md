# Task 1282 Stage 9 - 가로 확장 시 셀 폭 유지 방식 재수정

## 문제

Stage 8은 회전 그림의 인접 셀 침범을 막기 위해 picture frame을 owner cell 안쪽 폭으로 clamp했다.
이후 Stage 9 1차 시도에서 소유 셀/열 폭을 함께 확장했지만, 한컴오피스 기준과 다르다.

한컴오피스는 그림의 세로 크기가 커지면 셀/행 높이는 늘리지만, 그림의 가로 크기가 커져도 셀/열 폭은 바꾸지 않는다.
가로 방향은 picture 크기 값은 커지되 셀 경계에서 보이는 영역이 제한되어야 한다.

## 목표

- 셀 내부 그림을 셀 폭보다 크게 리사이즈할 수 있어야 한다.
- 그림 가로 크기 변경은 소유 셀/열/표 폭을 바꾸지 않아야 한다.
- 그림 세로 크기 변경은 owner cell 높이를 동기화해야 한다.
- 회전각 변경/축소 시 셀 높이 동기화는 유지한다.
- Stage 8에서 추가한 visual hull 회귀 검증은 작은 리사이즈/회전 정합에 유지하고, 과대 가로 리사이즈는 셀 폭 유지 조건을 따로 검증한다.

## 구현 방향

1. `object_ops.rs`의 picture 폭 clamp와 owner column 확장을 모두 제거한다.
2. direct owner cell 동기화는 세로 required height만 갱신한다.
3. full/partial 표 렌더의 non-inline picture 폭 clamp는 제거해 모델에 저장된 확장 폭이 유지되게 한다.
4. 표 셀 clip은 렌더 단계에서 정확한 cell bbox를 사용한다. 기존 Canvas 2D/CanvasKit fast-preview의 tableCell 우측 4px 여유는 그림까지 통과시키므로 제거한다.
5. E2E는 과대 리사이즈 후 picture width가 커졌고 owner cell width/bbox는 유지되는지 확인한다.
6. 사용자 비교 기준값(너비 97.45mm, 높이 115.07mm, 가로 offset 0mm, 세로 offset 36.82mm, 회전 0도)을 재현하고, 오른쪽 셀 내부 픽셀 샘플이 비어 있는지 검사한다.

## 한컴 비교 기준

- 파일: `samples/ta-pic-001-r.hwp`
- 셀 속성: 너비 84.0mm, 높이 154.8mm, 안쪽 여백 좌/우 1.8mm, 위/아래 0.5mm
- 그림 속성: 너비 97.45mm, 높이 115.07mm, 가로 `단/왼쪽/0mm`, 세로 `문단/위/36.82mm`, 회전 0도
- 기대 동작: 그림 모델 폭은 셀 폭보다 커질 수 있으나, 화면 렌더는 owner cell 경계에서 잘리고 오른쪽 셀로 보이지 않아야 한다.

## 시각 증적

- `mydocs/report/assets/task_m100_1282_stage9_hancom_props_clip.png`
- headless Chrome에서 위 한컴 비교 기준값을 직접 적용해 캡처했다.
- e2e 픽셀 샘플 결과: owner cell 오른쪽 경계 바깥 샘플 영역 `nonWhite=0`, `dark=0`

## 검증 대기

- [x] `cargo fmt --check`
- [x] `git diff --check`
- [x] `cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture`
- [x] `wasm-pack build --target web --out-dir pkg`
- [x] `node rhwp-studio/e2e/table-picture-resize-1282.test.mjs --mode=headless`
- [ ] 사용자 시각 판단
