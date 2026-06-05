# Task #1300: 수식 위첨자 과대 상승 — 구현계획서

- 이슈: [#1300](https://github.com/edwardkim/rhwp/issues/1300) / 브랜치 `local/task1300`
- 수행계획서: `task_m100_1300.md`
- 작성일: 2026-06-05

## 근본 원인 (프로브 측정 완료)

`layout.rs:460-468` `layout_superscript`:
```rust
let sup_shift = b.baseline - s.height * 0.7;   // (A)
if sup_shift >= 0.0 {
    sup_y = 0.0;
    base_y = sup_shift.max(s.height - b.height).max(0.0);   // (B)
    ...
}
```
`base_y`(= base를 아래로 미는 양)가 `b.baseline`에 비례 → 합성 baseline = `base_y + b.baseline ≈ 2·b.baseline − 0.7·s.height`. 키 큰 base에서 위첨자가 baseline 위로 과하게 치솟음.

프로브(fs=12):
| script | b.baseline | base_y | sup top above bl |
|--------|-----------|--------|------------------|
| `x^4` | 9.6 | 3.7 | 13.3 (정상) |
| `(1/6)^4` (LEFT/RIGHT) | 17.6 | 11.8 | **29.4 (과대)** |

`base_y`가 sup 높이(8.4)를 초과(11.8)하는 것이 비정상 — 위첨자를 base 위에 얹는 데 필요한 여유는 **sup 높이를 넘지 않아야** 한다.

## 수정 방안

`base_y`를 **sup 높이로 상한** 한다. 짧은 base(`sup_shift ≤ s.height`)는 불변, 키 큰 base만 보정.

```rust
let base_y = sup_shift.min(s.height).max(s.height - b.height).max(0.0);
//                     ^^^^^^^^^^^^^ 추가: base 밀어내기는 sup 높이까지만
```

- `x^4`: sup_shift=3.7 ≤ s.height=8.4 → `.min` 무영향 → **불변**.
- `(1/6)^4`: base_y 11.8 → 8.4. 합성 baseline 29.4 → 26.0. 위첨자가 괄호 우상단 근처로 내려옴.
- 단조 보정(값을 낮추기만) → 윗줄 침범 해소 방향.

## 구현 단계

### 1단계: 수정 + 단위 검증
- `layout.rs` `layout_superscript` base_y에 `.min(s.height)` 추가.
- 프로브/단위 테스트로 `x^4` 불변, `(1/6)^4` base_y≤s.height 확인.
- 기존 `test_superscript_fraction_baseline`(#532, 짧은 base+분수 sup) 통과 유지.
- 산출물: `working/task_m100_1300_stage1.md`

### 2단계: 시각 검증 (SVG + 캔버스)
- `cargo build --release` → `export-svg -p 16` → 17쪽 `(1/6)⁴` 지수가 윗줄 미침범 + PDF 정합 확인.
- WASM 재빌드 → studio 페이지 17 캔버스 캡처 동일 확인.
- 다른 수식 문서 표본(eq 포함)으로 `x²`/`a^n` 무회귀 시각 확인.
- 산출물: `working/task_m100_1300_stage2.md` (before/after 크롭)

### 3단계: 회귀 + 최종
- `cargo test` 전체 통과.
- 프로브 테스트를 정식 회귀 테스트로 승격(`(1/6)^4` base_y ≤ s.height 핀) 또는 제거 판단.
- 산출물: `working/task_m100_1300_stage3.md` + `report/task_m100_1300_report.md`

## 회귀 가드

- 단조성: `base_y`를 낮추기만 하므로 짧은 base는 `.min` 미발동으로 불변.
- 핀: `x^2`(test_superscript_layout), `25^{1/3}`(#532), 새 `(1/6)^4` 핀.
- 시각: `x²`, `a^{n+1}` 위치 불변 + `(1/6)⁴` PDF 정합.
