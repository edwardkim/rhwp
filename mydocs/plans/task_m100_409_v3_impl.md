# 구현계획서 v3 — Task #409 후속 (atomic TAC top-fit 시멘틱)

## 배경

v1 (layout vpos 가드) + v2 (pagination chart 누적) 후 잔여 결함:

- 23페이지 하단의 차트 (pi=208, TAC Picture, line_seg.lh=23700 HU = 316px) 가 SVG 에서 24페이지로 밀림 (PDF 와 불일치)
- `dump-pages`: pi=208 fit 검사에서 `619.3 + 316 = 935.3 > 933.5` (1.8px 초과) → split 분기 → 1-line atomic 이라 못 쪼개짐 → next page

## 근본 원인

HWP/PDF 는 atomic (분할 불가능한 단일 라인 TAC Picture/Shape) 항목에 대해 **top-fit** 시멘틱 사용:
- 항목 시작 y 가 본문 안이면 현재 페이지에 배치
- 항목 끝이 본문을 약간 초과하면 하단 여백(15mm = 56.7px)으로 흘림 허용
- footer 영역만 침범하지 않으면 OK

23페이지 케이스:
- pi=208 차트 시작 y = 721.37 (vpos 1460593 → +626.87px from body_top)
- 차트 끝 y = 1037.37 (chart 316px)
- body_bottom = 1028 → 9.37px 초과
- 하단 여백 끝 = 1085 (1028+56.7) → 차트 끝(1037) 이 안에 있음 ✓ → HWP 가 23페이지에 배치

우리 페이지네이션 (`typeset.rs:901, 936-940`) 은 strict bottom-fit:
```rust
if st.current_height + fmt.height_for_fit <= available {
    // place
}
// 못 들어감 → split → 1-line atomic 이라 advance_page
```

## 수정 방향

`typeset.rs::typeset_paragraph` split 분기 (line 936-940) 에 **atomic TAC top-fit** 분기 추가:

```rust
let line_count = fmt.line_heights.len();
let is_atomic_tac = line_count == 1 && para.controls.iter().any(|c| match c {
    Control::Picture(p) => p.common.treat_as_char,
    Control::Shape(s) => s.common().treat_as_char,
    _ => false,
});

let advance = if (st.current_height >= available || remaining < first_line_h)
    && !st.current_items.is_empty()
{
    if is_atomic_tac && st.current_height < available {
        // top-fit: 시작점이 본문 안이면 atomic TAC 항목 그대로 배치
        // (하단 일부가 하단 여백으로 흘러도 HWP 시멘틱에 부합)
        false
    } else {
        true
    }
} else {
    false
};
if advance {
    st.advance_column_or_new_page();
}
```

또는 더 명료하게 fit 분기 자체에 추가:
```rust
if st.current_height + fmt.height_for_fit <= available {
    // 일반 bottom-fit: place
} else if is_atomic_tac && st.current_height < available - safety && !st.current_items.is_empty() {
    // atomic TAC top-fit: place with bottom margin spillover
    // (HWP 시멘틱 — 분할 불가 항목은 시작점만 본문 안이면 현재 페이지 배치)
} else {
    // split 또는 advance
}
```

## 단계 구성 (2단계)

### Stage 6: atomic TAC top-fit 분기 추가 + 검증

**작업**:
1. `src/renderer/typeset.rs::typeset_paragraph` 의 fit 검사에 atomic TAC top-fit 분기 추가
2. 23페이지 SVG 재생성 → PDF 와 비교
3. 24페이지 SVG 재생성 → 차트 빠지고 정상 콘텐츠로 시작 확인
4. LAYOUT_OVERFLOW 변동 확인 (atomic TAC 의 의도된 본문 초과는 허용 범위)

**완료 조건**:
- 23페이지 SVG 에 차트(pi=208) 정상 표시 (PDF 일치)
- 24페이지 SVG 가 정상 후속 콘텐츠로 시작
- 6개 다른 샘플 LAYOUT_OVERFLOW 카운트 무회귀

**산출물**: `mydocs/working/task_m100_409_stage6.md`, `src/renderer/typeset.rs` (변경)

### Stage 7: 회귀 검증 + 통합 최종 보고서 v3

**작업**:
1. `cargo test --release` 전체 통과
2. 6개 다른 샘플 LAYOUT_OVERFLOW 무회귀 확인
3. PR 초안 (`task_m100_409_pr.md`) v3 통합본 업데이트
4. 최종 보고서 (`task_m100_409_report_v3.md`)

**산출물**: `mydocs/working/task_m100_409_stage7.md`, `mydocs/report/task_m100_409_report_v3.md`, `mydocs/report/task_m100_409_pr.md` (업데이트)

## 위험

| 위험 | 대응 |
|------|------|
| atomic TAC top-fit 가 너무 느슨하여 다른 샘플 회귀 | `is_atomic_tac` 조건을 line_count==1 + TAC Picture/Shape 한정. 표 (Table) 는 자체 분할 가능하므로 제외 |
| 본문 하단 초과량이 footer 영역 침범 | top-fit 적용 시 추가 가드: `current_height + fmt.height_for_fit ≤ base_available + bottom_margin_tolerance` 확인. 보수적으로 일단 추가 안 하고 회귀 검증으로 안전성 확인 |

## 커밋 분리

- Stage 6: `Task #409 Stage 6: atomic TAC top-fit 시멘틱 (typeset.rs)`
- Stage 7: `Task #409 Stage 7: 회귀 검증 + 통합 최종 보고서 v3`
