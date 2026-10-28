# Task #1280 (v2) 1단계 완료보고서 — 레이아웃 쿼리에 plane/zOrder/stableIndex 노출

## 목표

프런트 히트테스트가 겹침 시 "최상단 개체"를 선택할 수 있도록, Rust 레이아웃 쿼리
(`get_page_control_layout_native`)가 컨트롤별 `plane/zOrder/stableIndex`를 노출한다.
값은 **렌더 정렬키와 단일 진실 원천**이어야 한다.

## 변경 내용

### 1. `paper_node_sort_key` 를 `pub(crate)` 로 노출

**파일**: `src/renderer/layout.rs:775`

렌더 종이 노드 정렬키 `(plane, z_order, stable_index)`를 산출하는 `LayoutEngine::paper_node_sort_key`를
`pub(crate)`로 변경하여 레이아웃 쿼리에서 재사용 가능하게 했다. plane 계산(`render_layer_plane`:
BehindText→1, InFrontOfText→3, 그 외→2)과 layer 부재(inline) 시 폴백 `(2, 0, node.id)`까지 그대로
공유 — 렌더 정렬과 100% 동일.

### 2. `collect_controls` 가 컨트롤별 layer 필드 방출

**파일**: `src/document_core/queries/rendering.rs` `get_page_control_layout_native`

`collect_controls` 진입부에서 `LayoutEngine::paper_node_sort_key(node)`로 `(plane, zOrder, stableIndex)`를
구해 `layer_str`를 만들고, **9개 컨트롤 방출 지점 전부**(table/equation/image/group/shape(rectangle)/
shape(ellipse)/line/path-line/path-shape)에 부착했다. `wrap`만 이미지에 있던 것과 달리 plane/zOrder/
stableIndex는 모든 타입에 일관 노출된다(히트테스트 plane 비교 일관성).

## 검증

```
cargo test --lib task1280_v2_control_layout    → 1 passed
cargo test --lib control_layout                → 4 passed (기존 3 + 신규 1, 회귀 0)
cargo test --lib task1197_paper_nodes_sort     → 1 passed (정렬키 회귀 0)
cargo test --lib                               → 1581 passed; 0 failed; 6 ignored
rustfmt --check (rendering.rs, layout.rs)       → clean
cargo clippy --lib                              → 변경 코드 경고 0
```

### 신규 Rust 단위 테스트

`task1280_v2_control_layout_exposes_plane_z_order_stable_index`
(`src/document_core/queries/rendering.rs` 테스트 모듈):

- 한컴 권위 샘플 `samples/textbox-under-image.hwp` 로드 → `get_page_control_layout_native(0)`.
- `plane`/`zOrder`/`stableIndex` 필드 노출 단언.
- **글상자(shape)=InFrontOfText plane 3, 이미지=Square plane 2** 단언 → `image_plane < shape_plane`
  (이미지가 글상자 뒤 = 한컴 정합)의 근거 고정.

## 다음 단계

Stage 2 — 프런트 `ControlLayoutItem` 타입에 `plane/zOrder/stableIndex/wrap` 필드 추가(동작 무변화).

## 승인 대기

본 보고서와 소스 커밋 후 승인 요청. 승인 후 Stage 2 진행.
