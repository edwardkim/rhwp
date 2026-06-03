# Task #716 Stage 2 (분석) 완료 보고서

**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**Stage**: 2 — 분석 / instrument
**작성일**: 2026-05-08

---

## 산출물

- `RHWP_TASK716_DEBUG=1` 환경변수 가드 instrument (4 위치)
  - `layout.rs` column item 루프 진입/종료 (TASK716_ADV)
  - `layout.rs` FullParagraph 진입 + layout_paragraph 호출 직전 (TASK716_FP_ENTER, TASK716_FP_CALL)
  - `paragraph_layout.rs:layout_composed_paragraph` 진입 + spacing_before 적용 후 (TASK716_PARA_ENTRY)
  - `paragraph_layout.rs` line advance 시점 (TASK716_LINE)

→ Stage 3 GREEN 진입 시 instrument 모두 제거.

---

## 측정 결과

### 페이지 0 항목별 drift 추적

`samples/20250130-hongbo.hwp` page 0 첫 6개 항목:

| pi | 종류 | y_in (ADV) | y_start (PARA) | gap | y_out | layout advance | hwp delta(lh+ls) | drift |
|----|------|-----------|---------------|-----|-------|---------------|------------------|-------|
| 0 | Table TAC | 94.47 | — | — | 141.93 | 47.47 | 39.47 | **+8.00** |
| 1 | empty para | 141.93 | **149.93** | **+8.00** | 161.93 | 20.00 (line=12+gap=8) | 12.00 | +8.00 |
| 2 | Table TAC | 161.93 | — | — | 199.45 | 37.52 | 29.97 | **+7.55** |
| 3 | empty para | 199.45 | **211.45** | **+12.00** | 219.45 | 20.00 (line=8+gap=12) | 8.00 | +12.00 |
| 4 | empty para | 219.45 | 219.45 | 0 | 225.85 | 6.40 | 6.40 | 0 |
| 5 | Table TAC | 225.85 | — | — | 380.99 | 155.13 | 155.13 | 0 |

### 핵심 관찰

1. **pi=1, pi=3 의 layout 진입 직후 +8/+12 px gap 발생**
   - `task716_y_in` (ADV instrument) = layout_column_item 호출 직전 y_offset
   - `task716_para_entry_y` (PARA instrument) = layout_composed_paragraph 진입 직후 y
   - 두 시점 사이에 **+8 (pi=1), +12 (pi=3) px** 의 push 발생
2. **line advance 자체는 정확** — `lh + ls` 가 그대로 가산됨 (pi=1: 20-8=12, pi=3: 20-12=8). 음수 ls 처리 정상.
3. **pi=4 (Percent 160%, ls>0) drift = 0** — 양수 ls 케이스는 drift 없음
4. **pi=5 이후 drift = 0** (multi-line paragraph 의 ADV drift 표시는 instrument 의 hwp_delta 산출 한계 — first line_seg lh+ls 만 표시. 실제 line advance 는 정확)

### Push 발생 위치 식별

`src/renderer/layout.rs:1582-1595` 의 **Task #9 `fix_overlay_active` 블록**:

```rust
if fix_overlay_active {
    let is_fixed = paragraphs.get(item_para)
        .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
        .map(|ps| ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed)
        .unwrap_or(false);
    if !is_fixed {
        let table_bottom = fix_table_start_y + fix_table_visual_h;
        if y_offset < table_bottom {
            y_offset = table_bottom;   // ← push 발생
        }
        fix_overlay_active = false;
    }
}
```

`fix_overlay_active` 활성 조건: 직전 item 이 TAC 표(was_tac=true) + host paragraph first line_seg `line_spacing < 0` (`layout.rs:1607-1614`).

### 산수 검증

- pi=0 TAC 표 (lh=3560, ls=-600, sa=0): `fix_table_start_y = 141.93 - hwpunit_to_px(3560-600).max(0) - 0 = 141.93 - 39.47 = 102.46`. `fix_table_visual_h = hwpunit_to_px(3560) = 47.47`. → `table_bottom = 149.93` ✓ (pi=1 y_start)
- pi=2 TAC 표 (lh=3148, ls=-900): `fix_table_start_y = 199.45 - 29.97 = 169.48`. `fix_table_visual_h = 41.97`. → `table_bottom = 211.45` ✓ (pi=3 y_start)

### 누적 drift 와 LAYOUT_OVERFLOW 정합

- pi=1 push: +8.00 px
- pi=3 push: +12.00 px
- 합계: **+20.00 px**
- RED overflow: **+20.15 px** (Stage 1 측정)

→ **drift 의 99.3% 가 fix_overlay 의 빈 paragraph push 누적**. 잔여 0.15 px 는 미세 sub-pixel 드리프트로 설명 가능.

---

## 본질 재정의 (가설 갱신)

### 기존 가설 (수행계획서)

> 음수 line_spacing(ls<0) 미반영 → drift 누적 (TAC 표 호스트 + 빈 문단 advance)

→ **부정확**. line advance 자체는 음수 ls 를 정확히 반영 (`paragraph_layout.rs:2657 y += line_height + line_spacing_px`).

### 수정된 본질

**Task #9 의 `fix_overlay_active` push 가 빈 paragraph(text_len=0) 에까지 적용되어 drift 만 누적**:

- 시각적 의미: TAC 표(ls<0) 후속 paragraph 가 표 위에 그려지지 않도록 표 하단으로 push
- 텍스트 paragraph: push 가 시각 정합에 필요 (HWP 편집기 동작 일치)
- **빈 paragraph: 시각적으로 invisible → push 가 무의미하나 y_offset 만 +(table_bottom-y_offset) 누적**

### 정정 방향 (수정안)

**기존 H1/H2-1/H2-2 폐기**. 대신 **단일 좁은 정정**:

```rust
if fix_overlay_active {
    let is_fixed = ...;
    // [Task #716] 빈 paragraph 는 push 의미 없음 → drift 누적만 발생
    let is_empty_para = paragraphs.get(item_para)
        .map(|p| p.text.is_empty()
            || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}'))
        .unwrap_or(false);
    if !is_fixed && !is_empty_para {
        let table_bottom = fix_table_start_y + fix_table_visual_h;
        if y_offset < table_bottom {
            y_offset = table_bottom;
        }
        fix_overlay_active = false;
    } else if is_empty_para {
        // 빈 paragraph 는 push 건너뜀 — fix_overlay_active 는 유지하여
        // 후속 비-empty paragraph 가 push 받을 수 있도록 (pi=1 후 pi=2 가 TAC 표인
        // 본 케이스는 pi=2 가 TAC 라 push 대상 아님 → fix_overlay 자연 종료).
    }
}
```

→ Task #9 의 텍스트 paragraph push 동작 유지, 빈 paragraph 만 skip.

### 위험 / 비범위 영향

- **Task #9 의 본래 의도 (TAC 표 후속 텍스트 paragraph 의 표 위 침범 방지) 유지**
- `is_empty_para` 판정은 보수적 (text 가 진짜 비어있거나 control 문자만): TAC 표 host paragraph (FFFC 포함) 는 push 대상이 아니라 별도 경로
- **회귀 위험 낮음**: 빈 paragraph 는 시각적으로 보이지 않으므로 push 변경이 보이는 차이 없음

---

## Stage 2 → Stage 3 진입 사항

### 변경된 구현 계획 (impl 갱신)

| 항목 | 기존 (impl) | 갱신 |
|------|-------------|------|
| H2-1 lazy_base.max(0) | 적용 예정 | **취소** — drift 원인 다름 |
| H2-2 trailing_ls 음수 허용 | 적용 예정 | **취소** — 동일 사유 |
| H1 TAC 호스트 ls 가산 | 적용 예정 | **취소** — line advance 자체는 정확 |
| **新 H0**: fix_overlay 빈 paragraph skip | — | **신규 적용** — 본질 정정 |

### Stage 3 작업 (3-A 단일 step)

1. instrument 모두 제거 (layout.rs 4 위치, paragraph_layout.rs 2 위치)
2. `layout.rs:1582-1595` fix_overlay 블록에 `is_empty_para` 가드 추가
3. `cargo test --test issue_716` 실행 → PASS 확인
4. `LAYOUT_OVERFLOW_*` stderr 출력 0건 확인
5. 보고서 + 커밋

## 승인 요청

Stage 2 분석 완료. 수정된 정정 방향(빈 paragraph fix_overlay skip 단일 정정) 으로 Stage 3 GREEN 진입 승인 요청.
