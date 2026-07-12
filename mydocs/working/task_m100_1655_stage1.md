# Task M100 #1655 Stage 1

## 목적

HWPX 수식(`hp:equation`)의 `hp:pos@flowWithText`가 roundtrip에서 원본 IR 값대로 보존되는지 확인하고,
하드코딩된 직렬화 경로를 정정한다.

## 사전 확인

- GitHub 이슈 #1655는 `OPEN` 상태였다.
- 동일 범위를 처리하는 열린 PR은 없었다.
- parser는 `hp:pos@flowWithText`를 `CommonObjAttr.flow_with_text`로 읽을 수 있다.
- 모델은 `Equation.common`에 `CommonObjAttr`를 보유한다.
- serializer는 `render_equation`에서 `flowWithText="1"`을 고정 방출하고 있었다.
- roundtrip diff gate는 표의 `flowWithText`만 비교하고, 수식 arm에서는 누락되어 있었다.

## 구현 방침

- 수식 serializer는 입력 문서에서 읽은 `Equation.common.flow_with_text`를 그대로 방출한다.
- diff gate는 기존 `ObjectFlowWithText` 차이 타입을 재사용하되 path만 `eq`로 남긴다.
- 테스트는 수동 생성 IR에서 `flow_with_text=false` 수식을 만들고, serialize 후 XML 및 reparse 결과가
  모두 `false`인지 확인한다.

## 구현 결과

- `src/serializer/hwpx/section.rs`
  - `render_equation`의 `flowWithText="1"` 고정값을 제거하고 `Equation.common.flow_with_text`를 방출한다.
- `src/serializer/hwpx/mod.rs`
  - `task1655_equation_flow_with_text_roundtrips` 테스트를 추가했다.
  - `flow_with_text=false` 수식이 XML에서 `flowWithText="0"`으로 방출되고 재파싱 뒤에도 `false`로 유지되는지 확인한다.
- `src/serializer/hwpx/roundtrip.rs`
  - `Control::Equation` 비교에서도 `diff_flow_with_text`를 호출한다.
  - `task1655_equation_flow_with_text_in_gate` 테스트로 수식 차이가 `ObjectFlowWithText`로 검출되는지 확인한다.

## 검증

- `env CARGO_INCREMENTAL=0 cargo test --lib task1655 -- --nocapture`: 통과, 2 passed
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx -- --nocapture`: 통과, 275 passed
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings`: 통과

## 메모

- 변경은 HWPX serializer/roundtrip 게이트에 한정된다.
- renderer/layout/paint 변경이 아니므로 visual sweep 대상은 아니다.
