# task_m100_4333 Stage 1 — 인라인 도형 높이 정의 통일

- **이슈**: [#4333](https://github.com/edwardkim/rhwp/issues/4333)
- **브랜치**: `fix/issue-4333-height-measure-inline-shape`
- **분기 기준**: `upstream/devel` `a70797db4`
- **상태**: 정의 통일 완료(동작 중립 확인), 이슈가 지목한 발산 자체는 미해소 — 아래 6절
- **기록일**: 2026-08-11 KST

## 1. 실측 — 이슈의 수치는 재현되지만, 발산의 방향이 반대다

`samples/` 352개 문서에서 `PaginationResult` 기준 **단 최상단 문단**만 골라
`HeightMeasurer::measure_section` 과 scratch `LayoutEngine::layout_partial_paragraph` 를 대조했다.
이슈 본문의 인라인 도형 수치(n=142, 발산 126, 88.7%, 평균 62.4px, 최대 739.3px)가 그대로 재현된다.

여기에 **production 페이지네이션이 실제로 쓰는 정의**를 한 열 더 넣었다.
`DocumentCore::paginate_pass`(`document_core/queries/rendering.rs:4260`)는 기본 경로에서
`TypesetEngine::typeset_section_with_variant` 를 호출하고, 그 높이 정의는
`TypesetEngine::format_paragraph`(`renderer/typeset.rs:14318`)다.
`Paginator::paginate_with_measured_opts`(= `MeasuredSection.paragraphs` 소비자)는
`RHWP_USE_PAGINATOR=1` 일 때만 쓰인다(`rendering.rs:4180`).

| 종류 | n | 발산(>1px) | 비율 | 평균\|차\| | 최대\|차\| |
|---|---:|---:|---:|---:|---:|
| **measure vs render** — Shape tac=true | 142 | 126 | 88.7% | 62.39px | 739.32px |
| **typeset(=조판) vs render** — Shape tac=true | 142 | 126 | 88.7% | 62.39px | 739.32px |

발산 126행 전부에서 `measured == typeset` 이 **소수점까지 일치**한다. 즉 이슈가 "정의가 두 벌"
이라고 지목한 두 값(측정·조판)은 인라인 도형에서 서로 어긋나지 않는다. 어긋나는 쪽은 렌더다.

## 2. 발산의 원인 — 한 줄

최악 사례 `samples/2025 행정업무운영 편람(최종).hwp` 구역4 문단67 (739.32px):

```
seg[0] vpos=0 lh=55449 ls=0 th=55449          ← 저장 줄 높이 = 55449HU = 739.32px
ctrl[0] Shape tac=true common.h=55449 current_h=55449
measured.total_height = 739.32   typeset.total_height = 739.32   render = 0.00
```

`RHWP_DEBUG_PARA_TAC` 계측:

```
TAC_ADV pi=67 line_idx=0 y=0.0 raw_lh=739.3 lh=0.0 ls=0.0 whitespace=true
```

`raw_lh=739.3` 이 `lh=0.0` 이 되는 지점은
**`src/renderer/layout/paragraph_layout.rs:3228` `let font_lh = max_fs * 1.2;`** 이다.

- 이 분기의 게이트(`:3214-3217`)는 `has_tac_shape && !empty_tac_guide_has_explicit_shape_height
  && (cell_ctx.is_none() || max_fs > 0.0) && raw_lh > max_fs * 1.5` 다.
- `max_fs` 는 그 줄의 run 에서만 계산된다(`:3100-3111`, 문자모양 폴백 없음). 도형만 있는 줄은
  run 이 없어 `max_fs == 0` → `font_lh == 0`.
- 탈출구 `empty_tac_guide_has_explicit_shape_height`(`:3197-3209`)는
  `shape_attr().current_height > common().height`(엄격 부등호)를 요구한다. 위 문단은 둘이
  55449 로 같아 발동하지 않는다.

발산 행의 "일정한 32px 격차"도 같은 기제다 — 예: 구역10 문단21 은 `lh=[32.0] ls=[6.67]` 인데
렌더 20.0 = `ls + sa`, 즉 `lh` 32.0 이 통째로 0 이 된 값이다.

## 3. 그런데 렌더의 실제 흐름 위치는 그 0 을 쓰지 않는다

`build_single_column` 은 항목 사이에 `HeightCursor::vpos_adjust`(`renderer/layout.rs:5778`)로
저장 vpos 사다리에 스냅한다. 그래서 `layout_partial_paragraph` 의 고립 advance 는 본문 흐름에서의
문단 높이가 아니다.

이를 실험으로 확인했다. `:3228` 붕괴를 제거하면(도형 전용 줄이 저장 줄 높이를 유지) —

- 인라인 도형 발산: 126/142(88.7%) → **8/142(5.6%)**, 최대 739.32 → 19.61px
- `samples/` 355개 문서 **페이지 수 변화 0건**
- 그러나 `tests/issue_1116.rs` 의 한컴 PDF 핀 2건이 깨진다
  (`sample16_hwp5_page3_heading_positions_follow_lineseg_vpos`: 기대 y≈337.2 → 실제 347.63,
  `sample16_hwp5_2022_page3_bcp_tail_glyph_stays_on_hancom_line`: 881.35 → 891.75).
  두 건 모두 정확히 **+10.4px**, `hwp3-sample16-hwp5.hwp` 구역0 문단71 도형 줄의 후행 줄간격
  (780HU)이다. 그 문단은 저장 사다리 delta(16304−5760=10544HU=140.59px)가 측정값과 정확히 같아
  붕괴를 없애면 advance 가 정답이 되는데, 그 뒤 `vpos_adjust` 의 후방 스냅 클램프가 `spacing_before`
  만큼의 초과를 되돌리지 못한다.

즉 렌더의 문단 높이는 `layout_partial_paragraph` 단독이 아니라
`vpos_adjust ∘ layout_partial_paragraph` 의 합성이고, 붕괴와 스냅이 **짝으로** 한컴 정합을 이루고
있다(`:3219-3227` 주석이 그 사실을 스스로 기록한다). 한쪽만 떼면 정합이 깨진다.
그래서 이 변경은 **채택하지 않았다** — 되돌리는 데 필요한 것은 보상 기계의 재교정이고, 그건
이 이슈 범위 밖이다.

## 4. 실제로 있던 "정의 두 벌" — 인라인 도형의 흐름 높이

발산과 별개로, 같은 속성의 **정의식이 진짜로 두 벌**인 지점이 조판·렌더 경계에 있었다.

```
renderer/typeset.rs:2002           tac_picture_or_shape_height_px  → Shape: common().height
renderer/layout/paragraph_layout.rs:694  tac_picture_or_shape_height_px  → Shape: max(common.height, current_height)
```

이름까지 같은 두 함수가 도형에서 다른 값을 냈다. `samples/` 의 인라인 도형 868개 중
**354개(38개 문서)** 가 `current_height != common.height` 라 사장된 차이가 아니다.
같은 `max(...)` 식은 추가로 다섯 곳에 복제돼 있었다
(`paragraph_layout.rs:300/5320/6373`, `composer/line_breaking.rs:1065/1087`,
`document_core/commands/object_ops/picture.rs:821`, `renderer/layout.rs:10641`).

**조치**

1. `ShapeObject::flow_height_hu()`(`model/shape.rs`) — 유일 정의.
2. `renderer::tac_object_flow_height_px()` — 글자처럼 취급 개체(그림/도형)의 흐름 높이 px.
   조판·렌더가 공유. typeset 쪽 사본 삭제.
3. `renderer::line_owning_tac_object_height_px()` — "이 줄은 인라인 개체가 소유한 줄인가"
   술어. `format_paragraph`(조판)와 `layout_composed_paragraph`(렌더)에 각각 있던 동일 술어를
   하나로.
4. `paragraph_layout.rs:3197` 의 `current_height > common.height` 대리 지표를
   `flow_height_hu() > common().height` 로 표기(동치, 단일 정의 경유).

정의 유일성 확인:

```
$ grep -rn "shape_attr()\.current_height" --include="*.rs" src/
src/model/shape.rs:476:        (self.common().height as i32).max(self.shape_attr().current_height as i32)
```

(그 외 `shape_attr.current_height` 히트는 전부 `Picture` 타입 — 별도 정의
`renderer/layout/utils.rs:65 effective_picture_size` — 또는 파서·직렬화·편집 명령의 필드 접근이다.)

## 5. 동작 중립 확인

조판 쪽 도형 산식이 `common().height` → `flow_height_hu()` 로 **바뀌므로** 페이지네이션에
영향이 갈 수 있다. 실측:

- `samples/` 355개 문서 페이지 수 대조(`upstream/devel a70797db4` vs head): **차이 0건**
- 코퍼스 발산표: 변경 전후 동일 (Shape tac=true 126/142, 최대 739.32px — 3절 참조)
- 인라인 도형을 담은 5쪽 PNG 렌더 전후 **바이트 동일**
  (`draw-group.hwp p0`, `hwp3-sample16-hwp5.hwp p2`,
  `2025 행정업무운영 편람(최종).hwp p163·p290`, `3-09월_교육_통합_2022.hwp p10`)

## 6. 미해소 — 이슈 본체

이슈의 목표(`force_breaks` 되먹임 재가동을 위한 "측정 통일")는 이 Stage 로 달성되지 않았다.
남은 사실을 그대로 기록한다.

- 인라인 도형 발산 88.7% 는 **측정↔조판 사이가 아니라 렌더의 줄높이 붕괴**(`:3228`)다.
  측정과 조판은 이미 소수점까지 같다.
- 이슈가 지목한 `height_measurer.rs` 에 `shape_attr()/current_height` 인지 경로를 추가하는 것은
  방향이 반대다 — 측정은 이미 저장 `LINE_SEG.line_height` 로 도형 높이를 담고 있고, 그 값이
  저장 vpos 사다리 delta 와 일치한다(문단71: 140.59px = 10544HU).
- 붕괴 제거는 3절대로 `vpos_adjust` 보상 기계 재교정을 동반해야 한다. 별도 이슈로 분리한다.
- 이중 계상 재확인: `PageItem::Shape` 방출(`typeset.rs:6355-6385`)은 `st.current_height` 를
  올리지 않고, `pushdown_h`(`typeset.rs:6435`)는 `!treat_as_char && TopAndBottom && VertRelTo::Para`
  게이트라 떠 있는 도형 전용이다. 인라인 도형은 어느 쪽으로도 예산을 두 번 먹지 않는다.
- 떠 있는 쪽 1건(`textbox-under-image.hwp` 구역0 문단0, 21.33px)은 손대지 않았다. 빈 문단 기본값
  폴백(`height_measurer.rs:860`) vs 렌더의 `para_has_visible_textless_float_shape_item` 로,
  기제가 다르다.

## 7. 검증 (완료)

- `cargo fmt --all -- --check` 통과
- `cargo clippy --all-targets -- -D warnings` 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` — `ok` 블록 535, 통과 5760,
  FAILED 0
- `cargo test --profile release-test --features native-skia skia --lib` — 58 passed
- `--test issue_2225_missing_picture_placeholder` — 2 passed
- `--test render_p37_direct_pdf_export` — 4 passed
- `wasm-pack build --target web --out-dir pkg` 성공
- 신규 회귀 시험 `typeset_and_render_agree_on_inline_shape_flow_height`
  (`renderer/typeset.rs`) — 수정 전 RED 실증:

  ```
  assertion `left == right` failed
    left: 32.0            (조판: common.height 2400HU)
   right: 54.21333333333333  (렌더: current_height 4066HU)
  ```


## 후속 이슈 (2026-08-11)

- **[#4604(닫힘)](https://github.com/edwardkim/rhwp/issues/4604)** — 문단 높이가
  `measure_paragraph`(`height_measurer.rs:619`)와 `format_paragraph`(`typeset.rs:14318`)
  두 정의로 남아 있다. 이번에 인라인 도형 축만 통일했고, `treat_as_char == false` 인
  Picture 는 여전히 25/102 대 7/102 로 갈린다.
- **[#4605](https://github.com/edwardkim/rhwp/issues/4605)** —
  `MeasuredSection.paragraphs` 가 프로덕션 페이지네이션에서 죽어 있다. 이 계열에서 수정 위치를
  **두 번** 오도했다(`has_picture` 술어, 분해안 2). `has_picture`/`picture_height` 도
  읽는 곳이 0건이다.
- **[#4606(닫힘)](https://github.com/edwardkim/rhwp/issues/4606)** — 도형과 무관한 PlainText
  문단에도 발산이 288/5128(5.6%, 최대 18.88px) 남아 있다. 도형 발산에 가려 있던 기저선.

`textbox-under-image.hwp` 구역0 문단0 의 float 측 1건(21.33px)은 #4333 본문이 이미 별개
기제로 적어 두어 새로 열지 않았다.


## 정정 (2026-08-12)

Stage 2 실측으로 #4604·#4606 은 **"고치지 않는다"로 닫혔다.** 근거는
`task_m100_4333_stage2.md` 와 각 이슈 코멘트에 있다.
#4605 는 PR #4621 이 처리한다.
