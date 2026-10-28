# Task 1436 Stage 1

## 목적

`CommonObjAttr.size_protect`는 이미 HWP5/HWPX 파서와 직렬화 계층에 존재한다. 이번 단계에서는 이 값을 Studio가 쓰는 속성 JSON과 TypeScript 타입에 `sizeProtect`로 노출하고, 저장 경로에서 다시 반영되도록 만든다.

## 범위

- `getPictureProperties`/`getShapeProperties` 계열 JSON에 `sizeProtect` 포함
- `setPictureProperties`/`setShapeProperties` 계열 JSON에서 `sizeProtect` 반영
- HWP5 attr bit 20 및 HWPX `hp:sz@protect` 기존 매핑 유지 확인
- Studio 타입 정의 갱신

## 제외

- 개체 속성 대화상자 비활성화 UI
- 드래그/키보드 리사이즈 및 회전 조작 차단
- 모달 외부 클릭 정책 정리

위 항목은 Stage 2 이후에서 진행한다.

## 검증 계획

- `cargo test --test issue_1282_rotated_cell_picture_resize`
- 신규/기존 단위 테스트로 `sizeProtect` JSON 노출과 setter 반영 확인
- `git diff --check`

## 수행 내용

- `format_picture_properties_json` 결과에 `sizeProtect`를 추가했다.
- `common_obj_attr_to_json` 결과에 `sizeProtect`를 추가해 Shape/Equation 공용 속성 JSON에서도 같은 값을 볼 수 있게 했다.
- Picture 전용 setter와 CommonObjAttr 공용 setter에서 `sizeProtect` 입력을 받아 `CommonObjAttr.size_protect`와 HWP5 attr bit 20을 함께 갱신하도록 했다.
- Studio 타입 정의의 `PictureProperties`, `ShapeProperties`, `EquationProperties`에 `sizeProtect`를 추가했다.
- `tests/issue_1436_size_protect_properties.rs`를 추가해 표 셀 그림/도형 속성 JSON의 `sizeProtect` 왕복과 attr bit 20 반영을 검증했다.

## 검증 결과

- `cargo fmt --check` 통과
- `cargo test --test issue_1436_size_protect_properties -- --nocapture` 통과
  - `issue_1436_picture_properties_round_trip_size_protect`
  - `issue_1436_shape_properties_round_trip_size_protect`
- `cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture` 통과
  - 6 passed
- `git diff --check` 통과
