# Task #495 단계 1 보고서 — 원인 정확 진단

**이슈**: #495
**브랜치**: `local/task495`
**단계 목표**: y=224.26 라인이 어느 코드 경로에서 발행되는지 식별 (코드 미수정 / 진단용 임시 로그만 사용)

---

## 1. 진단 방법

다음 진단 로그를 임시 추가해 발행 경로를 추적했다 (이후 모두 제거):

1. **`src/main.rs` `Control::Shape` dump 분기**: 셀 안 사각형의 `text_box.paragraphs` 내용 출력
2. **`src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph`**: paragraph p[1] 호출 시 진입 로그 + 백트레이스
3. **동일 함수의 line emit 지점 (line_idx 루프)**: paragraph 단위 라인 발행 시 text_y / effective_col_x / line_height 출력

진단 코드는 단계 1 종료 시 모두 제거했다.

## 2. 진단 결과

### 2.1 사각형 자체 텍스트 (text_box)

```
ctrl[1] 사각형: tac=true, wrap=TopAndBottom, w=4724 h=1716
  rect text_box paragraphs=1
  rect.p[0] text_len=3 ls_count=1 ctrls=0 text="㉠"
```

사각형의 자체 글자는 **"㉠" 한 글자**(원형 한자 ㉠)이며 SVG의 18글자 텍스트와 무관함이 확정.

### 2.2 paragraph p[1]의 layout_composed_paragraph 호출

```
section=0 para=1 y=206.747 start=0 end=2 cell_ctx=Some(33) text_len=36
text_preview="◦ 분자당 구성 원자 수가 인 분자의 분자 모양은 모두  \u{2007}이다."
backtrace:
  layout_composed_paragraph
  table_layout::layout_table_cells
  table_layout::layout_table
  layout_column_item
  build_single_column
  build_render_tree
```

- **단 한 번만 호출됨** — paragraph 자체는 두 번 layout 되지 않는다.

### 2.3 paragraph p[1]의 라인 발행 (text_y 200~232 범위, 단 0)

```
pi=1 line_idx=0 text_y=206.75 effective_col_x=97.07 eff_margin_left=0.00 line_height=15.33 baseline=12.27
pi=1 line_idx=1 text_y=228.21 effective_col_x=97.07 eff_margin_left=20.00 line_height=22.88 baseline=12.27
```

- line_idx=0 baseline = 206.75 + 12.27 ≈ **219.02** ✓ (SVG y=219.01)
- line_idx=1 baseline = 228.21 + 12.27 ≈ **240.48** ✓ (SVG y=240.48)
- **y=224.26은 layout_composed_paragraph가 발행한 라인이 아니다.**

## 3. 원인 코드 위치 식별

다른 TextRun 발행 경로를 전수 검색한 결과 해당 코드를 발견했다.

### 3.1 코드 위치

**`src/renderer/layout/table_layout.rs:1568~1647`** — 셀 paragraph의 인라인 Shape(treat_as_char) 처리 분기.

요약 흐름:
```rust
Control::Shape(shape) => {
    if shape.common().treat_as_char {
        let shape_w = ...;
        // (A) tac_controls 에서 이 Shape 의 text_pos(=tac_pos) 검색
        if let Some(&(tac_pos, _, _)) = composed.tac_controls.iter().find(|&&(_, _, ci)| ci == ctrl_idx) {
            // (B) Shape 앞의 텍스트(text_before) 추출:
            //     composed.lines.first().runs 에서 char index 가
            //     [prev_tac_text_pos, tac_pos) 인 글자만 모음
            let text_before: String = composed.lines.first()
                .map(|line| {
                    let mut chars_so_far = 0usize;
                    let mut result = String::new();
                    for run in &line.runs {
                        for ch in run.text.chars() {
                            if chars_so_far >= prev_tac_text_pos && chars_so_far < tac_pos {
                                result.push(ch);
                            }
                            chars_so_far += 1;
                        }
                    }
                    result
                })
                .unwrap_or_default();
            if !text_before.is_empty() {
                // (C) text_before 를 사각형 옆 인라인 위치에 별도 TextRun 으로 발행
                ... cell_node.children.push(text_node);
            }
        }
        ... self.layout_cell_shape(...);
    }
}
```

### 3.2 결함 메커니즘

paragraph p[1] 의 IR:
- ls[0] vpos=1610: "◦ 분자당 구성 원자 수가 인 분자의 분자 모양은 모두" (35 글자)
- ls[1] vpos=3220: 사각형(ctrl[1]) + " 이다."

이 코드의 가정은 **"text_before 는 사각형이 위치한 줄의 앞 텍스트"** 이지만 실제 동작은:
- (B) 가 `composed.lines.first()` 즉 항상 **첫 줄만** 본다.
- ctrl[1] 사각형의 tac_pos 가 paragraph 텍스트 인덱스 기준이라 ls[1] 시작점(=첫 줄 끝점) 이상이다.
- 따라서 text_before = "ls[0] 첫 줄 전체 텍스트 (◦ 분자당...모두)" 가 추출된다.
- 사각형 위치 옆에 paragraph 첫 줄 텍스트 18글자가 통째로 발행된다.

### 3.3 SVG 좌표 일치 검증

| 발행 경로 | 시작 x | y(baseline) | 글자 간격 |
|---|---|---|---|
| layout_composed_paragraph (정상) | 97.07 | 219.01 | ~17px (composer 기반) |
| table_layout.rs:1612~1635 (중복) | 104.07 | 224.26 | ~20px (estimate_text_width 기반) |

- **x 차이 7px** = ctrl[0] 수식 폭(525 HU = 7px). 중복 발행 코드가 사각형 앞 inline_x 에 수식 폭을 누적한 후 시작했기 때문.
- **y 차이 5.25px** = layout_composed_paragraph 의 line_idx=0 (lh=15.33) 의 baseline(219.01) ≠ 인접 사각형 height 22.88 기반 baseline(text_y = 185.28 + (22.88-18.4) = 189.76 + bl_text 약 11.7 = 201.46... 직접 일치 안 됨; para_y_before_compose 와 adjacent_shape_h 산식 검증 필요. 단계 2에서 정밀 재현).
- **글자 간격 차이** = `estimate_text_width` (휴리스틱) ≠ composer 의 char_overlap-aware 폭. 발행 경로별 폭 계산 함수가 다름.

→ **두 라인이 동일 텍스트지만 서로 다른 위치/간격으로 그려져 시각적 겹침을 만든다.**

## 4. 정상 동작 시 가정 vs 실제

### 코드 의도 (정상 동작 가정)

이 분기는 **사각형이 paragraph 첫 줄에 있고 그 앞에 텍스트가 있는 경우**, 인라인 텍스트를 사각형 옆에 그리는 보강 코드로 보인다 (예: `[수식 박스] 다음 글자` 패턴).

이 가정 하에서는:
- `composed.lines.first()` 가 사각형이 속한 줄
- 그 줄에서 사각형 앞 텍스트만 추출 → 사각형 옆 발행
- `layout_composed_paragraph` 가 사각형 자리를 비워두므로 중복 X

### 실제 결함 케이스

paragraph p[1] 처럼 **사각형이 두번째 이후 줄에 있는 경우**:
- `composed.lines.first()` = ls[0] (첫 줄, 사각형 없는 줄)
- text_before = ls[0] 전체 텍스트
- `layout_composed_paragraph` 가 ls[0] 를 정상 발행 + 이 코드가 ls[0] 텍스트를 또 발행 → **중복**

## 5. 관련 Picture 분기

같은 함수 내 라인 1442~1541 의 `Control::Picture` 분기에는 **target_line 산출 코드**가 있다(라인 1483~1530):

```rust
let target_line = if composed.tac_controls.iter().find(...).is_some() {
    composed.tac_controls.iter().find(...).map(|&(abs_pos, _, _)| {
        composed.lines.iter().enumerate().rev()
            .find(|(_, line)| abs_pos >= line.char_start)
            .map(|(li, _)| li).unwrap_or(0)
    }).unwrap_or(0)
} else { 0 };

if target_line > current_tac_line {
    // 줄이 바뀜: inline_x 리셋, y 이동
    ...
}
```

**Picture 는 줄이 바뀌면 inline_x/y 를 리셋하고 다음 줄로 이동**한다. 즉 다중 줄 paragraph 에서 정상 동작.

**Shape 분기는 이 줄 추적 로직이 누락**되어 항상 첫 줄 기준으로 text_before 를 추출한다 — 이것이 결함의 본질.

## 6. #496 관계

12번 본문 줄간격 압축(#496)은 paragraph pi=61 의 인라인 수식(Equation) 9개 처리와 관련된 별개 문제로 보인다. 단계 1 진단 결과 **본 결함은 Shape 분기에 한정**되며 Equation/Picture 분기와 다른 코드 경로다. **#496 은 본 task 범위 밖**으로 분리한다.

## 7. 단계 2 입력

- 결함 위치: `src/renderer/layout/table_layout.rs:1573~1635` (Shape 인라인 분기의 text_before 추출/발행)
- 수정 방향 후보(단계 2 에서 결정):
  - (A) Picture 분기와 동일하게 target_line 산출 후 해당 줄에서만 text_before 추출
  - (B) 그러나 `layout_composed_paragraph` 가 이미 모든 줄 텍스트를 발행하므로, text_before 발행 자체가 중복일 수 있음 → 이 분기 자체의 필요성 재검토
  - (C) 분기가 필요한 케이스(예: layout_composed_paragraph 가 사각형 자리를 비우지 못해 텍스트가 사각형과 겹치는 케이스)를 회귀 샘플로 식별 후 정확한 가드 추가

## 8. 검증 단계 산출물

- 진단 코드는 모두 제거. `git diff src/` 빈 상태 확인.
- `cargo build --release` 정상.
- 본 보고서: `mydocs/working/task_m100_495_stage1.md`

## 9. 승인 요청

원인 진단 결과에 대한 승인을 요청합니다. 승인되면 단계 2 (구현 계획서 `task_m100_495_impl.md` 작성) 에 착수합니다.
