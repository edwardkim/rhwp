# Task M100 #363 최종 보고서: 기존 PageRenderTree 구현의 native bridge API 노출

## 1. 목표

native bridge에서 WASM API를 거치지 않고 Rust core의 기존 `PageRenderTree` 생성 경로를 사용할 수 있도록 public API를 정리했다.

## 2. 완료 내용

### 2.1 public API 추가

`DocumentCore`에 다음 API를 추가했다.

```rust
pub fn build_page_render_tree(
    &self,
    page_num: u32,
) -> Result<PageRenderTree, HwpError>

pub fn get_bin_data(
    &self,
    index: usize,
) -> Option<&[u8]>
```

`build_page_render_tree()`는 기존 내부 `build_page_tree()` 경로를 감싼다. 새 render tree 구조나 별도 renderer는 추가하지 않았다.

### 2.2 `PageRenderTree` 직렬화 지원

native bridge가 render tree를 JSON 등으로 변환할 수 있도록 `serde::Serialize` derive를 정리했다.

직렬화 대상:

- `PageRenderTree`
- `RenderNode`, `RenderNodeType`
- 텍스트/표/도형/이미지/수식/양식 개체 노드
- render node가 참조하는 style, equation layout, cell context 보조 타입

직렬화 제외:

- `ImageNode.data`
- `PageBackgroundImage.data`
- `PageRenderTree.next_id`
- `PageRenderTree.inline_shape_positions`

이미지 bytes는 render tree JSON에 직접 포함하지 않고 `get_bin_data(index)`로 별도 조회한다.

### 2.3 API 테스트 추가

다음 테스트를 추가했다.

- `build_page_render_tree_exposes_public_page_tree`
- `get_bin_data_returns_zero_based_content_slice`

### 2.4 문서화

다음 문서를 추가/보강했다.

- `mydocs/manual/native_render_tree_bridge_api.md`
- `mydocs/tech/hwp_spec_errata.md`

핵심 기준:

- `ImageNode.bin_data_id`는 1-based 참조값
- `DocumentCore::get_bin_data(index)`는 0-based `bin_data_content` 배열 인덱스
- render tree 이미지 노드에서 bytes 조회 시 일반적으로 `get_bin_data((bin_data_id - 1) as usize)` 사용

## 3. 검증

최종 검증 명령:

```bash
cargo build
cargo test
```

결과:

- `cargo build`: 통과
- `cargo test`: 통과
  - lib tests: 1010 passed, 1 ignored
  - integration tests: `hwpx_roundtrip_integration` 14 passed, `hwpx_to_hwp_adapter` 25 passed, `issue_301` 1 passed, `svg_snapshot` 6 passed, `tab_cross_run` 1 passed
  - doctest: 0 tests

기존 테스트 경고 4건이 출력되었으나 이번 변경과 무관한 기존 경고다.

## 4. 영향 범위

- 기존 WASM `getPageRenderTree()`의 수동 JSON 경로는 변경하지 않았다.
- 기존 SVG/HTML/Canvas 렌더링 경로는 변경하지 않았다.
- 이미지 binary payload를 render tree JSON에 포함하지 않아 payload 크기 증가를 피했다.

## 5. 커밋

- `b7eba16 Task #363: Stage 1 native render tree API`
- `f281290 Task #363: Stage 2 serialize render tree`
- `d11de91 Task #363: Stage 3 test native bridge APIs`
- Stage 4: 문서화 및 최종 검증

## 6. 남은 작업

- 작업지시자 승인 후 fork branch push
- upstream `devel` 대상 PR 생성
