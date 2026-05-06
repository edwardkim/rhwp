# Task #628 Stage 1: 분석

## IR 구조 비교

### 19번 문항 (pi=119, 정상)

```
--- 문단 0.119 --- ctrls=1
[0] 표: 2행×3열 treat_as_char=true
    셀[0] (rs=1, cs=3) paras=1
      p[0] ctrls=1
        ctrl[0] 그림: bin_id=1, w=28300 h=5872 (99.8×20.7mm), tac=false, wrap=TopAndBottom
```

표 nesting: **1단** (paragraph → 2x3 표 → 셀 → 그림)

### 20번 문항 (pi=127, 결함)

```
--- 문단 0.127 --- ctrls=1
[0] 표: 1행×1열 treat_as_char=true     ← 외부 글상자 1x1 wrapper
    셀[0] paras=4
      p[0] ctrls=2
        ctrl[0] (Other)
        ctrl[1] 표: 2행×3열 tac=true   ← 내부 표
          셀[0] (rs=1, cs=3) paras=1
            p[0] ctrls=1
              ctrl[0] 그림: bin_id=2, w=28249 h=7636 (99.7×26.9mm), tac=false, wrap=TopAndBottom
      p[1] "◦ 의 질량은 (가)에서가 (다)에서의 배이다."
      p[2] "◦ 실린더 속 기체의 단위 부피당 원자 수는 (나)에서"
      p[3] "◦ 전체 원자 수는 (가)에서가 (다)에서의 배이다."
```

표 nesting: **2단** (paragraph → 1x1 → 셀 → 2x3 → 셀 → 그림)

→ 외부 1x1 wrapper 가 **글상자** 역할 (이미지 + 캡션 (가)/(나)/(다) + 관찰 bullet 통합 박스)

## SVG 출력 비교 (수정 전)

```
<image x=279.68 y=1003.39 width=57.28  height=56.32  ...>  ← 작은 인라인 1
<image x=426.59 y=1003.39 width=59.52  height=57.6   ...>  ← 작은 인라인 2
<image x=560.45 y=415.48  width=377.33 height=78.29  ...>  ← bin_id=1 (19번)
                                                             ← bin_id=2 누락!
```

높이 78.29 px = 20.7 mm × 96/25.4 → 19번(`bin_id=1`) 매칭. 20번(`bin_id=2`, 26.9mm = 101.8 px) 은 SVG 에 없음.

## 트레이스로 confirm

`Control::Picture` 매치 진입 트레이스 (`depth, cp_idx, ctrl_idx, bin_id`):

```
[TRACE_PIC] depth=0 cp_idx=0 ctrl_idx=0 bin_id=3       ← 작은 1
[TRACE_PIC] depth=0 cp_idx=0 ctrl_idx=0 bin_id=4       ← 작은 2
[TRACE_PIC] depth=0 cp_idx=0 ctrl_idx=0 bin_id=1       ← 19번 ✓
                                                         ← 20번 (bin_id=2) 누락!
```

20번 그림은 Picture 처리 분기에 **도달조차 못함** → 그 위에 있는 내부 2x3 표 자체가 미렌더 의심.

## `layout_table` 호출 트레이스

```
[TRACE_TBL] enter depth=0 table=2x3 tac=true y_start=413.60   ← 19번 내부 표
[TRACE_PIC] depth=0 cp_idx=0 ctrl_idx=0 bin_id=1               ← 19번 그림 ✓

[TRACE_TBL] enter depth=0 table=1x1 tac=true y_start=770.71   ← 20번 외부 1x1
                                                                 ← 내부 2x3 호출 누락!
[TRACE_TBL] enter depth=0 table=1x10 tac=true y_start=1119.11 ← 다음 paragraph (pi=129)
```

20번 외부 1x1 → 내부 2x3 의 `layout_table(depth=1)` 재귀 호출이 발생하지 않음.

## `Control::Table(nested_table)` 분기 트레이스

`table_layout.rs:1893` 의 `already_rendered_inline` 체크:

```
[TRACE_NESTED] depth=0 cp_idx=15 ctrl_idx=0 is_tac=true already_rendered_inline=false  table=2x3   ← 19번 ✓
[TRACE_NESTED] depth=0 cp_idx=0  ctrl_idx=1 is_tac=true already_rendered_inline=true   table=2x3   ← 20번 ✗
```

→ 20번은 `already_rendered_inline=true` 로 판정되어 `layout_table` 호출이 스킵됨 (`inline_x += tac_w` 만 수행).

## `set_inline_shape_position(0, 0, 1, ...)` 백트레이스

`already_rendered_inline=true` 의 원인은 **누가 `(sec=0, para=0, ctrl=1)` 키를 미리 등록**했기 때문. 백트레이스 캡처:

```
[TRACE_SETINL] sec=0 para=0 ctrl=1 x=269.11 y=633.62
   2: PageRenderTree::set_inline_shape_position
   3: layout_composed_paragraph         ← inner cell paragraph 의 inline TAC 표 등록
   4: layout_table_cells                 ← inner table 처리
   5: layout_table                        ← inner table (depth=1)
   6: layout_table_cells                 ← outer table 처리
   7: layout_table                        ← outer table (depth=0)
```

다른 paragraph 의 double-nested 셀 처리가 `(0, 0, 1)` 키를 점유 → 20번 외부 1x1 처리 시 stale 값을 보고 오판.

## 근본 원인

`PageRenderTree.inline_shape_positions` 의 키 `(section, para, control)` 에서 `para` 가 **두 가지 의미로 혼용**:

1. `paragraph_layout` 호출 시 → 섹션 단위 paragraph 인덱스 (예: 119, 127)
2. `layout_table` → 셀 paragraph 호출 시 → 셀 내부 paragraph 인덱스 (`cp_idx`, 보통 0)

→ 서로 다른 셀 컨텍스트가 동일 키 namespace 를 공유하여 충돌.

## 결론

키에 `cell_path` 를 추가해 namespace 분리해야 함.
