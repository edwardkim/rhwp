# Task #1270 Stage 3 완료보고서 — 구현 + 집중 테스트

## 범위

- `layout_caption()`에 캡션 문단 원본 `para`와 `bin_data_content` 스레딩 적용
- `layout_caption()` 호출부 전체 컴파일 정합화
- 캡션 내 인라인 TAC picture 회귀 테스트 추가
- 신규 테스트와 관련 기존 TAC picture 테스트 실행
- 첨부 HWPX 1페이지 SVG 보조 관찰

## 구현 요약

### 1. 캡션 문단 원본/바이너리 스레딩

수정 파일:

- `src/renderer/layout/picture_footnote.rs`

`layout_caption()` 시그니처에 `bin_data_content: &[BinDataContent]`를 추가했다.

기존에는 `layout_composed_paragraph()` 호출 시 인라인 이미지 렌더링에 필요한 마지막 인자들을 `None`으로 고정 전달했다.

```rust
None,
None,
None,
None,
```

수정 후 캡션 문단 루프의 실제 `para`와 상위 렌더 경로의 `bin_data_content`를 전달한다.

```rust
None,
Some(para),
Some(bin_data_content),
None,
```

이에 따라 캡션 문단의 `treat_as_char` picture가 `paragraph_layout.rs`의 기존 인라인 이미지 방출 경로를 사용할 수 있게 되었다.

### 2. 호출부 정합화

수정 파일:

- `src/renderer/layout.rs`
- `src/renderer/layout/picture_footnote.rs`
- `src/renderer/layout/shape_layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`

각 호출부는 이미 상위 함수 scope에서 `bin_data_content`를 보유하고 있었다. 기존 좌표 계산, `cell_ctx`, caption y/height 처리 방식은 변경하지 않고 인자만 추가했다.

변경된 호출부는 다음 7곳이다.

```text
src/renderer/layout.rs:6752
src/renderer/layout/picture_footnote.rs:528
src/renderer/layout/shape_layout.rs:535
src/renderer/layout/table_layout.rs:1022
src/renderer/layout/table_partial.rs:1628
src/renderer/layout/table_partial.rs:1652
src/renderer/layout/table_partial.rs:1684
```

### 3. 신규 회귀 테스트

추가 파일:

- `tests/issue_1270_caption_inline_image.rs`

테스트 방식:

1. 작은 기존 fixture `samples/hwpx/hy-001.hwpx`를 로드한다.
2. 문서 안에서 실제 파싱된 TAC picture 포함 문단을 찾는다.
3. 해당 문단을 첫 번째 top-level 표의 TOP caption paragraph로 복제한다.
4. `build_page_render_tree(0)`를 실행한다.
5. caption sentinel context(`cell_index = 65534`)를 가진 `ImageNode`가 정확히 1개 방출되고, 이미지 payload가 존재하는지 확인한다.

검증 케이스:

- 텍스트가 함께 있는 캡션 TAC picture 문단
- 텍스트 없이 TAC picture만 있는 캡션 문단

검증 기준:

```rust
caption_images == vec![(caption_bin_id, true)]
```

이 테스트는 수정 전 구조에서는 `layout_caption()`이 `para` / `bin_data_content`를 넘기지 않아 caption 이미지가 방출되지 않는 조건을 직접 겨냥한다.

## 리뷰 보완

리뷰 중 표 캡션의 picture-only 문단 경계 조건을 추가 확인했다. 표 캡션은 실제 표 셀이 아니지만 `cell_ctx`에 `cell_index = 65534` 센티널을 사용한다. 기존 빈 줄 TAC picture 렌더링 분기는 `cell_ctx.is_none()`일 때만 동작하므로, 텍스트 없이 TAC picture만 있는 표 캡션 문단은 여전히 누락될 수 있었다.

보완 내용:

- `src/renderer/layout/paragraph_layout.rs`
  - 빈 줄 TAC picture 렌더링 허용 조건에 `cell_index == 65534` 캡션 센티널 예외 추가
  - 실제 표 셀 내부 중복 렌더링 방지 조건은 유지
- `tests/issue_1270_caption_inline_image.rs`
  - picture-only 캡션 TAC 문단 회귀 테스트 추가

이 보완은 구현 계획서에서 “필요 시”로 둔 좁은 예외 적용이다.

## 검증 결과

### 신규 테스트

```bash
cargo test --test issue_1270_caption_inline_image
```

결과:

```text
running 2 tests
test table_caption_inline_tac_picture_emits_image_node ... ok
test table_caption_picture_only_tac_paragraph_emits_image_node ... ok

test result: ok. 2 passed; 0 failed
```

### 관련 기존 테스트

```bash
cargo test --test issue_1352_table_cell_tac_picture_text
```

결과:

```text
running 1 test
test hy001_first_cell_tac_picture_stays_inside_center_aligned_cell ... ok

test result: ok. 1 passed; 0 failed
```

```bash
cargo test --test issue_1459_topbottom_picture_reflow
```

결과:

```text
running 3 tests
test turning_first_picture_off_reflows_remaining_tac_picture ... ok
test topbottom_second_picture_flows_before_tac_picture ... ok
test non_tac_topbottom_picture_is_not_caret_stop ... ok

test result: ok. 3 passed; 0 failed
```

### 포맷 체크

```bash
cargo fmt --check
```

결과: 통과.

## 첨부 샘플 보조 관찰

첨부 HWPX를 수정 후 다시 SVG로 내보냈다.

```bash
cargo run --quiet --bin rhwp -- export-svg \
  '/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx' \
  -p 0 \
  -o /private/tmp/rhwp-task1270-stage3-svg
```

1페이지 SVG의 `<image>` 요소 수:

```text
1
```

이미지 태그:

```text
<image x="69.90666666666667" y="109.79999999999998" width="200" height="118.4" .../>
```

따라서 첨부 샘플의 `image2`는 이번 Stage 3 수정 후에도 1페이지 SVG에 나타나지 않는다. 이는 구현 계획서에서 보조 관찰로 분리한 항목이며, 메인테이너 코멘트의 “플로팅 캡션 이미지는 후속” 범위와 충돌하지 않도록 이번 Stage 3 성공 기준에서 제외했다.

## 변경 파일

```text
src/renderer/layout.rs
src/renderer/layout/picture_footnote.rs
src/renderer/layout/shape_layout.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
src/renderer/layout/paragraph_layout.rs
tests/issue_1270_caption_inline_image.rs
```

## Stage 3 결론

- 캡션 문단의 인라인 TAC picture 렌더링에 필요한 `para` / `bin_data_content` 스레딩을 구현했다.
- 신규 회귀 테스트로 텍스트 포함 캡션 및 picture-only 캡션의 인라인 TAC picture가 `ImageNode`로 방출되는 것을 확인했다.
- 기존 셀 TAC picture / TopAndBottom picture 흐름 테스트는 통과했다.
- `paragraph_layout.rs`의 빈 줄 TAC 조건은 캡션 센티널에 한해서만 완화했고, 기존 표 셀 중복 방지 범위는 보존했다.
- 첨부 샘플의 완전 해소는 후속 플로팅 캡션 이미지 작업 범위로 유지한다.

## 다음 단계

작업지시자 승인 후 Stage 4 회귀 검증을 진행한다.
