---
PR: #720
제목: render — replay raw SVG fragments in native Skia PNG output (refs #536, follow-up #599/#626)
컨트리뷰터: @seo-rii (Seohyun Lee) — **7번째 사이클 PR** (Skia 영역 핵심 컨트리뷰터)
base / head: devel / render-p6
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: ALL SUCCESS
변경 규모: +178 / -11, 5 files (소스 2 + Cargo.toml + README ×2)
검토일: 2026-05-09
---

# PR #720 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #720 |
| 제목 | render — replay raw SVG fragments in native Skia PNG output |
| 컨트리뷰터 | @seo-rii (Seohyun Lee) — **7번째 사이클 PR** (Skia 영역 핵심 컨트리뷰터) |
| base / head | devel / render-p6 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 |
| CI | ALL SUCCESS (Build & Test, CodeQL ×3, Render Diff, Canvas visual diff) |
| 변경 규모 | +178 / -11, 5 files |
| 커밋 수 | 1 (단일 본질 정정) |
| refs | #536 (멀티 렌더러 트래킹) |
| follow-up | PR #599 (P4 Skia PNG 백엔드, 머지 완료) / PR #626 (P5 equation replay, 머지 완료) |

## 2. 컨트리뷰터 사이클

@seo-rii (Seohyun Lee) — 누적 7 PR (모두 close 영역, cherry-pick 머지 패턴):
- PR #165 (skia renderer 도입, 4/16)
- PR #419 (PageLayerTree API, 4/28)
- PR #456 (Canvas → PageLayerTree, 4/29)
- PR #498 (canvas visual diff pipeline, 4/30)
- PR #599 (native Skia PNG raster backend, 5/5)
- PR #626 (equation replay, 5/6)
- **PR #720 (raw SVG replay, 5/8) — 본 PR**

→ **7번째 사이클 핵심 컨트리뷰터** (Skia 영역 트래킹 #536 영역의 단계적 진전 영역). PR #599/#626 모두 본 환경 devel 머지 완료 확정.

## 3. 정정 본질 — RawSvg leaf 영역 placeholder → 실제 raster

### 3.1 결함 영역
P5 (PR #626) 까지 native Skia 경로 영역 의 image/equation 영역 처리 영역, **RawSvg 영역 placeholder fallback** 영역 (`renderer.rs:763` `PaintOp::RawSvg { bbox, .. } => draw_placeholder(*bbox, "svg")`).

### 3.2 정정 (P6 단계)

**`src/renderer/skia/image_conv.rs` (+82 LOC)**:
```rust
const MAX_SVG_FRAGMENT_BYTES: usize = 4 * 1024 * 1024;  // 4 MB 가드
const MAX_SVG_RASTER_PIXELS: u64 = 67_108_864;           // 67M (8192×8192)

pub fn draw_svg_fragment(canvas, svg_fragment, x, y, width, height, sampling) -> bool {
    let png = rasterize_svg_fragment_to_png(svg_fragment, width, height)?;
    draw_image_bytes(canvas, &png, ...);  // 기존 image draw 경로 영역 재사용
    true
}

fn rasterize_svg_fragment_to_png(svg_fragment, width, height) -> Option<Vec<u8>> {
    // 가드: empty / >4MB / non-finite / pixel >67M / size==0
    let svg = format!(
        "<svg xmlns=... width=... height=... viewBox=...>{svg_fragment}</svg>"
    );
    let tree = usvg::Tree::from_str(&svg, &svg_parse_options()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(...)?;
    resvg::render(&tree, ..., &mut pixmap.as_mut());
    pixmap.encode_png().ok()
}

fn svg_parse_options() -> usvg::Options<'static> {
    let mut options = usvg::Options::default();
    options.resources_dir = None;                              // 리소스 디렉터리 차단
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),  // data: URI 만 허용
        resolve_string: Box::new(|_, _| None),                  // external href 차단
    };
    let fontdb = options.fontdb_mut();
    fontdb.load_system_fonts();
    fontdb.set_sans_serif_family("Noto Sans CJK KR");
    fontdb.set_serif_family("Noto Serif CJK KR");
    fontdb.set_monospace_family("D2Coding");
    options
}
```

**`src/renderer/skia/renderer.rs` (+88/-6, line 760)**:
```rust
PaintOp::RawSvg { bbox, raw } => {
    if !draw_svg_fragment(
        canvas, raw.svg.as_str(),
        bbox.x as f32, bbox.y as f32,
        bbox.width as f32, bbox.height as f32,
        ImageSampling::linear(),
    ) {
        draw_placeholder(*bbox, "svg");  // invalid SVG fallback 보존
    }
}
```

### 3.3 의존성 (`Cargo.toml`)
- `native-skia` feature 영역 의 `dep:resvg` 추가
- `resvg = { version = "0.45", optional = true }` (이미 base 영역 의 `usvg = "0.45"` 영역과 정합)

## 4. 보안 영역 정합 점검 ✅

| 보안 가드 | 영역 |
|----------|------|
| MAX_SVG_FRAGMENT_BYTES = 4 MB | fragment 크기 가드 |
| MAX_SVG_RASTER_PIXELS = 67M | 8192×8192 영역 raster pixel 가드 |
| `resolve_string = Box::new(\|_, _\| None)` | **external href 차단** (file:// / http:// / https:// 등) |
| `resolve_data` | usvg 기본 data: URI resolver 만 허용 |
| `resources_dir = None` | 리소스 디렉터리 자동 탐색 차단 |
| Wrapper SVG | `<svg xmlns="..." width="..." height="..." viewBox="...">{fragment}</svg>` 안전 wrap |
| Invalid SVG fallback | 기존 placeholder 영역 보존 |

→ **외부 리소스 로딩 차단 완비**. PNG/VLM 경로 영역 의 예측 가능한 raster output 영역 정합.

## 5. 회귀 가드 테스트 (2건 신규)

### 5.1 `renders_raw_svg_fragment_as_colored_ink`
- green rect (`<rect ... fill="#00ff00"/>`) 영역 → 100+ green 픽셀 영역 검증
- placeholder 부재 영역 의 실제 raster 영역 출력 입증

### 5.2 `raw_svg_replay_does_not_load_external_file_hrefs`
- 외부 파일 href (`<image href="/tmp/...png"/>`) 영역 → red 픽셀 0건 영역 검증
- **보안 가드 작동 입증** (외부 file href 영역 차단 확정)

### 5.3 기존 테스트 보강 (line 1672)
- 기존 `renders_layer_tree_to_png` 영역 의 RawSvg 영역 `<rect/>` → `<invalid` 영역 변경
- invalid SVG 영역 fallback placeholder 영역 검증 영역 강화

## 6. 영향 범위

### 6.1 변경 영역 (PNG/VLM 경로 영역)
- native Skia PNG output 영역 의 차트/OLE/내장 SVG 영역 fragment 실제 렌더링
- 기존 placeholder 영역 회피 영역

### 6.2 무변경 영역
- WASM/browser SVG replay 영역 (별건)
- CanvasKit raw SVG replay 영역 (별건)
- form native replay 영역 (별건)
- 다른 PaintOp 영역 (Image, Equation, Path, Text 등)

### 6.3 비목표 영역 (PR 본문 명시)
- browser/WASM SVG replay 변경
- CanvasKit raw SVG replay
- full SVG security policy 설계
- network/file resource loading
- animated SVG 지원
- SVG filter 전체 parity
- form native replay
- VLM preset 확장 (#613)
- PNG DPI metadata (#614)

→ **비목표 명시 정합** (`feedback_process_must_follow` 정합).

## 7. 충돌 / mergeable

- merge-base = `ff0c7d3e` (PR #678 / Task #674 영역, **5/7 시점 — 상당히 오래된 base**)
- devel HEAD = `c9dd6f9c` (5/9 시점, **PR #599/#626 머지 후 + 다수 머지 후 영역**)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (Skia 영역 만 변경, 다른 영역 무관)

→ PR #599/#626 영역 의 후속 영역 영역 일관 영역.

## 8. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge (추천)

PR #720 영역 의 단일 commit 영역 + PR #599/#626 영역 의 cherry-pick 패턴 정합. PR #694~#719 패턴 일관.

```bash
git branch local/task720 c9dd6f9c
git checkout local/task720
git cherry-pick 39924845
git checkout local/devel
git merge --no-ff local/task720
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release --features native-skia` 통과
- [ ] `cargo test --release --features native-skia skia --lib` — 2 신규 + 기존 테스트 PASS
- [ ] `cargo test --release` — 전체 ALL GREEN
- [ ] `cargo clippy --release --features native-skia --lib -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (Skia 영역 무관 영역, 본 환경 sweep 영역 native-skia 미사용 영역)

### 시각 판정 게이트 (선택)
- 본 PR 영역 의 native Skia PNG 영역 의 RawSvg 영역 시각 영역 — `cargo run --release --features native-skia --bin rhwp -- export-png samples/...` 영역 영역 SVG fragment 영역 정합 확인 영역
- 결정적 검증 (CI ALL SUCCESS + 신규 회귀 가드 2건) 통과 영역 → 시각 판정 게이트 면제 가능 영역

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @seo-rii **7번째 사이클** 핵심 컨트리뷰터 (Skia 영역 트래킹 #536) |
| `feedback_process_must_follow` | 비목표 명시 + scope 좁힘 (PR 본문 영역) |
| `feedback_image_renderer_paths_separate` | native Skia 경로 영역 만 변경 — WASM/CanvasKit 영역 무영향 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + 회귀 가드 2건) 통과 영역 → 시각 판정 면제 가능 영역 |
| `feedback_pr_supersede_chain` | PR #599 (P4) → PR #626 (P5) → **PR #720 (P6)** 단계적 진전 영역 (Issue #536 트래킹) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 에서 1 commit cherry-pick (옵션 A)
2. 자기 검증 (cargo test --features native-skia + 회귀 가드 2건 + clippy)
3. 광범위 sweep (Skia 영역 무관 영역, 회귀 0 확증 영역)
4. (선택) export-png 영역 의 시각 판정
5. no-ff merge + push + archives 이동 + 5/9 orders 갱신
6. PR #720 close (refs #536 영역 — close 명시 부재, 수동 close 영역)

---

작성: 2026-05-09
