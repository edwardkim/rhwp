# Stage 2 완료보고서 - Task M100-1197

- 이슈: #1197
- 단계: Stage 2 - 렌더 레이어 메타데이터 계약 확정
- 브랜치: `local/task1197`
- 기준 커밋: `9b330e71`
- 작성일: 2026-06-02

## 1. 변경 요약

Picture/Table/Shape가 같은 z-order 축에서 렌더될 수 있도록 RenderNode 공통 레이어 메타데이터를 추가했다.
이번 단계는 실제 `layout.rs`의 용지 기준 객체 수집 경로에 메타데이터를 전파하는 단계가 아니라,
렌더 트리와 SVG 렌더러가 공통 계약을 표현하고 해석할 수 있게 만드는 단계다.

## 2. 구현 내용

### 2-1. RenderNode 공통 layer 메타데이터 추가

파일:

- `src/renderer/render_tree.rs`

추가:

```rust
pub struct RenderLayerInfo {
    pub text_wrap: Option<TextWrap>,
    pub z_order: i32,
    pub stable_index: u32,
}
```

`RenderNode`에 `layer: Option<RenderLayerInfo>`를 추가했다.

- 기본값은 `None`
- 일반 본문/표 셀/텍스트 노드는 기존 동작 유지
- paper/page anchored Picture/Table/Shape 노드만 이후 단계에서 `Some`으로 stamp 예정
- serde 직렬화에서는 `None`일 때 생략

편의 API:

- `RenderNode::with_layer(...)`
- `RenderNode::set_layer(...)`

### 2-2. SVG plane 판정이 layer를 우선 사용

파일:

- `src/renderer/svg.rs`

변경:

- `node_z_plane()`이 `RenderNode.layer.text_wrap`을 우선 확인한다.
- layer가 없으면 기존 #1167 호환 경로대로 `ImageNode.text_wrap` fallback을 사용한다.
- 정렬 키를 `(plane, z_order, stable_index)`로 확장했다.

의미:

- `BehindText` 표/도형도 layer가 부여되면 이미지와 같은 plane에서 정렬 가능하다.
- `ImageNode.text_wrap`만 보던 #1167 기존 동작은 유지된다.

### 2-3. #1197 테스트를 계약 검증으로 갱신

파일:

- `tests/issue_1197_svg_object_zorder.rs`

변경:

- synthetic tree의 표/그림/front shape에 `RenderLayerInfo`를 부여했다.
- root 삽입 순서는 일부러 `image -> low table`로 둔다.
- SVG 출력은 layer의 `(plane, z_order)` 기준으로 `low table -> image -> final table -> front shape`가 되어야 한다.

## 3. 검증 결과

### #1197 synthetic z-order 테스트

```sh
cargo test --test issue_1197_svg_object_zorder -- --nocapture
```

결과:

```text
1 passed; 0 failed
```

Stage 1 RED 실패(`image offset 310 before low table offset 332`)가 Stage 2 계약 도입으로 GREEN 전환됐다.

### #1167 BehindText 워터마크 회귀

```sh
cargo test --test issue_1167_svg_behindtext_zorder
```

결과:

```text
1 passed; 0 failed
```

이미지 단독 `ImageNode.text_wrap` fallback은 유지된다.

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

Stage 2는 렌더 트리/렌더러 계약만 확정했다. 실제 문서 렌더링에서 #1197이 고쳐지려면
Stage 3에서 `layout.rs`의 용지/페이지 기준 Picture/Table/Shape 생성 경로가 `RenderLayerInfo`를
부여해야 한다.

특히 다음 경로가 Stage 3 대상이다.

- paper/page based Picture
- paper/page based Table
- Rectangle/Path/Ellipse/Line/TextBox 등 Shape leaf 또는 group
- `paper_images` 버킷 root append 전 정렬

## 5. 다음 단계

Stage 3 승인 후 `layout.rs`의 paper/page anchored 객체 수집/정렬 경로에 `RenderLayerInfo`를 전파한다.
