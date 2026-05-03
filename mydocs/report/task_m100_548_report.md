# Task #548 최종 결과 보고서

**제목**: 셀 내부 paragraph 첫줄 inline TAC Shape 의 margin_left + first-line indent 누락 (Task #547 후속)
**브랜치**: `local/task548`
**이슈**: https://github.com/edwardkim/rhwp/issues/548
**Milestone**: M100 (v1.0.0)

---

## 1. 요약

페이지 8 보기 표 (pi=167) 셀 5 (3-col 병합 본문 셀) 의 첫 줄 [푸코] inline
rectangle Shape 가 cell 좌측 가장자리 (x=131.04 px) 에 렌더되어 PDF (한컴 2010,
x≈155.6 px) 보다 24.56 px 좌측에 있음.

원인: `table_layout.rs` 의 셀 안 inline TAC Shape 렌더링용 `inline_x` 초기화/
리셋이 paragraph 의 `margin_left + first_line_indent` 를 미적용. paragraph_layout
(텍스트 경로) 은 `effective_margin_left` 적용해 정확하지만, table_layout (shape
경로) 은 `inner_area.x` 만 사용 → 두 경로 위치 불일치.

**핵심 측정값** (페이지 8 셀 5 line 0 [푸코] box):

| 항목 | PDF (한컴 2010) | 수정 전 | 수정 후 |
|------|-----------------|---------|---------|
| Box left x | 155.6 | 131.04 (-24.6 px) | **155.60** (+0.0 ✓) |
| Box right x | ≈185.8 | 161.27 (-24.6) | 185.83 (+0.0 ✓) |
| 텍스트 "는" (직후) | ≈185.8 | 185.83 (paragraph_layout 정확) | 185.83 (변경 없음) |

**결과**: shape 와 직후 텍스트 위치 정확히 일치 (185.83 = 155.60 + 30.23).

## 2. 변경 사항

### 2.1 `src/renderer/layout/table_layout.rs` (+30 / -3 LOC)

**(1) 헬퍼 함수 추가**:

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

**(2) ParaShape 변수 추출**:

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

**(3) inline_x 산출 정정 — 3 위치**:
- 초기 inline_x (line 0)
- Picture target_line reset (line N)
- Shape target_line reset (line N)

```rust
let line_margin = effective_margin_left_line(para_margin_left_px, para_indent_px, line_n);
match para_alignment {
    Alignment::Center | Alignment::Distribute => { ... }  // 변경 없음
    Alignment::Right => { ... }                           // 변경 없음
    _ => inner_area.x + line_margin,                      // fix
}
```

### 2.2 `src/renderer/layout/integration_tests.rs` (+75 / -1 LOC)

`test_548_cell_inline_shape_first_line_indent_p8` 통합 테스트 추가.

## 3. 핵심 설계 결정

### 3.1 단일 룰 적용 [feedback_rule_not_heuristic]

paragraph_layout 의 `effective_margin_left` 산식과 동일 헬퍼 (`effective_margin_left_line`)
사용. 텍스트와 shape 두 경로가 같은 산식 → 위치 일치 보장.

### 3.2 Center/Right alignment 보존

Center/Right 분기는 본 fix 미적용 (변경 없음). 본 케이스 (Justify) 와 다른
분기는 별도 분석 영역. 회귀 위험 차단.

### 3.3 `set_inline_shape_position` 미사용

paragraph_layout 이 등록한 inline_pos 는 셀 shape 경로 (`layout_cell_shape`) 가
사용하지 않음. 본 fix 는 inline_pos 사용 변경 없이 cell shape 경로 자체 산식
정정 (영향 범위 명확).

## 4. 검증 결과

### 4.1 단위 테스트

```
test result: ok. 1122 passed; 0 failed; 2 ignored
test_548_cell_inline_shape_first_line_indent_p8 ... ok
```

기존 1121 단위 테스트 + Task #548 GREEN 1건. Task #534v2/#537/#539/#540/#544/#547
무회귀.

### 4.2 광범위 회귀 검증 (6 샘플)

| 샘플 | text +shift (max) | text -shift | rect +shift (max) | rect -shift |
|------|------------------|------------|------------------|------------|
| 21_언어_기출 | 2 (+24.56) | **0** | 1 (+24.56) | **0** |
| exam_kor | 21 (+2.00) | **0** | 7 (+2.00) | **0** |
| exam_math | 0 | **0** | 0 | **0** |
| exam_eng | 9 (+2.00) | **0** | 3 (+2.00) | **0** |
| exam_science | 31 (+20.00) | **0** | 11 (+20.00) | **0** |
| synam-001 | 601 (+6.67) | **0** | 154 (+6.67) | **0** |

**텍스트 + Rect 음의 시프트 0건 — 회귀 없음 ✓**.

### 4.3 페이지 8 보기 표 셀 5 직접 검증 (사용자 보고 케이스)

- [푸코] box left x: **131.04 → 155.60 px** (PDF 155.6 ±0.0)
- 텍스트 "는" 위치 보존 (185.83 px)
- shape 와 텍스트 위치 정확히 일치 (185.83 = 155.60 + 30.23)

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| 셀 내 inline shape 위치 변경 | 매우 큼 | 광범위 회귀 검증 (6 샘플, 음의 시프트 0건) |
| paragraph_layout 텍스트 경로 회귀 | 큼 | table_layout 만 변경, paragraph_layout 미변경 |
| Task #544 / #547 fix 와 충돌 | 큼 | 단위 테스트 무회귀 (test_544/#547 통과) |
| 다양한 paragraph 패턴 (positive/negative indent) | 중간 | paragraph_layout 의 line_indent 산식과 동일 헬퍼 사용 |
| Center/Right alignment | 낮음 | 분기 미변경 (회귀 차단) |
| PDF 절대 기준 아님 [feedback_pdf_not_authoritative] | 중간 | 한컴 2010 PDF 일치 + 한컴 2020/한컴독스 검증 권고 |

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | effective_margin_left_line 헬퍼 + 3 분기 fix (+30 / -3 LOC) |
| `src/renderer/layout/integration_tests.rs` | TDD 통합 테스트 (+75 / -1 LOC) |
| `examples/inspect_ps.rs` | ParaShape 검증 도구 |
| `mydocs/plans/task_m100_548.md` | 수행 계획서 |
| `mydocs/working/task_m100_548_stage{0,1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_548_report.md` | 본 보고서 |

## 7. 커밋 이력

- `1b89ad4a` Stage 0: 사전 분석 + 본 코드 진단
- `f4bced43` Stage 1: TDD 통합 테스트 (RED) + 광범위 사전 평가
- `9576f364` Stage 2: 셀 내부 inline TAC Shape effective_margin_left 적용
- (Stage 3) — 본 보고서 + 광범위 회귀 검증

## 8. 회고

Stage 0 사전 분석에서 두 경로 (paragraph_layout 텍스트 / table_layout shape) 의
산식 불일치를 정확히 식별. ps_id=19 ParaShape 검증 도구 (`examples/inspect_ps.rs`)
도입으로 cell paragraph margin_left=1704, indent=+1980 확인.

Stage 1 광범위 사전 평가에서 영향 분포 파악 (synam-001 27건, 21_언어_기출
2건, exam_kor 1건, exam_science 1건, exam_math/eng 0건). Stage 2 fix 적용 시
paragraph_layout.rs 의 line_indent 산식과 동일한 헬퍼 (`effective_margin_left_line`)
사용 — 단일 룰로 두 경로 일관성 확보. Stage 3 광범위 회귀 검증으로 6 샘플 모두
음의 시프트 0건 + Task #544/#547 무회귀 확인.

「본질 정정 회귀 위험」 [feedback_essential_fix_regression_risk] 적용: 광범위
샘플 검증으로 회귀 위험 완화. 「룰과 휴리스틱 구분」 [feedback_rule_not_heuristic]
적용: paragraph_layout 과 동일 산식 (`effective_margin_left_line`) 사용으로
단일 룰 보장.

closes #548
