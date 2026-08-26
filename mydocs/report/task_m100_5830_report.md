# task_m100_5830 처리결과 — 마지막 줄 dash leader 를 자연 폭으로 되살린다

- 이슈: [#5830](https://github.com/edwardkim/rhwp/issues/5830)
- 기준: `0f9ceeb19` (devel)
- 재현: `samples/issue1891/86712_regulatory_analysis.hwpx` · 정본 `pdf/issue1921/86712_regulatory_analysis-2024.pdf`
- 시각 증적: [`edit_demo_5830/compare_p34.png`](edit_demo_5830/compare_p34.png) · [`compare_p35.png`](edit_demo_5830/compare_p35.png)

## 1. 결함

양쪽정렬 배분 대상이 아닌 줄(문단 마지막 줄·강제 줄바꿈 줄)은 `needs_word_distribution` 이
false 라 `compute_line_extra_spacing` 이 **슬랙 자체를 계산하지 않았다.** dash leader 는
`char_width_decision` 의 클램프 하한 `min(자연폭, font_size × 0.3)` 에 그대로 남아,
한글 2022 정본 대비 **폭 절반**(0.300em vs 0.499~0.583em)으로 그려졌다.

## 2. 한글의 마지막 줄 규칙 — 정본 글리프 원점 실측으로 확정

이슈는 "한글은 여백까지 늘림"으로 추정했지만, 정본 PDF 의 하이픈 글리프 원점을 전수로
재 보니 **무한 신장이 아니었다**:

| 정본 런 | advance | 끝점 |
|---|---|---|
| p34 10자 | 7.00pt = **0.499em** | x 530 — **여백에 정확히 닿음** |
| p35 10자 | 8.00pt = **0.571em** | x 404 — 여백(537)에 **닿지 않음** |
| p35 18자 | 8.00pt = **0.571em** | x 496 — 닿지 않음 |

즉 규칙은 둘이다: **여백이 충분하면 자연 폭(≈0.571em)으로, 여백이 그보다 좁으면
여백까지만 좁힌다.** 슬랙 전량을 dash 에 쏟으면(구현 중 실제로 만들었던 오답)
p35 8자 런이 1.9em 까지 벌어진다 — 계약 테스트가 상한(0.75em)으로 그 방향도 막는다.

## 3. 변경

`src/renderer/layout/paragraph_layout.rs` — `compute_line_extra_spacing` 에 분기 추가:
`!needs_justify` 이고 정렬이 여백을 채우는 종류(Justify·Split)일 때, dash leader 에
`min(슬랙/leader수, 자연폭 − 클램프폭)` 만큼 `extra_dash_advance` 를 준다.

- **클램프를 자연 폭 한도 안에서만 되돌린다** — 슬랙이 남아도 자연 폭을 넘지 않는다.
- 여백이 좁으면 슬랙만큼만 — 정본의 "여백까지만" 동작.
- needs_justify 줄의 기존 탄력 흡수(Task #352)는 무변경.
- 왼쪽/가운데 정렬은 건드리지 않는다 — 짧은 dash 가 저자 의도일 수 있다.

## 4. 전/후 실측 (rhwp SVG, 96dpi)

| 런 | 수정 전 | 수정 후 | 정본 |
|---|---|---|---|
| p34 10자 | 0.300em, 끝 641 | **0.589em, 끝 690** | 0.499em, 여백 도달 |
| p34 30자 | 0.300em, 끝 578 | **0.482em, 끝 677** (슬랙 제약) | 0.530em, 여백 도달 |
| p35 8자* | 0.300em, 끝 469 | **0.589em, 끝 506** (여백 미도달) | 0.571em, 미도달 |
| p35 18자 | 0.300em, 끝 562 | **0.589em, 끝 654** (여백 미도달) | 0.570em, 미도달 |
| 양쪽정렬 런 4개 | 0.502~0.603em | **무변경** | 0.499~0.583em |

*rhwp 와 한글의 줄바꿈이 달라 런 길이는 8자 vs 10자로 다르다 — 비교 대상은 advance 와
도달/미도달 패턴이다. rhwp 자연 폭(0.589em)은 정본(0.571em)보다 3% 넓다(폰트 메트릭 차).

## 5. 검증

### red → green

수정 분기만 죽이고 돌리면:
```
p33 y=427.2 10자 런의 advance 0.300em — 정본 대역(0.45~0.75em) 밖
test result: FAILED. 1 passed; 1 failed
```
되돌리면 2/2 통과. 상한 테스트는 슬랙 전량 분배 오답(1.9em)도 잡는다(실제로 잡았다).

### 259문서 쪽수 게이트

`python tools/render_page_gate.py --root . --fixture tests/fixtures/render_page_samples.tsv`

- 일치 245/259 (기준선 244) — **개선 1건, 회귀 0건**
- 유일한 변화: `samples/issue3637/press_release_topbottom_float.hwpx` 3쪽(오답) → **2쪽(한글 정답)**

### 실행한 것

| 명령 | 결과 |
|---|---|
| `cargo test --lib -p rhwp` | **3,893 passed** / 13 ignored |
| `cargo test --test regression_suite_015` (신규 계약 포함) | 124 passed |
| `cargo test --test regression_suite_020` (쪽수 픽스처 계약) | 121 passed |
| `cargo test --test regression_suite_027` (#5804 dash 글리프 회귀) | 119 passed |
| `cargo test --test regression_suite_001` (#5799 탭 리더 회귀) | 75 passed |
| `rustfmt --edition 2021` 변경 파일 | 정리 완료 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4,225 (src 기준선 유지) |

## 6. 시각 증적 생성 방법

```
rhwp export-svg samples/issue1891/86712_regulatory_analysis.hwpx -p 33|34 -o <dir>   # 전/후
resvg --zoom 1.5 <svg> <png>                                                          # 래스터
PyMuPDF 로 정본 PDF p34·p35 래스터 → PIL 로 전/후/정본 3단 합성
```
