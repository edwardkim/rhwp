# Task #548 Stage 0 완료 보고서

**제목**: 사전 분석 + 본 코드 진단 + ParaShape 검증
**브랜치**: `local/task548`
**이슈**: https://github.com/edwardkim/rhwp/issues/548

---

## 1. 본질 진단

### 1.1 셀 5 paragraph (ps_id=19) ParaShape

`examples/inspect_ps.rs` 로 검증:

```
ps_id=19: margin_left=1704 margin_right=1704 indent=1980 border_fill_id=1 alignment=Justify
```

| 항목 | raw HU | px (style_resolver /2) |
|------|--------|----------------------|
| margin_left | 1704 | 11.36 |
| margin_right | 1704 | 11.36 |
| **indent (positive)** | **+1980** | **+13.20** |

positive indent → 첫줄 들여쓰기 (line 0 만 +13.20 px).

### 1.2 [푸코] inline shape 좌표 비교 (페이지 8 col 0 보기 표 셀 5)

| 위치 | SVG x | PDF (200dpi → SVG 환산) | 차이 |
|------|-------|------------------------|------|
| Cell 5 line 0 [푸코] box left | **131.04** | ≈155.6 | **-24.56 px** ✗ |
| Cell 5 line 0 텍스트 "는" first | 185.83 | ≈185.8 | 0 ✓ |
| Cell 5 line 1 첫 글자 "출" | 142.40 | ≈146.4 | -4 (tolerance) ✓ |

PDF 기대값 검증: cell_x (131.04) + margin_left (11.36) + indent (13.20) = **155.60** ≈ PDF 측정 155.6 ✓

### 1.3 두 경로 불일치

**텍스트 경로** (`paragraph_layout.rs`) — 정확:

```rust
// Line 858
let effective_margin_left = margin_left + line_indent;
// Line 1230
_ => effective_col_x + effective_margin_left + ...,
```

→ Line 0 cursor x 시작 = `col_x + margin_left + indent = 131.04 + 11.36 + 13.20 = 155.60`
→ 푸코 직후 "는" at `155.60 + tac_w (30.23) = 185.83` ✓

paragraph_layout 은 `set_inline_shape_position(...x=155.60, shape_y)` 도 등록 (line 1834).

**Shape 경로** (`table_layout.rs:1486-1497`) — 버그:

```rust
let mut inline_x = {
    let line_w = tac_line_widths.first().copied().unwrap_or(total_inline_width);
    match para_alignment {
        Alignment::Center | Alignment::Distribute => {
            inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
        }
        Alignment::Right => {
            inner_area.x + (inner_area.width - line_w).max(0.0)
        }
        _ => inner_area.x,  // ← Justify/Left: margin/indent 미적용
    }
};
```

→ Line 0 inline_x = `inner_area.x = 131.04` → `layout_cell_shape` 가 receive `inner_area.x = 131.04` → 푸코 rect 131.04 에 렌더 ✗

### 1.4 cell shape 렌더 경로

`shape_layout::layout_shape` 는 `layout.rs` (body level) 에서만 호출. 셀 내 shape
는 `table_layout::layout_cell_shape` 만 호출. 후자는 inline_pos 검사 없이
`inner_area.x` 기반 자체 계산.

→ paragraph_layout 의 `set_inline_shape_position` 등록값이 셀 shape 에는 무시됨.

## 2. 영향 범위

table_layout.rs 의 `inline_x` 초기화/리셋 위치 3곳:

| 위치 | 컨텍스트 | line_n |
|------|----------|--------|
| L1486 | 초기 | 0 |
| L1538 | Picture target_line reset | N (>0) |
| L1627 | Shape target_line reset | N (>0) |

세 위치 모두 `_ => inner_area.x` 분기 — `effective_margin_left_line(N)` 미적용.

## 3. fix 방향

### 3.1 본질 정정 (H1)

`paragraph_layout` 과 동일한 `effective_margin_left_line(N)` 적용:

```rust
fn effective_margin_left_line(margin_left: f64, indent: f64, line_n: usize) -> f64 {
    let line_indent = if indent > 0.0 {
        if line_n == 0 { indent } else { 0.0 }
    } else if indent < 0.0 {
        if line_n == 0 { 0.0 } else { indent.abs() }
    } else {
        0.0
    };
    margin_left + line_indent
}
```

table_layout 의 inline_x 분기에 `effective_margin_left_line(line_n)` 추가:

```rust
let line_margin = effective_margin_left_line(margin_left, indent, line_n);
match para_alignment {
    Alignment::Center | Alignment::Distribute => {
        inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
    }
    Alignment::Right => {
        inner_area.x + (inner_area.width - line_w).max(0.0)
    }
    _ => inner_area.x + line_margin,  // Left/Justify: 적용
}
```

### 3.2 영향

| 케이스 | 영향 |
|--------|------|
| margin_left=0 + indent=0 | 변경 없음 (line_margin=0) |
| margin_left=0 + indent=+N | Line 0 inline shape +N/2 px 우측 시프트 (PDF 정합) |
| margin_left=M + indent=0 | 모든 line 의 inline shape +M/2 px 우측 시프트 |
| margin_left=M + indent=+N | Line 0 +(M+N)/2, Line K≥1 +M/2 |
| margin_left=M + indent=-N (hanging) | Line 0 +M/2, Line K≥1 +(M+N)/2 |

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/plans/task_m100_548.md` | 수행 계획서 |
| `mydocs/working/task_m100_548_stage0.md` | 본 보고서 |
| `examples/inspect_ps.rs` | ParaShape 검증 도구 (참고용) |

## 5. 다음 단계 (Stage 1)

1. TDD 통합 테스트 추가: 페이지 8 셀 5 line 0 [푸코] box left x ≈ 155.6 ±2 (RED)
2. 광범위 사전 평가: 6 샘플 셀 내부 inline TAC Shape 분포
3. fix 위치 정밀 진단: table_layout.rs L1486 / L1538 / L1627
4. 셀 외부 inline shape 케이스 영향 평가

## 6. 승인 요청

Stage 0 완료. 본질 진단:
- **원인**: table_layout 의 `inline_x` 초기화/리셋이 paragraph 의 `effective_margin_left + line_indent` 를 미적용
- **fix 방향**: `effective_margin_left_line(line_n)` 적용 (paragraph_layout 과 동일 룰)

Stage 1 (TDD 테스트 + 광범위 사전 평가) 진행 승인 요청.
