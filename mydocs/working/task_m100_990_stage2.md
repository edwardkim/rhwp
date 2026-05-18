# Task #990 Stage 2 완료보고서 — 정정 (GREEN)

**이슈**: [#990](https://github.com/edwardkim/rhwp/issues/990)
**브랜치**: `local/task990`

---

## 1. 정정 내용

`src/renderer/layout.rs` — `layout_shape_item()` 의 `!has_real_text`
(treat-as-char 글상자가 빈 문단 위에 놓인) 분기:

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| `shape_y` | `y_offset` (선행 FullParagraph 진행 후 값) | `para_start_y[para_index]` (호스트 문단 시작) |
| `result_y` | 무조건 `shape_y + line_advance.max(shape_h)` 재진행 | `y_offset <= para_start` (FullParagraph 미선행 = 글상자 단독) 인 경우에만 진행. 선행 FullParagraph 가 이미 LINE_SEG advance 를 마친 경우 재진행 없음 |

`FullParagraph` PageItem 의 `layout_paragraph()` 가 빈 호스트 문단의 LINE_SEG
(`lh + ls`)만큼 이미 advance 했으므로, `Shape` PageItem 은 위치만 등록하고
재진행하지 않는다. Task #974(`c3e32151`)가 추가한 `set_inline_shape_position`
등록 자체는 보존 — 이중 가산만 제거.

## 2. RED → GREEN

```
변경 전: issue #990 box y=[344.08, 476.96, 609.84]  advance=[132.88, 132.88]  FAILED
변경 후: issue #990 box y=[344.08, 410.52, 476.96]  advance=[ 66.44,  66.44]  ok
```

박스 advance 132.88px → **66.44px** (LINE_SEG 1회분 `4983 HU`). 박스↔본문 줄
비율 = 66.44 / 34.67 = **1.92** — 한컴 PDF 비율(1.92)과 정확히 일치.

`export-svg` 의 `LAYOUT_OVERFLOW page=3 overflow=215.9px` 경고도 해소.

## 3. Task #974 의도 보존 검증

textbox 관련 테스트 4건 전부 통과:

```
test test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx ... ok   (Task #974 회귀 테스트)
test test_task78_rectangle_textbox_inline_images ................. ok
test test_textbox_render_tree_debug .............................. ok
test test_624_textbox_inline_shape_y_on_line2_p2_q7 .............. ok
```

## 4. 시각 검증

`samples/issue-990-tac-box.hwpx` 4쪽 SVG — 글상자 3개가 한컴 PDF 와 동일하게
밀착 배치(`※ 본 사업은 …` ×3) 후 `2. 추진배경 및 필요성` 이어짐.

## 5. 다음 단계 (Stage 3)

전체 `cargo test` 0 회귀, 광범위 `export-svg` 차분, `cargo clippy`/`fmt`.
