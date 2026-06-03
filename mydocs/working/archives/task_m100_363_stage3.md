# Task M100 #363 Stage 3 완료보고서: native bridge API 테스트 추가

## 1. 작업 범위

구현계획서의 Stage 3 범위에 따라 `DocumentCore` public native bridge API의 기본 동작을 검증하는 단위 테스트를 추가했다.

## 2. 변경 파일

- `src/document_core/queries/rendering.rs`

## 3. 구현 내용

### 3.1 `build_page_render_tree` 테스트

테스트명:

```rust
build_page_render_tree_exposes_public_page_tree
```

검증 내용:

- `DocumentCore::new_empty()` 문서를 페이지네이션한다.
- `build_page_render_tree(0)` 호출이 성공하는지 확인한다.
- 반환된 tree root가 `RenderNodeType::Page`이고 page index가 0인지 확인한다.

빈 문서의 기본 page size는 0일 수 있으므로, 이번 테스트는 public API 호출 가능성과 루트 노드 타입에 집중했다.

### 3.2 `get_bin_data` 테스트

테스트명:

```rust
get_bin_data_returns_zero_based_content_slice
```

검증 내용:

- synthetic `BinDataContent` 2개를 `document.bin_data_content`에 추가한다.
- `get_bin_data(0)`과 `get_bin_data(1)`이 각각 대응하는 byte slice를 반환하는지 확인한다.
- 범위 밖 index는 `None`을 반환하는지 확인한다.

## 4. 검증

```bash
cargo test build_page_render_tree_exposes_public_page_tree
cargo test get_bin_data_returns_zero_based_content_slice
cargo build
```

결과:

- 모두 통과
- 기존 테스트 경고 4건이 출력되었으나 이번 변경과 무관한 기존 경고다.

## 5. 특이사항

처음 작성한 `build_page_render_tree` 테스트는 빈 문서 page width/height가 0보다 크다고 가정했으나, 빈 문서 기본 layout에서는 0일 수 있어 assertion을 제거했다.

## 6. 다음 단계

Stage 4에서 `bin_data_id`와 `get_bin_data(index)`의 인덱스 기준을 문서화하고 최종 검증을 수행한다.
