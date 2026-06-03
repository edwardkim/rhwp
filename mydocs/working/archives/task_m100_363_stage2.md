# Task M100 #363 Stage 2 완료보고서: PageRenderTree 직렬화 지원

## 1. 작업 범위

구현계획서의 Stage 2 범위에 따라 native bridge에서 `PageRenderTree`와 그 하위 render node graph를 `serde` 기반으로 직렬화할 수 있도록 정리했다.

## 2. 변경 파일

- `Cargo.toml`
- `src/renderer/render_tree.rs`
- `src/renderer/mod.rs`
- `src/renderer/composer.rs`
- `src/renderer/layout.rs`
- `src/renderer/equation/ast.rs`
- `src/renderer/equation/layout.rs`
- `src/renderer/equation/symbols.rs`
- `src/model/control.rs`
- `src/model/image.rs`
- `src/model/style.rs`

## 3. 구현 내용

### 3.1 `serde` derive 의존성 추가

`Cargo.toml`에 다음 의존성을 추가했다.

```toml
serde = { version = "1", features = ["derive"] }
```

### 3.2 render tree graph 직렬화

다음 주요 타입에 `Serialize` derive를 추가했다.

- `PageRenderTree`
- `RenderNode`, `RenderNodeType`
- `BoundingBox`
- 페이지/본문/텍스트/표/도형/이미지/수식/양식 개체 노드
- render tree node가 참조하는 style, equation layout, cell context 보조 타입

### 3.3 이미지 bytes 제외

render tree JSON에 이미지 원본 bytes를 직접 포함하지 않도록 다음 필드는 `#[serde(skip)]` 처리했다.

- `ImageNode.data`
- `PageBackgroundImage.data`

`PageRenderTree` 내부 상태인 `next_id`, `inline_shape_positions`도 public bridge payload가 아니므로 `#[serde(skip)]` 처리했다.

## 4. 검증

```bash
cargo build
cargo test test_page_render_tree
```

결과:

- `cargo build` 통과
- `cargo test test_page_render_tree` 통과
- 기존 테스트 경고 4건이 출력되었으나 이번 변경과 무관한 기존 경고다.

## 5. 영향 범위

- 기존 WASM `getPageRenderTree()`의 수동 JSON 문자열 생성 경로는 변경하지 않았다.
- 기존 SVG/HTML/Canvas 렌더러는 동일한 render tree 타입을 계속 사용한다.
- 이미지 데이터는 별도 `get_bin_data(index)` 조회 API로 가져오는 구조를 유지한다.

## 6. 다음 단계

Stage 3에서 public API 동작 테스트를 추가한다.
