# Task M100 #1317 최종 결과보고서 — SVG 적분기호 path 렌더 (PR #1314 후속)

## 1. 개요

| 항목 | 내용 |
|------|------|
| 이슈 | edwardkim/rhwp #1317 |
| 배경 | PR #1314 (Task #1313) 메인테이너 시각 판정 보류 |
| 브랜치 | `local/task1317` (base: PR #1314 head `task1313-pr`) |
| 채택 방향 | 2안 — SVG에서 적분기호를 path/shape로 렌더 |

## 2. 문제 (보류 사유)

PR #1314는 Canvas/WASM 적분 배치는 개선했으나, **SVG export에서 적분 상·하한 위치가 어긋남**:

- `layout_subsup()` 적분 분기가 **고정 offset**(`fs*0.21`/`fs*0.55`/`fs*0.13`)으로 상·하한 배치, 글리프 박스를 `fs×INTEGRAL_SCALE` 명목 크기로 가정.
- 이 offset은 Canvas 웹폰트 메트릭 기준 튜닝.
- SVG는 글리프를 `<text font-family=...>`로만 출력하고 폰트 미임베딩(수식 폰트가 `ttfs/`에도 부재) → 뷰어 대체 폰트(rsvg=Times 등)의 ∫ 실제 bbox가 가정과 달라 상·하한이 위아래로 벌어지고 떨어져 보임.

## 3. 해결

적분기호 ∫를 폰트 `<text>`가 아닌 **stroke path(Bézier S-곡선)**로 렌더하고, 글리프 형상·상하한 attach point를 **단일 기준(`integral_geom`) 공유**로 산출. SVG/Canvas/Skia 3경로가 폰트 대체에 무관하게 정합.

### 단계별 변경

| Stage | 파일 | 내용 |
|-------|------|------|
| 1 | `layout.rs` | `IntegralGeom`/`integral_geom()` SSOT 신설, `layout_subsup` 적분 분기 geom 기반 재작성(매직 offset 제거), bare 적분 advance=geom.width |
| 2 | `svg_render.rs` | `integral_path()` 헬퍼, ∫ `<text>`→`<path>` (∑/∏ text 유지) |
| 3 | `canvas_render.rs`, `skia/equation_conv.rs` | 동일 geom path(`draw_integral`)로 3경로 정합 |

## 4. 검증

| 항목 | 방법 | 결과 |
|------|------|------|
| 네이티브 빌드 | `cargo build --release` | ✅ |
| 전체 테스트 | `cargo test` | ✅ 1602 passed |
| native-skia 테스트 | `cargo test --features native-skia --lib` | ✅ 1643 passed |
| Canvas 컴파일 | `cargo check --target wasm32-unknown-unknown --lib` | ✅ |
| Skia 빌드/렌더 | `cargo build --features native-skia` + `export-png` | ✅ |
| 네이티브 clippy | `cargo clippy --features native-skia` | ✅ clean |
| 포맷 | `cargo fmt -- <변경파일>` | ✅ 적용 |

### 시각 정합 (정답: `pdf/3-10월_교육_통합_2022.pdf` 9페이지)

- `∫_0^2 (2x³+3x²)dx`, `∫_0^4 {g(x)-f(x)}dx`, `2∫_0^2 (-2x²+6x)dx` — 상한 "상단 우측", 하한 "하단" 줄기 밀착, PDF 정합.
- **SVG ↔ Skia 동일 path glyph** (`output/poc/pr1314/cmp_3way.png`).
- 수정 전/후 비교: `cmp_int.png`.
- 회귀: ∑/∏는 `<text>` 유지(SVG `>∑<` 4, `>∫<` 0, 적분 path 3), 전체 페이지 레이아웃 정상.

## 5. 결론

메인테이너 요구("Canvas만이 아니라 SVG/native PNG/Skia까지 같은 기준 정합")를 `integral_geom` SSOT + path 렌더로 충족. PR #1314 갱신 대상.

## 6. 산출물

- 코드: `layout.rs`, `svg_render.rs`, `canvas_render.rs`, `skia/equation_conv.rs`
- 문서: 수행/구현 계획서, Stage 1~3 보고서, 본 보고서
- POC: `output/poc/pr1314/` (cmp_int.png, cmp_3way.png, skia_p9.png 등)
