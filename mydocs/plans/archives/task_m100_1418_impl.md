# 구현 계획서 — Task M100-1418: Paper 기준 글상자 host 문단 line advance 보존

- 이슈: https://github.com/edwardkim/rhwp/issues/1418
- 수행 계획서: `mydocs/plans/task_m100_1418.md`
- Stage 1: `mydocs/working/task_m100_1418_stage1.md`
- Stage 2: `mydocs/working/task_m100_1418_stage2.md`
- 작성일: 2026-06-16
- 브랜치: `local/task_m100_1418`

## 1. 진단 요약

교체된 정답 PDF `pdf-large/hwpx/2026_oss_rst.pdf`는 총 6페이지 문서다. 이번 이슈의 기준은
그 PDF의 1페이지와 `samples/2026_oss_rst.hwp` 1페이지다.

정답 PDF 1페이지에서 제목 글상자 흰 배경은 `y≈133.1..171.1px` 영역에 있고,
큰 1x1 안내 표 상단선은 그 중앙에 가까운 `y≈153.4px`를 지난다. 현재 rhwp는 제목 글상자
위치는 거의 맞지만 큰 표 상단선을 `y=132.3px`에 그린다.

차이 `≈21.1px`는 직전 빈 host 문단 `pi=0`의 line advance `21.3px`와 일치한다.

## 2. 원인

페이지네이션 단계는 `FullParagraph pi=0 h=21.3`을 가지고 있다. 그러나 render tree layout
단계에서 빈 non-TAC floating shape host 문단이 fast path로 처리되며, 이 샘플의 글상자가
`VertRelTo::Paper` 기준이라는 이유로 line advance 예약 대상에서 제외된다.

현재 경로:

- `src/renderer/layout.rs:359` `para_has_visible_textless_float_shape_item`
- `src/renderer/layout.rs:392` `textless_infront_para_host_requires_line_advance`
- `src/renderer/layout.rs:4437` 빈 floating shape host fast path

현재 `textless_infront_para_host_requires_line_advance`는 다음 조건만 true로 본다.

- non-TAC
- `TextWrap::InFrontOfText`
- `VertRelTo::Para`

문제 샘플의 제목 글상자는 non-TAC `InFrontOfText`이지만 `VertRelTo::Paper`다. 그래서 layout에서
`pi=0` 진행량이 0이 되고, 다음 `Table pi=1 ci=0`이 body top `132.3px`에서 바로 시작한다.

## 3. 수정 방안

### 3.1 `layout.rs` helper 조건 보강

`src/renderer/layout.rs`의 `textless_infront_para_host_requires_line_advance`를 좁게 확장한다.

유지할 기존 동작:

- `Control::Picture`의 `VertRelTo::Para + InFrontOfText` 예약 동작
- `Control::Shape`의 `VertRelTo::Para + InFrontOfText` 예약 동작
- `BehindText`, `TopAndBottom`, 일반 flow 객체, TAC 객체의 기존 처리

추가할 동작:

- `Control::Shape`
- `shape.common().treat_as_char == false`
- `shape.common().text_wrap == TextWrap::InFrontOfText`
- `shape.common().vert_rel_to == VertRelTo::Paper`
- `shape.drawing().and_then(|d| d.text_box.as_ref()).is_some()`

이 조건을 만족하는 Paper 기준 글상자 host 문단은 visible text가 없더라도 한컴처럼 host 문단의
line advance를 본문 flow에 예약한다.

예상 형태:

```rust
Control::Shape(shape) => {
    let cm = shape.common();
    if cm.treat_as_char || !matches!(cm.text_wrap, TextWrap::InFrontOfText) {
        return false;
    }
    matches!(cm.vert_rel_to, VertRelTo::Para)
        || (matches!(cm.vert_rel_to, VertRelTo::Paper)
            && shape.drawing().and_then(|d| d.text_box.as_ref()).is_some())
}
```

### 3.2 수정하지 않는 항목

- `src/renderer/typeset.rs`의 pagination cursor는 이번 원인 경로가 아니므로 변경하지 않는다.
- `src/renderer/layout/shape_layout.rs`의 Paper-relative 위치 계산은 변경하지 않는다.
- HWP5/HWPX parser lowering은 변경하지 않는다.
- `samples/hwpx/2026_oss_rst.hwpx`는 별도 1페이지 문서로 판정했으므로 이번 expected에 맞추지 않는다.
- text_box 없는 Paper 기준 장식 도형과 Picture는 기존 비예약 경로를 유지한다.

## 4. 기대 효과

수정 후 `samples/2026_oss_rst.hwp` 1페이지:

- 제목 글상자 배경은 기존처럼 `y≈133px` 근처에 유지된다.
- 큰 표 `pi=1 ci=0`의 상단 bbox y는 `132.3px`에서 `≈153.4px`로 내려간다.
- 표 상단선이 제목 흰 배경 중앙을 지나 정답 PDF와 같은 overlap 구조가 된다.

Paper 기준 도형 자체는 `shape_layout.rs`에서 paper/page 기준으로 위치가 계산되므로, host 문단
advance를 보존해도 글상자 자체가 같이 내려가지는 않아야 한다.

## 5. 테스트 계획

### 5.1 신규 회귀 테스트

신규 통합 테스트 파일:

- `tests/issue_1418_textbox_table_overlap.rs`

검증 방식:

1. `samples/2026_oss_rst.hwp`를 `HwpDocument::from_bytes`로 로드한다.
2. `build_page_render_tree(0)`로 1페이지 render tree를 생성한다.
3. `RenderNodeType::Table` 중 `para_index=Some(1)`, `control_index=Some(0)`인 큰 표 bbox를 찾는다.
4. 표 상단 y가 정답 기준 `153.4px` 근처인지 검증한다.
5. `RenderNodeType::Rectangle` 또는 `RenderNodeType::TextBox`에서 제목 글상자 bbox가 기존 기준
   `y≈133px` 근처에 남아 있는지 보조 검증한다.

예상 assertion:

- 큰 표 top: `153.4 ± 2.0px`
- 제목 글상자 top: `133.4 ± 2.0px`

정확한 노드 식별은 구현 중 render tree 구조를 확인하여 다음 중 더 안정적인 쪽을 사용한다.

- `TableNode { para_index: Some(1), control_index: Some(0) }`
- `TextBox` 노드 bbox 또는 `Shape` 자식 Rectangle bbox 중 `x≈271.9`, `w≈249.4`인 제목 배경

### 5.2 관련 회귀 테스트

필수 실행:

```bash
cargo fmt --check
cargo test --test issue_1418_textbox_table_overlap -- --nocapture
cargo test --test issue_775 -- --nocapture
cargo test --test issue_919_textbox_hit_test -- --nocapture
cargo test --test issue_1052_footnote_in_textbox -- --nocapture
```

상황에 따라 추가 실행:

```bash
cargo test --test issue_716 -- --nocapture
cargo test --test issue_986 -- --nocapture
cargo check --lib
```

## 6. 시각 검증 계획

수정 후 다음 산출물을 생성한다.

```bash
target/debug/rhwp export-svg samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-final-hwp --debug-overlay
target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-final-render-tree
target/debug/rhwp dump-pages samples/2026_oss_rst.hwp -p 0
```

시각 판정 대상:

- `output/poc/task1418-final-hwp/2026_oss_rst_001.svg`

기대 결과:

- 제목 글상자 흰 배경은 현재와 같은 위치에 남는다.
- 큰 표 상단선이 글상자 배경 위쪽이 아니라 중앙을 통과한다.
- HWP 전체 페이지 수는 6페이지를 유지한다.

## 7. 리스크

| 리스크 | 대응 |
|---|---|
| Paper 기준 장식 도형이 의도치 않게 본문을 미는 회귀 | `Shape + text_box + InFrontOfText`에만 확장하고 Picture/text_box 없는 도형은 제외 |
| 기존 Para 기준 InFront host 회귀 | 기존 `VertRelTo::Para` 조건은 그대로 유지 |
| 글상자 자체가 함께 내려가는 회귀 | render tree 테스트에서 제목 글상자 bbox y를 함께 검증 |
| 표 pagination 정책 회귀 | 수정 범위를 layout host advance 판정에 한정하고 `dump-pages`/render tree를 함께 확인 |

## 8. 승인 요청

위 계획대로 구현을 진행한다. 승인 후에만 `src/renderer/layout.rs`와
`tests/issue_1418_textbox_table_overlap.rs`를 수정한다.
