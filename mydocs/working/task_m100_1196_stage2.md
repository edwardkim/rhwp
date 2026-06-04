# Stage 2 보고 — Task M100-1196

## 범위

- 이슈: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 단계: 페이지별 여백 계산 API와 단위 테스트
- 브랜치: `local/task1196`

## 변경 파일

```text
src/model/page.rs
src/renderer/page_layout.rs
```

## 구현 내용

### PageAreas 페이지별 계산 API 추가

`src/model/page.rs`에 다음 API를 추가했다.

```rust
pub fn from_page_def_for_page(page_def: &PageDef, page_number: u32) -> Self
```

기존 `PageAreas::from_page_def(page_def)`는 호환을 위해 유지하고, 내부에서 page 1 기준 계산으로 위임한다.

`BindingMethod::DuplexSided` 처리 규칙:

```text
홀수쪽:
  effective_left  = margin_left + margin_gutter
  effective_right = margin_right

짝수쪽:
  effective_left  = margin_right
  effective_right = margin_left + margin_gutter
```

`page_number=0`은 아직 최종 쪽번호가 확정되지 않은 상태로 보고 기존 방향을 유지하도록 했다.

### PageLayoutInfo 페이지별 계산 API 추가

`src/renderer/page_layout.rs`에 다음 API를 추가했다.

```rust
pub fn from_page_def_for_page(
    page_def: &PageDef,
    column_def: &ColumnDef,
    dpi: f64,
    page_number: u32,
) -> Self
```

기존 `PageLayoutInfo::from_page_def()`는 page 1 기준으로 위임해 기존 호출부 결과를 유지한다.

## 추가 테스트

`src/model/page.rs`:

- `page_areas_single_sided_keeps_horizontal_margins_on_even_pages`
- `page_areas_duplex_sided_swaps_horizontal_margins_on_even_pages`
- `page_areas_top_flip_keeps_left_right_margins_for_now`

`src/renderer/page_layout.rs`:

- `page_layout_duplex_sided_even_page_swaps_body_and_columns`
- `page_layout_from_page_def_matches_page_one_layout`

테스트 의도:

- `SingleSided`는 page 1/2 좌표가 동일하다.
- `DuplexSided`는 짝수쪽에서 좌우 여백이 교대된다.
- `TopFlip`은 이번 이슈 범위에서 좌우 교대하지 않는다.
- `PageLayoutInfo`의 다단 column 영역이 body x 이동을 따라간다.
- 기존 `from_page_def()`는 page 1 layout과 동일하다.

## 검증

명령:

```text
cargo test --lib page_areas
cargo test --lib page_layout
cargo fmt --check
```

결과:

```text
cargo test --lib page_areas
test result: ok. 4 passed; 0 failed

cargo test --lib page_layout
test result: ok. 5 passed; 0 failed

cargo fmt --check
통과
```

## 판단

- 페이지별 여백 교대 계산 API가 추가됐다.
- 기존 `from_page_def()` 호출부는 page 1 기준 결과를 유지한다.
- Stage 2는 아직 실제 `PageContent.layout`에 최종 page number를 반영하지 않는다.
- Stage 3에서 `DocumentCore::paginate()` 결과의 각 페이지 layout을 최종 `page.page_number` 기준으로 갱신한다.
