# 구현계획서 — task_m100_3211 LayoutFrame 기반 LineSeg 재조판

- **이슈**: [#3211](https://github.com/edwardkim/rhwp/issues/3211)
- **선행 검증 PR**: [#4315](https://github.com/edwardkim/rhwp/pull/4315)
- **구현 PR**: [#4755](https://github.com/edwardkim/rhwp/pull/4755)
- **브랜치**: `renderer/lineseg-frame-reflow`
- **기준**: `upstream/devel` `fbca0aa6c`
- **작성일**: 2026-08-13
- **최종 정합**: 2026-08-14

## 1. 목표

단일 가용 폭으로 문단 전체의 `LineSeg`를 먼저 확정하던 경로를 물리 행 단위의
`LayoutFrame` 재조판으로 바꾼다. 그림 회피 영역이나 표 셀처럼 한 물리 행이 여러 가로 구간으로
나뉘어도 각 구간의 텍스트 진행과 공통 세로 지표를 한 행의 계약으로 유지한다.

저장 `LineSeg`는 새 행의 가로 geometry를 대신하지 않는다. 완전한 단일-segment 행만 보존할 수 있고,
FIRST부터 LAST까지 나뉜 행은 하나의 물리 행으로 다시 계산한다.

## 2. `LayoutFrame` 계약

`LayoutFrame`은 RenderTree 산출물이 아니라 레이아웃이 소유하는 가변 flow geometry다.

```rust
pub(crate) struct LayoutFrame {
    pub(crate) horizontal: Range<i32>,
    pub(crate) top: i32,
    pub(crate) exclusions: Vec<FrameExclusion>,
    pub(crate) current_intervals: Vec<Range<i32>>,
    pub(crate) next_geometry_event: Option<i32>,
    pub(crate) minimum_width: i32,
    rows: Vec<PhysicalRow>,
}
```

### `LayoutFrame::carve()`

```rust
pub(crate) fn carve(&mut self, band_height: i32) -> &[Range<i32>];
```

호출자는 내용에서 계산한 후보 행 높이만 전달한다. 현재 가로 범위, 세로 위치, exclusion과 최소 폭은
Frame이 이미 소유하므로 `Paragraph`나 물리 행 index를 인자로 다시 전달하지 않는다.

`carve()`는 다음 상태만 계산한다.

- 현재 미확정 행의 세로 band
- 왼쪽에서 오른쪽으로 정렬된 가로 구간
- 다음으로 이동할 수 있는 정확한 geometry event

고정 조건:

- band와 exclusion은 half-open 구간으로 교차한다.
- `BothSides`는 양쪽 구간을, `LargestSide`는 선택된 한쪽 구간을 남긴다.
- 사용할 수 있는 구간이 없으면 임의의 `+1` 없이 다음 geometry event에서 재시도한다.
- 최소 폭 미만 구간은 두 개 이상일 때만 제거하여 마지막 구간 하나를 보존한다.
- 더 높은 후보 행으로 다시 호출할 수 있도록 commit 이전 상태만 바꾼다.

`carve()`는 텍스트를 채우거나 `LineSeg`를 생성하지 않고, 행을 commit하거나 Frame을 다음 행으로
전진시키지 않는다.

## 3. 물리 행 transaction

`PhysicalRow`는 한 번 기록되는 `FrameRowMetrics`와 왼쪽에서 오른쪽으로 정렬된 `RowSegment`를
소유한다. `LineSeg`의 FIRST/LAST 경계 비트는 완성된 행을 투영할 때 만든다.

```rust
pub(crate) fn commit_carved_row(
    &mut self,
    metrics: FrameRowMetrics,
    segments: Vec<RowSegment>,
) -> Option<usize>;
```

commit은 segment 수와 각 가로 구간이 마지막 `carve()` 결과와 정확히 같은지 확인한다. 수용된 행의
모든 segment는 같은 `vertical_pos`, `line_height`, `text_height`, `baseline_distance`,
`line_spacing`을 가지며, Frame의 세로 위치는 행 전체에 대해 정확히 한 번만 전진한다.

문단 채움은 caller-owned Frame에서 다음 상태 전이로 수행한다.

```rust
let frame_checkpoint = frame.clone();
let row_frame_checkpoint = frame.clone();
let cursor_checkpoint = cursor.clone();

loop {
    frame.restore_checkpoint(row_frame_checkpoint.clone());
    cursor = cursor_checkpoint.clone();

    let intervals = frame.carve(candidate_height).to_vec();
    let segments = fill_each_interval(intervals, &mut cursor)?;
    let metrics = resolve_row_metrics(&segments);

    if metrics.line_height != candidate_height {
        candidate_height = metrics.line_height;
        continue;
    }

    frame.commit_carved_row(metrics, segments)?;
    break;
}

if transaction_failed {
    frame.restore_checkpoint(frame_checkpoint);
}
```

위 의사 코드는 `layout_paragraph_in_frame()`의 소유권과 rollback 순서를 고정한다. 작은 내부 계산을
그 이름의 새 함수로 추출하라는 요구는 아니다.

## 4. 연결 범위

### `src/renderer/layout_frame.rs`

- `LayoutFrame`, `PhysicalRow`, `RowSegment`, `FrameRowMetrics`를 레이아웃 계층에 둔다.
- `carve()`와 `commit_carved_row()`를 분리한다.
- 완성된 물리 행만 `project_line_segs()`에서 평탄한 `LineSeg`로 투영한다.

### `src/renderer/composer/line_breaking.rs`

- `FillCursor`가 가로 구간 사이의 token과 UTF-16 진행을 보존한다.
- `fill_one_interval()`은 Frame이 제공한 한 구간만 채운다.
- `layout_paragraph_in_frame()`은 행 높이가 달라지면 cursor와 Frame을 복원해 다시 carve한다.
- 일반 scalar 문단도 지원 범위 안에서는 같은 Frame transaction을 사용한다.

### 표와 그림 band

- 표 셀은 표가 소유한 실제 내용 폭과 padding으로 Frame을 만든다.
- 그림 회피 영역은 host부터 영향받는 마지막 문단까지 하나의 Frame에서 계산한다.
- 문서 편집은 영향받는 그림 band 전체를 shadow 상태에서 계산하고, 성공한 완성 결과만 한 번에 게시한다.
- 열 이동 뒤 geometry가 달라지면 새 열에서 다시 계산하며, 수렴하지 않으면 기존 공개 상태를 유지한다.

RenderTree는 완성된 `LineSeg`만 소비하며 `carve()`나 미확정 행 상태를 소유하지 않는다.

## 5. 검증 계획

핵심 계약은 실제 구현과 같은 모듈의 focused test로 고정한다.

```text
taller_candidate_recarves_before_the_row_is_committed
committed_row_projects_one_complete_lineseg_group_and_advances_once
one_physical_row_projects_each_carved_interval_with_shared_metrics
frame_reflow_projects_two_intervals_as_one_physical_row
frame_reflow_retries_a_taller_row_without_consuming_the_cursor
real_p325_picture_band_matches_the_stored_seven_paragraph_geometry
picture_frame_body_edit_publishes_complete_band_before_recompose
picture_frame_transaction_rejects_shadow_failure_without_publication
```

PR 후보는 다음 저장소 gate를 순차 통과시킨다.

```text
focused LayoutFrame, line-breaking, table-owner와 picture-band tests
release-test 전체 회귀
Native Skia 3종
cargo fmt --all -- --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
rhwp-studio TypeScript와 unit tests
wasm-pack build --target web --out-dir pkg
```

## 6. 범위 밖

- 저장 `LineSeg` cache의 재사용·무효화 정책
- `reflow_linesegs_on_demand()`의 persistence 소유권 변경
- 지원되지 않는 복수 그림 topology의 추정 처리
- Shape, Tight, Through 전체 geometry 지원
- 글꼴 shaping과 kerning에 따른 작은 줄 경계 차이

마지막 항목은 구조적 Frame 정확성과 섞지 않고 #4439에서 별도로 추적한다.
