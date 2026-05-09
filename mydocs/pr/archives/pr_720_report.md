---
PR: #720
제목: render — replay raw SVG fragments in native Skia PNG output (P6)
컨트리뷰터: @seo-rii (Seohyun Lee) — 7번째 사이클 (Skia 영역 핵심)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: 70121b1b
---

# PR #720 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge `70121b1b`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `70121b1b` (--no-ff merge) |
| refs | #536 (멀티 렌더러 트래킹, OPEN 유지) |
| 시각 판정 | 게이트 면제 (결정적 검증 + 회귀 가드 통과) |
| 자기 검증 | lib 1173 + 통합 ALL GREEN + native-skia 24/24 + clippy 0 + sweep 회귀 0 |

## 2. 정정 본질 — P6 단계 (RawSvg fragment replay)

### 2.1 기존 영역
PR #599 (P4 PNG raster backend) + PR #626 (P5 equation replay) 영역 의 후속. RawSvg leaf 영역 placeholder fallback (`renderer.rs:763`).

### 2.2 정정
- `image_conv.rs` (+82 LOC): `draw_svg_fragment` + `rasterize_svg_fragment_to_png` + `svg_parse_options`
- `renderer.rs` (+88/-6): `PaintOp::RawSvg` 영역 placeholder → 실제 raster (resvg + tiny-skia)
- `Cargo.toml`: `native-skia` feature 영역 의 `dep:resvg` 추가

### 2.3 보안 가드 ✅
| 가드 | 영역 |
|------|------|
| MAX_SVG_FRAGMENT_BYTES = 4 MB | fragment 크기 |
| MAX_SVG_RASTER_PIXELS = 67M | 8192×8192 |
| `resolve_string = None` | external href 차단 |
| `resolve_data` | data: URI 만 허용 |
| `resources_dir = None` | 디렉터리 탐색 차단 |
| Wrapper SVG | 안전 wrap |
| Invalid fallback | placeholder 보존 |

### 2.4 회귀 가드 (2건 신규)
- `renders_raw_svg_fragment_as_colored_ink`: green rect → 100+ green 픽셀
- `raw_svg_replay_does_not_load_external_file_hrefs`: external file href → 0 red 픽셀 (보안 입증)

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (1 commit)
```
6505e903 feat: replay raw svg in native skia
```
충돌 0건.

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (28.81s) |
| `cargo test --release --features native-skia skia --lib` | ✅ **24/24 PASS** (신규 2건 + 기존 22건) |
| `cargo test --release` | ✅ lib 1173 + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --features native-skia --lib -- -D warnings` | ✅ 통과 |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** ✅ |

### 3.3 시각 판정 게이트 면제
- 결정적 검증 + 회귀 가드 + 보안 가드 명시 영역 모두 통과
- native Skia PNG 영역 영역 — WASM/browser 영역 무관 영역
- `feedback_visual_judgment_authority` 정합 — CI ALL SUCCESS + 회귀 가드 통과 영역의 면제 합리

## 4. 영향 범위

### 4.1 변경 영역
- native Skia PNG/VLM 경로 영역 의 차트/OLE/내장 SVG 영역 fragment 실제 렌더링

### 4.2 무변경 영역
- WASM/browser SVG replay 영역
- CanvasKit raw SVG replay 영역
- form native replay 영역
- 다른 PaintOp 영역 (Image, Equation, Path, Text 등)

### 4.3 비목표 (PR 본문)
- WASM/CanvasKit / network loading / animated SVG / SVG filter parity / VLM preset / PNG DPI metadata 영역 — 모두 별건

## 5. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @seo-rii **7번째 사이클** (Skia 영역 핵심) |
| `feedback_process_must_follow` | 비목표 명시 + scope 좁힘 |
| `feedback_image_renderer_paths_separate` | native Skia 경로 영역 만 변경 — WASM/CanvasKit 영역 무영향 |
| `feedback_visual_judgment_authority` | 결정적 검증 + 회귀 가드 통과 영역 → 시각 판정 면제 합리 |
| `feedback_pr_supersede_chain` | PR #599 (P4) → #626 (P5) → **#720 (P6)** 단계적 진전 영역 |

## 6. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- Issue #536 OPEN 유지 (멀티 렌더러 트래킹 영역) — 후속 P7+ 영역 가능

---

작성: 2026-05-09
