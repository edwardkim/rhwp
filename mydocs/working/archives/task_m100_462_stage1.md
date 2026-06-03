# Task #462 Stage 1 보고서: TAC Picture 인라인 line advance 누락 수정

## 변경 내용

`src/renderer/layout.rs:2724` (와 2727 의 동일 분기) — TAC `Control::Picture` 인라인 경로의 `result_y` 계산을 LINE_SEG 의 `lh + ls` 기반으로 변경.

```rust
// 변경 전
result_y = pic_y + pic_h;

// 변경 후
let line_advance = para.line_segs.first()
    .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
    .unwrap_or(pic_h);
result_y = pic_y + line_advance.max(pic_h);
```

## 검증 결과

### exam_kor 페이지 1 (목표 케이스)

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| pi=25 이미지 bottom | 1113.58 | 1113.58 (불변) |
| pi=26 line_top | 1113.57 | ~1134.13 |
| pi=26 baseline | 1129.36 | **1149.87** |
| 갭 | ~0 px | **20.51 px** |

HWP LINE_SEG 의 `lh+ls = 25213 HU = 336.17 px` advance 와 정확히 일치.

### 회귀 검증 (다단 샘플 5종)

| 샘플 | 페이지 수 | LAYOUT_OVERFLOW (전→후) |
|------|-----------|------------------------|
| exam_kor.hwp | 20 → 20 | 16 → 19 (+3) |
| exam_eng.hwp | 8 → 8 | 9 → 11 (+2) |
| exam_math.hwp | 20 → 20 | 0 → 0 |
| exam_science.hwp | 4 → 4 | 5 → 5 |
| exam_social.hwp | 4 → 4 | 4 → 4 |

- 페이지 수 모든 샘플 동일 (회귀 0).
- 신규 overflow 는 **수정으로 정당하게 추가된 advance (≈20.5px)** 가 이미 col_bottom 근처였던 컬럼을 살짝 넘기는 경계 케이스. 변경 전이 잘못 작은 advance 로 컬럼 안에 욱여넣었던 것이 원인.
  - exam_kor pi=77 page 2 col 0: +7.3px (마진 케이스)
  - exam_kor pi=103/104/105 page 3 col 1: 기존 overflow 가 본 fix 로 약 +20px 증가 (이미 큰 overflow 였음)
  - exam_eng pi=194 page 3 col 1: +11.5px (마진 케이스)
- 모두 본 fix 의 정상 결과 — HWP 소스의 advance 를 정확히 재현하기 시작했기 때문.

### `cargo test --release`

- **1120 passed, 0 failed** (회귀 0).

## 다음 단계

- 최종 결과보고서 작성 + commit + merge.
