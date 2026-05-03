# Task #496 단계 1 보고서 — 원인 정확 진단

**이슈**: #496
**브랜치**: `local/task496`
**단계 목표**: y=1179, 1191, 1195, 1198 baseline 4 라인이 어느 paragraph/lineseg/코드 경로에서 발행되는지 + 어느 단계에서 baseline 압축이 발생하는지 식별

---

## 1. 진단 방법

다음 진단 로그를 임시 추가해 발행 경로를 추적했다 (단계 1 종료 시 모두 제거):

1. **`paragraph_layout::layout_composed_paragraph`** 진입 진단: paragraph_index, y_start, composed.lines 출력
2. **`paragraph_layout::layout_paragraph`** 진입 진단: paragraph 본체 layout 호출 추적
3. **`paragraph_layout::layout_inline_table_paragraph`** 진입 진단: 인라인 표 paragraph 분기 추적
4. **`paragraph_layout` line emit (text_y/baseline)**: 단 1 영역 내 라인 발행 추적
5. **`layout::layout_column_item`**: page 2 단 1 의 column items 추적

## 2. 진단 결과

### 2.1 단 1 column items 순서 (page 2)

```
pi=47 (10번 시작) → ... → pi=58 (빈) → pi=59 (빈) → pi=60 (12번 본문 4 lines)
→ Shape pi=60 ci=0~8 (그림 + 수식 8개)
→ pi=61 FullParagraph (12번 후반부, "는? (단, ...)")    ← 핵심
→ Shape pi=61 ci=1~9 (수식 9개)
→ Table pi=62 ci=0
→ pi=63, pi=64 (빈)
```

### 2.2 paragraph_layout 호출 누락

- pi=58, 59, 60 → `layout_paragraph` → `layout_composed_paragraph` 호출 ✓
- **pi=61** → `layout_paragraph` **미호출** ⚠️
- pi=61 → `layout_inline_table_paragraph` **호출** (별도 경로)

### 2.3 분기 결정 — `layout::layout_column_item:1944`

```rust
let has_inline_tables = para.controls.iter()
    .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
        && crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls)));

if has_inline_tables {
    ...
    y_offset = self.layout_inline_table_paragraph(...);
} else {
    ...
    y_offset = self.layout_paragraph(...);
}
```

pi=61 의 ctrl[0] 표 (2행1열, tac=true) 가 `is_tac_table_inline=true` 로 판정되어 `layout_inline_table_paragraph` 분기 진입.

### 2.4 pi=61 의 IR

```
pi=61 paragraph.text = "  는? (단, 는 임의의 원소 기호이고, , , , 의 원자량은 각각 (식) (식) (식) 의 (식) (식) 합은 (식) 이다."
                       (실제 paragraph.text 는 dump-pages preview 보다 길다 — 여러 인라인 수식/표 포함)

ls[0]: vpos=74118 lh=2864 bl=1432 ls=460 sw=18939   ← 표 포함 큰 줄
ls[1]: vpos=77442 lh=1150 bl=575  ls=460 sw=18939   ← bl 비정상적으로 작음
ls[2]: vpos=79052 lh=1150 bl=575  ls=460 sw=30562   ← sw 변경 (단 너비 다름)

controls:
  ctrl[0]: 표 (2x1, tac=true, wrap=TopAndBottom, size=14745×2864 HU)
  ctrl[1~9]: 수식 9개 (tac=true)
```

pi=60 (정상 paragraph) 비교:
```
ls[1]: lh=1150 bl=978 ← 정상 baseline
```

→ pi=61 ls[1]/ls[2] 의 `baseline_distance=575` 가 비정상적으로 작음 (정상 ls[1] bl=978 의 약 60%). HWP 파일이 표 인라인 paragraph 의 후속 줄에 작은 baseline 을 저장한 것.

## 3. SVG baseline 분포 (단 1 12번 영역)

### 수정 전 baseline 분포

```
1037.63 ← pi=60 line[0] "12.-그림은 원자 의 중성자수와"
1048.56 ← (검증 필요 — 라벨/마커?)
1060.56 ← pi=60 line[1] "전자 수의 차와 질량수를 나타"
1070.03 ← (라벨/마커?)
1082.03 ← pi=60 line[2] "낸 것이다. 는 원소 의 동위 원"
1091.49 ← (라벨/마커?)
1103.49 ← pi=60 line[3] "소이고, 의 중성자수 합은 이다."
   (그림 영역 점프)
1179.68 ← <g transform>: 수식 (글자 아님)
1191.68 ← pi=0 cell_ctx=Some(61) 표 row 0 글자 "의 에 들어 있는 중성자수"
1195.85 ← pi=61 paragraph 본문 "는?(단,는임의의원소기호" (layout_inline_table_paragraph 가 발행)
1198.77 ← <g transform>: 수식
```

### 결함 핵심

표 (ctrl[0], 2행1열) 가 인라인으로 baseline 1191.68 (행 1) + 1210.77 (행 2) 위치에 그려진다. 그러나 paragraph 본문 텍스트가 표의 행 1/행 2 영역과 겹친 baseline 1195.85 에 그려져 **시각적 겹침**.

paragraph 본문 baseline 1195.85 와 표 row 0 baseline 1191.68 차이 = 4px. 4px 안에 글자 + 표 글자 모두 있어 압축으로 보임.

## 4. `layout_inline_table_paragraph` 의 multi-line + multi-row 표 미지원

`layout_inline_table_paragraph` 코드 분석:

```rust
// 줄바꿈은 line_break_char_idx 기반 1회만 처리
let need_wrap = if let Some(break_idx) = line_break_char_idx {
    ch_idx >= break_idx && !wrapped_below_table
} else { ... };

if need_wrap {
    if !wrapped_below_table && max_table_bottom > y {
        current_y = max_table_bottom;
        wrapped_below_table = true;  // ← 한 번만 줄바꿈
    } else {
        current_y += line_step;
    }
    ...
}
```

- ls[1].text_start 만 사용 (`para.line_segs[1].text_start`). ls[2] 무시.
- `wrapped_below_table` 가 한 번 true 가 되면 더 이상 줄바꿈 안 함.

→ pi=61 처럼 ls 가 3 개 (인라인 표 + 텍스트 2 줄) 이고 표가 2 행이라 multi-row 인 경우 처리 모델이 부정확.

또한 `current_y = max_table_bottom` 로 줄바꿈 시 표 **하단** 으로 이동하는데, 표가 2 행이면 표 1 행 하단(=2 행 시작)으로 이동. 결과적으로 텍스트가 표 2 행과 겹침.

## 5. 가설 — 결함 본질

1. **pi=61 의 ls[1]/ls[2] bl=575** 는 HWP 가 표 인라인 paragraph 의 후속 줄을 작은 baseline 으로 저장한 것 — `ensure_min_baseline` 으로 보정되지만 텍스트 줄 위치 자체는 표 행과 정렬되어야 함.
2. **`layout_inline_table_paragraph` 가 multi-row 표 + multi-line 텍스트** 처리를 못함:
   - 표가 2 행이면 표 본체가 (행1 + 행2) 두 줄 차지
   - paragraph 본문 텍스트가 표 옆 / 표 아래 어디로 갈지 정책이 모호
   - 현재 코드는 표 1 행 하단으로 텍스트를 이동하지만 그 위치가 표 2 행과 겹침

## 6. 수정 방향 후보 (단계 2 에서 결정)

### (A) `layout_inline_table_paragraph` multi-row 표 보강
- `max_table_bottom` 계산 시 표의 모든 행 합산 높이 사용
- 텍스트 줄바꿈 시 표 전체 하단으로 이동
- 단점: 변경 범위 큼, 다른 케이스 회귀 위험

### (B) pi=61 분기 변경 — `layout_paragraph` 사용
- `is_tac_table_inline` 판정에서 multi-row 표는 인라인 처리 제외
- block 표 또는 일반 paragraph 처리로 fallback
- 단점: `is_tac_table_inline` 룰 변경 — 다른 paragraph 영향 가능

### (C) ls[1]/ls[2] vpos 기반 정렬 — 본 paragraph 만 가드
- pi=61 의 IR vpos 가 정확하므로, layout_inline_table_paragraph 가 ls[i].vpos 를 직접 사용하여 각 line baseline 을 계산
- 표/수식 인라인 위치는 별도 처리하지 말고 paragraph_layout 의 정상 multi-line 모드 재사용
- 가장 룰-정합적이지만 코드 통합 필요

### (D) 분기 자체 보류 — 본 task 범위 재정의
- pi=61 표 + paragraph 처리는 layout 의 본질적 한계. 단독 paragraph 수정으로 안전 해결 어려움
- #495 의 잔존 사각형 위치 결함 (#500) 과 동일한 본질적 inline tac 처리 한계
- 본 task 를 보류하고 더 큰 layout 리팩터링에 통합

## 7. PDF 정답과의 비교 (참고)

PDF 페이지 2 12번:
- "12. 그림은 원자 W~Z의 중성자수와 [그림]"
- "전자 수의 차와 질량수를 나타낸 것이다."
- 표 (2 행) — 그림 옆 또는 본문 옆 별도 위치
- 본문 마무리 "(단, X는 ..., X, Y, Z의 원자량은 각각 ...)"

→ 표가 paragraph 본문과 **분리되어** 다른 위치에 있다. 즉 한컴은 본 paragraph 의 표를 인라인으로 처리하지 않고 별도 블록으로 처리한 것으로 보임. 후보 (B) 가 한컴 동작과 일치.

## 8. 위험 / 권고

- 메모리 `feedback_essential_fix_regression_risk.md`: layout 본질 정정은 회귀 위험 큼.
- 후보 (B) 가 PDF 와 가장 정합적이지만 `is_tac_table_inline` 변경은 인라인 표 케이스 광범위에 영향. 다수 샘플 회귀 검증 필수.
- 후보 (D) 보류는 본 사이클에서 안전한 선택. #500 과 함께 layout 리팩터링 시 종합 해결.

## 9. 단계 2 입력

- 결함 위치: `src/renderer/layout/paragraph_layout.rs::layout_inline_table_paragraph` 의 multi-line + multi-row 표 처리 한계
- 분기 결정 위치: `src/renderer/layout.rs:1944` (`is_tac_table_inline` 판정)
- 수정 방향 후보: (A)/(B)/(C)/(D) — 단계 2 에서 결정
- 진단 코드 모두 제거. `git diff src/` 빈 상태 확인.

## 10. 단계 2 진입 권고

자동승인 모드이지만 본 결함은 회귀 위험 큰 layout 본질 정정. 단계 2 (구현 계획서) 작성 시 후보 (D) 보류 결정이 가장 안전한 선택일 수 있음. 작업지시자 판단을 단계 2 에서 명시적으로 받기 권고.
