# Task #429 최종 결과보고서 — 표 셀 배경 image fill 미구현

## 결과 요약

`samples/aift.hwp` 41쪽 표 셀 안 이미지가 SVG 출력에 누락되던 결함 정정. 이미지가 셀 안 paragraph control 이 아닌 **BorderFill 의 fill_type=Image** (셀 배경 그림) 였으나 rhwp 가 이 처리를 미구현.

**본질**: `render_cell_background` 함수가 `border_style.image_fill` 필드를 무시 — 신규 분기 추가로 정정.

## 결함 본질

### 정황

`samples/aift.hwp` 페이지 41 의 표 pi=496 (2x1, RowBreak, TAC):
- 셀[0] r=0, c=0, **bf=175** — `border_fill[175] fill_type=Image image(bin_id=14, mode=None)`
- 셀[0] paragraph p[0] **ctrls=0** (셀 안 control 없음)
- 셀[1] r=1, c=0, bf=1 (Solid white) — 캡션 텍스트

**작업지시자 통찰** (결정적): *"이건 셀의 배경으로 그림을 넣은 것입니다. 우리가 미구현한 것 같습니다."*

### 미구현 영역

`src/renderer/layout/table_layout.rs::render_cell_background` 가 `border_style` 의 `fill_color`, `pattern`, `gradient` 만 처리하고 `image_fill` 무시. `ResolvedBorderStyle::image_fill: Option<ResolvedImageFill>` 필드는 이미 구조에 있음 (style_resolver 단계에서 정상 해소되지만 렌더링 단계에서 사용 안 됨).

### 이미 구현된 비교 영역

| 영역 | 구현 |
|------|------|
| zone (영역 셀, table_layout.rs:303-316) | ✅ image_fill 처리 |
| Shape (Rectangle/Ellipse 등, shape_layout.rs::add_image_fill_node) | ✅ image_fill 처리 |
| 본문 배경 (layout.rs:668) | ✅ image_fill 처리 |
| **일반 표 셀 (render_cell_background)** | ❌ → ✅ (본 정정) |

## 정정 본질

### 1) `render_cell_background` 시그니처 확장 + image_fill 분기 추가

```rust
pub(crate) fn render_cell_background(
    &self,
    tree: &mut PageRenderTree,
    cell_node: &mut RenderNode,
    border_style: Option<&ResolvedBorderStyle>,
    cell_x: f64, cell_y: f64, cell_w: f64, cell_h: f64,
    bin_data_content: &[BinDataContent],  // 추가
) {
    // ... 기존 fill_color / pattern / gradient 처리 (변화 없음)
    
    // [Task #429] image fill 처리 — zone 처리와 동일 패턴
    if let Some(img_fill) = border_style.and_then(|bs| bs.image_fill.as_ref()) {
        if let Some(img_content) = find_bin_data(bin_data_content, img_fill.bin_data_id) {
            let img_node = RenderNode::new(
                tree.next_id(),
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

### 2) zone 의 별도 image_fill 처리 제거 (중복 차단)

`table_layout.rs:302-316` 의 zone image_fill 별도 처리는 `render_cell_background` 가 통합 처리 → 제거.

### 3) 호출부 6개소 정정

| # | 파일 | 라인 | 정정 |
|---|------|------|------|
| 1 | `table_layout.rs` | 264 (표 전체 배경) | `bin_data_content` 인자 추가 |
| 2 | `table_layout.rs` | 303 (zone 배경) | `bin_data_content` 인자 추가 + zone 의 별도 image_fill 처리 제거 |
| 3 | `table_layout.rs` | 1157 (일반 셀) | `bin_data_content` 인자 추가 |
| 4 | `table_partial.rs` | 240 (분할 표 전체 배경) | `bin_data_content` 인자 추가 |
| 5 | `table_partial.rs` | 341 (분할 표 셀) | `bin_data_content` 인자 추가 |
| 6 | `table_cell_content.rs` | 472 (표 배경) | `bin_data_content` 인자 추가 |

## 변경 파일 (3 파일 — 7개소)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | `render_cell_background` 시그니처 확장 + image_fill 분기 + 호출부 3개소 + zone 별도 처리 제거 |
| `src/renderer/layout/table_partial.rs` | 호출부 2개소 |
| `src/renderer/layout/table_cell_content.rs` | 호출부 1개소 |

## Stage 별 결과

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | 베이스라인 측정 | aift 41쪽 image=0 확인, image fill BorderFill 3개 (`bf=174/175/176`) 식별, 호출부 6개소 정합 점검 |
| 2 | 정정 적용 | render_cell_background 정정 + 호출부 6개소 + zone 처리 제거 |
| 3 | 광범위 회귀 검증 | aift 77 페이지 중 2 페이지 변화 (41쪽, 43쪽 모두 정정 의도 영역), 다른 샘플 회귀 0 |
| 4 | 작업지시자 시각 검증 | aift 41쪽 / 43쪽 모두 정상 ✅ |
| 5 | 최종 결과보고서 + 오늘할일 갱신 | 본 문서 |

## 정정 작동 확인

### aift.hwp 41쪽 (본 결함)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| `<image>` 갯수 | **0** | **1** ✅ |
| SVG bytes | 218,179 | 469,459 |

### aift.hwp 43쪽 (다른 셀 배경 image fill 케이스)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| `<image>` 갯수 | 1 | **3** ✅ |

### 광범위 회귀 점검

aift.hwp 77 페이지 중 변화 페이지: **2** (41쪽, 43쪽 — 모두 정정 의도 영역). 75 페이지 byte-equal.

다른 샘플 (synam-001, k-water-rfp, exam_kor, 2022 국립국어원) 의 image fill BorderFill 0개 — 본 정정 영향 영역 없음. SVG 추출 정상.

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| WASM 빌드 | 4,204,760 bytes ✅ |
| 작업지시자 시각 검증 | aift 41쪽 / 43쪽 정상 |

## 작업지시자 통찰 (보존)

본 작업의 결함 본질 식별은 작업지시자의 결정적 통찰:

1. *"디버깅 레이아웃으로 내보내기 해보세요"* — 진단 도구 활용
2. *"원인을 찾았습니다. 이건 셀의 배경으로 그림을 넣은 것입니다"* — **결함 본질 명확화**
3. *"우리가 미구현한 것 같습니다"* — 회귀가 아닌 미구현 기능 추가 작업으로 정합

## 메모리 원칙 정합

- **`feedback_search_troubleshootings_first`**: 트러블슈팅 사전 정독 — `table_reflow_and_cell_rendering.md`, `shape_fill_transparency.md` 정합. 본 결함 영역은 신규.
- **`feedback_visual_regression_grows`**: dump 정보로 본질 (BorderFill image fill) 식별 + 작업지시자 시각 판정 게이트로 확정

## 다음 단계

- 이슈 #429 close
- `local/task429` → `local/devel` 머지
- 오늘할일 갱신
