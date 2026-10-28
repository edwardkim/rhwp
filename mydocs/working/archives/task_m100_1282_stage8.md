# Task 1282 Stage 8 - 회전 그림 다단 경계 침범 보정

## 문제

`samples/ta-pic-001-r.hwp`에서 표 셀 안 회전 그림을 리사이즈/회전 보정한 뒤 rhwp-studio 렌더가 한컴오피스와 다르게
다단 또는 인접 셀 범위를 침범해 보인다.

## 기준

- 한컴오피스 화면 기준:
  - 회전 그림은 소유 셀의 시각 경계 안에서 배치되어야 한다.
  - 크기/회전각 보정 후에도 인접 다단/셀 콘텐츠 영역을 침범하지 않아야 한다.
  - 셀 높이 자동 조정은 유지하되, 그림의 기준 위치가 한컴과 다르게 튀면 안 된다.

## 조사 계획

1. 현재 `ta-pic-001-r.hwp` 렌더와 한컴 기준 스크린샷의 차이를 정리한다.
2. `object_ops.rs`의 회전 bounding-box/셀 높이 동기화와 renderer table layout의 offset 적용을 분리해서 확인한다.
3. 회전 후 저장/렌더 시 `horizontal_offset`, `vertical_offset`, 셀 제약, line segment 기준이 서로 다른 좌표계를 쓰는지 점검한다.

## 검증

- [x] `cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture`
- [x] `wasm-pack build --target web --out-dir pkg`
- [x] `node rhwp-studio/e2e/table-picture-resize-1282.test.mjs --mode=headless`
- [x] Browser/Playwright 시각 확인
- [x] `cargo fmt --check`
- [x] `git diff --check`
- [ ] `cargo clippy --all-targets -- -D warnings`

## 상태

시각 판정 실패로 PR 준비를 중단하고 재수정한다.

## 1차 수정

- `table_layout.rs`의 full table 경로와 달리 `table_partial.rs`는 분할/partial 렌더에서 셀 `clip`을 split row에만 켜고 있었다.
- 같은 partial 경로의 non-inline picture는 `layout_picture`에 원본 offset/alignment를 직접 전달해 full table 경로와 좌표계가 달랐다.
- partial 표 셀도 항상 clip하도록 맞추고, non-inline picture는 full 경로처럼 `compute_object_position`으로 최종 좌표를 만든 뒤 offset/alignment를 초기화한 picture로 렌더하도록 변경했다.

## 검증 결과

- Rust 집중 테스트, WASM 빌드, headless E2E가 통과했다.
- Browser 시각 확인에서 `ta-pic-001-r.hwp` 로드 직후 및 회전 그림 드래그 확대 후 실제 이미지 픽셀이 owner cell 내부에서 잘리고 우측 셀로 넘어가지 않는 것을 확인했다.
- E2E에 picture 표시 bbox가 owner cell bbox 안에 유지되는 회귀 assertion을 추가했다.

## 2차 수정

- 사용자 시각 확인에서 큰 리사이즈 후 회전 picture 실제 픽셀이 다단/셀 경계를 침범하는 것이 재확인되었다.
- 원인은 회전 picture의 `common.width/height`가 이미 회전 후 외접 프레임인데, 렌더 경로가 이를 다시 실제 이미지 크기로 사용해 회전을 한 번 더 적용한 점이다.
- 렌더러는 회전 picture에서 `current_width/current_height`를 실제 이미지 크기로 사용하고, `common.width/height` 프레임 중앙에 배치한 뒤 회전하도록 수정했다.
- 셀 높이 동기화는 회전 프레임인 `common.height`를 기준으로 바꾸고, 회귀 테스트/E2E 기대식도 같은 기준으로 갱신했다.

## 3차 수정

- E2E가 단순 picture bbox만 검사해 회전 후 실제 visual hull 침범을 놓칠 수 있어, 회전각 기준 visual bbox를 계산해 owner cell bbox 안에 들어오는지 확인하도록 보강했다.
- `setCellPicturePropertiesByPath` 후 owner cell 안쪽 폭보다 큰 회전 frame을 비율 유지 축소하고, horizontal offset도 셀 내부로 clamp하도록 보정했다.
- full/partial 표 렌더 경로의 non-inline cell picture도 셀 가용 폭을 넘으면 비율 유지 축소하도록 맞췄다.

## Stage 8 한계 및 Stage 9 이관

- 사용자 시각 검토 결과, Stage 8의 폭 clamp는 경계 침범은 막지만 한컴오피스처럼 그림이 셀보다 크게 확장되면서 셀/열 경계가 함께 커지는 동작을 막는다.
- Stage 9에서는 picture 자체를 셀 폭에 clamp하지 않고, 소유 셀/열/표 폭을 그림 frame에 맞춰 확장해 인접 셀 경계를 유지하는 방식으로 재수정한다.
