# Stage 3 보고 — Task M100-1196

## 범위

- 이슈: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 단계: 최종 `page_number` 기준 `PageContent.layout` 갱신
- 브랜치: `local/task1196`

## 변경 파일

```text
src/document_core/queries/rendering.rs
src/renderer/page_layout.rs
```

Stage 2에서 변경한 `src/model/page.rs`, `src/renderer/page_layout.rs`의 페이지별 계산 API를 실제 pagination 결과에 연결했다.

## 구현 내용

### PageLayoutInfo x 이동 helper 추가

`src/renderer/page_layout.rs`에 다음 helper를 추가했다.

```rust
pub fn apply_page_number_margins(&mut self, page_def: &PageDef, page_number: u32)
```

동작:

- `PageAreas::from_page_def_for_page(page_def, page_number)`로 최종 page number 기준 body/header/footer 영역을 계산한다.
- 현재 layout의 `body_area.x`와 목표 `body_area.x`의 차이(`delta_x`)를 계산한다.
- `body_area`, `header_area`, `footer_area`의 x/width를 목표 영역으로 갱신한다.
- `footnote_area`가 존재하면 같은 x delta를 적용한다.
- `column_areas`는 ColumnDef가 다른 zone layout일 수 있으므로 너비/간격을 재계산하지 않고 x만 이동한다.

추가 테스트:

```text
apply_page_number_margins_moves_existing_zone_layout_without_rebuilding_columns
```

### DocumentCore pagination 결과에 적용

`src/document_core/queries/rendering.rs`에 다음 helper를 추가했다.

```rust
fn apply_page_number_layouts_for_section(result: &mut PaginationResult, section: &Section)
```

동작:

- 각 `PageContent.layout`에 `apply_page_number_margins()`를 적용한다.
- 각 `ColumnContent.zone_layout`에도 같은 page number 기준 보정을 적용한다.

호출 시점:

```text
1. section-local pagination 수행
2. page_number_pos 상속
3. 구역 간 page_number carry 보정
4. apply_page_number_layouts_for_section()
5. assign_master_pages_for_section()
6. header/footer 상속 보정
```

이 순서로 둔 이유:

- #1276의 바탕쪽 선택과 같은 “최종 page_number 기준”을 사용한다.
- TypesetEngine과 fallback Paginator 모두 `DocumentCore::paginate()` 결과를 통과하므로 한 곳에서 정책을 맞출 수 있다.
- 페이지네이션 중 layout 폭을 바꾸지 않아 줄바꿈/표 분할 결과를 흔들지 않는다.

## 단위/회귀 검증

명령:

```text
cargo test --lib page_layout
cargo test --lib page_areas
cargo test --lib master_page_selection_uses_final_carried_page_number_parity
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
cargo fmt --check
```

결과:

```text
cargo test --lib page_layout
test result: ok. 6 passed; 0 failed

cargo test --lib page_areas
test result: ok. 4 passed; 0 failed

cargo test --lib master_page_selection_uses_final_carried_page_number_parity
test result: ok. 1 passed; 0 failed

cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
test result: ok. 3 passed; 0 failed

cargo fmt --check
통과
```

## 대상 샘플 dump-pages 검증

대상 문서:

```text
samples/hwpx/[2027] 온새미로 1 본교재.hwpx
```

명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 4
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 5
```

결과 요약:

```text
문서 로드: 46페이지

page 4: global_idx=3, section=1, page_num=4
  body_area: x=189.0 y=113.4 w=510.2 h=895.8
  첫 문단: "강의 01."

page 5: global_idx=4, section=1, page_num=5
  body_area: x=94.5 y=113.4 w=510.2 h=895.8

page 6: global_idx=5, section=1, page_num=6
  body_area: x=189.0 y=113.4 w=510.2 h=895.8
```

판단:

- Stage 1 baseline의 `x=94.5 / 94.5 / 94.5`가 `189.0 / 94.5 / 189.0`으로 교대된다.
- page 4는 여전히 section 1 본문 시작이며 `page_num=4`이다.
- body width는 `510.2`로 유지된다.

## fallback Paginator 확인

명령:

```text
env RHWP_USE_PAGINATOR=1 cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
env RHWP_USE_PAGINATOR=1 cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 4
```

결과 요약:

```text
page 4: body_area.x=189.0
page 5: body_area.x=94.5
```

판단:

- fallback Paginator 결과에도 최종 page number 기준 layout 교대 정책이 적용된다.
- fallback 경로는 이 샘플에서 47페이지로 로드되며, 이는 #1196 layout 적용과 별개인 기존 fallback 페이지네이션 차이다.

## Stage 3 결론

- `PageContent.layout`이 최종 `page.page_number` 기준으로 갱신된다.
- 기본 TypesetEngine 경로에서 page 4/5/6 body_area x 좌표가 PDF 기준 홀짝 방향으로 교대된다.
- fallback Paginator 경로에도 같은 layout 정책이 적용된다.
- #1271 회귀 테스트와 master page 최종 page number 테스트는 통과했다.
- Stage 4에서는 #1196 전용 통합 회귀 테스트를 추가한다.
