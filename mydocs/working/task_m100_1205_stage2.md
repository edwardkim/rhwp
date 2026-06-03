# Task M100-1205 Stage 2 완료 보고서 — side별 문단 border 렌더링

## 1. 작업 범위

구현계획서 Stage 2에 따라 문단 border group 렌더링 경로를 side별 `BorderLine` 기준으로 수정했다.

변경 파일:

- `src/renderer/layout.rs`
- `src/renderer/layout/integration_tests.rs`

## 2. 구현 내용

`src/renderer/layout.rs`의 문단 border group 렌더링에서 기존 top border 대표 stroke만 사용하는 방식을 보정했다.

주요 변경:

- `borderFill`의 side 배열 `[left, right, top, bottom]`을 직접 조회한다.
- `BorderLineType::None` side는 width/color 값이 있어도 invisible side로 처리한다.
- 4면이 모두 visible이고 `line_type/width/color`가 동일하며 partial skip이 없을 때만 기존 `RectangleNode` stroke 경로를 유지한다.
- 일부 side가 `NONE`이거나 side별 stroke가 다르면 fill-only `RectangleNode`와 visible side별 `LineNode`로 분리한다.
- side별 line 생성에는 기존 `border_rendering::create_border_line_nodes()`를 사용한다.

## 3. 테스트 보강

Stage 1 RED 테스트를 GREEN 테스트로 유지했다.

테스트명:

```text
task_1205_para_border_none_sides_do_not_render_vertical_edges
```

검증:

- left/right `NONE` 조합에서 4면 stroke rectangle 없음
- left/right `NONE` 조합에서 좌우 수직선 없음
- top/bottom `SOLID` 조합에서 가로선은 유지

## 4. 검증 결과

실행 명령:

```text
cargo fmt --all
cargo test --lib task_1205 -- --nocapture
cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture
cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture
```

결과:

```text
task_1205_para_border_none_sides_do_not_render_vertical_edges ... ok
test_469_partial_start_box_does_not_cross_col_top ... ok
test_471_cross_column_box_no_bottom_line_in_col0 ... ok
```

## 5. 판단

Stage 1에서 고정한 #1205 RED는 GREEN으로 전환됐다.

기존 문단 border partial/cross-column 회귀 가드인 #469, #471도 통과하므로, 현재 수정은 #1205의 side별 `NONE` 처리에 좁게 작동한다.

## 6. 다음 단계

작업지시자 승인 후 Stage 3으로 진행한다.

Stage 3 범위:

- 추가 회귀 테스트 검토
- 4면 SOLID 문단 border 기존 경로 유지 확인
- 필요 시 `cargo fmt --all --check`, 관련 `cargo test --lib` 확장 실행
