# Task #544 v2 Stage 3 보고서

## 목적

Phase C 재적용 — 셀 내부 inline TAC Shape (Picture / Shape) 의 left x 산식에 paragraph margin_left + first-line indent 반영. paragraph_layout 텍스트 경로와 셀 inline 경로의 위치 일관성 보장.

## 적용 변경

### 1. effective_margin_left_line 헬퍼 추가 (table_layout.rs:13~28)

```rust
/// [Task #548] paragraph 의 line N 에 적용되는 effective margin_left.
/// paragraph_layout.rs 의 line_indent 산식과 동일 (단일 룰).
/// - positive indent: line 0 에만 +indent 적용 (첫줄 들여쓰기)
/// - negative indent (hanging): line N≥1 에 +|indent| 적용
/// - indent=0: 모든 line 에 margin_left 만 적용
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

### 2. inline_x 산출 3 분기 (Left/Justify) 에 line_margin 가산

#### 분기 (1) — paragraph 시작 (line 0)

```rust
let mut inline_x = {
    let line_w = ...;
    let line_margin = effective_margin_left_line(para_margin_left_px, para_indent_px, 0);
    match para_alignment {
        Center | Distribute => inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0,
        Right => inner_area.x + (inner_area.width - line_w).max(0.0),
        _ => inner_area.x + line_margin,  // [Task #548] 추가
    }
};
```

#### 분기 (2) — Picture target_line reset (line ~1582)
#### 분기 (3) — Shape target_line reset (line ~1674)

각 분기에 `line_margin = effective_margin_left_line(..., target_line)` 추가, Left/Justify 케이스에 `+ line_margin` 가산.

### 3. para_margin_left_px / para_indent_px 추출 (line ~1512)

```rust
let para_margin_left_px = styles.para_styles.get(para.para_shape_id as usize)
    .map(|s| s.margin_left).unwrap_or(0.0);
let para_indent_px = styles.para_styles.get(para.para_shape_id as usize)
    .map(|s| s.indent).unwrap_or(0.0);
```

### 4. test_548 의 `#[ignore]` 제거

RED → GREEN 전환.

## 검증

### 단위 테스트

```
cargo test --lib
test result: ok. 1122 passed; 0 failed; 2 ignored; 0 measured
```

- Stage 2 baseline: 1121 passed / 3 ignored
- 현재: 1122 passed / 2 ignored
- Δ = **+1 GREEN, -1 ignored** (test_548)
- 회귀 **0건**

### 회귀 가드 통합 테스트

| Suite | 결과 |
|-------|------|
| issue_301 | 1 GREEN |
| issue_418 | 1 GREEN |
| issue_501 | 1 GREEN |
| issue_505 | 9 GREEN |
| issue_514 | 3 GREEN |
| issue_516 | 8 GREEN |
| issue_530 | 1 GREEN |
| issue_546 | 1 GREEN |

→ **24 / 24 GREEN, 회귀 0건**.

### test_548 측정값 변화

| 좌표 | Stage 1 (수정 전) | Stage 3 (수정 후) | PDF 기대 |
|------|--------------------|-------------------|----------|
| 셀 5 line 0 [푸코] puko_x | 131.04 | **155.6 ±2** | 155.6 |

→ paragraph_layout 텍스트 "는" 의 위치 (185.83) 와 정합. 텍스트와 shape 위치 일관성 회복.

## 코드 영향

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/layout/table_layout.rs` | +40 LOC (헬퍼 + 3 분기 + para 추출) | 정정 본문 |
| `src/renderer/layout/integration_tests.rs` | -1 ignore attribute | test_548 GREEN |

## 다음 단계

Stage 4 — 최종 보고서:
- 광범위 svg_snapshot 회귀 (의도된 정정 vs 회귀 분석)
- WASM 빌드 + clippy 0 확인
- `mydocs/report/task_m100_544_v2_report.md`
- `mydocs/orders/20260503.md` 갱신 ("잔존 (별도 이슈 후보)" 항목 정정)
