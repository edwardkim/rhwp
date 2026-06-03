# Stage 1 완료 보고서 — Task #1233: Σ/∏ 큰 연산자 trailing 간격

- **이슈**: #1233 (M100)
- **브랜치**: `feature/issue-1233-eq-bigop-spacing`
- **단계**: Stage 1 / 3
- **작성일**: 2026-06-02

## 변경 내용

`src/renderer/equation/layout.rs`:

1. 상수 추가: `const BIG_OP_TRAIL_PAD: f64 = 0.1;` (fs 비율, `layout_symbol` pad 관례 정합).
2. `layout_big_op`(∑/∏ limits): `width: max_w` → `width: max_w + fs * BIG_OP_TRAIL_PAD`.
   sup/sub 중앙정렬은 `max_w` 기준 유지 → 연산자는 좌측, 우측에 순수 trailing 공백.
3. 단위 테스트 `test_big_op_trailing_pad`: 트리에서 BigOp 박스를 찾아 width 가 내부
   sub/sup 우측 끝보다 `fs×PAD` 이상 큼을 단언.

## 설계 안전성 (재확인)

인라인 수식은 `svg.rs` 에서 컨트롤 advance(tac_w)로 가로 스케일되므로, BigOp width 증가는
오버플로 없이 흡수되고 Σ-피연산자 간격만 비례적으로 생김 → 다음 글자("의") 침범 없음.

## 검증

- `cargo test --lib equation` → **139 passed, 0 failed** (신규 테스트 포함).
- 1차 시각: `export-svg 3-09월_교육_통합_2023 -p5` 6쪽 문26 "∑bₙ의" 에 **간격 생성 확인**.
  - `output/poc/task1233/sigma_before_after_pdf.png` (변경 전 붙음 → 변경 후 간격, PDF 정합).

## 다음 단계

Stage 2: `layout_integral`(∫) width 에 동일 trailing 간격 추가 + 적분 포함 수식 시각 확인.
