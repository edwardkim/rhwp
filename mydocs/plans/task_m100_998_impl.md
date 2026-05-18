# Task #998 구현 계획서 — CHARS_PER_LINE 휴리스틱 조정

- 이슈: [#998](https://github.com/edwardkim/rhwp/issues/998)
- 선행: [Stage 1 진단](../working/task_m100_998_stage1.md)
- 브랜치: `local/task998` (base: `local/task994`)

## 1. 변경 위치

[src/renderer/composer.rs](src/renderer/composer.rs) 의 `compose_lines` fallback (`line_segs.is_empty()` branch) 의 `CHARS_PER_LINE` 상수.

## 2. 변경 내용

```rust
// Before (PR #997)
const CHARS_PER_LINE: usize = 35;

// After (Task #998)
const CHARS_PER_LINE: usize = 45;  // HWP3 reference 평균 43~46 chars/line
```

## 3. 예상 효과

| | Before | After |
|---|---|---|
| HWP5 sample16 페이지 수 | 67 | 65 |
| HWP3 reference 대비 | +3 | +1 (수용) |
| 시각 정상성 | OK | OK |
| 240 sample 회귀 | (PR #997) | 영향 없음 예상 |

## 4. 회귀 검증 계획

| 검증 | 통과 기준 |
|------|----------|
| cargo test --release --lib | 0 fail |
| 240 sample 페이지 수 | 변동 0 (HWP5 sample16 외) |
| 시각 | 작업지시자 판정 통과 |

## 5. Stage 진행

| Stage | 내용 | 산출물 |
|-------|------|--------|
| 2 | 구현 (이미 적용 — CHARS_PER_LINE=46 → 45 정리) | composer.rs 1 line |
| 3 | cargo test + 240 sample 회귀 + 시각 검증 | working/stage3.md |
| 4 | 최종 보고서 + PR | report + PR |
