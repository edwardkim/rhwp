# Stage 4 완료보고서 - Task M100-1197

- 이슈: #1197
- 단계: Stage 4 - PaintOp/CanvasKit replay 경로 레이어 소비
- 브랜치: `local/task1197`
- 기준 커밋: `1628058d`
- 작성일: 2026-06-02

## 1. 변경 요약

Stage 3에서 `RenderNode.layer`로 전달한 paper/page anchored 객체 레이어 정보를
`PageLayerTree`와 PaintOp replay 경로가 소비하도록 확장했다. 이제 이미지뿐 아니라
Table/Shape에서 내려온 vector/text leaf도 상위 `LayerNode.layer`를 상속받아
`BehindText` / flow / `InFrontOfText` replay plane에 배치된다.

## 2. 구현 내용

### 2-1. LayerNode에 layer 메타데이터 보존

파일:

- `src/paint/layer_tree.rs`
- `src/paint/builder.rs`

`LayerNode`에 `layer: Option<RenderLayerInfo>`를 추가했다.
`LayerBuilder`는 모든 RenderNode lowering 경로에서 `RenderNode.layer`를 `LayerNode.layer`로 복사한다.

### 2-2. PaintOp replay plane 판정 확장

파일:

- `src/paint/replay_order.rs`
- `src/paint/mod.rs`

추가 API:

- `paint_op_replay_plane_with_layer(op, layer)`
- `render_layer_replay_plane(layer)`

판정 순서:

1. `PageBackground`는 항상 background plane
2. `layer.text_wrap`이 있으면 해당 layer plane 우선
3. 없으면 기존 `ImageNode.text_wrap` fallback 유지
4. 그 외 op는 flow plane

### 2-3. CanvasKit/Skia replay 경로 적용

파일:

- `src/renderer/canvaskit_policy.rs`
- `src/renderer/skia/renderer.rs`

CanvasKit replay plan traversal과 native Skia raster traversal이 `LayerNode.layer`를 inherited state로 전달한다.
leaf op의 replay plane은 `paint_op_replay_plane_with_layer`로 판정한다.

### 2-4. WebCanvas layer filter 적용

파일:

- `src/renderer/web_canvas.rs`

기존 layer filter는 `ImageNode.text_wrap`만 확인했다. Stage 4부터는 `LayerNode.layer` 상속값을 포함한
PaintOp replay plane 기준으로 필터링한다.

의미:

- flow canvas는 BehindText/InFrontOfText table/shape op를 제외한다.
- behind/front overlay canvas는 이미지뿐 아니라 table/shape op도 포함할 수 있다.
- BehindText layer 존재 여부 판정도 이미지 전용에서 replay plane 기준으로 확장했다.

### 2-5. JSON export 보강

파일:

- `src/paint/json.rs`

`PageLayerTree::to_json()`에서 `LayerNode.layer`를 다음 형태로 직렬화한다.

```json
{"textWrap":"behindText","zOrder":7,"stableIndex":42}
```

브라우저 CanvasKit/Canvas2D replay 소비자가 Rust native와 같은 layer 메타데이터를 받을 수 있게 했다.

## 3. 테스트 추가/갱신

추가 테스트:

- `paint::builder::tests::copies_render_layer_metadata_to_layer_node`
- `paint::replay_order::tests::render_layer_metadata_overrides_non_image_paint_ops`
- `paint::json::tests::serializes_layer_node_metadata`
- `renderer::canvaskit_policy::tests::replay_plan_uses_layer_metadata_for_non_image_ops`
- `renderer::skia::renderer::tests::behind_text_layered_vector_replays_below_flow_across_tree_branches`

Skia 테스트는 `native-skia` feature에서 실행된다.

## 4. 검증 결과

### 신규/관련 단위 테스트

```sh
cargo test --lib replay_order
cargo test --lib copies_render_layer_metadata_to_layer_node -- --nocapture
cargo test --lib serializes_layer_node_metadata -- --nocapture
cargo test --lib replay_plan_uses_layer_metadata_for_non_image_ops -- --nocapture
cargo test --lib replay_plan_items_expose_paint_replay_planes -- --nocapture
```

결과: 모두 통과.

### native Skia feature 테스트

```sh
cargo test --features native-skia --lib behind_text_layered_vector_replays_below_flow_across_tree_branches -- --nocapture
```

결과:

```text
1 passed; 0 failed
```

### #1197 / #1167 회귀

```sh
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --lib task1197_paper_nodes_sort_by_plane_z_order_and_stable_index -- --nocapture
```

결과: 모두 통과.

비고: #1167 테스트는 기존과 동일하게 `LAYOUT_OVERFLOW` 진단 1건을 출력하지만 assertion은 통과했다.

### 공용 렌더 트리/포맷

```sh
cargo test --lib render_tree -- --nocapture
cargo fmt --all --check
git diff --check
```

결과: 모두 통과.

## 5. 남은 작업

Stage 4까지로 SVG, layout, PaintOp/CanvasKit/Skia layer 소비 계약은 연결됐다.
다음 단계에서는 최종 통합 검증과 작업 보고/정리 범위를 진행한다.

## 6. 다음 단계

Stage 5 승인 후 전체 변경 범위 리뷰, 최종 보고서 작성, 필요 시 PR 준비 단계를 진행한다.
