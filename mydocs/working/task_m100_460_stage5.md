# Task #460 Stage 5 완료 보고서: HWP3 표 셀 LINE_SEG line_spacing 수정

## 개요

HWP3 표(bibliography 17×2)에서 텍스트 행이 심하게 겹치는 현상을 수정하였다.

## 버그 원인

`src/parser/hwp3/mod.rs`의 `parse_paragraph_list()`에서 LineSeg를 생성할 때,
percent 줄간격임에도 `line_spacing = line_height - text_height`로 계산하였다.

- HWP5 IR 모델: percent 줄간격은 `line_height`에 이미 비율이 반영되어 있고
  `line_spacing = 0` 이어야 한다.
- 렌더러(`paragraph_layout.rs:2658`)는 매 줄마다 `y += line_height + line_spacing`으로
  이동하므로, `line_spacing`이 0이 아니면 줄 간격이 두 배로 계산된다.

**예시 (160% 줄간격, font size 1000 HU):**

| 항목 | 수정 전 (버그) | 수정 후 (정상) |
|------|-------------|-------------|
| line_height | 1600 HU | 1600 HU |
| line_spacing | 600 HU (= 1600−1000) | **0** |
| 렌더러 줄 이동 | 2200 HU = 29.33px | **1600 HU = 21.33px** |
| 3줄 셀 콘텐츠 높이 | 6600 HU = 88px | **4800 HU = 64px** |

## 수정 내용

**파일**: `src/parser/hwp3/mod.rs`

```rust
// 수정 전 (버그):
let fallback_line_spacing = fallback_line_height - fallback_text_height;
// ...
ls = lh - th;

// 수정 후 (정상):
// HWP5 IR 모델: percent 줄간격은 line_height에 이미 반영 → line_spacing=0
// fixed 줄간격은 line_height=fixed, line_spacing=fixed-th (추가 간격)
let fallback_line_spacing = if fixed_line_spacing.is_some() {
    fallback_line_height - fallback_text_height
} else {
    0
};
// ...
ls = if fixed_line_spacing.is_some() { lh - th } else { 0 };
```

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test` (전체) | 1068 lib + 6 svg_snapshot + 1 tab_cross_run = 전부 통과 |
| `cargo build --release` | 성공 |
| hwp3-sample.hwp SVG 내보내기 | 16페이지 정상 완료 (수정 전 22페이지) |
| bibliography 표 (pi=194) LINE_SEG | `lh=1600 ls=0` 전체 셀 확인 |
| SVG 줄 Y 좌표 간격 | 21.33px (= 1600 HU) 일정, 겹침 없음 |
| 표 페이지 분포 | p14: rows 0..3, p15: rows 2..17 — 정상 분할 |

### 페이지 수 변화

수정 전 22페이지 → 수정 후 16페이지. 줄간격 오류로 인한 과도한 콘텐츠 높이가
해소되어 자연스럽게 감소.

### 잔존 LAYOUT_OVERFLOW_DRAW 경고

`pi=18446744073709551615 overflow=36.0px` — 꼬리말 내부 LINE_SEG 높이 문제.
이번 Stage 범위 외.

## 수정 파일

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | percent 줄간격 line_spacing = 0 수정 (2곳, fallback + per-line) |
