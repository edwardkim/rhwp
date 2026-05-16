# Task #928 Stage 5: 잔존 회귀 ROOT CAUSE + Fix 구현

## 1. 잔존 회귀 ROOT CAUSE

### 회귀 1: 사각형 내부 폰트 축소 (15.33 → 6.88px)

**측정**: 임시 trace 로 사각형 IR 확인.

```
[928] shape sec=0 para=0 ci=0 orig=(2925, 975) cur=(6518, 1983)
       sw_ratio=2.228 sh_ratio=2.034 max_ratio=2.228
       base=(292.88, 408.70, 86.92, 26.45)
```

`shape_layout.rs:1206-1228` 의 Task #874 #3 ratio 축소 로직 (`max_ratio > 1.5 → font *= 1/2.228 = 0.449`) 이 인라인 사각형에 잘못 발동. 한컴은 본 케이스 (인라인 + 셀 내부) 에서 폰트 그대로 유지.

원래 의도: shortcut.hwp 마스터 페이지 글상자 (254pt → 작은 크기 강제 fit) 케이스. tac=false 인 large 글상자에만 적용되어야 함.

### 회귀 2: ㉢ 그림 중복 emit

**측정**: 임시 trace 로 Picture 분기 will_render_inline 확인.

```
[928pic] sec=0 cp_idx=2 ci=2 will=false
         lines=[(0, 14)] tac_controls=[(2,..,0), (8,..,1), (14,..,2)]
```

ctrl[2] (㉢) 의 `abs_pos=14` 와 line `char_start+line_chars=14` 가 같아 가드 식 `abs_pos < line_chars` boundary 미통과 → manual emit. paragraph_layout 의 run_tacs filter (`is_last_run && *pos == run_char_end` 포함) 는 emit → 중복.

## 2. Fix 구현

### Fix 1: `shape_layout.rs::layout_textbox_content`

`parent_treat_as_char: bool` 인자 추가 (4 호출자 모두 `shape.common().treat_as_char` 전달). ratio 축소 조건에 가드 추가:

```rust
// [Task #928] 인라인 도형 (treat_as_char=true) 은 폰트 자동 축소 적용 안 함.
if max_ratio > 1.5 && !parent_treat_as_char {
    // ratio 축소
}
```

shortcut.hwp 마스터 페이지 글상자는 tac=false 이므로 기존 동작 유지.

### Fix 2: `table_layout.rs` Picture/Shape 가드 — `inline_shape_position` 기반

기존 `tac_controls + line_chars` 기반 가드는 boundary 케이스 (abs_pos == line_chars) 에서 실패. 시도 1 (boundary 포함하도록 `<=` 조건) 은 hwp-img-001.hwp 4 그림 → 2 그림 회귀 (paragraph_layout 이 emit 안 한 케이스도 가드 true 로 만들어 manual emit 누락).

**정합 시그널**: paragraph_layout 이 inline emit 한 경우 항상 `set_inline_shape_position` 호출 (`paragraph_layout.rs:2019-2034` 등). 따라서 가드를 `tree.get_inline_shape_position(...).is_some()` 으로 변경하면 paragraph_layout 의 실제 동작과 정합.

```rust
let will_render_inline = tree
    .get_inline_shape_position(section_index, cp_idx, ctrl_idx, cell_context.as_ref())
    .is_some();
```

Picture 분기와 Shape 분기 양쪽 동일 패턴. Equation/Table 분기는 이미 같은 방식 사용 중 (라인 1986, 2079).

## 3. 검증 결과

### 시각 검증 (exam_kor.hwp 5쪽)

| 항목 | Before (Stage 1) | v1 fix (Stage 3) | v2 fix (Stage 5) |
|------|-----------------|------------------|-----------------|
| 텍스트 중복 | ✗ 2회 emit | ✅ 1회 emit | ✅ 1회 emit |
| 사각형 폰트 | ✗ 6.88px | ✗ 6.88px | ✅ **15.33px** |
| 사각형 outline | (확인 안 됨) | 작게 보임 | ✅ **x=292.88 w=86.92** |
| ㉠㉡㉢ 그림 | 4 emit (㉢ 중복) | 4 emit (㉢ 중복) | ✅ **3 emit** |

### 자동 회귀

```
cargo test --release
test result: ok. 1275 passed; 0 failed; 2 ignored
```

- svg_snapshot 8/8 통과
- `test_task76_img_001_four_pictures` 통과 (4 그림 정확)
- 모든 통합 테스트 통과

## 4. 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/layout/shape_layout.rs` | `layout_textbox_content` 시그니처에 `parent_treat_as_char` 추가, ratio 가드. 4 호출자 (Rectangle/Ellipse/Polyline/Curve) 에서 `shape.common().treat_as_char` 전달 |
| `src/renderer/layout/table_layout.rs` | Picture/Shape 가드를 `tree.get_inline_shape_position().is_some()` 으로 단순화 |

v1 의 Shape 분기 가드 추가 + 도형 좌표 inline_pos 사용 (Stage 3 commit 738d63fd) 은 그대로 유지. v2 가 가드 시그널만 정합 변경.

## 5. Stage 6 진입 조건

- ✅ exam_kor.hwp 5쪽 다이어그램 행 단일 baseline + 사각형 정상 크기/폰트 + 그림 3개 emit
- ✅ cargo test 전체 통과 (snapshot + hwp-img-001 회귀 차단)
- ⏳ Stage 6: 한컴 PDF 시각 정합 비교 + 최종 보고서 + 오늘할일 갱신
