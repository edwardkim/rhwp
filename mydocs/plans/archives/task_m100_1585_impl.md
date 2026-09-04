# Task M100 #1585 구현계획서 — 캡션 내 플로팅 이미지 렌더링

## 기준 상태

- 작업 브랜치: `local/task1585`
- Worktree: `/private/tmp/rhwp-task1585`
- 기준 커밋: 최신 `upstream/devel` `a94e2051`
- 선행 PR: #1551 `MERGED`, `mergedAt=2026-06-27T04:12:17Z`

작성 초기에는 #1551이 아직 merge되지 않았으므로 이 구현계획서는 소스 수정 전 설계 확정 문서로 작성했다. 이후 Stage 3 시작 시 #1551 병합을 확인했고, `local/task1585`를 최신 `upstream/devel` `a94e2051`로 rebase한 뒤 구현했다.

## 결론

#1585는 #1551이 merge된 뒤 최신 `upstream/devel`로 재기준화해서 구현한다.

그 이유는 #1585 구현이 #1551의 다음 변경을 전제로 하기 때문이다.

- `layout_caption()`이 caption paragraph 원본 `para`를 `layout_composed_paragraph()`에 전달
- `layout_caption()`이 `bin_data_content`를 받아 caption 내부 그림의 바이너리 데이터를 조회
- #1270 인라인 caption image 회귀 테스트

#1551이 merge되기 전에 Stage 3을 진행하는 stacked 변경은 피했고, 현재 브랜치는 #1551 merge commit 이후의 `upstream/devel`을 기준으로 한다.

## 구현 범위

대상은 caption 내부 `Control::Picture` 중 `pic.common.text_wrap == TextWrap::TopAndBottom`인 그림이다.

주의할 점은 #1270 첨부 샘플의 `image2`가 `hp:pos@treatAsChar="1"`도 가지고 있다는 점이다. 이번 후속 작업에서는 메인테이너의 분리 기준에 맞춰 `treat_as_char` 값이 아니라 `textWrap="TOP_AND_BOTTOM"`을 플로팅 caption image 판정 기준으로 삼는다.

## 비범위

- #1270 이슈 close
- caption 속 그림의 caption 재귀 렌더링 일반화
- 모든 `Square`, `BehindText`, `InFrontOfText` caption object 일반화
- HWP3 전용 분기
- rhwp-studio UI 수정

## 설계 방향

전역 paragraph/table 플로팅 로직을 확장하지 않고, `layout_caption()` 내부에서 caption 전용 후처리 helper를 호출한다.

이유:

- caption 영역은 이미 `layout_caption()`에서 `content_x`, `content_width`, `y_start`, `cell_ctx`를 알고 있다.
- 표 caption은 `cell_index=65534` 센티널로 식별된다.
- 전역 `paragraph_layout.rs`의 빈 문단/TAC 처리 조건을 바꾸면 본문·셀·글상자 회귀 범위가 커진다.
- #1585는 caption 내부 `TopAndBottom` 그림에 한정된 후속 작업이다.

## 구현 단계

### 1. Stage 3 preflight

소스 수정 직전에 다음을 다시 확인한다.

```bash
gh pr view 1551 --repo edwardkim/rhwp --json state,mergedAt,mergeCommit,baseRefName,headRefName
git fetch upstream devel
git rev-parse upstream/devel
```

판정:

- #1551 merge 완료: `local/task1585`를 최신 `upstream/devel`로 rebase한 뒤 구현한다.
- #1551 미merge: 소스 수정하지 않고 작업지시자에게 stacked 진행 여부를 확인한다.

### 2. caption helper 추가

`src/renderer/layout/picture_footnote.rs`에 caption 전용 helper를 추가한다.

예상 형태:

```rust
fn layout_caption_topbottom_pictures(
    &self,
    tree: &mut PageRenderTree,
    parent_node: &mut RenderNode,
    para: &Paragraph,
    caption_area: &LayoutRect,
    para_y_before_layout: f64,
    bin_data_content: &[BinDataContent],
    section_index: usize,
    para_index: usize,
    cell_ctx: Option<&CellContext>,
)
```

helper는 `layout_caption()`에서 각 caption paragraph 처리 직후 호출한다.

### 3. 후보 탐지

caption paragraph의 controls를 순회한다.

대상:

```rust
matches!(ctrl, Control::Picture(pic) if matches!(pic.common.text_wrap, TextWrap::TopAndBottom))
```

제외:

- 이미 같은 `(section, para, control, cell_ctx)`로 inline position이 등록된 그림
- `bin_data_content`에서 데이터를 찾을 수 없고 외부 path도 없는 그림은 placeholder 정책 확인 후 현행 `layout_picture()` 동작에 맡김

중복 방지는 `PageRenderTree::get_inline_shape_position()`으로 한다. #1551 인라인 경로가 이미 같은 control을 렌더한 경우 caption floating helper는 추가 emit하지 않는다.

### 4. 좌표 계산

caption local area를 기준으로 계산한다.

- 기준 rect: `LayoutRect { x: content_x, y: para_y_before_layout, width: content_width, height: ... }`
- `VertRelTo::Para`인 경우 첫 `LineSeg.vertical_pos`가 있으면 `caption_area.y + vertical_pos`를 anchor로 사용한다.
- `HorzRelTo::Column`/`Para`는 caption content box를 기준으로 처리한다.
- `horz_align`, `vert_align`, `horizontal_offset`, `vertical_offset`은 기존 `compute_object_position()`의 의미와 맞춘다.

#1270 첨부 샘플은 `horzRelTo="COLUMN"`, `vertRelTo="PARA"`, `horzAlign="LEFT"`, `vertAlign="TOP"`, offset 0이다. 이 케이스가 우선 검증 기준이다.

### 5. emit 방식

기존 `layout_picture()`/`layout_picture_full()`을 재사용한다.

다만 caption floating helper는 `TextWrap::TopAndBottom`을 우선하는 경로이므로, 배치용 clone에서는 다음을 적용한다.

- `horizontal_offset`/`vertical_offset`은 사전 좌표 계산에 반영한 뒤 0으로 정규화
- `horz_align`/`vert_align`은 `Left`/`Top`으로 정규화
- sample처럼 `treat_as_char=true`이더라도 floating helper에서 배치한 그림은 inline 위치 등록으로 다시 처리하지 않도록 clone의 `treat_as_char`를 `false`로 둔다.

이렇게 해야 `layout_picture_full()`의 TAC 분기로 되돌아가 caption floating 배치가 무력화되거나 inline 중복 등록되는 것을 피할 수 있다.

### 6. caption height / flow 정책

이번 구현은 기존 caption 높이 계산을 크게 바꾸지 않는다.

- `calculate_caption_height()`의 전체 정책은 유지한다.
- floating image가 caption text와 겹치는 경우에만 최소 높이 보정이 필요한지 Stage 3에서 실제 테스트로 확인한다.
- 보정이 필요하면 caption paragraph별 `TopAndBottom` 그림 높이를 `calculate_caption_height()`에 포함하는 별도 작은 helper를 추가한다.

처음부터 table row height나 pagination 예약 높이까지 확장하지 않는다.

### 7. 재귀 caption 차단

caption 속 picture가 다시 caption을 갖는 경우 이번 범위에서는 재귀 렌더링하지 않는다.

구현은 `layout_body_picture()`가 아니라 `layout_picture()` 또는 caption 전용 emit helper를 통해 진행한다. 이로써 caption 속 그림의 caption까지 다시 내려가는 depth 확장을 피한다.

## 테스트 계획

### 신규 회귀 테스트

파일 후보:

```text
tests/issue_1585_caption_floating_image.rs
```

테스트 방식:

1. 저장소에 이미 있는 작은 HWPX/HWP 샘플에서 문서와 BinData를 로드한다.
2. 기존 picture control 하나를 clone해 caption paragraph 내부로 넣는다.
3. clone한 picture를 다음 속성으로 조정한다.
   - `text_wrap = TextWrap::TopAndBottom`
   - `treat_as_char = true`
   - `vert_rel_to = VertRelTo::Para`
   - `horz_rel_to = HorzRelTo::Column`
   - `vert_align = VertAlign::Top`
   - `horz_align = HorzAlign::Left`
   - offsets 0
4. 표 top caption의 빈 paragraph 또는 image-only paragraph에 해당 picture control을 배치한다.
5. page 0 render tree를 생성한다.
6. `ImageNode` 중 `cell_context.path[0].cell_index == 65534`이고 `text_wrap == Some(TextWrap::TopAndBottom)`인 노드를 찾는다.
7. caption floating image가 정확히 1개 방출되는지, 동일 control 중복 방출이 없는지 검증한다.

이 테스트는 현재 기준에서는 실패해야 하고, Stage 3 구현 후 통과해야 한다.

### 실샘플 보조 검증

Git에 첨부 샘플을 추가하지 않고, 로컬 첨부 파일로 SVG 산출을 확인한다.

```bash
cargo run --quiet --bin rhwp -- export-svg \
  '/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx' \
  -p 0 \
  -o /private/tmp/rhwp-task1585-svg
```

검증:

- 1페이지 SVG의 `<image>` 개수가 기존 1개에서 `image2` 포함 2개 이상으로 증가
- `BinData/image2.png`에 해당하는 `SEOUL MY SOUL` 로고가 page 0 render tree 또는 SVG에 방출
- 기존 `image1` 누락 없음

### 회귀 테스트

```bash
cargo fmt --check
cargo test --test issue_1585_caption_floating_image
cargo test --test issue_1270_caption_inline_image
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1352_table_cell_tac_picture_text
cargo test --test issue_1459_topbottom_picture_reflow
cargo test --test issue_530
cargo test --test issue_1486_hwpx_partial_tac_table
cargo test --lib
cargo clippy --lib -- -D warnings
```

## 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| #1551 미merge 상태에서 중복 변경 포함 | Stage 3 preflight에서 merge 상태 확인 후, 미merge면 소스 수정 중단 |
| `treat_as_char=true`와 `TopAndBottom`이 충돌 | #1585에서는 `TopAndBottom`을 caption floating 판정 기준으로 삼고, emit clone은 non-TAC 배치로 정규화 |
| 기존 inline caption image와 중복 방출 | `get_inline_shape_position()` 기반 중복 가드 |
| caption height가 이미지 높이를 반영하지 못함 | 우선 최소 emit을 구현하고, 겹침이 재현되면 caption height 보정 helper 추가 |
| table/cell 플로팅 회귀 | caption 전용 helper로 범위 제한, 기존 TopAndBottom 회귀 테스트 동시 실행 |

## 승인 후 진행 조건

Stage 3 소스 수정은 다음 중 하나가 충족될 때만 시작한다.

1. #1551이 merge되어 최신 `upstream/devel`에 포함됨
2. 작업지시자가 #1551 미merge 상태에서 stacked 변경으로 진행하라고 명시 승인함

권장 조건은 1번이다.
