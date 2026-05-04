# Task #568 Stage 3 보고서 — 구현 + 검증

- **이슈**: [#568](https://github.com/edwardkim/rhwp/issues/568)
- **브랜치**: `local/task568`
- **단계**: Stage 3 (구현 + 검증)
- **선행 산출**: Stage 1 진단(`task_m100_568_stage1.md`), Stage 2 구현 계획(`task_m100_568_impl.md`)
- **작성일**: 2026-05-04

## 1. 변경 요약

`src/renderer/layout/paragraph_layout.rs` L857 의 `effective_col_x / effective_col_w` 분기 조건 확장.

### 1.1 코드 변경 (+25 / -2 LOC)

```rust
// [Task #568] 인라인 TAC 표(treat_as_char=true) 가 있는 줄도 동일 처리.
let line_has_inline_tac_table = !tac_offsets_px.is_empty() && para.map(|p| {
    let line_start = comp_line.char_start;
    let line_end = line_start + comp_line.runs.iter()
        .map(|r| r.text.chars().count()).sum::<usize>();
    tac_offsets_px.iter().any(|(pos, _, ci)| {
        *pos >= line_start && *pos <= line_end
            && matches!(p.controls.get(*ci),
                Some(Control::Table(t)) if t.common.treat_as_char)
    })
}).unwrap_or(false);

// [Task #568] 임계값에 column_start 포함 — 실제 가용 line 폭은 (sw + cs).
let line_avail_hu = comp_line.segment_width.saturating_add(comp_line.column_start);
let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap
    || line_has_inline_tac_table)
    && comp_line.segment_width > 0
    && line_avail_hu < col_area_w_hu - 200    // ← 임계값 변경: sw → sw+cs
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

### 1.2 핵심 설계 결정

| 항목 | 결정 | 근거 |
|------|------|------|
| 활성 조건 추가 | `\|\| line_has_inline_tac_table` | 인라인 TAC 표 보유 줄 검출 (per-line, per-paragraph 아님) |
| 임계값 | `sw + cs < col_w_hu - 200` | 단락 들여쓰기를 LINE_SEG.column_start 로 인코딩한 paragraph 의 정상 라인 (sw+cs ≈ col_w_hu) 미진입 보장 |
| 기존 분기 보존 | OR 결합 (Picture/Shape Square wrap) | cs=0 인 기존 케이스 동일 동작 |

### 1.3 임계값 보정 (Stage 2 → Stage 3)

Stage 2 계획은 기존 임계값 `sw < col_w_hu - 200` 유지였으나, 첫 빌드 trace 에서 pi=110/118/120 (sw=30562 cs=1130 col_w_hu=31692) 가 spurious 활성화 발견.

```
sw=30562 < col_w_hu - 200 = 31492  → TRUE (잘못 활성)
```

이는 paragraph margin 을 LINE_SEG.column_start 로 인코딩한 paragraph 의 full-width line 이 narrow 로 잘못 판정되는 결함. **`sw + cs` 로 임계값 변경하여 정정**:

```
sw + cs = 30562 + 1130 = 31692 < 31492  → FALSE (정상)
```

Picture/Shape Square wrap 케이스 (cs=0) 는 sw + 0 = sw 라 동일 동작.

## 2. 검증 결과

### 2.1 단위 테스트
```
cargo test --lib
test result: ok. 1125 passed; 0 failed; 2 ignored; 0 measured
```

### 2.2 SVG snapshot
```
cargo test --test svg_snapshot
test result: ok. 6 passed; 0 failed
```
6/6 통과 (table-text, issue-147, issue-157, issue-267, form-002, render_is_deterministic).

### 2.3 clippy
사전 결함 2건 (`table_ops.rs:1007`, `object_ops.rs:298` — `unwrap()` will always panic) 변경 전후 동일. **본 변경에 의한 신규 경고/오류 0**.

### 2.4 광범위 fixture sweep (66 SVG 파일)

대표 fixture 7개 (인라인 표/수식/그림 wrap 보유):

| Fixture | 페이지 | 변경 |
|---------|------|------|
| `21_언어_기출_편집가능본.hwp` (Picture Square wrap) | 15 | byte-identical |
| `atop-equation-01.hwp` | 1 | byte-identical |
| `equation-lim.hwp` | 1 | byte-identical |
| `eq-01.hwp` | 1 | byte-identical |
| `exam_eng.hwp` | 8 | byte-identical |
| `exam_math.hwp` | 20 | byte-identical |
| `exam_kor.hwp` | 20 | byte-identical |
| **총** | **66** | **byte-identical** |

**회귀 0**.

### 2.5 의도된 정정 — exam_science.hwp

| 페이지 | 변경 | 사유 |
|------|------|------|
| 1 | byte-identical | 영향 없음 (조건 미충족) |
| **2** | **변경됨** | pi=61 ls[0] 인라인 분수 (12번 응답) 위치 정정 |
| 3 | byte-identical | pi=110 (13번) 등 sw+cs=full → 새 분기 미진입 |
| 4 | byte-identical | pi=118/120 등 동일 |

#### Pi=61 정정 정밀 측정

| 항목 | Before | After | 기대값 |
|------|--------|-------|--------|
| 인라인 2×1 표 (분수) x | **739.87** | **584.93** | ~575-590 |
| 편위 | +175 px (severe) | ±5-10 px (residual) | 0 |

선두 공백의 `extra_word_spacing` 80 px/space → 2.9 px/space 로 감소. 분수 우측 편위 거의 해소.

### 2.6 활성화 분포 (TASK568_TRACE)

| paragraph | 라인 | sq | tac_tbl | sw | cs | avail | 분기 |
|-----------|------|-----|---------|-----|-----|-------|------|
| pi=21 | 0..5 | true | false | 19592 | 0 | 19592 | OLD = NEW |
| pi=37 | 0..5 | true | false | 17546 | 0 | 17546 | OLD = NEW |
| pi=60 | 0..3 | true | false | 20069 | 0 | 20069 | OLD = NEW |
| **pi=61** | **0** | **false** | **true** | **18939** | **1130** | **20069** | **NEW (의도된 활성)** |

pi=110/118/120 sw+cs=31692 col_w_hu=31692 → 미진입 (정확).

## 3. 잔여 / 한계

### 3.1 pi=61 ls[0] 잔여 ~10 px

cs(15.07 px) + paragraph margin_left(15.07 px, resolved /2) 합 30 px 들여쓰기 적용. 한컴 의도가 cs OR margin 단일 인지 둘 다 인지는 PDF/한컴 2010/2020 비교 필요 (메모리 `feedback_pdf_not_authoritative` 정합 — 보조 ref). 시각적으로 ±10 px 는 무시 가능 수준.

### 3.2 미해결 사용자 보고 항목

다음 항목은 본 fix 로 정정되지 않음 (정정 코드 경로 불일치):

- **Page 1 header LEFT-shift (item ①)** — 외곽 1×1 표 cell 의 inline sub-tables (성명/수험번호/제선택) 가 cell halign=Center 미적용 → 별도 task 필요.
- **Page 3 보기 셀 분수 단락 (13/15/16번)** — 셀 내부 paragraph 의 inline TAC 표 + 수식. 셀 paragraph 의 segment_width = full cell width (좁지 않음) 으로 본 fix 의 임계값 미충족. 별도 분석 필요.
- **Page 4 보기 셀 분수 단락 (19번)** — 동상.
- **페이지 쪽번호 폰트 색상 (item ③)** — 별도 task.

작업지시자 피드백 "심하진 않고 좌측 편위가 생기는 문제도 있음" 은 위 미해결 항목 (특히 item ①) 의 잔존 또는 pi=61 ~10 px 잔여로 추정.

## 4. 변경 파일

```
src/renderer/layout/paragraph_layout.rs    +25 / -2 LOC
mydocs/working/task_m100_568_stage1.md     (Stage 1)
mydocs/plans/task_m100_568.md              (Stage 0 수행계획)
mydocs/plans/task_m100_568_impl.md         (Stage 2 구현계획)
mydocs/working/task_m100_568_stage3.md     (본 보고서)
```

## 5. 승인 요청

본 Stage 3 검증 결과를 바탕으로 Stage 4 (시각 판정 + 최종 보고) 진입을 승인 요청합니다.

다음 단계 후보 (Stage 4 후속 또는 별도 task):
- 미해결 항목 (item ①, 보기 셀 분수, item ③) 별도 issue 등록 → 추가 task
- 또는 본 task 의 잔여 처리에 한해 추가 진단 후 재정정
