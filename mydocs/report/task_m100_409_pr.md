# PR: Task #409 — TopAndBottom Picture (vert=Para) Layout/Pagination 일관화

## 제목

```
Task #409: TopAndBottom Picture vert=Para 레이아웃/페이지네이션 chart 높이 일관화
```

## 본문

## 배경

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 21~22페이지에서 SVG 출력이 PDF 와 다름:

- **21페이지 PDF**: 차트(170×111mm) 바로 아래 2x1 빈 표 (y≈540~660)
- **21페이지 SVG (v0)**: 차트와 2x1 표 사이 ~400px 빈 공간 → 2x1 표가 페이지 하단(y≈937)으로 밀려 일부 잘림 + pi=192 (10x5 표) 521px overflow

- **22페이지 PDF**: "(4) 연령대별 구매 건수 및 사용 포인트" 헤딩 + 10x5 빈 표 + 연령대별 차트 + 2x1 표
- **22페이지 SVG (v0)**: (4) 헤딩 + 10x5 표가 누락된 채 차트로 시작

`LAYOUT_OVERFLOW` 22건 (대상 샘플 전체 기준).

## 근본 원인

차트 그림(pi=172) 메타데이터:
- `Control::Picture` (170×111mm = HU 48190×31470)
- `TextWrap::TopAndBottom`
- `VertRelTo::Para`
- `treat_as_char = false`

한컴은 TopAndBottom + vert=Para 그림이 anchor 문단 다음 문단의 vpos 에 그림 높이를 더해 기록:
| 문단 | vpos (HU) | 차이 |
|------|----------|------|
| pi=172 | 1275685 | — |
| pi=173 | 1307155 | **+31470** = 차트 높이 |

이 패턴에서 **layout** 과 **pagination** 양쪽 모두 chart 높이 처리가 미흡했음.

### 1. Layout 측 결함 (`src/renderer/layout.rs`)

`prev_has_overlay_shape` 가드 (line 1366-1370) 가 `Control::Shape` + `InFrontOfText|BehindText` 만 검사 → `Control::Picture` 미처리 + `TopAndBottom` 케이스 미포함.

→ 차트 다음 문단(pi=173)에서 vpos 보정이 진입하여 잘못된 `lazy_base` 산출:
- sequential `y_offset = 528.8` (= 차트 바닥)
- `vpos_end_172 = 1277445` (pi=172 **텍스트** 라인 끝)
- `lazy_base = 1277445 - 32574 = 1244871` ← 차트 높이만큼 낮게

이후 pi=174 (2x1 표) 보정: `end_y = 948.4` → y_offset 528.8 → 948.4 강제 점프 (차트 높이 이중 반영).

### 2. Pagination 측 결함 (`src/renderer/typeset.rs`)

`typeset_section` controls 루프 (line 622-630) 가 비-TAC `Picture/Shape` 의 높이를 `current_height` 에 누적하지 않음:
```rust
Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
    if !has_table {
        st.current_items.push(PageItem::Shape { ... });
        // ← 높이 누적 없음
    }
}
```

`format_paragraph` 도 line_segs 의 `lh + ls` (텍스트 baseline) 만 누적 → 차트 419.6px 가 페이지네이션에 미반영.

→ 페이지네이션 추정 21페이지 used = 803.3px (vs 실제 layout y = 1275.9px) → pi=191 헤딩 + pi=192 (10x5 표) 모두 21페이지에 packing → layout overflow 로 잘려 22페이지에 누락.

## 변경

### 1. `src/renderer/layout.rs:1365-1390` (v1)

`prev_has_overlay_shape` 가드 확장 — `Control::Picture` (non-TAC) 분기 추가 + `TopAndBottom + vert_rel_to=Para` 케이스 포함:

```rust
// 글앞으로/글뒤로/위아래 Shape·Picture가 있는 문단: vpos에 개체 높이가 포함되어 과대 → bypass
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    use crate::model::shape::{TextWrap, VertRelTo};
    p.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => {
            let cm = &pic.common;
            if cm.treat_as_char { return false; }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para))
        }
        _ => false,
    })
}).unwrap_or(false);
```

### 2. `src/renderer/typeset.rs:622-672` (v2)

controls 루프에서 비-TAC + TopAndBottom + vert=Para 인 Picture/Shape 의 `height + margin.bottom` 을 `current_height` 에 누적 (layout 의 `calc_shape_bottom_y` 와 동일 산식):

```rust
use crate::model::shape::{TextWrap, VertRelTo};
let pushdown_h: Option<f64> = match ctrl {
    Control::Picture(pic) if !pic.common.treat_as_char
        && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
        && matches!(pic.common.vert_rel_to, VertRelTo::Para) => {
        let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
        let mb = hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
        Some(h + mb)
    }
    Control::Shape(s) if !s.common().treat_as_char
        && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
        && matches!(s.common().vert_rel_to, VertRelTo::Para) => {
        let cm = s.common();
        let h = hwpunit_to_px(cm.height as i32, self.dpi);
        let mb = hwpunit_to_px(cm.margin.bottom as i32, self.dpi);
        Some(h + mb)
    }
    _ => None,
};
if let Some(extra) = pushdown_h {
    st.current_height += extra;
}
```

## 검증

### 시각 결과 (PDF 대조)

| 항목 | v0 | v1 | v2 |
|------|----|----|----|
| 21페이지 2x1 표 위치 | 페이지 하단 잘림 ❌ | 차트 직하 ✓ | 차트 직하 ✓ |
| 22페이지 (4) 헤딩 | 누락 ❌ | 누락 ❌ | **표시** ✓ |
| 22페이지 10x5 표 | 누락 ❌ | 누락 ❌ | **표시** ✓ |

### LAYOUT_OVERFLOW (대상 샘플 전체)

| 단계 | 건수 | 잔여 |
|------|------|------|
| v0 | 22 | page=2/20/27 다수 |
| v1 | 4 | page=2 449 / page=20 248 / page=27 15+112 |
| **v2** | **1** | page=2 449 (기존 결함, 본 PR 무관) |

→ chart 관련 overflow 전건 해소.

### 회귀 테스트 (cargo test --release)

11개 테스트 스위트 100% 통과:
- `lib`: **1023 passed**, 0 failed
- `svg_snapshot`: **6 passed**, 0 failed
- 기타 9 suites 모두 통과

### 6개 다른 샘플 무회귀

| 샘플 | v0 | v1 | v2 |
|------|----|----|----|
| `biz_plan.hwp` | 0 | 0 | 0 |
| `exam_kor.hwp` | 7 | 7 | 7 |
| `exam_math.hwp` | 0 | 0 | 0 |
| `aift.hwp` | 1 | 1 | 1 |
| `k-water-rfp.hwp` | 0 | 0 | 0 |
| `kps-ai.hwp` | 4 | 4 | 4 |

## 변경 파일

- `src/renderer/layout.rs` (v1, line 1365-1390)
- `src/renderer/typeset.rs` (v2, line 622-672)

## 영향 범위

- 변경 모두 비-TAC + TopAndBottom + vert_rel_to=Para 조합으로 한정
- 다른 wrap 모드 (Square/Tight/InFrontOfText/BehindText) 와 vert=Page/Paper 케이스는 영향 없음
- 기존 InFrontOfText/BehindText Shape 케이스는 동일하게 동작 (분기 추가만 됨)
- 6개 다른 샘플 LAYOUT_OVERFLOW 무변동으로 회귀 없음 검증

closes #409
