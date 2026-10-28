# Task #1270 구현 계획서 — 캡션 문단 인라인 이미지 스레딩

## 기준

- **수행 계획서**: `mydocs/plans/task_m100_1270.md`
- **Stage 1 보고서**: `mydocs/working/task_m100_1270_stage1.md`
- **브랜치/worktree**: `local/task1270` / `/private/tmp/rhwp-task1270`
- **base**: `upstream/devel` `48e96704`
- **메인테이너 지시 범위**: 캡션 내 `treat_as_char` 인라인 이미지 스레딩, depth 1 한정. 플로팅 캡션 이미지는 후속.

## 구현 원칙

1. `layout_caption()`에서 이미 순회 중인 캡션 문단 `para`를 `layout_composed_paragraph()`에 넘긴다.
2. `layout_caption()` 호출부에서 이미 보유한 `bin_data_content`를 캡션 경로로 전달한다.
3. 기존 캡션 배치 좌표, caption height 계산, y advance 계산은 변경하지 않는다.
4. 캡션 속 그림의 caption 재귀 렌더링은 구현하지 않는다.
5. 플로팅 캡션 이미지 배치 일반화는 구현하지 않는다.

## 수정 대상

### 1. `src/renderer/layout/picture_footnote.rs`

`layout_caption()` 시그니처에 이미지 바이너리 인자를 추가한다.

```rust
pub(crate) fn layout_caption(
    &self,
    tree: &mut PageRenderTree,
    parent_node: &mut RenderNode,
    caption: &Caption,
    styles: &ResolvedStyleSet,
    _col_area: &LayoutRect,
    content_x: f64,
    content_width: f64,
    y_start: f64,
    auto_counter: &mut AutoNumberCounter,
    bin_data_content: &[BinDataContent],
    cell_ctx: Option<super::CellContext>,
)
```

`layout_composed_paragraph()` 호출의 마지막 인자 묶음을 다음처럼 바꾼다.

```rust
0.0,
None,                    // multi_col_width_hu
Some(para),              // 원본 캡션 문단
Some(bin_data_content),  // 이미지 바이너리
None,                    // wrap_anchor
```

현재 `section_index` / `para_index` 인자는 기존처럼 `0, 0`을 유지한다. 캡션 이미지의 실제 소유 위치는 기존 `cell_ctx` 또는 호출부 context가 보존하는 범위 안에서만 다룬다. 이번 작업에서 캡션 문단 identity 체계를 새로 설계하지 않는다.

### 2. `layout_caption()` 호출부

모든 호출부에 `bin_data_content`를 추가 전달한다.

| 파일 | 호출 위치 | 전달 근거 |
|------|-----------|-----------|
| `src/renderer/layout/picture_footnote.rs` | `layout_body_picture()` 내부 | 함수 인자로 이미 `bin_data_content: &[BinDataContent]` 보유 |
| `src/renderer/layout/table_layout.rs` | `layout_table()` 캡션 렌더링 | table layout 함수 인자로 이미 `bin_data_content` 보유 |
| `src/renderer/layout/table_partial.rs` | top/bottom/left/right partial caption | partial table layout 함수 인자로 이미 `bin_data_content` 보유 |
| `src/renderer/layout/shape_layout.rs` | `layout_shape()` 캡션 렌더링 | 함수 인자로 이미 `bin_data_content` 보유 |
| `src/renderer/layout.rs` | 본문 picture caption 경로 | 해당 렌더 함수 scope에 이미 `bin_data_content` 보유 |

호출부의 `cell_ctx` 생성 방식은 그대로 둔다.

### 3. 빈 문단 TAC 캡션 주의점

`paragraph_layout.rs`에는 runs가 없는 빈 줄에서 TAC picture를 렌더링하는 분기가 있다.

현재 조건은 다음 의도를 가진다.

```rust
// 테이블 셀 내부에서는 table_layout.rs가 layout_picture로 이미 처리하므로 스킵.
// 셀 외부에서 해당 줄 범위에 걸린 TAC만 여기서 렌더링.
if cell_ctx.is_none() && !line_tac_offsets.is_empty() {
    ...
}
```

캡션은 실제 표 셀이 아니지만 hit-test/context 보존을 위해 `cell_ctx`를 사용한다. 특히 표 캡션은 `cell_index = 65534` 센티널을 쓴다.

따라서 Stage 3 구현 검증은 두 단계로 판정한다.

1. 우선 `layout_caption()` 스레딩만 적용한다.
2. 신규 테스트 또는 첨부 샘플 관찰에서 빈 문단 caption TAC picture가 여전히 누락되고, 해당 컨트롤이 `treat_as_char=true`로 분류되는 경우에만 `paragraph_layout.rs`의 빈 줄 TAC picture 조건을 캡션 센티널에 한해 완화한다.

조건 완화가 필요할 경우의 후보:

```rust
let is_caption_ctx = cell_ctx
    .as_ref()
    .and_then(|ctx| ctx.path.last())
    .is_some_and(|entry| entry.cell_index == 65534);

if (cell_ctx.is_none() || is_caption_ctx) && !line_tac_offsets.is_empty() {
    ...
}
```

이 완화는 실제 표 셀 내부 중복 렌더링 방지 조건을 보존하면서, 캡션 센티널만 셀 외부 흐름으로 취급하기 위한 좁은 예외다. 단, 메인테이너 지시의 1차 범위는 스레딩이므로 이 예외는 Stage 3 증거가 있을 때만 적용한다.

## 회귀 테스트 계획

### 신규 테스트 파일

```text
tests/issue_1270_caption_inline_image.rs
```

### 테스트 방식

1. 기존 작은 HWPX fixture를 로드한다.
   - 1차 후보: `samples/hwpx/hy-001.hwpx`
   - 이유: 이미 표 셀 내부 TAC picture + text 렌더링 회귀 테스트에서 사용 중이고, fixture 크기/렌더 비용이 작다.
2. 테스트 안에서 문서 IR을 조작해 첫 번째 표에 TOP caption을 부여한다.
3. caption paragraphs에는 기존 문서에서 파싱된 TAC picture 포함 문단을 복제하거나, 최소 `Paragraph` + `Picture`를 구성한다.
4. `build_page_render_tree(0)` 실행 후 `RenderNodeType::Image`를 순회한다.
5. caption sentinel context를 가진 이미지가 방출되는지 확인한다.

검증 기준:

```rust
matches!(node.node_type, RenderNodeType::Image(img)
    if img.cell_index == Some(65534)
        && img.cell_context.as_ref().is_some_and(|ctx| ...)
)
```

가능하면 다음도 확인한다.

- `img.data.is_some()` 또는 `img.bin_data_id`가 문서 `bin_data_content`에 존재
- 이미지가 1개만 방출되어 중복 렌더링되지 않음
- 기존 셀 내부 이미지는 기존 context로 유지됨

### 첨부 샘플 검증

첨부 HWPX는 Stage 3/4에서 보조로 확인한다.

```bash
cargo run --quiet --bin rhwp -- export-svg \
  '/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx' \
  -p 0 \
  -o /private/tmp/rhwp-task1270-stage3-svg
```

단, 첨부 샘플의 `image2`는 `textWrap="TOP_AND_BOTTOM"` 특성과 메인테이너의 플로팅 후속 범위 언급이 있으므로, 이번 PR의 통과 조건은 신규 인라인 캡션 테스트로 둔다. 첨부 샘플 결과는 “보조 관찰”로만 보고한다.

## 구현 단계

### Stage 3-1 — 캡션 스레딩 구현

- `layout_caption()` 시그니처에 `bin_data_content` 추가
- `layout_composed_paragraph()` 호출에 `Some(para)` / `Some(bin_data_content)` 전달
- 모든 호출부 컴파일 정합화
- `cargo check` 또는 신규 테스트 컴파일로 타입 오류 확인

### Stage 3-2 — 신규 회귀 테스트 추가

- `tests/issue_1270_caption_inline_image.rs` 작성
- 작은 fixture + IR 조작 방식으로 캡션 내 인라인 이미지 방출 검증
- 수정 전 실패, 수정 후 성공하는 조건 확인

### Stage 3-3 — 필요 시 빈 문단 caption sentinel 예외 적용

- 스레딩만으로 신규 테스트가 통과하면 이 단계는 생략한다.
- 빈 문단 caption TAC picture만 실패할 경우, `paragraph_layout.rs`의 빈 줄 TAC picture 조건에 `cell_index == 65534` 예외를 추가한다.
- 실제 표 셀 내부 TAC picture 중복 방지 조건은 유지한다.

### Stage 3-4 — 집중 검증 및 보고

- 신규 #1270 테스트 실행
- 관련 기존 테스트 선별 실행
- 첨부 HWPX SVG 1페이지 보조 관찰
- `mydocs/working/task_m100_1270_stage3.md` 작성

## Stage 4 회귀 검증 계획

Stage 3 승인 후 다음 검증을 실행한다.

```bash
cargo test --test issue_1270_caption_inline_image
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1352_table_cell_tac_picture_text
cargo test --test issue_1459_topbottom_picture_reflow
cargo test --lib
cargo clippy --lib -- -D warnings
```

필요 시 표/캡션 경로 추가 검증:

```bash
cargo test --test issue_530
cargo test --test issue_1486_hwpx_partial_tac_table
```

## 성공 기준

- 신규 #1270 테스트가 통과한다.
- 캡션 내 인라인 `treat_as_char` picture가 `ImageNode`로 방출된다.
- 실제 표 셀 내부 TAC picture가 중복 렌더링되지 않는다.
- 기존 인라인 picture 중복 방지 테스트가 통과한다.
- `cargo test --lib`와 clippy가 통과한다.
- 보고서에 depth 1 인라인 한정과 플로팅 후속 범위를 명시한다.

## 비성공/후속 기준

다음은 이번 작업의 실패가 아니라 후속 범위로 기록한다.

- 캡션 안 `treat_as_char=false` picture 누락
- 캡션 안 `textWrap=TOP_AND_BOTTOM` 플로팅 배치 불완전
- 캡션 속 그림의 caption 미렌더링
- 첨부 샘플 `image2`가 메인테이너가 정의한 플로팅 후속 범위 때문에 여전히 누락되는 경우

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| `layout_caption()` 호출부 누락으로 컴파일 실패 | `rg "layout_caption\\(" src/renderer`로 모든 호출부 확인 후 컴파일 |
| 실제 표 셀 내부 TAC picture 중복 렌더링 | 빈 줄 TAC 조건은 기본적으로 유지하고, 필요 시 caption sentinel만 예외 |
| 캡션 이미지 metadata가 기존 문단 index `0,0`으로 남음 | 이번 범위에서는 기존 caption text 경로와 동일하게 유지, cell context로만 식별 |
| 첨부 샘플 완전 해소 여부 혼동 | 신규 인라인 테스트를 성공 기준으로 삼고 첨부 샘플은 보조 관찰로 분리 |
| 플로팅 일반화로 범위 확장 | `treat_as_char` 인라인 렌더링 외 로직 변경 금지 |

## 승인 후 다음 작업

작업지시자 승인 후 Stage 3 구현을 시작한다.
