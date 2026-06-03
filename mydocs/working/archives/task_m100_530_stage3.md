# Task #530 Stage 3 완료보고서 — TAC Top caption 표 본문 y 오프셋 정정

## 대상

- **Issue**: [#530](https://github.com/edwardkim/rhwp/issues/530)
- **브랜치**: `local/task530`
- **수정 범위**:
  - `src/renderer/layout/table_layout.rs`
  - `tests/issue_530.rs`

## Red 테스트

신규 회귀 테스트를 먼저 추가했다.

```bash
cargo test --test issue_530
```

수정 전 실패:

```text
issue #530 table_top=525.49, max_caption_baseline=551.32
TAC Top caption 이 표 머리행 위에 겹침
```

이는 Stage 1 분석과 동일하게 표 본문 top 이 caption 둘째 줄 baseline 보다 위에 있음을 증명한다.

## 구현 내용

`layout_table` 의 `inline_x_override.is_some()` 경로에서 Top caption offset 을 표 본문 y 에 반영했다.

기존:

```rust
let table_y = if inline_x_override.is_some() {
    y_start
} else {
    self.compute_table_y_position(...)
};
```

정정:

```rust
let inline_top_caption_offset = if inline_x_override.is_some() && depth == 0 {
    if let Some(ref caption) = table.caption {
        use crate::model::shape::CaptionDirection;
        if matches!(caption.direction, CaptionDirection::Top) {
            caption_height + caption_spacing
        } else {
            0.0
        }
    } else {
        0.0
    }
} else {
    0.0
};

let table_y = if inline_x_override.is_some() {
    y_start + inline_top_caption_offset
} else {
    self.compute_table_y_position(...)
};
```

정책:

- `inline_x_override` 의 x/y anchor 의미는 유지한다.
- Top caption 이 있는 경우에만 표 본문 y 를 아래로 내린다.
- caption 자체는 기존처럼 `y_start` 에 렌더링한다.
- Bottom/Left/Right caption 과 caption 없는 표는 변경하지 않는다.
- 반환 y 계산은 기존 `table_height + caption_extra` 정책을 유지한다.

## Green 테스트

```bash
cargo test --test issue_530 -- --nocapture
```

결과:

```text
test issue_530_tac_top_caption_does_not_overlap_header_row ... ok

issue #530 table_top=560.83, max_caption_baseline=551.32
```

정정 후 표 본문 top 이 caption 둘째 줄 baseline 아래로 이동했다.

## SVG 좌표 확인

명령:

```bash
cargo run --quiet --bin rhwp -- export-svg "samples/basic/treatise sample.hwp" -p 4 --debug-overlay -o output/debug
```

생성 파일:

```text
output/debug/treatise sample_005.svg
```

정정 후 좌표:

```text
cell-clip-233 rect y=560.8266666666669
Top caption 1줄 "표" y=535.6933333333336
Top caption 2줄 "T"  y=551.3200000000003
```

정정 전/후 비교:

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| 표 머리행 top | 525.49 | 560.83 |
| caption 둘째 줄 baseline | 551.32 | 551.32 |
| 판정 | 겹침 | 분리 |

## Stage 3 결론

TAC Top caption 표에서 caption 이 머리행 위에 덮이는 직접 원인을 정정했다. 신규 회귀 테스트는 Red → Green 으로 전환되었고, SVG 좌표에서도 caption 과 표 본문이 분리됨을 확인했다.

## 다음 단계

작업지시자 승인 후 Stage 4 회귀 검증을 진행한다.
