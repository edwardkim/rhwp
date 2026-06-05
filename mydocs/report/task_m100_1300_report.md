# Task #1300: 수식 위첨자 과대 상승 — 최종 결과보고서

- 이슈: [#1300](https://github.com/edwardkim/rhwp/issues/1300) / 브랜치 `local/task1300`
- 마일스톤: v1.0.0 (M100) / 일자: 2026-06-05
- 결과: **해결** — 위첨자 상단 정렬 보정.

## 문제

괄호 분수 등 키 큰 base의 위첨자(지수)가 baseline 위로 과하게 치솟아 **윗줄을 침범**. 17쪽 [다른 풀이] `(1/6)⁴`의 `4`가 윗줄 "의 2가지 경우이므로"로 떠올라 "이므₄로"처럼 보임.

## 원인

`src/renderer/equation/layout.rs` `layout_superscript`: `base_y`(base 밀어내기)가 `sup_shift = b.baseline − 0.7·s.height`로 **base baseline에 비례** → 키 큰 base에서 합성 baseline이 자연 baseline의 약 2배가 되어 위첨자가 한 줄 위로 치솟음.

## 해결

위첨자 상단을 base 상단에 맞춘다(상단 정렬). base를 아래로 밀지 않음:
```rust
// before: base_y = sup_shift.max(s.height - b.height).max(0.0);
// after  (#1300):
base_y = (s.height - b.height).max(0.0);   // base가 sup보다 낮을 때만 내림
```
- 키 큰 base: base_y=0 → 합성 baseline = base 자연 baseline, 위첨자 상단 = base 상단.
- 짧은 base: base_y=0 → 위첨자가 base 상단 우측에 정렬.

> 1차 시도(base_y를 sup 높이로 상한)는 여전히 위첨자가 높다는 피드백으로 상단 정렬로 재설계.

## 검증

- 시각: 17쪽 `(1/6)⁴` 지수가 괄호 우상단 모서리, 윗줄 미침범 — 한글 2022 PDF 정합. SVG·studio 캔버스(WASM 재빌드) 양쪽 확인.
- 짧은 base `6⁴`/`x⁴` 정상.
- `cargo test` **2037 passed, 0 failed**. 회귀 테스트 `test_superscript_tall_base_no_overshoot` 추가(상단 정렬 핀), 기존 #532 통과.

## 변경 파일

- `src/renderer/equation/layout.rs` — `layout_superscript` base_y 상단 정렬 + 회귀 테스트.
- 문서: `plans/task_m100_1300*.md`, `working/task_m100_1300_stage{1,2,3}.md`, 본 보고서.
