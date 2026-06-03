# 최종 결과 보고서 — Task #1233: 수식 큰 연산자(Σ/∏/∫) 피연산자 간격

- **이슈**: #1233 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1233-eq-bigop-spacing` (base: `stream/devel` f83c43b5)
- **기간**: 2026-06-02
- **성격**: 수식 렌더 레이아웃 버그 수정 (간격), 레이아웃 코어 무관

## 1. 문제

합산기호 ∑(및 ∏·∫)의 **피연산자가 연산자에 붙어** 렌더됨(예: 6쪽 문26 "∑bₙ의"). 한컴
PDF 정답지는 적정 간격(thin/med space)을 둠. (#1224 폰트 조사 중 발견 — 폰트 무관.)

## 2. 원인

`equation/layout.rs::layout_row` 는 형제 노드를 `x += b.width` 로 **간격 0** 배치 → 큰
연산자와 피연산자 간격은 **BigOp 박스 width** 가 결정. 그러나 limits(`layout_big_op`)·적분
(`layout_integral`, bare 적분)의 width 에 **trailing 간격이 없어** 피연산자가 붙었다.
(일반 연산자 `+ - =` 는 `layout_symbol` 에서 pad 를 가지나 큰 연산자만 누락.)

## 3. 해결

`BIG_OP_TRAIL_PAD = 0.45`(fs 비율) 상수를 도입하고, 큰 연산자 3경로의 box width 에 trailing
간격을 더함:

| 경로 | 대상 | 변경 |
|------|------|------|
| `layout_big_op` | ∑·∏ (limits) | `width: max_w` → `+ fs*PAD` |
| `layout_integral` | ∫ (첨자) | `width: total_w` → `+ fs*PAD` |
| `layout_math_symbol` | bare ∫ | `width: w` → `+ fs*PAD` |

### 안전성 (자기완결)

인라인 수식은 `svg.rs`(L443-453)에서 `scale_x = 컨트롤 advance(tac_w) ÷ layout_box.width`
로 가로 스케일되어 tac_w 에 맞춰진다. 따라서 BigOp width 증가는 **오버플로(다음 글자 침범)
없이 흡수**되고 연산자-피연산자 간격만 비례적으로 생긴다. 문단 advance·메트릭 DB·페이지네이션
무변경.

### pad 값 = 0.45 (0.1 → 0.25 → 0.45, 작업지시자 시각 판정)

분수·괄호를 포함한 큰 수식은 자연폭이 tac_w 보다 커 `scale_x` 가 0.6~0.9 로 **압축**된다.
자연폭에 더한 pad 도 그만큼 줄어 렌더되므로 초기 0.1 은 큰 수식(첫 ∑(…)) 에서 간격이
부족했다. 압축 후에도 충분한 간격이 남도록 **0.45** 로 확정했다. 0.25는 부족 판정이며,
첫 ∑(...), 둘째 ∑b, 적분 ∫ 모두 PDF 정합이고 둘째 ∑b·적분에서는 과간격이 없었다.

### 렌더러 중앙정렬 보정 (full-trailing)

`svg_render`/`canvas_render` 의 limits 연산자(∑/∏)는 본래 `op_x = (lb.width - op_w)/2` 로
**padding 포함 폭에 중앙정렬**해, (1) pad 의 절반만 trailing 이 되고 (2) 연산자가 첨자(=
`max_w` 중앙정렬)보다 우측으로 밀리는 문제가 있었다. 두 렌더러를 `center_w = lb.width -
fs*BIG_OP_TRAIL_PAD` 에 중앙정렬하도록 보정 → **pad 전체가 순수 trailing** 이 되어 간격이
충분해지고 연산자가 첨자와 정렬된다. WASM(canvas) 경로도 동일 보정 적용.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 시각(문26 ∑bₙ) | base 붙음 → current 간격, **PDF 한컴 정합** (pad 0.45 확정) |
| 적분(문25 ∫) | 피연산자 적정 간격, 오버플로 없음 |
| lim 등 비대상 | 미변경, 부작용 없음 |
| 레이아웃 불변 | `dump-pages` 3문서 base 대비 **바이트 동일** |
| 회귀 | `cargo test --lib` **1522 passed, 0 failed** (신규 단위 테스트 포함) |

시각 자료: `output/poc/task1233/{sigma_before_after_pdf,sigma_aligned_3way}.png`

## 5. 변경 파일

- `src/renderer/equation/layout.rs` (상수 + 3경로 width + 단위 테스트)

## 6. 범위 밖 (후속 여지)

- `layout_row` 일반 커닝(연산자 간 leading 간격 등) — 본 건은 큰 연산자 trailing 한정.
- serif/기타 수식 노드 간격 점검 — 별도 필요 시.
