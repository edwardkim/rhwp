# 구현계획서 v2 — Task #409 후속 (페이지네이션 chart 높이 누적)

## 배경

이슈 #409 의 v1 (commits d88f204~0a59329) 으로 `src/renderer/layout.rs` 의 `prev_has_overlay_shape` 가드를 확장하여 21페이지 2x1 표가 차트 바로 아래로 정상 위치 복원. 그러나 잔여:

- `pi=191` 헤딩 ("(4) 연령대별 구매…") 과 `pi=192` (10x5 빈 표) 가 21페이지에 묶임 → layout overflow 247.9px → 22페이지 SVG 에서 누락 (PDF 와 불일치)

## 근본 원인 (페이지네이션 측)

`src/renderer/typeset.rs::typeset_section` (line 622-630) 컨트롤 루프가 비-TAC `Picture/Shape` 의 높이를 `current_height` 에 더하지 않음:

```rust
Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
    if !has_table {
        st.current_items.push(PageItem::Shape { ... });
        // ← 높이 누적 없음
    }
}
```

`format_paragraph` 도 `line_segs` 의 `lh + ls` 만 누적 → 차트 (170×111mm = 419.6px) 의 높이가 페이지네이션에 미반영 → pagination 추정 used=803.3px (vs 실제 layout y=1275.9px).

## 수정 방향

`typeset_section` 의 컨트롤 루프에서 비-TAC + TopAndBottom + vert_rel_to=Para 인 `Picture/Shape` 의 경우 `common.height` 를 `current_height` 에 추가.

```rust
Control::Picture(pic) if !pic.common.treat_as_char
    && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(pic.common.vert_rel_to, VertRelTo::Para) => {
    st.current_items.push(...);
    let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
    let outer_bottom = hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
    st.current_height += h + outer_bottom;
}
```

Shape 도 동일 분기 추가. 다른 wrap 모드 (InFrontOfText/BehindText/Square/Tight) 는 건드리지 않음.

## 단계 구성 (2단계 추가)

---

### Stage 4: 페이지네이션 chart 높이 누적 + 검증

**작업**:
1. `src/renderer/typeset.rs:622-630` controls 루프 분기 확장
   - `Control::Picture` (non-TAC, TopAndBottom, vert=Para) → `current_height += common.height + margin.bottom`
   - `Control::Shape` (non-TAC, TopAndBottom, vert=Para) → 동일
2. 빌드 + 21/22 페이지 SVG 재생성하여 PDF 와 비교
   - 21페이지: 차트 + 2x1 표만 (변동 없어야 함)
   - 22페이지: pi=191 헤딩 + pi=192 (10x5 표) 정상 표시
3. LAYOUT_OVERFLOW 잔여 0 확인 (pi=192 overflow 247.9px 제거)

**완료 조건**:
- 22페이지에 pi=191 헤딩 + pi=192 표가 PDF 와 동일하게 출력
- 21페이지 OVERFLOW 0건

**산출물**: `mydocs/working/task_m100_409_stage4.md`, `src/renderer/typeset.rs` (변경)

---

### Stage 5: 회귀 검증 + 통합 최종 보고서

**작업**:
1. `cargo test --release` 전체 통과
2. 10개 샘플 LAYOUT_OVERFLOW 비교 (v1 후 vs v2 후)
3. PR 초안 (`mydocs/report/task_m100_409_pr.md`) 업데이트하여 v1 + v2 통합 커버
4. 최종 보고서 (`task_m100_409_report_v2.md`) 작성

**완료 조건**:
- 1023 lib + 6 svg_snapshot + 통합 테스트 100% 통과
- 다른 샘플 무회귀
- PR 초안 업데이트 완료

**산출물**: `mydocs/working/task_m100_409_stage5.md`, `mydocs/report/task_m100_409_report_v2.md`, `mydocs/report/task_m100_409_pr.md` (업데이트)

---

## 위험 요소

| 위험 | 대응 |
|------|------|
| outer_margin (top/bottom) 추가가 layout 과 어긋남 | layout 의 `calc_shape_bottom_y` 가 `margin.bottom` 만 더함 → 동일하게 적용 |
| 다른 페이지네이션 케이스에 부작용 (e.g. 차트 한 장만 있는 페이지) | 6개 샘플 LAYOUT_OVERFLOW 동일성 검증 + svg_snapshot 가드 |
| 후속 paragraph (pi=173) 가 차트 다음 line 으로 들어가야 하는데 chart 높이 추가하면 빈 paragraph 일 시 페이지 종료 트리거 가능 | layout 의 vpos 흐름과 `compute_body_wide_top_reserve_for_para` 패턴 따라 안전 검증 |

## 커밋 분리

- Stage 4: `Task #409 Stage 4: 페이지네이션 chart 높이 누적 (typeset.rs)`
- Stage 5: `Task #409 Stage 5: 회귀 검증 + 통합 최종 보고서`
