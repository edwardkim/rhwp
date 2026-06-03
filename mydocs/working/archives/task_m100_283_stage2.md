# Task #283 단계 2 완료보고서 — 튜닝안 프로토타입 + 최종 선정

## 수행 내역

6개 변형을 동일 문맥(`f(2+h)-f(2)`) 으로 렌더, 시각 비교:

| # | 변형 | 결과 |
|---|------|------|
| 0 | baseline (paren_w=0.30, ctrl=0%) | 얇은 moon, 글자와 gap |
| 1 | A_conservative (0.27 / 10%) | 약간 개선, 여전히 moon |
| 2 | A_aggressive (0.25 / 15%) | 더 타이트, 여전히 moon |
| 3 | **B_glyph** (`<text>(</text>`) | **자연스러운 본 파렌 형상** |
| 4 | extra_A (0.28 / 20%) | moon 깊어짐, 삼각형화 시작 |
| 5 | extra_B (0.24 / 25%) | 부자연스러움 심화 |

산출물: `mydocs/working/task_m100_283_stage2/variants/` (7개 PNG + 6개 SVG + `_compare_all.png` 합성)

## 핵심 발견

**모든 path 변형이 moon 형상을 벗어나지 못함** — 단일 제어점 quadratic Bezier 의 수학적 한계. Times `(` 는 비대칭 바울 + 세리프 끝단을 갖는 복잡한 곡선이라 path 로 모사 어려움.

**3_B_glyph 만 글자 폰트와 일관된 타이포그래피** 제공. 단계 1 측정(Times advance 4.89 = fs*0.333, 글리프 bbox 꽉 참) 과 정확히 부합.

## 선정: 옵션 B (글리프 전환 + 임계치 분기)

- **텍스트 높이 파렌** (`body.height / fs ≤ 1.2`) → `<text>(</text>` / `<text>)</text>`
- **스트레치 파렌** (`> 1.2`) → 기존 path 유지 (분수·sum·매트릭스)

임계치 `1.2 * fs` 는 직관적 "텍스트 높이 근접" 기준. 민감 조정 여지 있으나 첫 시도로는 충분.

## 구현 범위 (단계 3 에서 반영)

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/layout.rs` | `layout_paren` 의 `paren_w: fs * 0.3 → fs * 0.333` (Times advance 매치) |
| `src/renderer/equation/svg_render.rs` | `LayoutKind::Paren` arm — 높이 분기 (glyph / path) |
| `src/renderer/equation/canvas_render.rs` | 동일 분기 (svg 와 동기) |

**범위 밖** (기존 유지):
- `LayoutKind::Matrix` — 항상 스트레치 path
- `{`, `[`, `|` 등 기타 괄호 — 본 타스크 범위 외
- `draw_stretch_bracket` 내부 수정 (path 품질 개선은 후속)

## 완료 조건

- [x] 6 변형 렌더 + 시각 비교
- [x] 측정 근거 기반 결정 (옵션 B 채택)
- [x] 구현 파일 + 임계치 확정
- [x] `selected.md` 에 결정 근거 문서화

## 다음 단계

단계 3: 코드 변경 + 회귀 테스트.
