# Stage 2 완료 보고서 — Task M100-1418 원인 분리

- 이슈: #1418
- 제목: 첫 페이지 글상자와 표 상단 중앙 overlap 정합
- 작성일: 2026-06-16
- 브랜치: `local/task_m100_1418`
- 참조 PDF: `pdf-large/hwpx/2026_oss_rst.pdf`
- 참조 PDF SHA-256: `bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e`

## 1. 기준 정리

교체된 참조 PDF는 총 6페이지 문서다. 이번 overlap 정합 기준은 Stage 1에서 산출한 것처럼
그 참조 PDF의 1페이지와 `samples/2026_oss_rst.hwp` 1페이지 조합으로 유지한다.

Stage 1의 수치 기준:

- 제목 글상자 흰 배경: `x≈271.7px`, `y≈133.1px`, `w≈249.5px`, `h≈38.0px`
- 정답 PDF의 큰 표 상단선: `y≈153.4px`
- 현재 rhwp의 큰 표 상단선: `y=132.3px`
- 차이: 약 `21.1px`
- 직전 빈 host 문단 `pi=0` 높이: `21.3px`

즉 글상자 자체의 paper-relative 위치는 거의 맞고, 뒤따르는 큰 표가 빈 host 문단의
line advance 한 줄만큼 위로 당겨져 있다.

## 2. 재현 변형

`--respect-vpos-reset` 옵션을 켜도 첫 페이지 배치와 `dump-pages` 결과는 변하지 않았다.

```text
FullParagraph  pi=0  h=21.3
Shape          pi=0 ci=2  wrap=InFrontOfText tac=false
Table          pi=1 ci=0  1x1  566.9x852.0px  wrap=TopAndBottom tac=true
```

따라서 이번 결함은 vpos reset 옵션으로 갈리는 경로가 아니라, render tree 배치 단계에서
빈 floating shape host 문단의 진행량을 어떻게 반영하는지의 문제로 좁혀진다.

관련 산출물:

| 구분 | 경로 |
|---|---|
| vpos reset render tree | `output/poc/task1418-render-tree-hwp-vpos/render_tree_001.json` |
| vpos reset SVG | `output/poc/task1418-stage2-hwp-vpos/2026_oss_rst_001.svg` |
| TAC cursor debug render tree | `output/poc/task1418-stage2-debug-tac/render_tree_001.json` |
| table drift debug render tree | `output/poc/task1418-stage2-debug-drift/render_tree_001.json` |

## 3. 디버그 로그

`RHWP_DEBUG_TAC_CURSOR=1`:

```text
TAC_CURSOR  FullPara pi=0 y_in=132.3 y_out=132.3 dy=0.0 was_tac=false
TAC_CURSOR  Shape pi=0 ci=2 y_in=132.3 y_out=132.3 dy=0.0 was_tac=false
TAC_CURSOR  Table pi=1 ci=0 y_in=132.3 y_out=994.1 dy=861.8 was_tac=true
```

`RHWP_TABLE_DRIFT=1`:

```text
LAYOUT_Y: page=0 sec=0 ord=0 pi=0 y_after=132.3 (body_top=132.3)
LAYOUT_Y: page=0 sec=0 ord=1 pi=0 y_after=132.3 (body_top=132.3)
WHOLE_TABLE_Y: pi=1 sec=0 tac=true table_y_start=132.3 table_y_end=986.1 table_h=853.8 para_y=132.3 table_y_before=132.3
LAYOUT_Y: page=0 sec=0 ord=2 pi=1 y_after=994.1 (body_top=132.3)
```

해석:

1. 페이지네이션에는 `FullParagraph pi=0 h=21.3`이 존재한다.
2. 실제 layout pass에서는 `FullPara pi=0`의 `y_out`이 `y_in`과 같아 진행량이 0이 된다.
3. 그 결과 다음 `Table pi=1 ci=0`이 `body_top=132.3px`에서 시작한다.
4. 참조 PDF처럼 `153.4px`에서 시작하려면 `pi=0`의 `21.3px` line advance가 보존되어야 한다.

## 4. 코드 경로

주요 원인 후보는 `src/renderer/layout.rs`의 빈 floating shape host 문단 fast path다.

`src/renderer/layout.rs:359`:

```rust
fn para_has_visible_textless_float_shape_item(
    page_content: &PageContent,
    para: &Paragraph,
    para_index: usize,
) -> bool
```

이 함수는 visible text가 없고, 같은 페이지 content에 non-TAC `Picture`/`Shape`가 있으면
해당 문단을 floating shape host로 판정한다.

`src/renderer/layout.rs:392`:

```rust
fn textless_infront_para_host_requires_line_advance(para: &Paragraph) -> bool
```

현재 조건은 다음 세 가지다.

- `treat_as_char == false`
- `TextWrap::InFrontOfText`
- `VertRelTo::Para`

문제 샘플의 제목 글상자는 `TextWrap::InFrontOfText`, `treat_as_char=false`이지만
세로 기준이 `VertRelTo::Paper`다. 그래서 이 함수가 `false`를 반환하고,
아래 layout branch에서 `y_offset`을 그대로 반환한다.

`src/renderer/layout.rs:4437`:

```rust
if para_has_visible_textless_float_shape_item(page_content, para, *para_index) {
    para_start_y.entry(*para_index).or_insert(y_offset);
    if textless_infront_para_host_requires_line_advance(para) {
        let advance = paragraph_line_advance_px(
            para,
            composed.get(*para_index),
            self.dpi,
        );
        return (y_offset + advance, false);
    }
    return (y_offset, false);
}
```

따라서 Stage 2 기준 원인 후보는 다음이다.

- pagination 단계: 빈 host 문단 높이 `21.3px`를 알고 있다.
- layout 단계: Paper 기준 `InFrontOfText` 글상자 host 문단을 line advance 예약 대상에서 제외한다.
- 결과: 제목 글상자는 제 위치에 있으나, 뒤 표가 `21.3px` 위로 시작한다.

## 5. 글상자 위치 영향

Paper 기준 글상자는 host 문단 y가 아니라 paper/page 기준 계산으로 위치가 잡힌다.

`src/renderer/layout/shape_layout.rs:2915`:

```rust
pub(crate) fn calc_shape_bottom_y(
    &self,
    common: &CommonObjAttr,
    col_area: &LayoutRect,
    body_area: &LayoutRect,
) -> (f64, f64)
```

여기서 `VertRelTo::Paper`는 `ref_y=0.0` 기반으로 계산한다. 또한 shape pass는
`para_start_y`를 넘겨도 Paper/Page 기반 여부를 별도로 판단한다.

그래서 `pi=0`의 line advance를 보존하더라도 제목 글상자 자체를 아래로 밀 가능성은 낮다.
기대 효과는 뒤따르는 flow/TAC 표의 시작 y만 `21.3px` 내려가는 것이다.

## 6. HWPX 샘플 판정

`samples/hwpx/2026_oss_rst.hwpx`는 1페이지 문서이며 첫 문구가
`출품작 중복수혜 여부 확인서`다. 반면 참조 PDF와 HWP primary fixture는 6페이지 문서이며
1페이지 제목이 `< 결과보고서 작성 안내 >`다.

`dump-pages samples/hwpx/2026_oss_rst.hwpx -p 0`에서도 문제의 `pi=0 ci=2`
paper-relative `InFrontOfText` 글상자 구조가 나타나지 않는다. HWPX 샘플은 이번 overlap
수정의 직접 기준이 아니라 별도 fixture 또는 보조 비교 대상으로 둔다.

## 7. 구현 계획 방향

Stage 3 구현 계획서에서는 소스 수정 범위를 다음처럼 좁히는 것이 타당하다.

1. `src/renderer/layout.rs`의 `textless_infront_para_host_requires_line_advance` 조건을 보강한다.
2. 기존 `VertRelTo::Para` + `InFrontOfText` host 예약 동작은 유지한다.
3. `VertRelTo::Paper`인 경우는 전역 확대하지 않고, `Control::Shape`이면서 `drawing().text_box.is_some()`인 글상자에 한정하는 방향을 우선 검토한다.
4. `BehindText`, 일반 배경/워터마크성 Picture, text_box 없는 장식 도형은 기존 비예약 경로를 유지한다.
5. 회귀 테스트는 HWP primary fixture의 첫 페이지 큰 표 상단 y가 참조 기준 `153.4px` 근처로 내려오는지 확인하고, 기존 글상자/TextBox 관련 테스트를 함께 돌린다.

소스 코드는 아직 수정하지 않았다.

## 8. 실행 명령

```bash
target/debug/rhwp dump-pages samples/2026_oss_rst.hwp -p 0 --respect-vpos-reset
target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-render-tree-hwp-vpos --respect-vpos-reset
target/debug/rhwp export-svg samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-stage2-hwp-vpos --respect-vpos-reset
RHWP_DEBUG_TAC_CURSOR=1 target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-stage2-debug-tac
RHWP_TABLE_DRIFT=1 target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-stage2-debug-drift
target/debug/rhwp dump-pages samples/hwpx/2026_oss_rst.hwpx -p 0
```
