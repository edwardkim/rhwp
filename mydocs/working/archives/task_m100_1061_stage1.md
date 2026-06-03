# Task M100-1061 Stage 1 — HWPX 어댑터 Equation arm + GenShape attr bit 27 보강

- 이슈: [#1061](https://github.com/edwardkim/rhwp/issues/1061)
- 단계: Stage 1
- 브랜치: `local/task1061`
- 일시: 2026-05-22

## 1. 목표

HWPX → HWP 어댑터 (`src/document_core/converters/hwpx_to_hwp.rs`) 에서 누락된
`Control::Equation` 처리 추가. CTRL_HEADER attr bit 27 (`0x08000000`) 보강 —
Table 어댑터의 `HWPX_TABLE_NUMBERING_BIT` 와 동일 패턴.

## 2. 정정 영역

### 2.1 `src/document_core/converters/hwpx_to_hwp.rs`

- `AdapterReport` struct: 신규 필드 2개 추가
  - `equation_ctrl_header_attr_materialized: u32`
  - `equation_font_version_normalized: u32` (Stage 2 에서 사용)
- `changed_anything()` 에 신규 필드 누적
- `adapt_paragraph` 의 control match 에 `Control::Equation(eq) => adapt_equation(eq, report)` arm 추가
- 신규 함수 `adapt_equation`:
  ```rust
  fn adapt_equation(eq: &mut Equation, report: &mut AdapterReport) {
      const HWPX_EQUATION_NUMBERING_BIT: u32 = 0x0800_0000;
      let before = eq.common.attr;
      eq.common.attr = pack_common_attr_bits(&eq.common) | HWPX_EQUATION_NUMBERING_BIT;
      let raw_was_present = !eq.raw_ctrl_data.is_empty();
      eq.raw_ctrl_data.clear();
      if eq.common.attr != before || raw_was_present {
          report.equation_ctrl_header_attr_materialized += 1;
      }
  }
  ```

### 2.2 `examples/repro_1061_equation_save.rs` 신규

HWPX → 어댑터 → HWP 저장 → 재파싱 → 정답지와 비교하는 reproduce 도구.

## 3. 정량 입증

### 3.1 정답지 vs Stage 1 저장본 IR 비교

**Stage 1 적용 전** (saved/111math-001.hwp 기존):
```
attr=0x042A2211 (bit 27 누락)
```

**Stage 1 적용 후** (output/poc/issue_1061/repro_stage1.hwp):
```
attr=0x0C2A2211  ← 정답지 정확 정합
font_name="HYhwpEQ"               ← Stage 2 에서 처리
version_info="Equation Version 60" ← Stage 2 에서 처리
```

### 3.2 본 단계 검증 항목

| 항목 | 결과 |
|------|------|
| common.attr (정답지 0x0C2A2211 vs 저장본) | **정확 정합** |
| cargo build --release --lib | success |
| cargo test --release --lib | **1323 passed / 0 failed** |
| script / font_size / color / baseline | 보존 |
| common.size / common.pos | 보존 |

## 4. Stage 2 미해결 영역

- font_name/version_info 정답지 정합 (Stage 2 에서 HWPX parser 정정)
- Stage 1 결과는 attr 만 정합 — 한컴 시각 판정 게이트는 Stage 2 후 수행

## 5. 메모리 룰 정합

- `feedback_diagnosis_layer_attribution` — 정답지 vs 저장본 raw byte 정밀 비교로 본질 정확 식별
- `feedback_hancom_compat_specific_over_general` — case-specific contract (Equation 만)
- `project_hwpx_to_hwp_adapter_limit` 정합 + **단순 어댑터 한계 점진 돌파**

## 6. 작업지시자 승인 요청

Stage 1 단독 검증 결과 (attr 정합 완료) → Stage 2 (parser font/version 매핑 정정) 진행 승인 여부.
