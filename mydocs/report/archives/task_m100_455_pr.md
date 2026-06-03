# PR — Task #455: 인라인 글상자가 있는 줄의 외부 본문 텍스트 누락 수정

## Title

```
Task #455: 인라인 글상자(tac=true + TextBox) 가 있는 줄의 외부 본문 텍스트 누락 수정
```

## Body

### Summary

- `samples/exam_kor.hwp` 페이지 2 좌측 단의 문단 pi=33 line 2 에서 본문 39자가 누락되고 인라인 글상자 내부 "개화" 두 글자만 표시되던 버그 수정
- `paragraph_layout.rs` 의 `skip_text_for_inline_shape` 분기 제거 — 글상자 내부 텍스트와 외부 문단 본문을 혼동해 외부 본문까지 통째로 스킵하던 로직
- 글상자 자체와 내부 텍스트는 `shape_layout` 의 `inline_shape_position` 경로로 별도 렌더되므로 외부 본문을 항상 렌더해도 중복 없음

### 증상 (수정 전 → 후)

수정 전 (좌측 단):
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 본문 39자 누락
y=347.6 "수없는 대세로 자리잡았다..."
```

수정 후:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 글상자 (별도 패스)
y=322.6 "척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를"   ← 본문 39자 복원 (글상자 좌·우 분리)
y=347.6 "수없는 대세로 자리잡았다..."
```

### Root Cause

`paragraph_layout.rs::layout_composed_paragraph` 의 tac 분기 처리 블록에서:

```rust
// (제거 전) 잘못된 의도: "글상자 텍스트는 별도 패스에서 렌더되므로 스킵"
let skip_text_for_inline_shape = has_tac_shape && para.map(|p| {
    tac_offsets_px.iter().any(|(_, _, ci)| {
        if let Some(Control::Shape(s)) = p.controls.get(*ci) {
            s.drawing().map(|d| d.text_box.is_some()).unwrap_or(false)
        } else { false }
    })
}).unwrap_or(false);

if !skip_text_for_inline_shape {
    line_node.children.push(sub_run_node);  // tac 앞 텍스트
}
// ...
if !skip_text_for_inline_shape {
    line_node.children.push(sub_run_node);  // 마지막 tac 이후 텍스트
}
```

여기서 스킵된 "텍스트" 는 글상자의 *외부 문단 본문* 이지 *내부* ("개화") 가 아니다. 외부 본문은 글상자 좌·우를 흐르는 일반 텍스트이며 항상 렌더되어야 한다.

### Fix

- `skip_text_for_inline_shape` 변수 + 두 곳의 가드 제거
- 두 텍스트 푸시 지점의 `if !skip_text_for_inline_shape { ... }` 를 단순 블록 `{ ... }` 로 변경 (변수 스코프 보존)
- Task #455 라벨 주석 추가

### Test Plan

- [x] `cargo build --release` — 빌드 성공
- [x] `cargo test --release` — 1117 passed, 1 ignored, 0 failed
- [x] `rhwp export-svg samples/exam_kor.hwp` — 20 페이지, 좌측 단 pi=33 line 2 의 39자 정상 렌더 확인
- [x] 회귀 검증 (페이지 수 동일):
  - `exam_kor.hwp` 20
  - `exam_eng.hwp` 8
  - `2010-01-06.hwp` 6
  - `exam_math_8.hwp` 1
  - `biz_plan.hwp` 6
  - `draw-group.hwp` 1
  - `atop-equation-01.hwp` 1
  - `equation-lim.hwp` 1

### Known Minor Issue

본문 글자 baseline y=322.6, 글상자 내부 "개화" baseline y=321.4 — 1.2px 차이. 줄 높이가 5mm 글상자에 맞춰 1417 HU 로 늘어난 상황에서 baseline 정렬을 본문 폰트 기준으로 재계산하지 않아 생기는 미세 어긋남. 시각상 큰 문제는 아니나 별도 이슈로 분리 가능.

### Files Changed

- `src/renderer/layout/paragraph_layout.rs`
- `mydocs/plans/task_m100_455.md`
- `mydocs/plans/task_m100_455_impl.md`
- `mydocs/working/task_m100_455_stage{1,2,3}.md`
- `mydocs/report/task_m100_455_report.md`
- `mydocs/report/task_m100_455_pr.md`
- `mydocs/orders/20260429.md`

Closes #455.
