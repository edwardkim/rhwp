# Task #1012 최종 결과 보고서 — wrap 옵션 paragraph 의 라벨 텍스트 y 위치 정정

이슈: [#1012](https://github.com/edwardkim/rhwp/issues/1012)
배경: PR #1011 (Task #1006) 진행 중 발견

## 1. 목표

`samples/test-image.hwp` page 1 의 paragraph 라벨 텍스트 (자리차지/글앞으로/어울림/글뒤로) 가 한컴 viewer 와 다르게 표시되는 문제 해결.

## 2. 결과

| 항목 | Before | After |
|------|--------|-------|
| 라벨 텍스트 y 위치 | 143.6px (Picture 영역 내부) | **346px** (Picture 영역 아래) ✓ |
| 시각 overlap | text 가 image 에 가려짐 | 완전 분리 ✓ |
| 라벨-그림 대응 정렬 | 어긋남 | 각 그림 아래 정합 ✓ |
| `cargo test` | 1306 passed | 1306 passed ✓ |
| `cargo clippy` | 0 warnings | 0 warnings ✓ |
| 회귀 sweep (5 sample) | — | 모두 페이지 수 보존 ✓ |

## 3. 핵심 fix

`src/renderer/layout/paragraph_layout.rs` 의 spacing_before 처리 분기 확장:

```rust
// [Task #1012] paragraph 첫 line vpos > 0 인데 spacing_before=0 으로
// 위 블록 진입 안한 경우 — line_seg.vpos 를 직접 y 에 가산하여 텍스트가
// wrap shape 아래로 위치하도록 함. wrap 메커니즘이 별도로 처리하지 못하는
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

## 4. Root cause 와 fix 범위

### 4-1. Root cause

`paragraph_layout.rs::layout_composed_paragraph` 의 spacing_before 처리 로직 (line 982-1000) 이 `spacing_before > 0` 인 경우만 vpos 클램프 분기 진입. test-image.hwp pi=0 의 spacing_before=0 → vpos=15180 HU 무시 → 텍스트가 body_top 에 그려짐.

### 4-2. fix 범위

- **column-top + para_index==0** 으로 한정 — 일반 paragraph 의 y 누적은 영향 없음
- **vpos > 0** 인 경우만 → 인코더가 명시한 push-down 만 적용
- wrap 메커니즘 (typeset 의 wrap_around_paras) 이 별도로 처리하지 못하는 case 의 fallback

## 5. z-order 결함 — 자동 해소

Stage 1 진단 시 발견한 z-order 부 결함 (모든 image 가 모든 text 뒤 push) 은 본 fix 적용 후 시각 무관:
- text 가 image 영역 아래 (y=346) 에 위치 → 시각 overlap 없음
- 별도 z-order fix 불필요

## 6. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | vpos 0 + spacing 0 column-top case 의 fallback 추가 |

## 7. 검증 trail

- Stage 1: Root cause 진단 ([`stage1`](../working/task_m100_1012_stage1.md))
- Stage 2: line_seg.vpos 반영 fix + 회귀 검증 ([`stage2`](../working/task_m100_1012_stage2.md))

## 8. 결론

작업지시자 시각 판정 정합 — 4 종 wrap 옵션 그림 아래에 라벨이 정확히 배치. 다른 fixture 회귀 없음. Hancom 정합.
