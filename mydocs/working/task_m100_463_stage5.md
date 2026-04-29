# Task #463 Stage 5 완료보고서 (후속 박스 정합)

## 추가 발견 (Stage 4 머지 후 사용자 검토)

작업지시자가 14페이지 출력 추가 검토 결과:

1. **좌측 단 [A][B][C] 마커가 박스 밖으로 밀려 그림** — 인용 따옴표 ｢｣ 표시용 3×2 floating 표가 col_area 우측을 horizontal_offset 만큼 넘는데, paragraph border 가 paragraph margin 만 따라 그려져서 표가 박스를 벗어남.
2. **우측 단 박스가 좁음** — `merge` 가 첫 항목 (pi=50 PartialParagraph 의 좁은 wrap_area geometry x=604.7 w=351.2) 을 그룹 box geometry 로 굳히고 후속 wider paragraphs 가 박스 밖으로 튀어나옴.

## 변경 요약

### Fix 1: merge 시 박스 geometry 를 max-extent 로 확장 (`layout.rs:1639+`)

merge 시 `last.x` / `last.x+w` 를 `min(x)` / `max(x+w)` 로 업데이트.

```rust
// merge 본문 추가
let last_right = last.1 + last.3;
let cur_right = x + w;
let new_x = last.1.min(x);
let new_right = last_right.max(cur_right);
last.1 = new_x;
last.3 = new_right - new_x;
```

→ 우측 단 박스가 후속 wider paragraph 의 geometry 까지 확장됨.

### Fix 2: wrap host paragraph 의 box override (`layout.rs:220, 270, paragraph_layout.rs:2516`)

`LayoutEngine` 에 `border_box_override: Cell<Option<(f64, f64)>>` 추가. wrap host text rendering 직전에 원래 col_area 를 override 로 설정. paragraph_layout 의 push 가 override 가 있으면 그 geometry 사용.

→ wrap host 의 box 가 좁은 wrap_area 로 굳지 않고 원래 col_area 너비로 그려짐.

### Fix 3: floating 표 우측까지 box override 확장 (`layout.rs:2351, 2590, 2837, +helper`)

`compute_square_wrap_tbl_x_right` 헬퍼 추가 — `table_layout.rs:920-967` 와 동일 공식으로 인라인 wrap=Square 표의 우측 x 계산:

```rust
fn compute_square_wrap_tbl_x_right(t: &Table, col_area: &LayoutRect, dpi: f64) -> f64 {
    let tbl_w = hwpunit_to_px(t.common.width as i32, dpi);
    let h_offset = hwpunit_to_px(t.common.horizontal_offset as i32, dpi);
    let tbl_x = match t.common.horz_align {
        HorzAlign::Right | HorzAlign::Outside =>
            col_area.x + col_area.width - tbl_w + h_offset,
        HorzAlign::Center =>
            col_area.x + (col_area.width - tbl_w) / 2.0 + h_offset,
        _ => col_area.x + h_offset,
    };
    tbl_x + tbl_w
}
```

`layout_wrap_around_paras` 에 `tbl_x_right: Option<f64>` 매개변수 추가. 두 호출 지점 (line 2351, 2590) 에서 위 헬퍼로 계산해 전달. wrap_around 가 override 의 width 를 `max(col_area.width, tbl_x_right - col_area.x)` 로 설정.

### Fix 4: override 활성 시 margin_right 미차감 (`paragraph_layout.rs:2516`)

```rust
let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
    (ox + box_margin_left, ow - box_margin_left)  // margin_right 미차감
} else {
    (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
};
```

floating 표 우측까지 확장된 box width 에서 다시 margin_right 를 빼면 표가 박스 밖으로 다시 밀려나기 때문.

## 검증 결과

### 14p 좌측 단 박스 우측 좌표

| 단계 | box right | 표 right | 결과 |
|------|-----------|----------|------|
| Stage 4 | 528.5 | 549.95 | 마커 박스 밖 |
| Stage 5 v1 (override 만) | 528.5 | 549.95 | 동일 |
| Stage 5 v2 (+max-extent merge) | 528.5 | 549.95 | 좌측 동일, 우측 단 fix |
| Stage 5 v3 (+tbl_x_right override) | 538.55 | 549.95 | 11.4 px 부족 (margin_right 차감) |
| Stage 5 v4 (margin_right 미차감) | **549.89** | 549.95 | **마커 박스 안** ✓ |

좌측 박스 width = 421.39 (col_area.width 423.3 + h_offset 9.45 - margin_left 11.33)
우측 박스 width = 421.39 (동일 공식)

### 시각 확인 (zoom)

PDF 와 일치하게 [A] [B] [C] 마커가 박스 우측 가장자리에 정확히 둘러싸여 표시됨.

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed
```

### 회귀 (다른 샘플 6종)

- `2010-01-06.hwp` (6p) ✓
- `2022년 국립국어원 업무계획.hwp` (40p) ✓
- `biz_plan.hwp` (6p) ✓
- `21_언어_기출_편집가능본.hwp` (15p) ✓
- `exam_eng.hwp` (8p) ✓
- `exam_math_8.hwp` (1p) ✓

모두 정상 SVG 내보내기.

## 남은 한계

- `compute_square_wrap_tbl_x_right` 는 `horz_rel_to=Column` 케이스만 정확. `Paper`/`Page` 에서 floating 표 위치는 다른 ref_x/ref_w 사용 — 별도 케이스 발견 시 보강 필요.
- override 가 wrap host paragraph 에만 활성화. 비-wrap host 의 일반 paragraph border 는 영향 없음.

## 변경 파일

| 파일 | 변경 라인 |
|------|----------|
| `src/renderer/layout.rs` | +50 (state cell, merge max-extent, wrap_around override, helper) |
| `src/renderer/layout/paragraph_layout.rs` | +12 (override 분기) |
