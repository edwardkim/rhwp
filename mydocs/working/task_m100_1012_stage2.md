# Task #1012 Stage 2 — line_seg.vpos 반영 fix

이슈: [#1012](https://github.com/edwardkim/rhwp/issues/1012)
Stage 1: [task_m100_1012_stage1.md](task_m100_1012_stage1.md)

## 1. 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | spacing_before=0 + column-top + para_index==0 + vpos > 0 case 의 fallback 추가 |

### 1-1. fix 코드

```rust
// [Task #1012] paragraph 첫 line vpos > 0 인데 spacing_before=0 으로
// 위 블록 진입 안한 경우 (test-image.hwp pi=0: TopAndBottom Picture +
// 인라인 wrap 조합) — line_seg.vpos 를 직접 y 에 가산하여 텍스트가 wrap
// shape 아래로 위치하도록 함. wrap 메커니즘이 별도로 처리하지 못하는
// case 의 fallback. start_line==0 + column-top + para_index==0 으로 한정.
if start_line == 0 && spacing_before == 0.0 && is_column_top && para_index == 0 {
    let vpos0_px = para
        .and_then(|p| p.line_segs.first())
        .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
        .unwrap_or(0.0);
    if vpos0_px > 0.0 {
        y += vpos0_px;
    }
}
```

## 2. 단위 검증

### 2-1. test-image.hwp page 1

| 항목 | Before | After |
|------|--------|-------|
| 텍스트 y | 143.6 | **346** ✓ |
| 이미지 y 영역 | 86~334 | 86~334 (불변) |
| 시각 overlap | text 가 Pic[2] 내부 | text 가 모든 Pic 아래 ✓ |

### 2-2. 라벨 정렬

| 라벨 | x 영역 | 대응 그림 |
|------|--------|-----------|
| 자리차지 | 153~192 | Pic[2] (x=113~325) ✓ |
| 글앞으로 | 332~371 | Pic[5] InFrontOfText (x=243~455) ✓ |
| 어울림 | 490~516 | Pic[3] Square (x=353~565) ✓ |
| 글뒤로 | 609~635 | Pic[4] BehindText (x=471~683) ✓ |

각 라벨이 대응 그림 영역 아래에 배치됨 — Hancom 정합.

## 3. 회귀 검증

| Sample | 페이지 수 | 비고 |
|--------|----------|------|
| hwp3-sample16.hwp | 64 | 변동 없음 |
| exam_kor.hwp | 20 | 변동 없음 |
| aift.hwp | 74 | 변동 없음 |
| biz_plan.hwp | 6 | 변동 없음 |
| hwpspec.hwp | 175 | 변동 없음 |

전체 lib tests: **1306 passed**, 0 failed ✓
`cargo clippy --release -- -D warnings`: 0 warnings ✓
`cargo fmt --check`: clean ✓

## 4. z-order 결함 — 자동 해소

Stage 1 진단 시 발견한 z-order 보조 결함 (모든 image 가 모든 text 뒤 push) 은 결함 A (vpos 반영) 해결 후 시각적으로 무관해짐:
- 결함 A 후: text 가 image 영역 (y=86~334) 아래 (y=346) 에 배치 → 시각 overlap 없음
- BehindText (글뒤로) image 가 text 를 가릴 가능성도 사라짐 (서로 다른 y 영역)

따라서 별도 Stage 3 z-order fix 불필요.

## 5. Stage 3 → 보고서 + PR

z-order 단독 fix 제거하고 Stage 3 = 보고서 + PR 로 압축.
