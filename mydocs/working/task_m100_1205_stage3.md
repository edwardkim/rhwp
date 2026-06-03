# Task M100-1205 Stage 3 완료 보고서 — 회귀 테스트 확장

## 1. 작업 범위

구현계획서 Stage 3에 따라 #1205 직접 회귀 테스트를 정리하고, 기존 문단 border 경로를 보호하는 추가 단언을 넣었다.

변경 파일:

- `src/renderer/layout.rs`
- `src/renderer/layout/integration_tests.rs`

## 2. 테스트 구조 정리

`src/renderer/layout/integration_tests.rs`에 합성 문단 border 렌더링 helper를 추가했다.

helper:

```text
render_synthetic_para_border_counts()
```

역할:

- synthetic 문단 1개와 지정한 `[left, right, top, bottom]` border 배열을 렌더링한다.
- 렌더 트리 전체에서 stroke rectangle, vertical line, horizontal line 개수를 수집한다.
- #1205처럼 특정 샘플 파일이 없어도 문단 border renderer의 side별 동작을 검증할 수 있다.

## 3. 추가 회귀 테스트

### 3.1 NONE side 수직선 방지

테스트:

```text
task_1205_para_border_none_sides_do_not_render_vertical_edges
```

검증:

- `[NONE, NONE, SOLID, SOLID]` 조합에서 4면 stroke rectangle 없음
- 좌우 수직선 없음
- 가로선은 유지

### 3.2 Rectangle stroke 경로 조건 고정

테스트:

```text
task_1205_rect_stroke_path_requires_four_visible_same_stroke
```

검증:

- 4면이 모두 visible이고 동일 stroke이며 partial skip이 없을 때만 기존 `RectangleNode` stroke 경로를 사용할 수 있다.
- `NONE` side가 있으면 Rectangle stroke 경로를 사용할 수 없다.
- side별 stroke가 다르면 Rectangle stroke 경로를 사용할 수 없다.
- partial skip이 있으면 Rectangle stroke 경로를 사용할 수 없다.

이를 위해 `src/renderer/layout.rs`의 분기 조건을 작은 helper로 분리했다.

## 4. 검증 결과

실행 명령:

```text
cargo fmt --all
cargo test --lib task_1205 -- --nocapture
cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture
cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture
cargo fmt --all --check
```

결과:

```text
task_1205_para_border_none_sides_do_not_render_vertical_edges ... ok
task_1205_rect_stroke_path_requires_four_visible_same_stroke ... ok
test_469_partial_start_box_does_not_cross_col_top ... ok
test_471_cross_column_box_no_bottom_line_in_col0 ... ok
cargo fmt --all --check ... ok
```

## 5. 판단

Stage 3 기준으로 #1205 직접 회귀와 기존 partial/cross-column 문단 border 가드는 모두 통과했다.

4면 동일 visible stroke는 기존 Rectangle stroke 경로 조건을 유지하고, `NONE` side 또는 partial edge가 있으면 side별 line 경로로 내려가도록 조건이 고정됐다.

## 6. 다음 단계

작업지시자 승인 후 Stage 4로 진행한다.

Stage 4 범위:

- 전체 관련 검증 확장
- 실제 재현 샘플이 접근 가능하면 SVG 산출 및 시각 확인
- 최종 보고서 작성
- 오늘 할일 상태 갱신
