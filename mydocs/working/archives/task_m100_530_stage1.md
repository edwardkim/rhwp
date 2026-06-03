# Task #530 Stage 1 완료보고서 — TAC 표 Top caption 겹침 원인 확정

## 대상

- **Issue**: [#530](https://github.com/edwardkim/rhwp/issues/530)
- **브랜치**: `local/task530`
- **기준 커밋**: `ce98bf5`
- **샘플**: `samples/basic/treatise sample.hwp`
- **문제 위치**: page=5, global_idx=4, section=0, column=1, `Table pi=60 ci=0`

## 실행 명령

```bash
cargo run --quiet --bin rhwp -- dump-pages "samples/basic/treatise sample.hwp" -p 4
cargo run --quiet --bin rhwp -- dump "samples/basic/treatise sample.hwp" -s 0 -p 60
cargo run --quiet --bin rhwp -- export-svg "samples/basic/treatise sample.hwp" -p 4 --debug-overlay -o output/debug
```

## 재현 결과

`dump-pages` 에서 이슈와 동일한 표 위치를 확인했다.

```text
Table pi=60 ci=0  3x3  283.9x61.7px  wrap=TopAndBottom tac=true  vpos=27708
```

`dump` 에서 머리행 셀은 2개 문단을 가지고 있으며, 두 문단의 line segment 위치가 서로 다르다.

```text
셀[0] text="시스템|SecParam"
  p[0] vpos=0    lh=900 ls=180
  p[1] vpos=1080 lh=900 ls=180

셀[1] text="라우터 시스템|(P3-870MHz)"
  p[0] vpos=0    lh=900 ls=180
  p[1] vpos=1080 lh=900 ls=180

셀[2] text="일반노드 시스템1|(P3-696MHz)"
  p[0] vpos=0    lh=900 ls=180
  p[1] vpos=1080 lh=900 ls=180
```

따라서 셀 내부 두 문단의 y advance 자체가 0 으로 붕괴한 상태는 아니다.

## SVG 좌표 증거

생성 파일:

```text
output/debug/treatise sample_005.svg
```

첫 번째 머리행 셀 clip rect:

```text
x=431.97333333333336
y=525.4933333333336
w=88.33333333333333
h=30.16
```

첫 번째 머리행 셀 텍스트:

```text
시스템   y=537.5733333333336
SecParam y=551.9733333333336
```

같은 표의 Top caption:

```text
표 1. CGA 생성시간              y=535.6933333333336
Table 1. Generation times...   y=551.3200000000003
```

caption 두 줄이 머리행 두 줄과 거의 같은 y 영역에 렌더링된다. 즉, 시각 증상은 **머리행 셀 텍스트끼리의 중첩**이 아니라 **Top caption 이 머리행 위에 덮여 그려지는 중첩**이다.

## 코드 경로 확인

### `src/renderer/layout.rs`

`treat_as_char=true` 표는 `tree.get_inline_shape_position(...)` 에서 paragraph layout 이 계산한 inline position 을 가져온다. 해당 position 이 있으면 `layout_table(...)` 호출 시 `inline_x_override` 가 전달되고, `y_start` 도 inline position 의 y 값으로 들어간다.

관련 경로:

```rust
let inline_pos = if is_tac {
    tree.get_inline_shape_position(page_content.section_index, para_index, control_index)
} else {
    None
};

let tbl_inline_x = if let Some((ix, _)) = inline_pos {
    Some(ix)
...
let table_y_start = if let Some((_, iy)) = inline_pos { iy } else { y_offset };
y_offset = self.layout_table(... table_y_start, ... tbl_inline_x, ...);
```

### `src/renderer/layout/paragraph_layout.rs`

paragraph layout 은 TAC 표의 inline 좌표를 계산하고 별도 `PageItem::Table` 중복 렌더링을 막기 위해 위치를 등록한다.

```rust
let table_y = (y + baseline + om_bottom - table_h).max(y);
self.layout_table(... table_y, ... Some(x), ...);
tree.set_inline_shape_position(section_index, para_index, tac_ci, x, table_y);
```

이 `table_y` 는 표 본문 자체의 기준 y 로 계산되며, Top caption 높이만큼 표 본문을 아래로 내리는 처리가 없다.

### `src/renderer/layout/table_layout.rs`

`layout_table` 은 `inline_x_override.is_some()` 인 경우 y 위치 계산을 우회한다.

```rust
// inline_x_override가 있으면 외부에서 이미 위치를 계산했으므로 y_start 그대로 사용
let table_y = if inline_x_override.is_some() {
    y_start
} else {
    self.compute_table_y_position(...)
};
```

반면 일반 경로의 `compute_table_y_position()` 은 Top caption 일 때 표 본문 y 에 caption offset 을 적용한다.

```rust
if matches!(caption.direction, CaptionDirection::Top) {
    y_start + caption_height + caption_spacing + v_offset
}
```

caption 자체는 Top caption 일 때 `y_start` 에 렌더링된다.

```rust
CaptionDirection::Top => (table_x, table_width, y_start)
```

결과적으로 TAC 표 + Top caption 조합에서는 다음 상태가 된다.

| 요소 | 현재 y 기준 |
|------|-------------|
| Top caption | `y_start` |
| 표 본문 | `y_start` |

이 때문에 caption 과 머리행이 같은 y 영역에 겹친다.

## 비교 선례

`src/renderer/layout/table_partial.rs` 의 partial table 경로는 Top caption 이 있을 때 표 본문 y 를 명시적으로 내린다.

```rust
let table_y = if render_top_caption {
    y_start + caption_height + caption_spacing
} else {
    y_start
};
```

그림/도형 caption 경로도 Top caption 이 있는 객체 본문을 caption 아래로 내리는 구조를 사용한다. 따라서 #530 의 정정 방향은 기존 코드베이스 선례와도 일치한다.

## Stage 1 결론

원인은 **TAC 표의 inline 경로에서 Top caption offset 을 표 본문 y 에 적용하지 않는 것**으로 확정한다.

정정 방향은 다음 단계에서 구현 계획서로 확정한다.

1. `inline_x_override.is_some()` 경로에서도 Top caption 일 때 `caption_height + caption_spacing` 을 표 본문 y 에 적용한다.
2. caption 렌더링 y 는 기존처럼 `y_start` 로 유지한다.
3. 반환 y 는 기존 `table_height + caption_extra` 기반과 정합하도록 유지한다.
4. 회귀 테스트는 `samples/basic/treatise sample.hwp` page 5 표에서 caption y 와 표 본문/header y 가 같은 영역에 겹치지 않는지 확인하는 방식으로 추가한다.

## 다음 단계

작업지시자 승인 후 `mydocs/plans/task_m100_530_impl.md` 구현 계획서를 작성한다.
