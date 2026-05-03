# Task #466 최종 보고서

**이슈**: [#466](https://github.com/edwardkim/rhwp/issues/466) — `compute_square_wrap_tbl_x_right` `horz_rel_to=Paper/Page` 케이스 보강
**선행**: #463 Stage 5 (Column 케이스 도입)
**브랜치**: `local/task466`
**마일스톤**: M100 (v1.0.0)
**완료일**: 2026-05-01

---

## 1. 결과

### 결함 보강
- `compute_square_wrap_tbl_x_right` 의 `horz_rel_to` 분기 보강:
  - `Paper`: `ref_x=0, ref_w=paper_width`
  - `Page`: `ref_x=body_area.x, ref_w=body_area.width`
  - `Para/Column`: `ref_x=col_area.x, ref_w=col_area.width` (기존 동작)
- free function → `LayoutEngine` 메서드 변환으로 `current_paper_width`, `current_body_area` 직접 접근

### 회귀
- **0 건** — 9 종 샘플 202 SVG 비교 결과 변경 0 건.
- issue 명시 "현재 검증된 샘플에서는 발생 케이스 없음" 일치 — Paper/Page case 가 발견되지 않아 동작 변화 없음.

### 잠재적 효과
- 향후 wrap=Square + horz_rel_to=Paper/Page floating 표가 발견되면 paragraph border box 가 정확한 우측 끝까지 확장.

### 테스트
- `cargo test --release`: 1094 + 다수 통합 테스트 모두 통과
- `cargo clippy`: 본 task 변경부 경고 0

## 2. 결함 본질

### 결함 위치 — `layout.rs:3382` (수정 전)

```rust
fn compute_square_wrap_tbl_x_right(
    t: &Table,
    col_area: &LayoutRect,
    dpi: f64,
) -> f64 {
    // horz_rel_to 무시 — 항상 col_area 기준
    let tbl_x = match t.common.horz_align {
        HorzAlign::Right | HorzAlign::Outside =>
            col_area.x + col_area.width - tbl_w + h_offset,
        ...
    };
    tbl_x + tbl_w
}
```

### 비교 — `table_layout::compute_table_x_position:937~967` (정확한 공식)

```rust
let (ref_x, ref_w) = match horz_rel_to {
    HorzRelTo::Paper => (0.0, paper_w),
    HorzRelTo::Page => {
        let body = self.current_body_area.get();
        if body.2 > 0.0 { (body.0, body.2) } else { (col_area.x, col_area.width) }
    }
    HorzRelTo::Para => (col_area.x + host_margin_left, col_area.width - host_margin_left),
    _ => (col_area.x, col_area.width),
};
```

→ Paper/Page case ref 가 다름. 헬퍼는 Column case (col_area) 만 정확.

## 3. 수정

### 3.1 `compute_square_wrap_tbl_x_right` — `LayoutEngine` 메서드 변환

```rust
impl LayoutEngine {
    fn compute_square_wrap_tbl_x_right(
        &self,
        t: &Table,
        col_area: &LayoutRect,
    ) -> f64 {
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
        // ... 정렬 공식 (h_offset 가산식, table_layout 와 일관)
    }
}
```

### 3.2 caller 2 곳 (`layout.rs:2407, 2641`)

```rust
let tbl_x_right = self.compute_square_wrap_tbl_x_right(t, col_area);
```

`self.dpi` 인자 제거 (메서드 내부에서 직접 접근).

## 4. 진행 경로

### 단계 1 — 수행계획서 + 베이스라인 (`task_m100_466.md` 통합)
- `compute_table_x_position` 의 정확한 공식 확인
- `current_paper_width`, `current_body_area` 접근 방식 확인
- 9 종 샘플 202 SVG 베이스라인 보존

### 단계 2 — 코드 수정 + 회귀 검증 (`task_m100_466_stage2.md`)
- 헬퍼 메서드 변환 + `horz_rel_to` 분기 추가
- caller 2 곳 수정
- 202 SVG 회귀 0 건
- cargo test 통과

### 단계 3 — 최종 정리 (본 보고서)

## 5. 변경 통계

| 파일 | 변경 |
|---|---|
| `src/renderer/layout.rs` | +37 -19 |

## 6. 검증 체크리스트

- [x] free function → 메서드 변환 (self 접근)
- [x] horz_rel_to 분기 (Paper/Page/Para/Column)
- [x] caller 2 곳 수정 (`self.compute_*`)
- [x] cargo build, test 통과
- [x] 9 종 샘플 202 SVG 회귀 0
- [x] cargo clippy 본 task 변경부 경고 0

## 7. 핵심 룰 정합 (`feedback_rule_not_heuristic.md`)

본 수정은 **`table_layout::compute_table_x_position` 의 동일 룰** 을 헬퍼에 적용:
- HWP 표준: `horz_rel_to` 가 표 위치 기준점 결정
- 룰: Paper/Page/Para/Column 각각 다른 ref_x/ref_w
- 기존 헬퍼는 Column 만 처리 (휴리스틱) → 룰 정합으로 4 케이스 모두 처리

## 8. 산출물

- 수행계획서: `mydocs/plans/task_m100_466.md`
- 단계 보고서: `mydocs/working/task_m100_466_stage2.md`
- 최종 보고서: 본 문서
- orders 갱신: `mydocs/orders/20260501.md`
