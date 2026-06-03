# Task #521 Stage 2 — 추가 진단 + Stage 1 정정

**날짜**: 2026-05-04
**브랜치**: `pr-task521`
**선행**: Stage 1 진단 (`mydocs/working/task_m100_521_stage1.md`)

## 1. ⚠️ Stage 1 정정

**Stage 1 의 "박스 셀 본문 텍스트 완전 미렌더링" 발견은 거짓 양성**.

원인: SVG 의 영문 텍스트가 `x="..." y="..."` 속성 형식으로 렌더되는데, Stage 1 검증 regex 가 `translate(...)` 형식만 검사했음. 영문 본문 (이메일 내용) 은 정상 렌더링 됨.

### 1.1 정정된 측정 결과

페이지 2 우측 단 18번 박스 안 본문 (정상 렌더링):

| y | 본문 |
|---|------|
| 280 | "Dear Rosydale City Marathon Racers," |
| 305 | "We are really grateful to all of you who have signed up for" |
| ... | (이메일 본문 9줄) |
| 488 | "Sincerely," |
| 505 | "Martha Kingsley" |
| **523** | "Race Manager" (마지막 본문) |

박스 / ① 측정:

| 항목 | y |
|------|---|
| 박스 top (rect y) | 243.59 |
| 박스 bottom (rect y + h) | **531.68** |
| Race Manager baseline | 523.00 |
| ① 첫 답안 baseline | 543.95 |

**gap (박스 bottom → ① top)**: 543.95 - 531.68 = **12.27 px**.

## 2. 본질 재진단

### 2.1 IR vpos 정합 분석

- pi=104 vpos=2254 HU = 30.05 px from col top
- pi=104 ls[0] lh=22207 HU = 296.09 px (셀 h 21607 + outer_margin_bottom 600)
- pi=104 종료: vpos+lh = 2254 + 22207 = 24461 HU
- pi=105 IR vpos = 24805 HU
- IR gap: 24805 - 24461 = 344 HU = 4.59 px (= ls trailing line_spacing)

→ IR 기준 pi=105 위치 (body_y=209.76 가정):
- pi=105 expected y = 209.76 + 24805/75 = **540.49 px**

### 2.2 실제 vs IR vs PDF

| 위치 | y | 비고 |
|------|---|------|
| IR 기준 pi=105 | 540.49 | (body_y + 24805/75) |
| 본 devel SVG ① | **543.95** | IR + 3.46 px |
| PDF 기대 ① | **~551.68** | (box_bottom + 20 px gap) |

→ 본 devel 은 IR 보다 +3.46 px, PDF 보다 -7.7 px (-577 HU). PDF 가 IR 보다도 +11.19 px (+839 HU) 더 아래.

### 2.3 PDF deficit 본질 후보

**PDF 가 IR 보다 추가 11.19 px 아래에 ① 배치 → 본 devel 미반영 spacing**:

| 후보 | 가능성 | 본질 |
|------|------|------|
| **A** | 표 outer_margin_bottom 추가 적용 | 600 HU = 8 px (현 lh 에 이미 포함됐다고 가정) |
| **B** | spacing_after (pi=104 ParaShape) | spacing_after 미적용 가능성 |
| **C** | spacing_before (pi=105) | pi=105 spacing_before 미적용 |
| **D** | TopAndBottom wrap 표의 outer_margin_bottom 별도 적용 | 한컴 명세 정밀 확인 필요 |

→ pi=104 ParaShape: `spacing_before=0 after=0 line=130/Percent`. spacing_before/after 미적용 안 됨.
→ pi=105 ParaShape: `spacing_before=0 after=0`. 동일.

→ **D (표 outer_margin_bottom 별도 적용) 가 가장 가능성 높음**. lh 가 이미 outer_margin 포함하지만, 한컴은 wrap=TopAndBottom 표의 경우 추가로 outer_margin_bottom 을 next paragraph 앞에 적용하는 가능성.

## 3. 다른 fixture 영향 sweep

`tac=true + wrap=TopAndBottom + 셀 본문 텍스트` 패턴 fixture:

| fixture | 셀 본문 미렌더 결함 | 비고 |
|---------|------------------|------|
| exam_eng | ❌ 정상 (Stage 2 정정 후) | 본 cycle 대상 |
| exam_kor / exam_science / hwpspec / 등 | (확인 필요) | 광범위 sweep 필요 |

**Stage 2 정정 후, 결함은 "박스 본문 미렌더링" 이 아니라 "박스 ↔ 다음 paragraph spacing" 영역**. 

다른 fixture 의 같은 패턴 (overlay Picture + tac=true Table) sweep:

| fixture | 매칭 paragraph 수 | 비고 |
|---------|----------------|------|
| exam_eng | 2 (pi=0.0, pi=0.104) | pi=0.0 = 페이지 헤더 |
| exam_kor / hwpspec / aift / mel-001 / exam_science / synam-001 / kps-ai | 0 | 동일 패턴 부재 |

→ exam_eng pi=104 가 **유일한 발현 케이스**. 이슈 본문의 "BehindText 그림 + 인라인 표 + 다음 문단 패턴" 추정 영향 범위 (광범위) 와 달리 본 devel 에서는 단일 케이스.

## 4. 본질 후보 재확정

Stage 1 의 후보 (셀 paragraph 미렌더 root cause) 는 **거짓 양성** 으로 폐기.

새 본질 후보:

| 후보 | 영역 | 가능성 |
|------|------|-------|
| **D** | 표 outer_margin_bottom 의 다음 paragraph 앞 적용 | **높음** — 8 px deficit 와 outer_margin_bottom 600 HU = 8 px 일치 |
| E | TopAndBottom 표의 sequential advance 보정 | 중 — pi=104 advance 가 lh 만 사용하는지 확인 필요 |
| F | BehindText overlay 의 부수 영향 (이슈 본문 가설) | 낮음 — 실측 추가 spacing 11.19 px 와 BehindText offset/h 직접 일치 안 함 |

## 5. 옵션 평가

### 5.1 옵션 A — outer_margin_bottom 추가 적용 (후보 D)

**본질**: TopAndBottom 표의 outer_margin_bottom 을 lh 에 포함시키지 않고, 다음 paragraph 앞에 별도 spacing 으로 적용.

```rust
// 가설: 다음 paragraph 시작 시
y_offset += hwpunit_to_px(prev_table.outer_margin_bottom, dpi);
```

회귀 위험: **광범위** — 모든 TopAndBottom 표 + 다음 paragraph 영향. 회귀 가드 검증 필수.

### 5.2 옵션 B — Stage 3 추가 진단

`pagination/engine.rs` 의 paragraph height 산출 코드 추적. lh 와 outer_margin_bottom 분리 적용 여부 확인. 한컴 명세 (HWPML / HWP5 binary spec) 의 outer_margin 처리 명세 확인.

### 5.3 옵션 C — 단일 케이스 보류

발현 fixture 1건 (exam_eng pi=104) + 본 cycle 의 visual 임계 이하 (-7.7 px) → 우선순위 재평가.

## 6. 작업지시자 결정 사항

**Stage 2 정정 결과 종합**:

1. Stage 1 의 root cause 후보 (셀 미렌더) 거짓 양성 폐기
2. 발현 케이스: exam_eng pi=104 단일 (광범위 패턴 sweep 결과 부재)
3. 본질 후보 재확정: D (표 outer_margin_bottom 적용 영역) — 8 px deficit + outer_margin 600 HU = 8 px 정합

**진행 결정 후보**:

- **E1**: 옵션 A (outer_margin_bottom 별도 적용) 진행 — Stage 3 본질 정정 + 광범위 회귀 검증
- **E2**: 옵션 B (Stage 3 추가 진단) — 한컴 명세 확인 후 결정
- **E3**: 옵션 C (보류) — 우선순위 재평가, 단일 케이스 + 시각 임계 이하

권장: **E2** (한컴 명세 확인 후 옵션 A 진행 또는 보류 결정). 진행 결정 부탁드립니다.

---

## 7. E2 추가 진단 — 코드 추적 결과 (Stage 2 완결)

### 7.1 outer_margin_bottom 호출처 매핑

`grep -n "outer_margin" src/renderer/layout.rs`:

| 위치 | 함수 | 적용 컨텍스트 |
|------|------|-------------|
| `layout.rs:2192-2206` | `layout_table_item` | TAC 표 **위 (top)** outer_margin |
| `layout.rs:2642-2647` | `layout_partial_table_item` | TAC 표 **아래 (bottom)** outer_margin (partial table only) |
| `layout.rs:2478-2497` | `layout_table_item` (TAC after-spacing) | **outer_margin_bottom 미적용** ❌ |

### 7.2 본질 본 코드

`layout.rs:2478-2497` (TAC table after-spacing in `layout_table_item`):

```rust
if tac_seg_applied {
    if let Some(seg) = para.line_segs.get(control_index) {
        if seg.line_spacing > 0 {
            y_offset += hwpunit_to_px(seg.line_spacing, self.dpi);
        } ...
    }
    if let Some(ps) = styles.para_styles.get(ps_id) {
        if ps.spacing_after > 0.0 {
            y_offset += ps.spacing_after;
        }
    }
    return (y_offset, true);
}
```

vs `layout.rs:2638-2647` (`layout_partial_table_item`):

```rust
if is_tac {
    if para_style.spacing_after > 0.0 {
        y_offset += para_style.spacing_after;
    }
    let outer_margin_bottom_px = ...;  // ← outer_margin_bottom 적용
    if outer_margin_bottom_px > 0.0 {
        y_offset += outer_margin_bottom_px;
    }
}
```

### 7.3 pi=104 발현 경로

1. PageItem::Table (pi=104, ctrl=2) → `layout_table_item`
2. line 2374: `y_offset = self.layout_table(...)` → 표 cell content 높이만큼 advance (cell h=21607/75 = 288.09 px)
3. line 2386-2422: TAC 줄간격 처리
   - seg_idx=control_index=2
   - pi=104.line_segs 는 1개 (ls[0])
   - `para.line_segs.get(2)` = None → 모든 분기 skip
   - line 2421: `tac_seg_applied=true`
4. line 2478: `if tac_seg_applied`
   - line 2479: `seg.line_spacing` — `para.line_segs.get(2)` = None → skip
   - line 2491: `ps.spacing_after=0` → skip
   - **`outer_margin_bottom` 미적용**
   - return

→ 최종 y_advance = cell_h (21607 HU = 288.09 px). 누락된 outer_margin_bottom (600 HU = 8 px).

### 7.4 PDF 정합 검증

| 위치 | 현 devel | + outer_margin_bottom 8 px | PDF 기대 |
|------|---------|---------------------------|---------|
| ① first answer y | **543.95** | **551.95** | **551.68** |

→ outer_margin_bottom 적용 시 PDF 와 0.27 px 차이 (±2 px tolerance 내). **정확한 root cause** 확정.

### 7.5 한컴 명세 정합

HWP IR ls[0] lh=22207 = **cell h (21607) + outer_margin_bottom (600)** 으로 lh 정의. 즉 한컴 명세 상 lh 가 outer_margin_bottom 을 이미 포함.

`layout_table` 가 cell h 만큼만 advance 하고, lh 의 추가 600 HU 는 outer_margin_bottom 으로 별도 적용해야 함. `layout_partial_table_item` 는 정확히 적용, `layout_table_item` 는 누락.

→ **한컴 명세 정합 fix**: `layout_table_item` 의 TAC after-spacing 에 outer_margin_bottom 추가.

## 8. 본질 정정 본문 (제안)

`src/renderer/layout.rs:2491-2497` 정정:

```rust
if tac_seg_applied {
    if let Some(seg) = para.line_segs.get(control_index) {
        ...
    }
    let comp = composed.get(para_index);
    let ps_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
    if let Some(ps) = styles.para_styles.get(ps_id) {
        if ps.spacing_after > 0.0 {
            y_offset += ps.spacing_after;
        }
    }
+   // [Task #521] TAC 표 outer_margin_bottom 적용 (한컴 명세 정합).
+   // layout_partial_table_item:2642-2647 과 동일 처리.
+   let outer_margin_bottom_px = if let Some(Control::Table(t)) = para.controls.get(control_index) {
+       hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
+   } else { 0.0 };
+   if outer_margin_bottom_px > 0.0 {
+       y_offset += outer_margin_bottom_px;
+   }
    return (y_offset, true);
}
```

**변경 본질**: 5 LOC 추가 (조건 분기 없는 단일 룰).

## 9. 회귀 위험 평가

영향 범위:
- TAC + 외부 outer_margin_bottom > 0 인 표 후 paragraph
- 본 fix 가 적용되는 케이스 = 위 패턴 + `layout_table_item` 경로 (full table, not partial)

회귀 가드 후보:
- exam_eng pi=104: ① 위치 정합 (PDF ±2 px)
- 다른 fixture 의 TAC + outer_margin_bottom > 0 인 표 영향 sweep

`layout_partial_table_item` 의 동일 패턴이 이미 검증된 동작이므로, **`layout_table_item` 의 누락 케이스만 정합** → 회귀 위험 **낮음**.

## 10. 작업지시자 결정 사항

**E2 진단 결과**:

1. ✅ 한컴 명세 (lh = cell_h + outer_margin_bottom) 확인
2. ✅ 본질 본 코드 식별 (`layout_table_item:2478-2497`)
3. ✅ 정정 본문 식별 (5 LOC + outer_margin_bottom 추가)
4. ✅ 회귀 위험 평가 (낮음 — partial table 의 동일 패턴 정합)

**진행 결정**:

- **E1' (변경 후 Stage 3 진행 권장)**: 옵션 A 정정 적용 진행 (Stage 3-5)
- E3 (보류): 우선순위 재평가
