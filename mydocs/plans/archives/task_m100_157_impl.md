# Task #157 구현계획서: 비-TAC wrap=위아래 표 out-of-flow 배치

> 구현계획서 | 2026-04-23
> Issue: [#157](https://github.com/edwardkim/rhwp/issues/157)
> Milestone: v1.0.0
> Branch: `local/task157`

---

## 1. 근본 원인 (조사 결과)

### 수행계획서의 원인 분석 수정

수행계획서에서 "페이지네이터가 비-TAC TopAndBottom+Para 표를 in-flow로 처리"를 근본 원인으로 지목했으나, **코드 조사 결과 페이지네이터는 정상**이다.

### 실제 근본 원인: `layout.rs:1449–1454` vpos 기준점 리셋

`layout.rs`의 레이아웃 루프에서 **어떤 Table/Shape 아이템이든** 처리 후 vpos 기준점 두 개를 초기화한다:

```rust
// layout.rs:1449–1454 (현재 코드)
if was_tac || is_table_or_shape {
    vpos_page_base = None;
    vpos_lazy_base = None;
}
```

vert=Para 자리차지 표(TopAndBottom, non-TAC)는 앵커 문단에 attach되므로 후속 문단의 vpos 교정 기준점을 초기화하면 안 된다. 초기화가 일어나면:

1. 한컴이 Para-float 표를 기준으로 후속 문단 vpos를 기록한 값(Pi=8~25, vpos 큰 점프)이 잘못된 `lazy_base`로 교정됨
2. Pi=25(표 앵커)의 vpos 교정 결과 `anchor_y ≈ 939.2px`로 상승 (정상: 768.4px)
3. `compute_table_y_position` 내 `body_bottom` clamp → 표가 894.7px에 고정
4. `layout_table` 반환값(894.7) + 후행 `line_spacing`(9.6) = `y_offset 1102.9px`
5. `col_bottom 1093.3px` 초과 → `LAYOUT_OVERFLOW 9.6px`

### 수치 검증

| 항목 | 버그 상태 | 수정 후 |
|------|-----------|---------|
| lazy_base | 63965 HU (잘못됨) | 사용 안 함 → page_base 77497 |
| Pi=25 anchor_y | 939.2 px | 768.4 px |
| table_y (raw) | 788.0 → clamp 894.7 | 788.0 (clamp 불필요) |
| table_bottom | 1093.3 | 986.8 |
| 최종 y_offset | 1102.9 | 996.4 |
| LAYOUT_OVERFLOW | 9.6 px | 0 px |

---

## 2. 구현 단계

### 단계 1 — `layout.rs` vpos 기준점 리셋 예외 처리

**파일**: `src/renderer/layout.rs`  
**위치**: lines 1449–1454

Para-relative float 표(vert=Para, TopAndBottom, non-TAC)는 vpos 기준점 초기화 대상에서 제외한다.

**변경 전:**
```rust
if was_tac || is_table_or_shape {
    vpos_page_base = None;
    vpos_lazy_base = None;
}
```

**변경 후:**
```rust
let is_para_float_table = if let PageItem::Table { para_index, control_index } = item {
    paragraphs
        .get(*para_index)
        .and_then(|p| p.controls.get(*control_index))
        .map(|c| {
            matches!(
                c,
                Control::Table(t)
                if !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                    && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
            )
        })
        .unwrap_or(false)
} else {
    false
};

if was_tac || (is_table_or_shape && !is_para_float_table) {
    vpos_page_base = None;
    vpos_lazy_base = None;
}
```

### 단계 2 — `engine.rs` effective_table_height 보정 (방어 코드)

**파일**: `src/renderer/pagination/engine.rs`  
**위치**: lines 1099–1117 (`paginate_table_control` 내)

Para-relative float 표가 페이지 body 내에 완전히 들어오는 경우 `effective_table_height`를 0으로 처리하여 불필요한 `split_table_rows` 호출을 방지한다.

현재 코드 (lines 1099–1117):
```rust
// Para 상대 계산
let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
let effective_table_height = (abs_bottom - st.current_height)
    .max(effective_height + host_spacing);
```

**변경 후:**
```rust
// Para 상대 계산
let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
let effective_table_height = if abs_bottom <= base_available_height + 0.5 {
    // 표가 body 범위 내에 완전히 들어옴 → flow height 기여 없음
    0.0
} else {
    (abs_bottom - st.current_height).max(effective_height + host_spacing)
};
```

> **비고**: issue_157.hwpx 재현 파일에서는 단계 1만으로 수정이 완료된다.  
> 단계 2는 `base_available_height` 경계 케이스(Para-float 표가 body 내에 들어오는데 fit 체크 실패하는 엣지 케이스)를 방어하기 위한 추가 보완이다.

### 단계 3 — 검증

1. `cargo test` — 전체 테스트 통과 확인
2. `rhwp dump-pages samples/hwpx/issue_157.hwpx -p 1` — Pi=25 표 `y=788.0` 확인, LAYOUT_OVERFLOW 없음
3. `rhwp export-svg samples/hwpx/issue_157.hwpx --debug-overlay` — 표와 텍스트 비중첩 시각 확인
4. `hwpspec.hwp 93페이지` (#103 케이스) — 비정상 gap 해소 확인
5. Golden SVG 등록: `tests/golden_svg/issue-157/`

---

## 3. 수정 파일 목록

| 파일 | 단계 | 변경 내용 |
|------|------|-----------|
| `src/renderer/layout.rs` | 1 | vpos 기준점 리셋 예외 처리 (~10줄 추가) |
| `src/renderer/pagination/engine.rs` | 2 | `effective_table_height = 0.0` 조건 추가 (~5줄) |
| `tests/golden_svg/issue-157/` | 3 | golden SVG 신규 등록 |

---

## 4. 검증 기준

| 항목 | 합격 조건 |
|------|-----------|
| cargo test | 941개 전체 통과, 신규 실패 없음 |
| issue_157.hwpx p.2 | LAYOUT_OVERFLOW 로그 없음 |
| issue_157.hwpx p.2 | Pi=25 표 y≈788px, 텍스트와 비중첩 |
| hwpspec.hwp p.93 | 비정상 gap 없음 |
| 기존 TopAndBottom 표 | regression 없음 |

---

## 5. 참고

- 수행계획서: `mydocs/plans/task_m100_157.md`
- 수행계획서 원인 분석 수정: 페이지네이터는 정상. 원인은 `layout.rs` vpos 리셋.
- 조사 기록: `mydocs/orders/20260422.md`
