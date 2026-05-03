# Task #548 Stage 2 완료 보고서

**제목**: 셀 내부 inline TAC Shape margin_left + first-line indent 적용
**브랜치**: `local/task548`
**이슈**: https://github.com/edwardkim/rhwp/issues/548

---

## 1. fix 적용 내용

### 1.1 헬퍼 함수 추가 (`table_layout.rs:14-26`)

```rust
/// [Task #548] paragraph 의 line N 에 적용되는 effective margin_left.
/// paragraph_layout.rs:851-858 의 line_indent 산식과 동일 (단일 룰).
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

### 1.2 ParaShape 변수 추출 (`table_layout.rs:1490-1499`)

```rust
let para_margin_left_px = styles.para_styles
    .get(para.para_shape_id as usize)
    .map(|s| s.margin_left)
    .unwrap_or(0.0);
let para_indent_px = styles.para_styles
    .get(para.para_shape_id as usize)
    .map(|s| s.indent)
    .unwrap_or(0.0);
```

### 1.3 inline_x 산출 정정 — 3 위치

**위치 1: 초기 inline_x (line 0)**

```rust
let line_margin = effective_margin_left_line(para_margin_left_px, para_indent_px, 0);
match para_alignment {
    Alignment::Center | Alignment::Distribute => { ... }  // 변경 없음
    Alignment::Right => { ... }                           // 변경 없음
    _ => inner_area.x + line_margin,                      // fix
}
```

**위치 2: Picture target_line reset**

```rust
let line_margin = effective_margin_left_line(
    para_margin_left_px, para_indent_px, target_line);
inline_x = match para_alignment {
    ...
    _ => inner_area.x + line_margin,
};
```

**위치 3: Shape target_line reset**

```rust
let line_margin = effective_margin_left_line(
    para_margin_left_px, para_indent_px, target_line);
inline_x = match para_alignment {
    ...
    _ => inner_area.x + line_margin,
};
```

## 2. 검증

### 2.1 단위 테스트 — RED → GREEN

```
test test_548_cell_inline_shape_first_line_indent_p8 ... ok
test result: ok. 1122 passed; 0 failed; 2 ignored
```

페이지 8 보기 표 셀 5 line 0 [푸코] box left x = **155.60 px** (PDF 155.6 ±0.0 ✓).
Stage 1 RED 측정 131.04 → 155.60 (+24.56 px 정정).

기존 1121 단위 테스트 모두 통과. Task #534v2/#537/#539/#540/#544/#547 무회귀.
Task #548 통합 테스트 1건 추가 (총 1122).

### 2.2 핵심 측정값 (페이지 8 보기 표 셀 5)

| 항목 | Stage 0 측정 | Stage 2 fix 후 | PDF 기대 | 차이 |
|------|-------------|--------------|----------|------|
| Line 0 [푸코] box left | 131.04 | **155.60** | 155.6 | +0.0 ✓ |
| Line 0 텍스트 "는" | 185.83 | 185.83 (변경 없음) | ≈185.8 | 0 ✓ |
| Line 1 첫 글자 "출" | 142.40 | 142.40 (변경 없음) | ≈146.4 | -4 (tolerance) |

**텍스트와 shape 위치 일치 보장** — paragraph_layout 텍스트 경로 (185.83) 와
shape 의 직후 위치 (155.60 + 30.23 = 185.83) 가 정확히 일치.

## 3. 영향 범위

### 3.1 변경 적용 케이스 (의도된 시프트)

| paragraph 패턴 | line 0 시프트 | line N≥1 시프트 |
|---------------|-------------|---------------|
| margin_left=0 + indent=0 | 0 | 0 (변경 없음) |
| margin_left=M + indent=0 | +M | +M |
| margin_left=0 + indent=+N (positive) | +N | 0 |
| margin_left=M + indent=+N (positive) | +(M+N) | +M |
| margin_left=M + indent=-N (hanging) | +M | +(M+N) |

### 3.2 paragraph_layout 과 일관성

본 fix 는 paragraph_layout 의 `effective_margin_left` 산식 (line 858) 과 동일
산출 → **단일 룰** [feedback_rule_not_heuristic]. 텍스트와 shape 위치 일치
보장.

### 3.3 Center/Right alignment

Center/Right 분기는 본 fix 미적용 (변경 없음). 해당 정렬에서 inline_x 산출에
margin_left 가 어떻게 반영되는지는 별도 분석 영역 — 현재 회귀 위험 없음.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | effective_margin_left_line 헬퍼 + 3 분기 fix (+30 / -3 LOC) |
| `src/renderer/layout/integration_tests.rs` | `#[ignore]` 제거 (RED→GREEN) |
| `mydocs/working/task_m100_548_stage2.md` | 본 보고서 |

## 5. 핵심 설계 결정

### 5.1 단일 룰 적용

paragraph_layout 의 `effective_margin_left` 산식과 동일 헬퍼 (`effective_margin_left_line`)
사용. 텍스트와 shape 두 경로가 같은 산식 → 위치 일치.

### 5.2 Center/Right 보존

Center/Right 분기는 paragraph_layout 에서도 align_offset 계산 시 effective_margin_left
간접 반영. 현재 fix 는 Left/Justify 만 적용, Center/Right 분기는 변경 없음으로
회귀 위험 차단.

### 5.3 paragraph_layout `set_inline_shape_position` 미사용

paragraph_layout 이 등록한 inline_pos 는 셀 shape 경로 (`layout_cell_shape`) 가
사용하지 않음. 본 fix 는 inline_pos 사용 변경 없이 cell shape 경로 자체의 산식
정정으로 해결 (영향 범위 명확).

## 6. 다음 단계 (Stage 3)

1. 광범위 회귀 검증 (6 샘플) vs Stage 1 baseline
2. synam-001 27건 cell shape 검증 (가장 광범위 영향 후보)
3. 셀 외부 inline shape (pi=166 르포르/푸코) 무회귀 확인
4. Task #547 박스 outline / 텍스트 inset 무회귀 확인
5. Stage 3 보고서 + 최종 보고서

## 7. 승인 요청

Stage 2 완료. RED → GREEN, 1122 단위 테스트 통과.
- Line 0 [푸코]: 131.04 → 155.60 px (PDF 일치)
- 텍스트 "는" 위치 보존 (185.83 px)

Stage 3 (광범위 회귀 검증 + 최종 보고서) 진행 승인 요청.
