# Task #283 최종 보고서 — 수식 SVG 괄호 path 폭 조정

## 1. 배경

Task #280 Phase 2. #280 에서 폰트 스택 재정렬로 "볼드 인상" 문제를 해결했으나, **괄호 `(` `)` 가 얇은 moon 모양으로 렌더되어 폰트 글리프와 불일치** 하는 별도 문제가 남음.

`samples/equation-lim.hwp` 렌더 결과에서 `f(2+h)` 의 파렌이:
- 박스 할당 폭은 크고 (`fs * 0.3 = 4.40px`)
- 실제 곡선 시각 폭은 작음 (`1.54px`)
- 결과: 박스 내부에 "그림자" 처럼 떨어진 얇은 moon + 글자와 gap 벌어짐

## 2. 조사 결과 (단계 1)

### Times New Roman `(` 실측 (Chrome headless)

| 항목 | 값 (px, fs=14.67) | em 비율 |
|------|-------------------|---------|
| advance_width | 4.89 | **0.333** |
| bbox width | 5.0 | 0.341 |
| bbox height | 14.0 | 0.955 |

### 현재 rhwp path 파렌 분석

```
M41.25,2.93 Q38.17,10.27 41.25,17.60
```

| 항목 | 값 (px) |
|------|---------|
| 박스 할당 `paren_w` | 4.40 |
| 곡선 시각 폭 | **1.54** (Times bbox 의 31%) |
| 박스 내 whitespace | 69% |

### 진단

**단일 제어점 quadratic Bezier 의 수학적 한계** — Times `(` 는 비대칭 bowl + 세리프 끝단을 갖는 복잡한 곡선이라 quadratic 으로 모사 불가. 어떤 제어점 이동도 moon 형상을 벗어나지 못함.

## 3. 선정 (단계 2, 6 변형 프로토타입)

| # | 변형 | 결과 |
|---|------|------|
| 0 | baseline (paren_w=0.30, ctrl=0%) | 얇은 moon |
| 1 | A_conservative (0.27, 10%) | 약간 개선, 여전히 moon |
| 2 | A_aggressive (0.25, 15%) | 더 타이트, 여전히 moon |
| 3 | **B_glyph** (`<text>(</text>`) | **본 파렌 형상** |
| 4 | extra_A (0.28, 20%) | moon 깊어짐 |
| 5 | extra_B (0.24, 25%) | 부자연스러움 심화 |

**path 튜닝(0·1·2·4·5) 모두 moon 극복 실패 — B_glyph 만 글자와 일관된 타이포그래피 제공.**

**옵션 B (글리프 전환 + 임계치 분기)** 채택:
- 텍스트 높이 파렌 (`body.height / fs ≤ 1.2`) → `<text>(</text>` 글리프
- 스트레치 파렌 (`> 1.2`) → 기존 path 유지 (분수·sum·매트릭스 감쌈)

## 4. 변경 범위 (단계 3)

| 파일 | 위치 | 변경 |
|------|------|------|
| `src/renderer/equation/layout.rs` | `layout_paren:832` | `paren_w: fs * 0.3 → fs * 0.333` (Times advance 매치) |
| `src/renderer/equation/svg_render.rs` | `LayoutKind::Paren` arm | 높이 분기: `lb.height ≤ fs*1.2` + `(`/`)` → `<text>`, 외는 `draw_stretch_bracket` |
| `src/renderer/equation/canvas_render.rs` | `LayoutKind::Paren` arm | 동일 분기 (`ctx.fill_text` / path) |

**변경 제외**:
- `LayoutKind::Matrix` arm — 항상 스트레치 path (변경 없음)
- `{`, `[`, `|` 등 기타 괄호 — 범위 밖
- `draw_stretch_bracket` 내부 path 품질 개선 — 후속 이슈 여지

**테스트 갱신**:
- `test_paren_svg` — `<path>` → `<text>` assertion 으로 전환 (텍스트 높이 파렌)
- `test_paren_stretch_svg` 신규 — 스트레치 path 보전 확인 (`a over b` 감쌈)

## 5. 검증 (단계 3·4)

### 회귀 테스트

| 명령 | 결과 |
|------|------|
| `cargo check` | ✅ |
| `cargo test --lib equation` | ✅ **49/49** (신규 스트레치 테스트 포함) |
| `cargo test --test svg_snapshot` | ✅ 3/3 |
| `cargo clippy --lib --bins --tests` | ✅ 에러 없음 |
| `cargo check --lib --target wasm32-unknown-unknown` | ✅ |
| `cargo test --lib` 전체 | 950 pass / 14 fail (기존 CFB writer, #280 단계에서 확인 완료) |

### 실제 SVG 출력

`samples/equation-lim.hwp` 변경 전/후:

| 파렌 형태 | 단계 1 (before) | 단계 4 (after) |
|-----------|----------------|----------------|
| `<path>` | **4건** | **0건** |
| `<text>(/)</text>` | 0건 | **4건** (2 `(` + 2 `)`) |

### 3면 시각 비교

`mydocs/working/task_m100_283_stage4/compare.png`:
- **BEFORE**: 얇은 moon, 글자와 gap
- **AFTER**: Times 글리프, 글자와 자연 밀착
- **PDF 레퍼런스**: 자연 Times 글리프

→ **AFTER ≈ PDF**. 목표 달성.

### exam_math.hwp 회귀 (20페이지 중 4개 샘플링)

| 페이지 | 관찰 | 결과 |
|--------|------|------|
| p001 | 표지 | 정상 |
| p005 | `f(1)=f(2)=0`, `y=f(x)` 다수 글리프 파렌 | **글리프 렌더 정상** |
| p009 | `P(A\|B)` (글리프) / `P(A∩B)`·`P(A∪B)` (스트레치 path) | **분기 올바름** |
| p013 | 극한·분수·적분 (`lim x→0 3x²/sin²x`, `∫₀¹⁰ (x+2)/(x+1) dx`) | 정상 |

**임계치 `body.height ≤ fs * 1.2` 가 텍스트/스트레치 경계를 의도대로 분기 확인.**

## 6. 산출물

- `mydocs/plans/task_m100_283{,_impl}.md`
- `mydocs/working/task_m100_283_stage{1,2,3,4}.md`
- `mydocs/working/task_m100_283_stage1/` — 기준선 + Times 글리프 실측 (`glyph_metrics.json`, `metrics.md`)
- `mydocs/working/task_m100_283_stage2/` — 6 변형 프로토타입 + 선정 근거 (`variants/_compare_all.png`, `selected.md`)
- `mydocs/working/task_m100_283_stage4/` — before/after/PDF 3면 비교 + exam_math 회귀 페이지

## 7. 후속 과제 후보

이번 타스크 범위 밖 (필요 시 별도 이슈 등록):

- **기타 괄호 `{`·`[`·`|` 글리프 전환** — 본 변경은 `(`·`)` 만 대상. 동일 패턴으로 확장 가능하나 bracket 별 Unicode 상단/중단/하단 글리프 매칭 조사 필요.
- **스트레치 path 품질 개선** — 단일 제어점 quadratic Bezier 한계 (moon 형상). cubic Bezier 또는 다중 세그먼트 path 로 재설계 시 스트레치 파렌도 Times 근접 가능. 분수·sum·행렬에서 관찰 가능한 문제.
- **Matrix arm 동일 임계치 적용** — 현재 Matrix 는 셀 높이와 무관하게 항상 path. 실제로 `(` 하나만 있는 행은 텍스트 높이일 수 있음. 경계 케이스.

## 8. 교훈

1. **프로토타입 다수 비교가 수학적 한계 확인에 결정적** — path 튜닝 3안을 모두 돌리지 않았다면 "제어점을 더 움직이면 해결될 것" 가설에 갇혔을 것. 데이터가 "어떻게 해도 못 벗어남" 을 증명해 글리프 전환 결정이 자연스러웠음.
2. **임계치 분기는 "범위를 작게 잡는" 도구** — `body.height ≤ fs * 1.2` 로 텍스트 높이만 공략하고 스트레치는 건들지 않아 회귀 가능성 최소화. 스트레치 path 품질은 별개 문제로 분리 가능.
3. **폰트 글리프가 있는데 path 로 재구현하지 말자** — Times `(` 는 이미 존재. 50년간 다듬어진 글리프를 40줄의 Bezier 로 이기려 하는 건 구조적으로 불리한 싸움.
