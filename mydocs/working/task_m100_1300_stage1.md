# Task #1300 Stage 1: 위첨자 상승량 보정 + 단위 검증

- 브랜치: `local/task1300`
- 일자: 2026-06-05

## 수정

`src/renderer/equation/layout.rs` `layout_superscript`:
```rust
// before
base_y = sup_shift.max(s.height - b.height).max(0.0);
// after (#1300)
base_y = sup_shift.min(s.height).max(s.height - b.height).max(0.0);
```
`base_y`(base 밀어내기)를 **sup 높이로 상한**. 위첨자를 base 위에 얹는 여유는 sup 높이를 초과할 필요가 없다.

## 프로브 측정 (fs=12, 수정 전→후)

| script | base_y | sup top above baseline |
|--------|--------|------------------------|
| `x^4` (짧은 base) | 3.7 → **3.7 (불변)** | 13.3 → **13.3 (불변)** |
| `LEFT ( {1} over {6} RIGHT )^4` (키 큰 base) | 11.8 → **8.4** | 29.4 → **26.0** |

- 짧은 base는 `sup_shift(3.7) ≤ s.height(8.4)` 이므로 `.min` 미발동 → **불변**(회귀 없음).
- 키 큰 base만 base_y가 sup 높이로 제한되어 위첨자가 괄호 우상단으로 내려옴.

## 단위 테스트

- `test_superscript_layout`(`x^2`) ✅
- `test_superscript_fraction_baseline`(#532, `25^{1/3}` 짧은 base+분수 sup) ✅
- 위첨자 관련 6 테스트 전부 통과.

## 다음

Stage 2: SVG/캔버스 시각 검증 (17쪽 `(1/6)⁴` 윗줄 미침범 + PDF 정합).
