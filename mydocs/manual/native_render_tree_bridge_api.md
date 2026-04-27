# Native Render Tree Bridge API

## 목적

native viewer/renderer가 WASM API를 거치지 않고 Rust core에서 페이지 단위 render tree를 직접 가져오기 위한 API를 정리한다.

이번 API는 새 렌더러를 추가하지 않는다. 이미 존재하는 `PageRenderTree` 생성 경로를 `DocumentCore` public API로 노출한다.

## API

```rust
pub fn build_page_render_tree(
    &self,
    page_num: u32,
) -> Result<PageRenderTree, HwpError>
```

- `page_num`은 0-based page index다.
- 내부 `build_page_tree(page_num)` 경로를 사용한다.
- 반환 타입은 `rhwp::renderer::render_tree::PageRenderTree`다.
- native bridge는 이 값을 직접 순회하거나 `serde` 기반으로 JSON 직렬화해 platform model로 변환할 수 있다.

```rust
pub fn get_bin_data(
    &self,
    index: usize,
) -> Option<&[u8]>
```

- `index`는 0-based `Document.bin_data_content` 배열 인덱스다.
- 반환값은 원본 이미지/OLE binary bytes에 대한 slice다.
- 범위 밖 index는 `None`을 반환한다.

## 이미지 조회 기준

`ImageNode.bin_data_id`는 1-based 참조값이다.

따라서 render tree의 이미지 노드에서 실제 bytes를 가져올 때는 다음 기준을 사용한다.

```rust
let bin_data_id = image_node.bin_data_id;

let data = if bin_data_id == 0 {
    None
} else {
    document.get_bin_data((bin_data_id - 1) as usize)
};
```

`bin_data_id`와 storage id는 별개다. storage id가 비순차인 문서에서는 storage id로 직접 배열 접근을 하면 이미지가 잘못 매핑될 수 있다.

## 직렬화 기준

`PageRenderTree`와 주요 render node 타입은 `serde::Serialize`를 구현한다.

단, render tree JSON에 binary payload를 직접 포함하지 않기 위해 다음 필드는 직렬화에서 제외한다.

- `ImageNode.data`
- `PageBackgroundImage.data`

이미지 bytes는 render tree의 `bin_data_id`를 기준으로 `get_bin_data(index)`에서 별도 조회한다.

## WASM 경로와의 관계

WASM의 `getPageRenderTree(pageNum)`은 기존 수동 JSON 경로를 유지한다.

native bridge API는 Rust core 사용자용이며, SVG/HTML/Canvas/WASM 렌더링 경로의 동작을 변경하지 않는다.
