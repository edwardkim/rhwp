# Task #568 최종 결과 보고서 — exam_science.hwp 본문 인라인 표(분수)+수식 단락 우측 편위 정정

- **마일스톤**: v1.0.0 (M100)
- **이슈**: [#568](https://github.com/edwardkim/rhwp/issues/568)
- **브랜치**: `local/task568` (분기점: `local/devel`)
- **작성일**: 2026-05-04
- **선행 task**: #565 (closed) — 같은 단락의 인라인 수식 좌표 stack 정정
- **상태**: **Stage 4 시각 판정 대기**

## 1. 배경

`samples/exam_science.hwp` 본문에서 인라인 표(treat_as_char=true) + 인라인 수식이 함께 있는 단락의 첫 줄이 단락 left margin 기준이 아닌 column 우측으로 약 +175 px (≈ 21.5 mm) 편위되어 출력. 사용자 시각 보고로 식별됨.

영향 단락 (작업지시자 보고):
- p2 우단 12번 응답 (pi=61) — 분수 "1g 의 X 에 들어 있는 중성자수"
- p3 13번 / 15번 / 16번 / p4 19번 — 보기 셀 분수 단락

선행 task #565 의 fix(`a35bdbe` — 인라인 표+수식 단락을 일반 `layout_paragraph` 경로로 우회) 가 가린 별개의 결함이 표면화된 것.

## 2. 본질 결함

`src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph` L857-866 의 `effective_col_x / effective_col_w` 분기가 **인라인 TAC 표 보유 줄의 `comp_line.segment_width` 를 무시**.

HWP 는 인라인 TAC 표가 있는 줄의 segment_width 를 표 폭 + 잔여로 좁게 인코딩하지만 (wrap=TopAndBottom 영향), layout 은 컬럼 전체 폭(31692 HU = 422.6 px)으로 `effective_col_w` 를 잡았다. 결과:

```
available_width = 407.5 - 15.07 = 392.4 px         (실제로는 252.5 - 15.07 = 237.4 px 가 맞음)
slack = 392.4 - 231.6 = 160.8 px                    (실제 5.8 px 가 맞음)
extra_word_sp = 160.8 / 2 = 80.4 px / space         (실제 2.9 px / space)
table x = 564.94 + 2 × (5 + 80.4) = 735.74          (실제 ≈ 580 px 가 맞음)
```

선두 공백 2 개 × 80 px 부풀림으로 인라인 표가 +160 px 우측 편위.

## 3. 정정

### 3.1 코드 변경 — `paragraph_layout.rs` L857 분기 확장

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
    && line_avail_hu < col_area_w_hu - 200
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

### 3.2 핵심 설계

| 항목 | 결정 | 근거 |
|------|------|------|
| 활성 조건 추가 | `\|\| line_has_inline_tac_table` (줄 단위 검출) | 인라인 TAC 표 보유 paragraph 도 줄 단위 활성. line char range × tac_offsets_px × Control::Table(tac=true) 교차 |
| 임계값 보정 | `sw + cs < col_w_hu - 200` | 단락 들여쓰기를 LINE_SEG.column_start 로 인코딩한 paragraph 의 정상 full-width line (sw+cs ≈ col_w_hu) 미진입 보장. 기존 Picture wrap (cs=0) 동등 |
| 분기 보존 | OR 결합 | 기존 `has_picture_shape_square_wrap` 케이스 (pi=21/37/60) 동일 출력 |

### 3.3 변경 LOC

`src/renderer/layout/paragraph_layout.rs`: **+25 / -2**

## 4. 검증 결과

### 4.1 자동 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` | **1125 passed**, 0 failed, 2 ignored |
| `cargo test --test svg_snapshot` | **6/6 passed** |
| `cargo clippy --release --lib` | 본 변경 신규 경고 0 (사전 결함 2건 변경 전후 동일) |

### 4.2 광범위 fixture sweep — 회귀 0

대표 7 fixture / **66 SVG 페이지** byte-identical:

| Fixture | 페이지 | 결과 |
|---------|------|------|
| `21_언어_기출_편집가능본.hwp` (Picture Square wrap) | 15 | byte-identical |
| `atop-equation-01.hwp` | 1 | byte-identical |
| `equation-lim.hwp` | 1 | byte-identical |
| `eq-01.hwp` | 1 | byte-identical |
| `exam_eng.hwp` | 8 | byte-identical |
| `exam_math.hwp` | 20 | byte-identical |
| `exam_kor.hwp` | 20 | byte-identical |

### 4.3 의도된 정정 — `exam_science.hwp`

| 페이지 | 변경 | 사유 |
|------|------|------|
| 1 | byte-identical | 영향 없음 |
| **2** | **변경됨** | pi=61 ls[0] 인라인 분수 (12번 응답) 위치 정정 |
| 3 | byte-identical | pi=110 (13번) 등 sw+cs=full → 새 분기 미진입 |
| 4 | byte-identical | pi=118/120 등 동일 |

**Pi=61 인라인 분수 위치 측정**:

| 항목 | Before | After | 기대값 | 편위 |
|------|--------|-------|--------|------|
| 인라인 2×1 표 x | **739.87** | **584.93** | ~575-590 | ±5-10 px (잔여) |

선두 공백 `extra_word_spacing`: 80 px/space → 2.9 px/space.

### 4.4 활성화 분포 (TASK568_TRACE)

| paragraph | 라인 | sq | tac_tbl | sw | cs | avail | 분기 |
|-----------|------|-----|---------|-----|-----|-------|------|
| pi=21 | 0..5 | true | false | 19592 | 0 | 19592 | OLD = NEW (활성) |
| pi=37 | 0..5 | true | false | 17546 | 0 | 17546 | OLD = NEW (활성) |
| pi=60 | 0..3 | true | false | 20069 | 0 | 20069 | OLD = NEW (활성) |
| **pi=61** | **0** | **false** | **true** | **18939** | **1130** | **20069** | **NEW (의도된 활성)** |
| pi=110/118/120 | 다양 | false | true | 30562 | 1130 | 31692 | OLD/NEW 모두 미활성 (정확) |

## 5. 잔여 / 미해결 항목 (별도 task 후보)

본 fix 는 인라인 표 + 수식 + **narrow segment_width** 조합만 정정. 다음 항목은 별도 메커니즘:

| 항목 | 현상 | 코드 경로 | 권고 |
|------|------|-----------|------|
| ① **Page 1 header LEFT-shift** | 외곽 1×1 표 cell 의 inline sub-tables (성명/수험번호/제선택) 가 cell halign=Center 미적용 | cell halign 처리 | 별도 issue + task |
| **Page 3/4 보기 셀 분수 (13/15/16/19)** | 셀 paragraph 의 inline TAC 표 — 셀 paragraph segment_width = full cell width (좁지 않음) → 본 fix 임계값 미충족 | cell paragraph layout | 별도 진단 (다른 결함) |
| ③ **페이지 쪽번호 색·굵기** | 바탕쪽 쪽번호 fill=#000000 font-weight=bold | 바탕쪽 CharShape 적용 | 별도 issue + task |

### 5.1 작업지시자 피드백 분석

> "심하진 않고, 좌측 편위가 생기는 문제도 있음"

- "심하진 않고" → pi=61 ±5-10 px 잔여 (non-severe). HWP 의 cs / margin 결합 의도 (단일 vs 합산) 가 PDF/한컴 2010·2020 비교 필요. 메모리 `feedback_pdf_not_authoritative` 정합 — PDF 절대 기준 아님.
- "좌측 편위 도 있음" → 미해결 항목 ① (page 1 header) 잔존, 또는 위 잔여 ~5 px LEFT 편위 인지로 추정.

### 5.2 메모리 정합 검토

- `feedback_essential_fix_regression_risk`: ✓ 광범위 sweep 66 페이지 byte-identical, exam_science 4 페이지 중 page 2 만 의도된 정정.
- `feedback_rule_not_heuristic`: ✓ HWP 표준 룰 (LINE_SEG.cs/sw 기반 effective col 산출) 단일 룰. 휴리스틱 임계값 미도입 (200 HU 기존 임계값 재사용).
- `feedback_pdf_not_authoritative`: ✓ PDF 기준 미사용. SVG 좌표 + dump IR 데이터로 검증.

## 6. 산출물

```
src/renderer/layout/paragraph_layout.rs           +25 / -2 LOC
mydocs/plans/task_m100_568.md                     수행 계획서
mydocs/working/task_m100_568_stage1.md            Stage 1 진단 (코드 무수정)
mydocs/plans/task_m100_568_impl.md                Stage 2 구현 계획
mydocs/working/task_m100_568_stage3.md            Stage 3 구현+검증
mydocs/report/task_m100_568_report.md             본 최종 보고서
output/svg/exam_science_after/                    시각 판정용 SVG (4 페이지)
output/svg/exam_science_after_dbg/                debug-overlay SVG (4 페이지)
```

## 7. 커밋 이력

```
ebf0f99c Task #568 Stage 0: 수행 계획서 (인라인 표+수식 단락 우측 편위 정정)
e08c8897 Task #568 Stage 1: 정밀 진단 보고서 (코드 무수정) — 본질 결함 식별
ec5aea8c Task #568 Stage 2: 구현 계획서 (안 (a) 상세화)
5fba0abf Task #568 Stage 3: layout_composed_paragraph 분기 확장 …
```

## 8. 작업지시자 검토 사항

1. **시각 판정 — 본 task 효과**:
   - `output/svg/exam_science_after/exam_science_002.svg` — pi=61 (12번 응답) 인라인 분수 위치 정상화 확인
   - `output/svg/exam_science_after_dbg/exam_science_002.svg` — debug-overlay 좌표 (`s0:pi=61 ci=0 2x1`) 확인
2. **시각 판정 — 비회귀**:
   - `exam_science_001.svg`, `exam_science_003.svg`, `exam_science_004.svg` byte-identical (변경 없음)
   - 광범위 sweep 7 fixture 66 페이지 byte-identical
3. **rhwp-studio web Canvas 시각 판정**: WASM 빌드 후 browser 검증 (Docker 가동 환경)
4. **이슈 #568 close + local/devel merge** 결정: 시각 판정 통과 시
5. **잔여 항목 후속 처리 결정**:
   - Page 1 header sub-tables LEFT-shift (item ①) — 별도 issue + task 등록 여부
   - Page 3/4 보기 셀 분수 단락 (13/15/16/19) — 별도 진단 task 여부
   - Page 쪽번호 색·굵기 (item ③) — 별도 issue + task 등록 여부

## 9. 승인 요청

본 최종 보고서로 Task #568 완료 승인 요청합니다.
