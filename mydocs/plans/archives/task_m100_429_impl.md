# Task #429 구현 계획서 — render_cell_background 에 image_fill 분기 추가

## 정정 영역

### 1. 함수 시그니처 확장

`src/renderer/layout/table_layout.rs::render_cell_background` (라인 874-903):

```rust
// 정정 후 — bin_data_content 인자 추가
pub(crate) fn render_cell_background(
    &self,
    tree: &mut PageRenderTree,
    cell_node: &mut RenderNode,
    border_style: Option<&crate::renderer::style_resolver::ResolvedBorderStyle>,
    cell_x: f64, cell_y: f64, cell_w: f64, cell_h: f64,
    bin_data_content: &[BinDataContent],  // 추가
) {
    // 기존 fill_color / pattern / gradient 처리 (변화 없음)
    let fill_color = border_style.and_then(|bs| bs.fill_color);
    let pattern = border_style.and_then(|bs| bs.pattern);
    let gradient = border_style.and_then(|bs| bs.gradient.clone());
    if fill_color.is_some() || gradient.is_some() || pattern.is_some() {
        // ... Rectangle 배경 push
    }

    // [Task #429] image fill 처리 — zone 처리 (table_layout.rs:303-316) 와 동일 패턴
    if let Some(img_fill) = border_style.and_then(|bs| bs.image_fill.as_ref()) {
        if let Some(img_content) = crate::renderer::layout::find_bin_data(bin_data_content, img_fill.bin_data_id) {
            let img_id = tree.next_id();
            let img_node = RenderNode::new(
                img_id,
                RenderNodeType::Image(ImageNode {
                    fill_mode: Some(img_fill.fill_mode),
                    ..ImageNode::new(img_fill.bin_data_id, Some(img_content.data.clone()))
                }),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );
            cell_node.children.push(img_node);
        }
    }
}
```

### 2. 호출부 정정 (5개소)

| # | 파일 | 라인 | 정정 |
|---|------|------|------|
| 1 | `table_layout.rs` | 264 (표 전체 배경) | `bin_data_content` 인자 추가 |
| 2 | `table_layout.rs` | 301 (zone 배경) | `bin_data_content` 인자 추가 + **별도 image_fill 처리 (라인 302-316) 제거** (중복 차단) |
| 3 | `table_layout.rs` | 1154 (일반 셀 — **결함 영역**) | `bin_data_content` 인자 추가 |
| 4 | `table_partial.rs` | 240 (분할 표 전체 배경) | `bin_data_content` 인자 추가 |
| 5 | `table_partial.rs` | 340 (분할 표 셀 — **결함 영역**) | `bin_data_content` 인자 추가 |

### 3. zone 의 별도 image_fill 처리 제거

`table_layout.rs:302-316` 의 zone image_fill 처리는 `render_cell_background` 정정 후 중복 — 제거:

```rust
// 제거 대상
// 이미지 채우기
if let Some(ref img_fill) = zone_bs.image_fill {
    if let Some(img_content) = crate::renderer::layout::find_bin_data(bin_data_content, img_fill.bin_data_id) {
        // ... image_node push
    }
}
```

zone 는 `render_cell_background` 정정으로 자동 처리.

## Stage 별 작업

### Stage 1: 베이스라인 측정 + 호출부 정합 점검

```bash
# 베이스라인 SVG 추출 (image=0 확인)
mkdir -p output/svg/task429-baseline
cargo run --release --quiet --bin rhwp -- export-svg samples/aift.hwp -p 40 -o output/svg/task429-baseline/

# 광범위 회귀 baseline (셀 배경 image fill 케이스 점검)
mkdir -p output/svg/task429-baseline-svg
cargo run --release --quiet --bin rhwp -- export-svg samples/aift.hwp -o output/svg/task429-baseline-svg/aift/
```

### Stage 2: 정정 적용

1. `render_cell_background` 시그니처 확장 + image_fill 분기 추가
2. 호출부 5개소 `bin_data_content` 인자 추가
3. zone 의 별도 image_fill 처리 (라인 302-316) 제거

### Stage 3: 회귀 검증

```bash
cargo test --lib
cargo test --test svg_snapshot
cargo test --test issue_418
cargo clippy --lib -- -D warnings

# 정정 결과 SVG (image > 0 확인)
mkdir -p output/svg/task429-test
cargo run --release --quiet --bin rhwp -- export-svg samples/aift.hwp -p 40 -o output/svg/task429-test/

# 광범위 byte 비교 (zone image_fill / 셀 배경 image_fill 회귀 0)
```

### Stage 4: 시각 검증 (작업지시자)

- aift.hwp 41쪽 — 표 셀 배경 그림 정상 표시
- 다른 셀 배경 image fill 케이스 (찾아내기)

### Stage 5: 최종 결과보고서 + 오늘할일 갱신

## 검증 게이트

- `cargo test --lib`: 회귀 0건 (1080 passed 유지)
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1
- `cargo clippy --lib -- -D warnings`: 0건
- WASM 빌드 정상
- 작업지시자 시각 판정 (aift 41쪽 + 다른 셀 배경 케이스)

## 위험 정황 + 회피

- **zone image_fill 회귀 위험**: 정정 후 zone 도 `render_cell_background` 분기를 타므로 — 별도 처리 제거하지 않으면 **이중 출력**. 광범위 byte 비교로 회귀 점검 필수.
- **bin_data_content 빈 배열 케이스**: 호출부에서 `&[]` 가 전달되는 경우 — `find_bin_data` 가 `None` 반환 + 정상 동작 (기존 fill_color 등 처리만 적용).
