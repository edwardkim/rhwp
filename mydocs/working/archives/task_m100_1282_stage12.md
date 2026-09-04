# Task 1282 Stage 12 - 쪽 영역 제한 그림 이동/해제 정합화

## 문제

- 한컴오피스에서는 표 셀 내부 그림의 `쪽 영역 안으로 제한`이 켜져 있으면 그림을 셀 밖으로 끌어낼 수 없다.
- rhwp-studio에서는 같은 샘플에서 그림을 드래그하면 셀 왼쪽/아래쪽 밖으로 이동할 수 있다.
- `ta-pic-001-r-쪽영역안제한.hwp`에서 `쪽 영역 안으로 제한`을 끄면 저장 샘플 `ta-pic-001-r-쪽영역안제한no.hwp`와 같은 배치가 되어야 하는데, Stage11까지는 표 내부 흐름 높이와 렌더 트리가 그대로 남았다.

## 조사 대상

- `samples/ta-pic-001-r-쪽영역안제한.hwp`
- `samples/ta-pic-001-r-쪽영역안제한no.hwp`
- `samples/ta-pic-001-r-쪽영역안제한.hwpx`

## 원인 분석

- 마우스 드래그와 방향키 이동은 최종적으로 `setCellPicturePropertiesByPath`에 `horzOffset`/`vertOffset`을 전달한다.
- WASM의 `set_cell_picture_properties_by_path_native`는 Stage11까지 `restrictInPage=true`인 셀 내부 picture도 offset을 그대로 저장했다.
- 따라서 JavaScript에서 음수 offset을 `u32`로 전달하면 Rust 쪽에서 signed 음수 위치로 해석되어 그림이 셀 왼쪽/위쪽 밖으로 나갈 수 있었다.
- 방향키 이동 기록 명령은 셀 내부 picture의 `cellPath`를 `MovePictureCommand`에 넘기지 않아 Undo/Redo 경로가 셀 내부 by-path API를 타지 않는 허점도 있었다.
- 제한을 끈 저장 샘플과 제한 ON 샘플의 IR을 비교하면 그림 자체의 셀 소유권은 유지되지만, 바깥 문단 `LINE_SEG.vertical_pos`가 `vertOffset + 그림 높이`로 바뀌고 표 공통 높이는 원래 표 높이로 줄어든다.
- 즉 `쪽 영역 안으로 제한=false`와 `본문과의 배치=자리 차지` 조합은 셀 높이를 그림만큼 계속 키우는 것이 아니라, 그림이 표 앞쪽 자리 차지 흐름으로 분리되고 표는 그 아래로 밀려나는 모델이다.

## 수정

- `set_cell_picture_properties_by_path_native`에서 직접 소유 표 셀 picture이고 `flow_with_text=true`이면 이동 offset을 셀 내부 기준으로 클램프한다.
- `horzOffset`은 `0..=(셀 내부 폭 - 그림 폭)` 범위로 제한한다. 그림 폭이 셀 내부 폭보다 크면 한컴처럼 `0`에 고정한다.
- `vertOffset`은 최소 `0`으로 제한한다. 아래쪽 이동은 기존 Stage11 동작처럼 행/셀 높이 동기화가 받아서 셀 내부에 남도록 한다.
- `restrictInPage=false` 샘플은 기존처럼 offset을 클램프하지 않는다.
- 방향키 이동 명령 생성 시 `cellPath`를 넘겨 셀 내부 picture 이동 기록도 by-path API를 사용하게 했다.
- picture 속성창에는 `TopAndBottom` 배치를 한컴 UI의 `자리 차지`로 표시하는 pseudo 선택지를 추가했다.
- 제한을 끄는 속성 변경에서는 저장 `no` 샘플처럼 셀 picture를 표 셀 렌더 평면 밖으로 배치하고, 부모 문단 line segment의 `vertical_pos`/`line_height`/`text_height`/`baseline_distance`를 동기화한다.
- 제한을 다시 켜거나 제한 ON 상태에서 크기/회전 변경을 하는 경우에는 기존 Stage11의 셀 높이 동기화 모델을 유지한다.

## 기대 동작

- `restrictInPage=true`인 셀 내부 picture는 이동 후 표시 bbox가 소유 셀 경계를 넘지 않는다.
- `restrictInPage=false`로 전환하면 `ta-pic-001-r-쪽영역안제한no.hwp`와 같은 picture/table/owner cell 렌더 bbox가 나온다.
- 크기 조절로 셀 높이가 늘어나는 기존 Stage11 동작은 유지한다.

## 검증

- `cargo fmt --check`
- `cargo test --test issue_1282_rotated_cell_picture_resize`
  - `issue_1282_restrict_in_page_clamps_cell_picture_move_offsets`
  - `issue_1282_unrestricted_cell_picture_move_offsets_are_not_clamped`
  - `issue_1282_turning_off_restrict_in_page_releases_picture_from_cell_flow`
- `wasm-pack build --target web --out-dir pkg`
- `node rhwp-studio/e2e/table-picture-resize-1282.test.mjs --mode=headless`
  - 제한 on 그림의 좌/상/우 이동 offset 클램프 확인
  - 제한 on 그림의 방향키 이동 클램프 확인
  - `쪽영역안제한.hwp`에서 제한 off 전환 후 `쪽영역안제한no.hwp`와 picture/table/owner cell bbox 동일성 확인

## 상태

- 자동 검증 완료.
- Stage13에서 제한 OFF 상태의 상단 이동 좌표 폭주와 `자리 차지` 후속 표 당김 문제를 별도로 처리한다.
