# Task #466 단계 2 보고서 — 코드 수정 + 회귀 검증

**이슈**: #466
**브랜치**: `local/task466`
**전제**: 단계 1 (베이스라인 + 코드 위치 확인) 완료 (수행계획서 통합)

---

## 1. 코드 수정

### 1.1 `compute_square_wrap_tbl_x_right` — free function → `LayoutEngine` 메서드

```rust
impl LayoutEngine {
    fn compute_square_wrap_tbl_x_right(
        &self,
        t: &crate::model::table::Table,
        col_area: &LayoutRect,
    ) -> f64 {
        use crate::model::shape::{HorzAlign, HorzRelTo};
        let paper_w = self.current_paper_width.get();
        let body = self.current_body_area.get();
        let (ref_x, ref_w) = match t.common.horz_rel_to {
            HorzRelTo::Paper => {
                if paper_w > 0.0 { (0.0, paper_w) } else { (col_area.x, col_area.width) }
            }
            HorzRelTo::Page => {
                if body.2 > 0.0 { (body.0, body.2) } else { (col_area.x, col_area.width) }
            }
            HorzRelTo::Para | HorzRelTo::Column => (col_area.x, col_area.width),
        };
        let tbl_w = crate::renderer::hwpunit_to_px(t.common.width as i32, self.dpi);
        let h_offset = crate::renderer::hwpunit_to_px(t.common.horizontal_offset as i32, self.dpi);
        let tbl_x = match t.common.horz_align {
            HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - tbl_w) + h_offset,
            HorzAlign::Center => ref_x + (ref_w - tbl_w) / 2.0 + h_offset,
            _ => ref_x + h_offset,
        };
        tbl_x + tbl_w
    }
}
```

### 1.2 caller 2 곳 수정 (`layout.rs:2407, 2641`)

```rust
let tbl_x_right = self.compute_square_wrap_tbl_x_right(t, col_area);
```

`self.dpi` 인자 제거 (메서드 내부에서 직접 접근).

## 2. 회귀 검증

### 2.1 9 종 샘플 202 SVG 비교

| 샘플 | 페이지 |
|---|---|
| exam_kor | 20 |
| exam_eng | 8 |
| exam_math | 20 |
| exam_science | 4 |
| exam_social | 4 |
| synam-001 | 35 |
| k-water-rfp | 28 |
| aift | 77 |
| biz_plan | 6 |
| **합계** | **202 SVG** |

비교 결과: **변경 0 건** (회귀 0).

### 2.2 검증된 샘플에서 동작 변화 없음

issue #466 명시: "현재 검증된 샘플에서는 발생 케이스 없음".

- `horz_rel_to=Para/Column` (검증된 샘플의 모든 wrap=Square 표) 분기는 기존과 동일 (`col_area` 기준).
- `horz_rel_to=Paper/Page` 분기만 새로 활성화. 검증된 샘플에 발생 케이스 없어 변경 0 건.

→ **잠재적 결함 보강** 만 적용. 기존 동작 보존.

### 2.3 cargo test 통과

```
test result: ok. 1094 passed; 0 failed; 1 ignored; 0 measured
```

기타 통합 테스트 모두 통과.

## 3. 단계 3 진입

자동승인. 단계 3 (clippy + 최종 보고서 + orders + 커밋 + merge + push + Issue close) 즉시 진입.
