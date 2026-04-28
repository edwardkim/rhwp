# Task #402 최종 결과보고서

## 이슈

샘플 `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 의 7쪽 SVG 출력에서 빈 12행 표(pi=57 ci=0)와 파이 차트(pi=57 ci=1)가 거의 같은 y 좌표(578~581)에서 시작하여 **표 위에 파이 차트가 겹쳐** 그려짐. PDF 원본은 7쪽에 표만 두고 파이 차트는 다음 쪽으로 분할.

GitHub Issue: [#402](https://github.com/edwardkim/rhwp/issues/402)

## 원인

같은 paragraph(pi=57)에 inline 컨트롤(`treat_as_char=true`) 2개가 서로 다른 line_seg(ls[0]=표, ls[1]=그림)에 배치된 구조에서 **두 가지 누락**이 동시에 작용:

1. **레이아웃의 y 좌표 결정 누락 (`layout.rs::layout_shape_item`)**
   - 그림의 `pic_y`가 `para_start_y[para_index]` 단일값에 고정.
   - 표가 먼저 처리되어 `y_offset`은 진행되었지만 `para_start_y`는 paragraph 시작 y(=ls[0] 시작)에 머무름.
   - → 그림이 표 시작 위치에 겹쳐 그려짐.

2. **페이지네이션의 높이 누적 누락 (`typeset.rs::typeset_table_paragraph`)**
   - inline TAC 그림이 `PageItem::Shape`로 push만 되고 `current_height` 누적 없음.
   - 결과적으로 그림 line의 높이(369px)가 페이지 사용량에 반영되지 않아 페이지 분할이 트리거되지 않음.
   - → 7쪽에 표+그림이 함께 스케줄되어 SVG viewBox 초과·후속 항목 위치 어긋남.

## 수정

### 1. `src/renderer/layout.rs::layout_shape_item` (line 2513~)

`layout_table_item`(line 1995~)이 가진 "TAC 컨트롤이 다른 TAC 다음에 올 때 `para_start_y` 갱신" 패턴을, 그림에 맞게 적용. 단순 비교(`y_offset > existing + 1.0`)만으로는 단일 inline 그림 케이스(pi=51 ci=0, FullParagraph + Shape 구조)를 잘못 갱신하므로, **`control_index`보다 앞선 인덱스에 같은 paragraph의 TAC 컨트롤이 존재하는지**를 추가 조건으로 사용.

```rust
let has_prior_tac_in_para = paragraphs.get(para_index)
    .map(|p| p.controls.iter().take(control_index).any(|c| match c {
        Control::Table(t) => t.common.treat_as_char,
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => s.common().treat_as_char,
        _ => false,
    }))
    .unwrap_or(false);
if has_prior_tac_in_para {
    let needs_update = para_start_y.get(&para_index)
        .map(|&existing| y_offset > existing + 1.0)
        .unwrap_or(true);
    if needs_update {
        para_start_y.insert(para_index, y_offset);
    }
} else {
    para_start_y.entry(para_index).or_insert(y_offset);
}
```

### 2. `src/renderer/typeset.rs::typeset_table_paragraph` (line 1123~)

기본 페이지네이션 엔진은 `paginate_with_measured`(engine.rs)가 아닌 **`TypesetEngine::typeset_section`(typeset.rs)** 임을 진단 로깅으로 확인. `RHWP_USE_PAGINATOR=1` 일 때만 engine.rs로 fallback. 따라서 typeset.rs에 동일 수정 적용.

inline TAC 그림이 같은 paragraph에 선행 TAC 컨트롤을 가질 때:
- 자기에 해당하는 `line_segs[prior_tac_count]` 의 `line_height + line_spacing`을 line 높이로 산출.
- `current_height + line_h > available_height + 0.5` 이면 `advance_column_or_new_page()` 호출 → 다음 페이지로 분할.
- 분할 후 `PageItem::Shape` 를 push하고 `current_height += line_h`.

```rust
Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
    let tac_separate_line_h: Option<f64> = match ctrl {
        Control::Picture(p) if p.common.treat_as_char => Some(()),
        Control::Shape(s) if s.common().treat_as_char => Some(()),
        _ => None,
    }.and_then(|_| {
        let prior_tac_count = para.controls.iter().take(ctrl_idx).filter(|c| match c {
            Control::Table(t) => t.common.treat_as_char,
            Control::Picture(p) => p.common.treat_as_char,
            Control::Shape(s) => s.common().treat_as_char,
            _ => false,
        }).count();
        if prior_tac_count == 0 { return None; }
        para.line_segs.get(prior_tac_count).map(|seg| {
            let lh = hwpunit_to_px(seg.line_height, self.dpi);
            let ls_extra = if seg.line_spacing > 0 {
                hwpunit_to_px(seg.line_spacing, self.dpi)
            } else { 0.0 };
            lh + ls_extra
        })
    });
    if let Some(line_h) = tac_separate_line_h {
        if !st.current_items.is_empty()
            && st.current_height + line_h > st.available_height() + 0.5
        {
            st.advance_column_or_new_page();
        }
    }
    st.current_items.push(PageItem::Shape { para_index, control_index });
    if let Some(line_h) = tac_separate_line_h {
        st.current_height += line_h;
    }
}
```

## 검증

### 타겟 샘플 (이슈 재현 대상)
- 명령: `cargo run --bin rhwp -- export-svg "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"`
- 페이지 수: 27 → 30 (분할에 의한 정상적 증가)
- 7쪽 (qlmanage 렌더): 막대차트 + 빈 박스 + "(5) 기부 금액별 기부 건수" + 빈 12행 표 → **PDF 7쪽과 일치, 파이 차트 겹침 없음** ✅
- 8쪽 (qlmanage 렌더): "기부 금액 구간별 기부 건수 비중" 파이 차트 + 범례 + 빈 박스 → **파이 차트 정상 배치** ✅

### 회귀 테스트
- `cargo test --quiet`: **1023 passed, 0 failed, 1 ignored**

### 샘플 LAYOUT_OVERFLOW 카운트 비교 (수정 전 vs 후)

| 샘플 | 수정 전 | 수정 후 | 차이 |
|------|--------:|--------:|------|
| aift.hwp | 3 | 3 | 0 |
| biz_plan.hwp | 0 | 0 | 0 |
| endnote-01.hwp | 0 | 0 | 0 |
| equation-lim.hwp | 0 | 0 | 0 |
| exam_math.hwp | 0 | 0 | 0 |
| exam_kor.hwp | 30 | 30 | 0 |
| footnote-01.hwp | 0 | 0 | 0 |
| group-box.hwp | 0 | 0 | 0 |
| hwp_table_test.hwp | 0 | 0 | 0 |
| field-01.hwp | 0 | 0 | 0 |

10개 대표 샘플에서 회귀 없음. (대상 샘플 자체는 4 → 0으로 개선)

### Clippy
- `cargo clippy`는 `commands/table_ops.rs:1007`, `commands/object_ops.rs:298`의 `panicking_unwrap` 2건 에러를 보고하나, **둘 다 base 브랜치(0054a27)에 이미 존재하는 기존 문제**로 이번 수정과 무관함을 `git stash` + 비교 빌드로 확인. 본 타스크 범위 밖.

## 검증 기준 (수행계획서) 충족 여부

| # | 기준 | 결과 |
|---|------|:----:|
| 1 | 7쪽 SVG에 파이 차트가 더 이상 표시되지 않거나, 표와 겹치지 않음 | ✅ |
| 2 | 7쪽 표 영역이 PDF와 동일한 위치에 그려짐 | ✅ |
| 3 | 파이 차트가 다음 페이지(8쪽)로 흘러감 | ✅ |
| 4 | 기존 회귀 테스트 모두 통과 | ✅ |

## 비범위 / 후속 과제

- **engine.rs(`paginate_with_measured`) 동일 누락**: 현재 default 경로가 typeset이므로 본 타스크 범위에서 손대지 않음. `RHWP_USE_PAGINATOR=1` fallback에서는 동일 문제가 잔존. 별도 이슈로 다루는 것이 적절.
- **TAC 컨트롤이 line_seg와 1:1 매핑되지 않는 일반 케이스**: 현재 휴리스틱(`prior_tac_count` 인덱스로 line_seg 매핑)은 표/그림/도형이 텍스트 없이 line별로 1개씩 배치되는 경우에 정확. 텍스트 줄과 inline shape가 섞이는 더 복잡한 케이스에는 추가 검토가 필요.

## 커밋 이력

- `0054a27` Task #402 Stage 1: 진단 로깅으로 가설 확정
- `38bea10` Task #402 Stage 2: inline TAC 그림 페이지네이션 수정
- (Stage 3 본 보고서 + orders 갱신)
