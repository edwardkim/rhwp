# Task #894 Stage 3 완료 보고서 — HWP3 페이지 외곽선 좌표 기준 정합

**Stage**: 3 / 3 (항목 A)
**상태**: ✅ 완료

## 1. 문제

sample16 페이지 2 (목차) 의 우측 페이지 번호 (1, 3, 5, 6, ...) 가 페이지 외곽선 박스 **밖에** 표시. 한컴 viewer (HWP5 변환본 PDF) 는 외곽선 안에 표시.

## 2. 진단

### 2.1 기존 IR 변환 (Task #877 c8ba53b)

`src/parser/hwp3/mod.rs:2361`:
```rust
section_def.page_border_fill = crate::model::page::PageBorderFill {
    attr: 0,  // body_based
    spacing_left: (doc_info.border_margin_left as i16) * 4,  // 355 × 4 = 1420 HU ≈ 23.6 px (5 mm)
    ...
};
```

### 2.2 renderer 의 좌표 계산 (layout.rs:732~764)

```rust
let paper_based = (pbf.attr & 0x01) != 0;
let (base_x, base_y, base_w, base_h) = if paper_based {
    (0.0, 0.0, layout.page_width, layout.page_height)
} else {
    (layout.body_area.x, layout.body_area.y, layout.body_area.width, layout.body_area.height)
};
// border = base + spacing
```

### 2.3 좌표 측정

| 항목 | body_based (attr=0) | paper_based (attr=1) | 한컴 viewer |
|------|--------------------|--------------------|------------|
| 외곽선 x (좌, 우) | 80.3 ~ 713.4 px | **18.93 ~ 774.77 px** | (paper_based 정합) |
| 페이지 번호 x | 728.0 px | 728.0 px | — |
| 페이지 번호 위치 | **외곽선 밖** ❌ | **외곽선 안** ✅ | 외곽선 안 |

→ **paper_based (attr=1) 가 한컴 정합**.

## 3. Fix

`src/parser/hwp3/mod.rs:2361~2370`:

```rust
// attr bit 0 = paper_based (1) vs body_based (0).
// HWP3 spec 명시 없으나 한컴 viewer 의 PDF 출력 정합 비교 결과 paper_based 가 정답.
section_def.page_border_fill = crate::model::page::PageBorderFill {
    attr: 1,
    spacing_left: (doc_info.border_margin_left as i16) * 4,
    spacing_right: (doc_info.border_margin_right as i16) * 4,
    spacing_top: (doc_info.border_margin_top as i16) * 4,
    spacing_bottom: (doc_info.border_margin_bottom as i16) * 4,
    border_fill_id: bfid,
};
```

## 4. 검증

### 4.1 sample16 페이지 2 정합

| 항목 | 결과 |
|------|------|
| 외곽선 박스 (x 범위) | 18.93 ~ 774.77 px ✅ |
| 페이지 번호 (x=728.0) | 외곽선 안 ✅ |

### 4.2 회귀 점검

| 항목 | 결과 |
|------|------|
| `cargo test --release --all-targets` | **1355 passed**, 0 failed |
| HWP3 sample 6종 페이지 수 | 모두 동일 (회귀 없음) |
| HWPX/HWP5 페이지 수 | 모두 동일 (회귀 없음) |

## 5. 커밋

- `ddb7fa4` — Task #894 Stage 3: HWP3 page border 좌표 기준 paper_based 로 정합

## 6. 산출물

- 본 Stage 보고서: `mydocs/working/task_m100_894_stage3.md`
- Fix: `src/parser/hwp3/mod.rs` (+4 lines, -1 line)
