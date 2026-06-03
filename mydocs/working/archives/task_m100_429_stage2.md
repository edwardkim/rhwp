# Task #429 Stage 2 — 정정 적용

## 정정 본질

`render_cell_background` 에 image_fill 분기 추가 + zone 의 별도 image_fill 처리 제거 (중복 차단).

## 변경 영역 (3 파일 — 7개소)

| # | 파일 | 변경 |
|---|------|------|
| 1 | `src/renderer/layout/table_layout.rs::render_cell_background` (라인 ~862) | 시그니처 확장 (`bin_data_content` 인자 추가) + image_fill 분기 추가 |
| 2 | `src/renderer/layout/table_layout.rs:264` (표 전체 배경) | `bin_data_content` 인자 추가 |
| 3 | `src/renderer/layout/table_layout.rs:301-316` (zone 배경) | `bin_data_content` 인자 추가 + zone image_fill 별도 처리 제거 |
| 4 | `src/renderer/layout/table_layout.rs:1157` (일반 셀) | `bin_data_content` 인자 추가 |
| 5 | `src/renderer/layout/table_partial.rs:240` (분할 표 전체 배경) | `bin_data_content` 인자 추가 |
| 6 | `src/renderer/layout/table_partial.rs:341` (분할 표 셀) | `bin_data_content` 인자 추가 |
| 7 | `src/renderer/layout/table_cell_content.rs:472` (표 배경) | `bin_data_content` 인자 추가 |

## 정정 코드 (핵심 — render_cell_background image_fill 분기)

```rust
// [Task #429] image fill 처리 — zone 처리와 동일 패턴
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
```

## 정정 작동 확인

### aift.hwp 41쪽

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| `<image>` 갯수 | **0** | **1** ✅ |
| SVG bytes | 218,179 | 469,459 (+251,280) |

### aift.hwp 43쪽 (다른 셀 배경 image fill 케이스)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| `<image>` 갯수 | 1 | **3** ✅ (셀 배경 그림 2개 추가) |

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 다음 단계

Stage 3: 광범위 회귀 검증 — aift.hwp 77 페이지 + 다른 샘플 점검.
