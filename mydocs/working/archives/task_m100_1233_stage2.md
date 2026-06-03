# Stage 2 완료 보고서 — Task #1233: 적분(∫) trailing 간격

- **이슈**: #1233 (M100)
- **브랜치**: `feature/issue-1233-eq-bigop-spacing`
- **단계**: Stage 2 / 3
- **작성일**: 2026-06-02

## 변경 내용

`src/renderer/equation/layout.rs` — 적분 2경로에 동일 `BIG_OP_TRAIL_PAD` trailing 간격 추가:

1. `layout_integral`(첨자 있는 ∫, nolimits 스타일): `width: total_w` →
   `width: total_w + fs * BIG_OP_TRAIL_PAD`.
2. `layout_math_symbol`의 bare 적분(첨자 없는 ∫): `width: w` → `width: w + fs * BIG_OP_TRAIL_PAD`.

→ Σ/∏(Stage 1)과 함께 **모든 큰 연산자(∑·∏·∫·∮ 등)가 일관된 trailing 간격**을 가짐.

## 검증

- `cargo test --lib equation` → **139 passed, 0 failed**.
- 시각: 6쪽 문25 "∫₁^e(1+1/x)f(x)dx의 값은?" — 적분 첨자 뒤 피연산자 "(1+1/x)" 가
  붙지 않고 적정 간격 유지, 오버플로(다음 글자 침범) 없음 확인.

## 다음 단계

Stage 3: PDF 대조로 `BIG_OP_TRAIL_PAD` 값 확정 + 다수식 표본(∑/∏/∫) + `dump-pages`
레이아웃 불변 + 전체 회귀 + 최종 보고서.
