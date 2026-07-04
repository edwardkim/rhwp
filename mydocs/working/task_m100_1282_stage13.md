# Task 1282 Stage 13 - 제한 해제 그림 상단 이동 자리 차지 보정

## 문제

- `ta-pic-001-r-쪽영역안제한.hwp`에서 `쪽 영역 안으로 제한`을 끈 뒤 그림을 화면 상단으로 이동하면 rhwp-studio 속성창의 세로 위치가 `문단/위/15151671.70mm`처럼 비정상적으로 커진다.
- 한컴오피스에서는 같은 조작 후 정상 범위의 위치 값으로 표시된다.
- 해당 그림은 `본문과의 배치=자리 차지`이므로 그림을 위로 옮기면 아래 표가 그림 아래로 당겨져야 하는데, rhwp-studio에서는 표 흐름 동기화가 맞지 않는다.

## 조사 대상

- `samples/ta-pic-001-r-쪽영역안제한.hwp`
- `samples/ta-pic-001-r-쪽영역안제한no.hwp`
- 그림 속성: `TopAndBottom`, `VertRelTo::Para`, `VertAlign::Top`, `HorzRelTo::Column`, `HorzAlign::Left`, `restrictInPage=false`

## 분석 계획

1. 제한 OFF 상태에서 마우스 이동/속성 저장이 어떤 좌표계를 `vertOffset`에 넣는지 확인한다.
2. table cell picture가 제한 OFF일 때 렌더 좌표와 저장 좌표의 기준점이 동일한지 확인한다.
3. `자리 차지` 흐름에서 부모 문단 line segment가 그림 높이와 offset 변화에 맞춰 표를 다시 배치하는지 확인한다.
4. 한컴 비교 기준에 맞춰 좌표 변환과 line segment/table 높이 동기화를 보정한다.

## 원인

- 프론트엔드 이동/리사이즈 경로가 `horzOffset`/`vertOffset`을 `>>> 0`으로 unsigned 변환해 음수 이동을 큰 양수로 래핑했다.
- Rust 속성 JSON도 `u32` offset을 그대로 노출해 속성창에 `15151671.70mm` 같은 비정상 수치가 표시됐다.
- `TopAndBottom` 자리차지 그림의 흐름 높이를 `max(vertOffset, 0) + height`로 계산해, 그림을 위로 이동해도 아래 표가 함께 당겨지지 않았다.

## 수정

- 그림 속성 JSON과 적용 경로에서 offset을 signed 값으로 읽고 쓰도록 변경했다.
- rhwp-studio의 그림 이동/리사이즈/방향키 이동/undo command 경로에서 offset unsigned 래핑을 제거했다.
- 자리차지 그림의 부모 문단 line segment 높이를 `vertOffset + visualHeight` 기준으로 갱신해, 제한 OFF 상태에서 그림을 위로 이동하면 뒤따르는 표도 한컴처럼 위로 당겨지게 했다.

## 검증 예정

- Rust 통합 테스트에 제한 OFF 상단 이동 좌표 폭주 방지와 표 당김 검증 추가.
- headless E2E로 제한 OFF 후 상단 이동 시 속성값 정상 범위와 table bbox 이동 확인.

## 검증 결과

- `cargo fmt --check && cargo test --test issue_1282_rotated_cell_picture_resize`
  - 6개 테스트 통과
  - `issue_1282_unrestricted_take_place_negative_offset_pulls_table_up` 추가 검증
- `wasm-pack build --target web --out-dir pkg`
  - 성공
- `node rhwp-studio/e2e/table-picture-resize-1282.test.mjs --mode=headless`
  - 성공
  - 제한 OFF 후 상단 이동 속성값: `vertOffset=-5890`, `horzOffset=2030`
  - 제한 OFF 후 그림 y: `273.3 -> 55.6`
  - 제한 OFF 후 표 y: `709.1 -> 491.4`
