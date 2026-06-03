# Task #957 — 최종 보고서

- 이슈: [#957](https://github.com/edwardkim/rhwp/issues/957)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task957`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위

원 issue #952 의 Issue 2 (sample16 page 18 본문 다음 페이지 밀림) 해결. typeset multi-TAC-shape cursor over-advance root cause 식별 + fix.

## 2. Root cause

### 2.1 RHWP_DEBUG_TAC_CURSOR 추적 결과

```
Before fix:
  Shape pi=394 ci=1 y_in=767.3 y_out=1197.9 dy=430.6 ⚠️
  FullPara pi=395 y_in=1197.9 ...

After fix:
  Shape pi=394 ci=1 y_in=767.3 y_out=767.3 dy=0.0 ✓
  FullPara pi=395 y_in=759.8 y_out=795.0 dy=35.3
```

→ Shape pi=394 ci=1 (picture diagram) 의 layout_shape_item 이 phantom +430.6px advance.

### 2.2 위치 식별

`src/renderer/layout.rs:3470-3475` (Bottom caption 의 result_y advance):

```rust
if matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

### 2.3 본질

pi=394 ci=1 picture (diagram) 의 caption:
- `dump`: `caption: dir=Bottom width=0 paras=1 text=""` (빈 caption)
- 한컴 viewer 에서는 invisible

rhwp 의 phantom advance 계산:
- pic_y = 767.3 (has_prior_tac 로 갱신된 잘못된 para_start_y)
- image_bottom = 767.3 + 411.89 (pic_h) = 1179.19
- cap_y = 1179.19 + 0 = 1179.19
- caption_h = 18.7 (empty paragraph default line height)
- cap_bottom = 1179.19 + 18.7 = 1197.89

→ result_y 가 767.3 에서 1197.89 로 +430.59px 누적. 다음 paragraph (pi=395) 가 1197.89 부터 시작 → body 외 영역 emit.

## 3. Fix

**Fix A** (`src/renderer/layout.rs:3465-3486`):

```rust
// [Task #957] 빈 caption (text 없음 + controls 없음) 은 SVG 에 invisible.
let caption_is_empty = caption.paragraphs.iter().all(|p|
    p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}')
        && p.controls.is_empty()
);
if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
    let cap_bottom = cap_y + caption_h;
    if cap_bottom > result_y {
        result_y = cap_bottom;
    }
}
```

+ `RHWP_DEBUG_TAC_CURSOR` 환경변수 영구화 (paragraph item 별 y_offset 추적 도구).

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**
- golden SVG diff: 0 regression

### 4.2 단위 검증 (sample16 page 18)
- pi=394 ci=1 dy: 430.6 → 0.0 ✓
- pi=395 y 위치: 1220 → 782 ✓
- 본문 (○ 통합모델..., ◦ 업무특성..., ◦ 공사 주요업무..., - 하드웨어..., - ORACLE RDBMS..., ○ UNIX...): 같은 페이지 표시 ✓
- 한컴 viewer page 16 정합 ✓

### 4.3 회귀 검증 (caption text 보유 sample)
- hwp3-sample14 (Task #864 영역):
  - non-empty caption "Cut&Paste 할 영역" 정상 위치 ✓
  - 다수 empty caption 도 phantom advance 제거 (개선)
- hwp3-sample10/11/13: 정상
- exam_kor/math: 정상

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| Empty caption picture | result_y advance 제거 (회귀 fix) |
| Non-empty caption picture | 영향 없음 (기존 동작 유지) |
| Caption None picture | 영향 없음 (본 분기 미진입) |
| Caption Top direction | 영향 없음 (별도 offset 처리) |

## 6. 관련 작업

- 원 issue #952 + PR #956 — Issue 1 (페이지 외곽선) 해결
- archive/task936 — 본 영역의 이전 fix 시도 history (미완)
- PR #918 — 본 영역의 시도 history (close)

## 7. 후속

- 원 #952 의 Issue 3 (시험지 page 1 문9 vertical) 별도 task 필요
