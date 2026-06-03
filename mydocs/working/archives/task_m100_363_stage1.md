# Task M100 #363 Stage 1 완료보고서: native bridge public API 추가

## 1. 작업 범위

구현계획서의 Stage 1 범위에 따라 `DocumentCore`에 native bridge용 public API를 추가했다.

## 2. 변경 파일

- `src/document_core/queries/rendering.rs`

## 3. 구현 내용

### 3.1 `DocumentCore::build_page_render_tree`

```rust
pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError>
```

- 기존 내부 구현인 `build_page_tree(page_num)`를 호출한다.
- 기존 SVG/HTML/Canvas native 렌더링 API와 동일하게 `layout_engine.take_overflows()`를 호출해 overflow 상태를 정리한다.
- 새 render tree 구조나 별도 layout 경로는 추가하지 않았다.

### 3.2 `DocumentCore::get_bin_data`

```rust
pub fn get_bin_data(&self, index: usize) -> Option<&[u8]>
```

- `document.bin_data_content[index].data.as_slice()`를 반환한다.
- 이 API의 `index`는 0-based `bin_data_content` 배열 인덱스다.
- `ImageNode.bin_data_id`의 1-based 참조값과의 관계는 Stage 4 문서화에서 명확히 정리한다.

## 4. 검증

```bash
cargo build
```

결과:

- 통과
- 최초 sandbox 실행은 crates.io DNS 제한으로 실패했다.
- 승인된 네트워크 실행에서 의존성 다운로드 후 `dev` profile 빌드가 성공했다.

## 5. 영향 범위

- 기존 WASM `getPageRenderTree()` 경로는 변경하지 않았다.
- 기존 SVG/HTML/Canvas 렌더링 경로는 변경하지 않았다.
- 이번 단계에서는 직렬화 derive와 문서화는 아직 적용하지 않았다.

## 6. 다음 단계

Stage 2에서 `PageRenderTree` 직렬화를 위한 최소 `serde::Serialize` derive를 정리한다.
