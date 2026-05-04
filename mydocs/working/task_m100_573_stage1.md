# Task #573 Stage 1 진단 보고서 — 본질 결함 식별

- **이슈**: [#573](https://github.com/edwardkim/issues/573)
- **브랜치**: `local/task573`
- **단계**: Stage 1 — 정밀 진단 (코드 무수정)
- **작성일**: 2026-05-04

## 1. 결론 (요약)

작업지시자 보고 "셀 paragraph 인라인 분수 우측 편위" 의 **실제 본질 결함은 surrounding text 미렌더**.

`samples/exam_science.hwp` 보기 셀 내부 paragraph (예: pi=68 cell[5] p[2] "ㄷ. \[분수\] \[수식\] 이다.") 에서 **인라인 분수 cell content 와 inline 수식만 SVG 에 렌더되고 surrounding text "ㄷ. " 와 "이다." 가 누락됨**. 분수가 cell content 좌단(x=97.07)에 단독으로 보이므로 사용자는 이를 "우측 편위" 로 인지 (실제로는 "좌측 텍스트 누락 + 분수만 노출").

가설 A/B/C 모두 부정 — **새 가설 D**: cell paragraph 의 inline 표 + inline 수식 단락에서 surrounding text run 이 layout pipeline 에서 누락.

## 2. 정밀 측정 — pi=68 cell[5] (page 3 13번 보기)

### 2.1 IR 데이터 (rhwp dump)

```
[0] 외곽 표 3×3 (pi=68 ci=0): 보기 표
[0]   셀[5] r=2,c=0 rs=1,cs=3 h=2850 w=30557 paras=3
[0]     p[0] ps_id=59 ctrls=2 text_len=56 ls[0] vpos=0 lh=1148 ls=516
        text="ㄱ. (나)와 (다)에서 [수식 B(s)] 는 환원제로 작용한다."
[0]     p[1] ps_id=59 ctrls=1 text_len=12 ls[0] vpos=1947 lh=2580 ls=516
        text="ㄴ. [수식 b/c=2/3] 이다."
[0]     p[2] ps_id=59 ctrls=2 text_len=12 ls[0] vpos=5326 lh=2864 ls=516
        text="ㄷ. [분수 (다)에서 반응한.../(나)에서 생성된...] [수식 mol] 이다."
[0]     p[2] 내부표: 2행×1열, 셀=2
[0]       셀[0] text="(다)에서 반응한 의 양()"
[0]       셀[1] text="(나)에서 생성된 의 양()"
```

### 2.2 SVG 측 실측 (`/tmp/task573/baseline_dbg/exam_science_003.svg`)

#### 2.2.1 외곽 보기 표 (pi=68 ci=0)
- `<rect x="85.73" y="583.56" width="407.43" height="149.07"...>`
- 보기 셀 cell-clip-161: `x=85.73, y=600.76, width=407.43, height=131.87`
- cell content area: x=97.07..481.43 (cell.x + padding 11.34)

#### 2.2.2 paragraph p[0] (ㄱ. line) — 정상 렌더
```
y=625.11:
  x=97.07  "ㄱ"
  x=111.15 "."
  x=121.73 "("
  x=128.54 "나"
  ... (모든 visible 글자 + 수식 inline) ...
  x=399.43 "."  (마지막)
```
✓ 모든 visible 텍스트 + 수식 정상 렌더.

#### 2.2.3 paragraph p[1] (ㄴ. line) — 정상 렌더
```
y=660.41:
  x=97.07  "ㄴ"
  x=111.15 "."
  x=122.07 (수식 b/c=2/3, transform y=639.69, font-size 14.67)
  x=165.48 "이"
  x=179.56 "다"
  x=193.64 "."
```
✓ 모든 visible 텍스트 + 수식 정상 렌더.

#### 2.2.4 paragraph p[2] (ㄷ. line) — **surrounding text 누락**
```
y=683.11..702.20  (cell-clip-171 — 분수 상단 cell):
  x=103.87  "(다)에서 반응한 의 양"  (분수 셀 0 내부 텍스트)
  + 수식 X (variable)
y=702.20..721.29  (cell-clip-178 — 분수 하단 cell):
  x=103.87  "(나)에서 생성된 의 양"  (분수 셀 1 내부 텍스트)
  + 수식 X (variable)

** "ㄷ" / "." / 분수 후 "이다." / "." 모두 SVG 에 미존재 **
```

✗ surrounding text "ㄷ. ", "이다.", "." 모두 미렌더. 단지 분수 cell content + inline 수식만 보임.

### 2.3 가설 검증

| 가설 | 결과 | 근거 |
|------|------|------|
| **A** Last line + Justify x_start | ✗ 부정 | Surrounding text 자체 미렌더. Justify slack 분배 무관. |
| **B** Cell halign / paragraph alignment | ✗ 부정 | Alignment 와 무관. Text run 이 layout 단계에서 누락. |
| **C** run_tacs 의 인라인 표 char_offset | ✗ 부분 부정 | char_offset 결함이라면 분수 위치가 어긋날 텐데 분수는 정상 위치. |
| **D (신규)** cell paragraph + inline 표 + inline 수식 단락의 surrounding text run 미렌더 | ✓ **확정** | 정상 paragraph (p[0], p[1]) 와 비정상 paragraph (p[2]) 의 차이가 inline 표 보유 여부. |

### 2.4 정상 / 비정상 paragraph 대조 매트릭스

| paragraph | inline 표 | inline 수식 | text 렌더 | 분수 위치 |
|-----------|----------|-----------|-----------|-----------|
| p[0] (ㄱ.) | ✗ | 1개 (B(s)) | ✓ 정상 | (분수 없음) |
| p[1] (ㄴ.) | ✗ | 1개 (b/c=2/3) | ✓ 정상 | (분수 없음) |
| **p[2] (ㄷ.)** | **1개** | **1개 (mol)** | **✗ surrounding 누락** | x=97.07 (cell 좌단) |

**결정적 차이**: inline 표 보유 여부. inline 표 + inline 수식 동시 보유 시 surrounding text 누락.

## 3. 사용자 보고와의 정합

작업지시자: "p3 13번 (다)에서 반응한 의 양() — 우측 편위"

실제: 분수 cell content "(다)에서 반응한 의 양" 은 x=97.07 (cell content 좌단). 이는 ㄱ./ㄴ./ㄷ. 문자가 시작하는 x 좌표와 동일. **분수가 우측 편위된 것이 아니라, ㄷ. 등 surrounding text 가 누락되어 분수가 line 좌단에 단독 노출** → 시각적으로 "분수가 어색하게 위치" 인지.

비슷한 시각 인지:
- p3 15번 "제 N 이온화 에너지" — pi=82 cell[5]
- p3 16번 "ㄴ. ㄷ." — pi=75 cell[5]
- p4 19번 "제 N 이온화 에너지" — page 4 보기 셀

(15/16/19번 동일 패턴 검증은 Stage 2 에서 — 본 Stage 는 13번 본질 식별로 충분.)

## 4. 코드 경로 추정

### 4.1 cell paragraph 진입점

`src/renderer/layout/table_layout.rs:1480`:
```rust
para_y = self.layout_composed_paragraph(
    tree, &mut cell_node, composed, styles,
    &inner_area, para_y, 0, end_line,
    section_index, cp_idx,
    cell_context.clone(),    // Some(...)
    is_last_para,
    0.0,
    None, Some(para), Some(bin_data_content),
);
```

→ `layout_composed_paragraph` (paragraph_layout.rs:665) 동일 함수 호출. Top-level paragraph 와 같은 코드 경로.

### 4.2 Task #565 fix 와의 관계

`src/renderer/layout.rs::layout_column_item` 의 `has_inline_tables && !has_other_inline_ctrls` 가드 — **top-level paragraph 만 분기**. cell paragraph 는 이 분기를 통과하지 않고 직접 `layout_composed_paragraph` 호출.

→ Task #565 fix 는 top-level paragraph 의 인라인 표+수식 라우팅을 정정 (layout_inline_table_paragraph → layout_paragraph). cell paragraph 는 항상 layout_composed_paragraph 직행이므로 영향 없음.

### 4.3 추정 본질 (Stage 2 에서 정밀화)

`layout_composed_paragraph` 의 run rendering 분기에서:
- **인라인 표만**: 정상 (Task #568 fix 적용)
- **인라인 수식만**: 정상 (Task #565 fix 후)
- **인라인 표 + 인라인 수식 동시**: 결함 가능

Cell paragraph 의 경우 inline 표가 cell child 로 등록되어 layout_table 호출 시점에서 surrounding text run 이 누락될 수 있음.

특히 `tac_offsets_px` 처리 loop (L1493-1939) 에서 인라인 표 처리 시 `seg_start = tac_rel` 후 다음 tac 까지 `seg = run_chars[seg_start..tac_rel]` 부분이 surrounding text 를 emit 해야 하는데, 어떤 분기에서 누락 가능성.

## 5. 우선순위 제안

본 결함은 단순 위치 편위가 아닌 **텍스트 누락** — 가독성 / 정확성 영향 큼. M100 milestone 에서 Task #568 (closed) 다음 우선 처리 권고.

## 6. Stage 2 권고

Stage 2 에서:
1. `layout_composed_paragraph` 의 run_tacs loop 정밀 추적 — cell paragraph 케이스에서 surrounding text run 처리 분기 확인
2. p[2] 의 ComposedParagraph.lines, runs, tac_offsets_px 데이터 다이렉트 dump (임시 example)
3. 정상 paragraph (p[1] 인라인 수식만) 과 비정상 paragraph (p[2] 인라인 표+수식) 의 코드 경로 분기점 식별
4. 4-5 안 비교 후 정정 방향 확정

## 7. 회귀 위험 (Stage 2 사전 평가)

본 정정은 cell paragraph + inline 표/수식 처리 분기 — Task #565/#568 와 같은 코드 영역 재건드림. 메모리 `feedback_essential_fix_regression_risk` 정합 — 광범위 sweep 필수.

## 8. 산출물

- 본 보고서: `mydocs/working/task_m100_573_stage1.md`
- baseline SVG (visual 검증용): `/tmp/task573/baseline_dbg/exam_science_003.svg`

## 9. 승인 요청

본 진단 결과 (가설 D 확정 — surrounding text 미렌더) 를 바탕으로 Stage 2 (구현 계획서 작성, 코드 분기 추적 + 안 비교) 진입을 승인 요청합니다.
