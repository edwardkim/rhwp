# Stage 4 보고서 — Task #409 v2

## 변경 내용

`src/renderer/typeset.rs:622-672` controls 루프 분기 확장:

비-TAC + TopAndBottom + vert_rel_to=Para 인 `Picture/Shape` 의 경우 `current_height` 에 `common.height + margin.bottom` 누적.

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

`margin.bottom` 까지 더하는 것은 layout 의 `calc_shape_bottom_y` (`shape_layout.rs:2052`) 가 `shape_y + shape_h + margin_bottom` 을 반환하는 것과 일치.

## 검증 결과

### 21페이지 / 22페이지 SVG

- **21페이지**: 차트 + 2x1 빈 표 (이전과 동일, PDF 일치 유지)
- **22페이지** (이전 결함):
  - 수정 전: (4) 헤딩 + 10x5 표 누락, 22페이지가 차트로 시작
  - 수정 후: (4) 헤딩 + 10x5 표 + 차트 + 2x1 빈 표 모두 정상 표시 (PDF 일치)

### LAYOUT_OVERFLOW (대상 샘플 전체)

| 단계 | overflow 건수 | 잔여 항목 |
|------|--------------|-----------|
| v0 (수정 전) | 22 | page=2/20/27 |
| v1 (layout 가드 확장) | 4 | page=2 449.2, page=20 247.9, page=27 15.0+111.9 |
| **v2 (typeset chart 누적)** | **1** | page=2 449.2 (본 변경과 무관, 기존 결함) |

→ chart 관련 모든 overflow 해소. page=27 의 잔여까지 부수적으로 해결 (27페이지에도 동일 패턴의 chart 가 존재했음을 시사).

### 회귀 테스트

- `cargo test --lib --release`: **1023 passed**, 0 failed
- `cargo test --release --test svg_snapshot`: **6 passed**, 0 failed

### 6개 샘플 LAYOUT_OVERFLOW 비교

| 샘플 | v1 후 | v2 후 |
|------|------|------|
| `biz_plan.hwp` | 0 | 0 |
| `exam_kor.hwp` | 7 | 7 |
| `exam_math.hwp` | 0 | 0 |
| `aift.hwp` | 1 | 1 |
| `k-water-rfp.hwp` | 0 | 0 |
| `kps-ai.hwp` | 4 | 4 |
| `2025년 기부·답례품_양식.hwpx` | 4 | **1** |

→ 다른 샘플 무회귀, 타겟 샘플 추가 3건 개선.

## 결론

- 22페이지 누락 결함 해소 (PDF 일치)
- chart 관련 overflow 전건 해소
- 1023 lib + 6 svg_snapshot 통과, 6개 다른 샘플 무회귀

Stage 5 (전체 회귀 + 통합 보고서) 진행 승인 요청.
