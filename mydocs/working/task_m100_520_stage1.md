# Task #520 Stage 1 — 진단 로그 + 원인 확정

## 1. 진단 로그 출력

`paragraph_layout.rs:1840` 와 `shape_layout.rs:222` 에 `RHWP_DEBUG_T520` 가드 로그 삽입 후 `samples/exam_science.hwp` 빌드 실행:

```
[T520-shape] sec=0 pi=0 ci=0 tac=false wrap=BehindText inline_pos=None para_y=149.28      ← 바탕쪽 도형
[T520-shape] sec=0 pi=0 ci=0 ...                                                          (반복, 머리말/바탕쪽)
[T520]       sec=0 pi=1 ci=1 line_idx=1 y=228.21 baseline=12.27 shape_h=22.88
              shape_y=228.21 wrap=TopAndBottom tac=true
              ls_count=2 ls_vpos=[(1610, 1150), (3220, 1716)]
[T520-shape] sec=0 pi=0 ci=0 ...                                                          (반복)
```

**관찰 1**: 셀 내부 ㉠ 사각형(sec=0, pi=1, ci=1)은 paragraph_layout 가 **올바른 line_idx=1, shape_y=228.21** 로 등록한다.

**관찰 2**: `[T520-shape]` 는 cell 내부 inline shape(sec=0, pi=1, ci=1)에 대해 **한 번도 호출되지 않는다**. 즉 `layout_column_shapes_pass` → `shape_layout::layout_shape` 경로는 본 케이스에 진입하지 않으며, `inline_pos` 룩업이 사용되지 않는다.

## 2. 실제 SVG 출력 좌표

```
<rect x="97.07" y="249.68" width="62.99" height="22.88"
      fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
```

paragraph_layout 가 등록한 `shape_y=228.21` 와 **실제 그려진 y=249.68 가 21.47 px 차이**.
21.47 px = 1610 HWPUNIT 의 px 환산 = `ls[0].vpos`.

## 3. 진짜 렌더 경로

`samples/exam_science.hwp` 셀 내부 inline shape 의 실제 렌더는 `table_layout.rs:1597-1728` 의 두 번째 패스에서 수행된다 (paragraph_layout 의 등록은 사용되지 않음).

해당 경로 라인 1631-1633:
```rust
if let Some(seg) = para.line_segs.get(target_line) {
    tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
}
```
이어서 1718-1724 에서 `tac_img_y` 를 그대로 `layout_cell_shape` 에 전달하여 사각형 rect 를 그린다.

## 4. 좌표계 분석

HWP 파일의 `LineSeg.vertical_pos` 는 **셀 origin 기준** 절대 vpos 이다 (paragraph 시작 기준이 아님). 본 케이스 cell:

| 셀 paragraph | line_seg | vpos (HU) | 의미 |
|------|------|----|------|
| p[0] "[가설]" | ls[0] | 0 | 셀 origin (185.28 px) |
| p[1] "...모두" | ls[0] | 1610 (21.47 px) | p[0] 다음 줄 |
| p[1] | ls[1] | 3220 (42.93 px) | "...이다." + ㉠ 줄 |

`para_y_before_compose` 는 paragraph 시작 시 누적된 y (= 206.75 px = cell_origin + ls[0].vpos).

`para_y_before_compose + ls[1].vpos` = 206.75 + 42.93 = **249.68** ← `ls[0].vpos` 가 이미 `para_y_before_compose` 에 누적되어 있는데 다시 더해지는 **이중 합산** 결과.

올바른 값: `cell_origin + ls[1].vpos` = 185.28 + 42.93 = **228.21** ← paragraph_layout 가 등록한 값.

## 5. 시각적 결과

| y 범위 (px) | 내용 |
|-------------|------|
| ~228.21 | ls[1] 라인 "이다." 텍스트 (paragraph_layout 가 생성, 정상 위치) |
| 228–251 | ㉠ 사각형이 들어가야 할 자리 (실제로는 그려지지 않음, paragraph_layout 만 inline_pos 등록 후 실제 rect 미생성) |
| ~257 | p[2] "[탐구 과정 및 결과]" 라인 시작 |
| 249.68–272.56 | ㉠ 사각형이 **실제로 그려진 위치** (table_layout 가 잘못된 y 로 발행) |
| 269.49 | p[2] 텍스트 baseline ("탐", "구"...) |

→ ㉠ 사각형이 [탐구 과정 및 결과] 텍스트를 침범. 사용자가 "[탐구 과정 및 결과] 위치가 올라가 있다" 고 본 것은 실제로는 ㉠ 사각형이 한 줄 내려와 [탐구...] 라인을 가린 결과.

## 6. 수정 방향 (Stage 2 제안)

`table_layout.rs:1547-1549` (Picture 인라인) 와 `1631-1633` (Shape 인라인) 두 위치의 공식:

```rust
// Before
tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);

// After
let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
tac_img_y = para_y_before_compose
    + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
```

근거: `ls[0].vpos` 는 이미 `para_y_before_compose` 누적치에 반영되어 있으므로, `target_line` 의 상대 오프셋만 더해야 한다.

대안 A — paragraph_layout 가 등록한 inline_pos 를 그대로 재사용 (`tree.get_inline_shape_position` 호출). Stage 2 에서 두 안 중 영향 범위가 작은 쪽을 선택.

회귀 우려: `Task #500` 가 이 분기를 수정한 직전 상태(즉 `seg.vertical_pos` 그대로 사용) 에서 별다른 회귀 보고가 없었다는 점은, ls[0].vpos = 0 인 케이스(셀 첫 paragraph)에선 두 공식이 동일함을 시사. 따라서 본 수정은 **셀의 두 번째+ paragraph** 에서만 결과가 바뀌며 그 외 케이스에는 영향 없음.

## 7. 결론

원인: `table_layout.rs` 의 인라인 TAC 도형/그림 렌더 경로가 `ls[target].vpos` 를 `para_y_before_compose` 에 더하면서, vpos 가 셀 origin 기준이라는 점을 고려하지 않아 ls[0].vpos 를 이중 합산함. paragraph_layout 의 등록 좌표(올바름)는 사용되지 않음.

Stage 2 에서 해당 두 라인 공식 수정으로 해결 가능.
