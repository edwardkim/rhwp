# Task #548 Stage 1 완료 보고서

**제목**: TDD 통합 테스트 (RED) + 광범위 사전 평가 + fix 위치 정밀 진단
**브랜치**: `local/task548`
**이슈**: https://github.com/edwardkim/rhwp/issues/548

---

## 1. TDD 통합 테스트 추가 (RED 확인)

`integration_tests.rs` 에 `test_548_cell_inline_shape_first_line_indent_p8` 추가.

페이지 8 보기 표 셀 5 line 0 [푸코] box 좌측 x 검증:
- y in [685, 690] (셀 5 첫줄)
- width ≈ 30.23 (푸코 box width = curr_w 2267 HU)
- height ≈ 18.89 (푸코 box height = curr_h 1417 HU)
- PDF 기대값 = 155.6 px (±2 px)

```
test test_548_cell_inline_shape_first_line_indent_p8 ... FAILED
셀 5 line 0 [푸코] box left x=131.04 가 PDF 기대값 155.60 (±2 px) 와
일치해야 함. 버그(수정 전): puko_x=131.04 (-24.6 px, table_layout
inline_x 가 effective_margin_left + first_line_indent 미적용).
```

Stage 0 진단 정확히 일치 (-24.56 px). `#[ignore]` attribute 적용. 1121 단위
테스트 모두 통과.

## 2. 광범위 사전 평가

### 2.1 셀 내부 사각형 TAC Shape 분포

| 샘플 | 사각형 tac=true 건수 | 비고 |
|------|---------------------|------|
| 21_언어_기출 | 2 | (1) pi=167 셀 5 [푸코] (본 케이스), (2) 다른 표 빈 문단 |
| exam_kor | 1 | 후속 검증 필요 |
| exam_math | 0 | 영향 없음 |
| exam_eng | 0 | 영향 없음 |
| exam_science | 1 | 후속 검증 필요 |
| synam-001 | **27** | 광범위 영향 가능성 |

### 2.2 영향 분기 분석

table_layout.rs 의 `inline_x` 분기:

| 분기 | 위치 | 영향 |
|------|------|------|
| Center / Distribute | L1486-1490 | margin/indent 미적용 → **center 정렬에는 적용 안됨** (변경 없음 권고) |
| Right | L1486, L1492 | margin/indent 미적용 → **right 정렬에는 적용 안됨** (변경 없음 권고) |
| **Left / Justify** | L1486, L1495 | margin + line_indent 적용 필요 → **fix 대상** |

본 케이스 (셀 5 ps_id=19 alignment=Justify) 는 Left/Justify 분기.

### 2.3 회귀 위험

| 케이스 | 영향 | 비고 |
|--------|------|------|
| margin_left=0 + indent=0 cell paragraph | 변경 없음 | 대다수 cell paragraph |
| margin_left>0 + indent=0 cell paragraph | shape +M/2 px 우측 시프트 | PDF 정합 (의도) |
| margin_left=0 + indent>0 cell paragraph | line 0 shape +N/2 px 우측 시프트 | PDF 정합 (의도) |
| margin_left>0 + indent>0 cell paragraph | line 0 shape +(M+N)/2 px 시프트 | PDF 정합 (의도) — **본 케이스** |
| hanging indent (indent<0) | line N≥1 shape 시프트 | PDF 정합 (의도) |

synam-001 27건 중 ParaShape 분포 확인 필요 (Stage 3 회귀 검증).

### 2.4 paragraph_layout 과 일관성

본 fix 는 paragraph_layout 의 `effective_margin_left` (line 858) 와 동일 산식
적용 → **단일 룰** [feedback_rule_not_heuristic]. 텍스트와 shape 위치 일치
보장.

## 3. fix 위치 정밀 진단

### 3.1 본 코드 위치 (table_layout.rs)

**위치 1: 초기 inline_x (line 0)**

```rust
// L1486-1497
let mut inline_x = {
    let line_w = tac_line_widths.first().copied().unwrap_or(total_inline_width);
    match para_alignment {
        Alignment::Center | Alignment::Distribute => {
            inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
        }
        Alignment::Right => {
            inner_area.x + (inner_area.width - line_w).max(0.0)
        }
        _ => inner_area.x,  // ← fix 대상
    }
};
```

**위치 2: Picture target_line reset (L1538-1546)**

```rust
inline_x = match para_alignment {
    ...
    _ => inner_area.x,  // ← fix 대상 (line N)
};
```

**위치 3: Shape target_line reset (L1627-1635)**

```rust
inline_x = match para_alignment {
    ...
    _ => inner_area.x,  // ← fix 대상 (line N)
};
```

세 위치 모두 `effective_margin_left_line(margin_left, indent, line_n)` 적용.

### 3.2 변수 접근

table_layout 함수 내부에 paragraph 의 `margin_left`, `indent` 가 이미 있는지
확인 필요 (Stage 2). 없으면 `styles.para_styles[para_shape_id]` 조회 추가.

## 4. fix 방향 정리

### 4.1 헬퍼 함수 추가 (table_layout 또는 공용 유틸)

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

### 4.2 적용 분기

```rust
let line_margin = effective_margin_left_line(margin_left, indent, line_n);
match para_alignment {
    Alignment::Center | Alignment::Distribute => { ... }  // 변경 없음
    Alignment::Right => { ... }                           // 변경 없음
    _ => inner_area.x + line_margin,                      // fix
}
```

Center/Right 정렬에는 미적용 — paragraph_layout 의 Center/Right 도 align_offset
계산 시 effective_margin_left 차감 (`available_width = effective_col_w -
effective_margin_left - margin_right`). 이는 별도 검토 필요 (Stage 2).

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | TDD 테스트 1건 (RED, +75 LOC) |
| `mydocs/working/task_m100_548_stage1.md` | 본 보고서 |

## 6. 다음 단계 (Stage 2)

1. `effective_margin_left_line` 헬퍼 함수 추가
2. table_layout.rs L1486 / L1538 / L1627 fix 적용
3. RED → GREEN 확인
4. 1121 단위 테스트 무회귀 확인
5. Stage 2 보고서 + 커밋

## 7. 승인 요청

Stage 1 완료. 본질 정정 (table_layout `inline_x` 에 effective_margin_left_line
적용) 진행 OK?

승인 후 Stage 2 (fix 적용) 진행합니다.
