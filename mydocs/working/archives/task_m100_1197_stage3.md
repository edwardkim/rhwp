# Stage 3 완료보고서 - Task M100-1197

- 이슈: #1197
- 단계: Stage 3 - layout paper/page anchored 객체 레이어 전파
- 브랜치: `local/task1197`
- 기준 커밋: `7efb2528`
- 작성일: 2026-06-02

## 1. 변경 요약

`layout.rs`에서 paper/page 기준으로 out-of-flow 수집되는 Picture/Table/Shape 최상위 `RenderNode`에
`RenderLayerInfo`를 부여했다. 이제 SVG 렌더러가 Stage 2에서 확정한 공통 `(plane, z_order, stable_index)`
계약을 실제 문서 레이아웃 결과에서도 사용할 수 있다.

## 2. 구현 내용

### 2-1. layer 생성/정렬 헬퍼 추가

파일:

- `src/renderer/layout.rs`

추가 헬퍼:

- `object_stable_index(para_index, control_index)`
- `render_layer_from_common(common, para_index, control_index)`
- `push_layered_paper_children(paper_images, temp_parent, layer)`
- `sort_paper_render_nodes(paper_images)`

`stable_index`는 문단/컨트롤 인덱스 기반으로 계산하여 같은 z-order에서도 결정적 순서를 유지한다.

### 2-2. paper_images 수집 경로에 layer stamp

대상 경로:

- body 밖으로 렌더되는 paper/page 기준 TopAndBottom Table
- paper/page 기준 non-TAC Picture
- shape pass의 paper/page 기준 Table
- shape pass의 paper/page 기준 Shape

각 경로에서 임시 parent의 최상위 children만 `set_layer(...)` 처리했다.
표 셀/도형 leaf 등 내부 children에는 layer를 재귀 적용하지 않아 내부 렌더 순서에는 영향을 주지 않는다.

### 2-3. root append 전 paper_images 정렬

`paper_images`를 root에 붙이기 전에 `(plane, z_order, stable_index)` 기준으로 정렬한다.
plane 의미는 SVG와 동일하다.

- `BehindText`: 1
- 일반 flow wrap: 2
- `InFrontOfText`: 3

Stage 3에서는 레이아웃 트리에 공통 메타데이터와 안정 정렬을 전파했다. PaintOp/CanvasKit 재생 경로 반영은 Stage 4 범위로 남긴다.

### 2-4. 레이아웃 정렬 단위 테스트 추가

파일:

- `src/renderer/layout/tests.rs`

추가 테스트:

- `task1197_paper_nodes_sort_by_plane_z_order_and_stable_index`

검증 내용:

- `BehindText`가 flow/front plane보다 먼저 정렬된다.
- 같은 plane에서는 `z_order`가 우선한다.
- 같은 `z_order`에서는 `stable_index`가 tie-breaker로 동작한다.

## 3. 검증 결과

### Stage 3 신규 단위 테스트

```sh
cargo test --lib task1197_paper_nodes_sort_by_plane_z_order_and_stable_index -- --nocapture
```

결과:

```text
1 passed; 0 failed
```

### #1197 synthetic z-order 테스트

```sh
cargo test --test issue_1197_svg_object_zorder -- --nocapture
```

결과:

```text
1 passed; 0 failed
```

### #1167 BehindText 워터마크 회귀

```sh
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
```

결과:

```text
1 passed; 0 failed
```

비고: 기존 테스트 경로에서 `LAYOUT_OVERFLOW` 진단 1건이 출력되지만 assertion은 통과했다.

### render_tree 단위 테스트

```sh
cargo test --lib render_tree -- --nocapture
```

결과:

```text
21 passed; 0 failed
```

### PaintOp replay plane 기존 계약

```sh
cargo test --lib replay_order
```

결과:

```text
5 passed; 0 failed
```

### 포맷/공백

```sh
cargo fmt --all --check
git diff --check
```

결과: 통과.

## 4. 남은 작업

Stage 4에서 PaintOp/CanvasKit 경로가 `RenderNode.layer` 기반의 공통 z-order 계약을 소비하도록 확장해야 한다.
현재 Stage 3까지는 SVG 렌더링과 레이아웃 트리 메타데이터 전파가 완료된 상태다.

## 5. 다음 단계

Stage 4 승인 후 PaintOp/CanvasKit replay 경로의 Picture/Table/Shape 공통 z-order 소비를 구현한다.
