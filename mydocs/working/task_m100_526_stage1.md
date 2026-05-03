# Task #526 Stage 1 — 진단·재현

## 요약

**원인 확정.** `layout_inline_table_paragraph` (paragraph_layout.rs:88-603) 가 **인라인 표만** 처리하고 동일 문단의 다른 TAC 컨트롤(수식·Picture·Form)에 대해서는 `set_inline_shape_position` 등록을 누락한다. 이로 인해 후속 shape 패스의 `shape_layout::layout_shape` (shape_layout.rs:132-182) 가 `inline_pos = None` 으로 보고 fallback 경로 (140-152) 로 진입하여 모든 수식을 동일 좌표 `(col_area.x, para_y)` 에 stack 한다.

**영향 범위는 pi=61 단독이 아닌 5개 문단**: pi=61, 79, 110, 118, 120. 사용자는 가장 시각적으로 두드러지는 pi=61 (9개 수식 stack) 만 보고했다.

## 1. 라우팅 확정

`layout.rs:2003-2025` 에서 인라인 표(`tac=true && is_tac_table_inline=true`) 보유 시 `layout_inline_table_paragraph` 로 dispatch:

```rust
let has_inline_tables = para.controls.iter()
    .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char
        && crate::renderer::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls)));

if has_inline_tables {
    y_offset = self.layout_inline_table_paragraph(...);
} else {
    y_offset = self.layout_paragraph(...);  // → layout_composed_paragraph
}
```

`layout_composed_paragraph` 경로(606+)는 `set_inline_shape_position` 을 정상 호출(1842 Shape, 1899 Equation, 1919 Table). 그러나 `layout_inline_table_paragraph` 는 이 호출이 전혀 없다 (전체 메서드에서 `set_inline_shape_position` grep = 0건).

## 2. `layout_inline_table_paragraph` 구조 분석

| 라인 | 처리 |
|------|------|
| 117-124 | `inline_tables` 만 추출 (Control::Table && tac=true) |
| 151-179 | `char_offsets` gap 분석으로 텍스트 세그먼트 분리 |
| 211-229 | 텍스트 세그먼트 폭 계산 (수식/그림 폭 미반영) |
| 336-563 | 텍스트 세그먼트 + 표를 순차 배치 — **수식/Picture 컨트롤은 segments 처리 루프에 진입하지 않음** |

수식·Picture·Form TAC 컨트롤은 segments 분할 시 단순히 "갭"으로 인식되어 텍스트가 끊어지지만, 그 갭에 해당하는 컨트롤이 무엇인지는 검사하지 않는다. 즉 `layout_inline_table_paragraph` 는 **인라인 표를 가진 문단에서 수식·그림이 함께 등장할 가능성을 전혀 고려하지 않은 코드**.

## 3. pi=61 측정 데이터

`RHWP_LAYOUT_DEBUG=1 cargo run --release -- export-svg samples/exam_science.hwp -p 1` 결과:

```
LAYOUT_INLINE_TABLE_PARA: pi=61 sec=0 col_x=534.8 col_w=422.6 y_start=1119.1
                          y=1119.1 sb=0.0 sa=6.7 ml=15.1 mr=0.0 align=Justify
                          ls_count=3 tables=1
  LAYOUT_LS[0]: vpos=74118 lh=2864 ls=460 bl=1432 text_start=0    sw=18939
  LAYOUT_LS[1]: vpos=77442 lh=1150 ls=460 bl=575  text_start=13   sw=18939
  LAYOUT_LS[2]: vpos=79052 lh=1150 ls=460 bl=575  text_start=60   sw=30562
  LAYOUT_INLINE_TBL[0]: ctrl_idx=0 rows=2 cols=1 w=14745 h=2864 wrap=TopAndBottom
  LAYOUT_BREAK_INDICES: pi=61 indices=[5, 28] (from ls[1..])
```

- `col_area.x = 534.8` (페이지 우측 단)
- `y_start = 1119.1`
- ls[0] = 표 줄 (lh=2864 → 38.2 px), ls[1]/ls[2] = 텍스트 줄 (각 lh=1150 → 15.3 px)
- 텍스트 줄 분기 위치: char 5, char 28 (체인 BREAK_INDICES)

이슈에 보고된 stack 좌표 `gx=534.8, gy=1206.91`:
- `gx = col_area.x` ✓ → fallback 의 `eq_x = col_area.x` (Justify 분기) 와 일치
- `gy ≈ y_start + 88` → fallback 의 `eq_y = para_y` (호출자가 전달하는 Shape pass 의 anchor y, ls[2] 부근으로 추정)

## 4. pi=61 컨트롤 구성

```
ctrl[0] = 표 (2x1, tac=true)        ← 인라인 표
ctrl[1] = 수식 "rmX"     (X)         ← stack 대상
ctrl[2] = 수식 "rmA"     (A)         ← stack 대상
ctrl[3] = 수식 "rmB"     (B)         ← stack 대상
ctrl[4] = 수식 "rmC"     (C)         ← stack 대상
ctrl[5] = 수식 "rmD"     (D)         ← stack 대상
ctrl[6] = 수식 "m-4"     (m-4)       ← stack 대상
ctrl[7] = 수식 "m-2"     (m-2)       ← stack 대상
ctrl[8] = 수식 "m+2"     (m+2)       ← stack 대상
ctrl[9] = 수식 "m+4"     (m+4)       ← stack 대상
```

기대 배치:
- ls[1] (vpos=77442, gy≈1145): "  는? (단, [X]는 임의의 원소 기호이고, [A], [B], [C], [D], 의 원자량은 각각"
- ls[2] (vpos=79052, gy≈1167): "[m-4], [m-2], [m+2], [m+4] 이다.) [3점]"

## 5. 영향 범위 확정

`RHWP_LAYOUT_DEBUG=1` 으로 `samples/exam_science.hwp` 전체 렌더링 시 `LAYOUT_INLINE_TABLE_PARA` 로그 5건. 모두 인라인 표 + TAC 수식 혼재:

| pi  | controls | 인라인 표 | TAC 수식 수 |
|-----|----------|-----------|------------|
| 61  | 10       | ctrl[0]    | 9 (ctrl[1..9]) |
| 79  | 11       | ctrl[1], ctrl[3] | 9 (ctrl[0,2,4..10]) |
| 110 | 8        | ctrl[1]    | 7 (ctrl[0,2..7]) |
| 118 | 9        | ctrl[7]    | 8 (ctrl[0..6,8]) |
| 120 | 2        | ctrl[1]    | 1 (ctrl[0]) |

→ **본 결함은 pi=61 단독이 아닌 5개 문단에 동시 발생** 가능성. pi=120 은 수식 1개라 stack 시각이 불명확할 수 있으나 위치 자체는 잘못된 fallback 위치로 갈 것으로 예상. 시각 확인은 Stage 2 후 회귀 검증 단계에서 진행.

## 6. 비교 — pi=18 (4번 문제 후속) 정상 동작

페이지 1 `pi=18` ("  ㉠과 ㉡으로 가장 적절한 것은?") 7개 인라인 수식 → 인라인 표 없음 → `layout_paragraph` → `layout_composed_paragraph` 경로 → `set_inline_shape_position` 호출 (paragraph_layout.rs:1899) → shape_layout fallback 우회 → 정상 인라인 배치.

이는 본 가설이 맞다는 강력한 대조군. 인라인 표 유무가 유일한 차이.

## 7. 수정 후보

### A안 (권장) — `layout_inline_table_paragraph` 안에서 인라인 수식·Picture·Form 처리 추가

`layout_composed_paragraph:1842/1899/1919` 의 inline TAC 처리 패턴을 `layout_inline_table_paragraph` 의 segments 처리 루프에 이식. 컨트롤 위치(char_offsets gap)에서 컨트롤 종류를 검사하여:
- 표 → 기존 `layout_table` 경로 유지
- 수식 → `EquationNode` 직접 추가 + `set_inline_shape_position` 등록
- Picture → `ImageNode` 추가 + `set_inline_shape_position` 등록
- Form → `FormObjectNode` 추가 + `set_inline_shape_position` 등록

장점: `layout_inline_table_paragraph` 자체 완성. 텍스트 흐름과 정확히 정렬.
단점: 메서드 확장 (~150-200 라인 추가).

### B안 — 인라인 표 + 수식 혼재 문단을 `layout_composed_paragraph` 로 라우팅

`layout.rs:2003-2025` 에서 `has_inline_tables && (no_other_tac_shapes)` 일 때만 `layout_inline_table_paragraph` 사용. 그렇지 않으면 `layout_composed_paragraph` 로.

장점: 코드 추가 없음. layout_composed_paragraph 의 검증된 inline 처리 재사용.
단점: layout_composed_paragraph 가 인라인 표 1개 처리는 가능하나(:1902-1922), 여러 표 + 줄바꿈 + 수식 혼재의 복잡한 경우는 검증 필요. 다단/줄간격/세그먼트 폭 계산이 layout_inline_table_paragraph 의 전용 로직과 다를 수 있어 회귀 위험.

### C안 — 최소 변경: set_inline_shape_position 등록만 추가

`layout_inline_table_paragraph` 안에서 모든 TAC 수식·Picture 에 대해 `set_inline_shape_position` 을 임의 위치(예: y_start, x=col_area.x)로 등록. shape_layout fallback 진입 자체를 막는다. 그러나 실제 텍스트 위치와 다르므로 결국 잘못된 위치에 렌더 → 의미 없음.

→ **A안 채택 권장**. 회귀 위험은 Stage 3 svg_regression_diff 로 통제.

## 8. 결론 — 완료 기준 충족

수행 계획서 Stage 1 완료 기준: "9개 수식이 fallback 으로 떨어지는 정확한 코드 경로/조건 식별" — 충족.

- 코드 경로: `layout.rs:2013` → `paragraph_layout.rs:88 layout_inline_table_paragraph` (set_inline_shape_position 누락) → shape pass `shape_layout.rs:135 inline_pos.is_none() == true` → `shape_layout.rs:140-152 fallback`
- 조건: 인라인 표(`tac=true && is_tac_table_inline()`) 와 TAC 수식·Picture 가 같은 문단에 공존
- 영향: pi=61, 79, 110, 118, 120 — 본 샘플에서 5건

다음 단계 Stage 2 의 구현 계획서는 A안 기준으로 작성한다.

---

승인 요청: 본 진단 결과 + A안 채택 + Stage 2 진행 가능 여부.
