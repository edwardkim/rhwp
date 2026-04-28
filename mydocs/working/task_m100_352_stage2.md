# Task #352 Stage 2 완료보고서: dash advance 자연 폭 보정

> 2026-04-28 | Branch: `local/task352`

---

## 변경 사항

### `src/renderer/layout/text_measurement.rs`

1. 신규 헬퍼 `is_dash_leader_run(chars: &[char], i: usize) -> bool` 추가 (line ~947).
   - chars[i] 가 ASCII 하이픈(`-`) 이고 좌·우 합산 ≥ 3 개 연속일 때 true.
   - 단발 dash(`stimulus-driven`, `32.-`) 는 false → 영향 없음.

2. 세 개의 `char_width` 클로저 (`estimate_text_width`, `compute_char_positions`, `estimate_text_width_unrounded`) 에 동일 패턴 적용:
   ```rust
   let base_w_raw = /* 기존 분기 (메트릭/cjk/narrow/half) */;
   let base_w = if is_dash_leader_run(&chars, i) {
       base_w_raw.min(font_size * 0.3)
   } else {
       base_w_raw
   };
   ```
   - leader 시퀀스만 좁은 폭으로 강제. 메트릭이 이미 좁으면 그대로 유지.

코드 라인: line 197~211, 304~318, 884~898.

---

## 검증

### Q32 블랭크 라인 (s0 p221 L6, 5 페이지)

| 항목 | Stage 1 측정 | Stage 2 측정 | PDF 기대 |
|------|-------------|-------------|----------|
| dash advance | 12.11 px | **4.36 px** | ~3.5 px |
| 29 dash 시퀀스 폭 | 351 px | **126 px** | ~135 px |
| "of being" 시작 x | ~953 (우측 끝) | **~770 (~컬럼 중앙)** | 컬럼 중앙쯤 |
| `,` 우측 잘림 | 발생 | **해소** | 정상 |

### SVG 측정 (post Stage 2)

```
translate(597.12, 1039.75) → translate(601.48, ...) → translate(605.84, ...)
delta = 4.36 px ✓
```

### cargo test --release

전 테스트 통과:
- svg_snapshot: 6/6
- tab_cross_run: 1/1
- 메인 1023 + 0 failed
- 회귀 없음

### 다른 dash 사용처 영향 점검

문서 전체 dash 통계 (Stage 1 측정):
- HY신명조 dash 1017 개 — 다수가 단발 dash 로 추정
- Times New Roman dash 295 개

`is_dash_leader_run` 은 ≥ 3 연속에만 발동하므로 단발 dash(예: "stimulus-driven", "32.-", "* taint--altruistic--") 는 영향 없음. 단, "--altruistic--" 같은 2 개 연속도 leader 가 아니라 영향 없음 (3 개 미만이므로).

---

## 남은 작업 (Stage 3, 4)

- Stage 3: 시각 측면에서 dash 글리프 사이 갭 (스케일 0.95 적용 후 시각 폭 1.4 px ≪ advance 4.36 px) 을 단일 라인으로 통합 — `svg.rs` / `web_canvas.rs` 의 가운데점 패턴 모방
- Stage 4: 최종 회귀 + report + orders 갱신
