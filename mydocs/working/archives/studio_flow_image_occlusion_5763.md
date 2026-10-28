---
kind: working
status: active
issue: 5763
---

# studio flow 그림이 canvas 에 가려지는 결함 (#5763)

작업 브랜치: `fix/5763-studio-flow-image-occluded`
대상: `src/paint/replay_order.rs` · `src/paint/mod.rs` ·
`src/document_core/queries/rendering.rs` · `rhwp-studio/src/view/page-renderer.ts` ·
`rhwp-studio/tests/render-backend.test.ts`

## 한 줄

flow 그림 밑에 불투명 채우기가 깔린 페이지는 flow-static 분리를 하지 않는다 — 분리하면
그 채우기가 canvas 에 남아 아래 평면의 그림을 덮는다.

## 이슈가 요구한 것

studio 에서 `156550355` 문서 3·4·11쪽 그림이 **빈 흰 상자**로 보인다. `export-svg`·
`export-pdf` 는 정상이다.

실측(headless studio):

- `<img>` 는 DOM 에 있고 `complete=true`, `naturalWidth=469`, `fetch(src)` → `200 / 165,013 B`
- `display:block visibility:visible opacity:1`, `getBoundingClientRect()` 도 제자리
- `document.elementFromPoint(그림 중심)` → **CANVAS(z=1)**, 그 픽셀은 `[255,255,255,255]`

원인은 평면 분리다. flow-static 분리(#516·#3315)는 flow plane 의 `Image`/`RawSvg` 만 canvas
**아래** 평면(DOM `<img>` layer 또는 flow-static canvas)으로 내리고, 나머지 본문은
`flow-dynamic` 으로 그 **위에** 그린다. 그림을 담은 표 칸의 흰 배경 사각형은 `flow-dynamic`
에 남으므로 그림을 덮는다.

`export-svg` 산출이 그 사각형을 그대로 보여주고, 좌표가 flow op 의 `clip` 과 같다.

```
flow op :  "clip":{"x":119.400,"y":567.067,"width":274.493,"height":146.360}
SVG     :  <rect x="119.400…" y="567.067" width="274.493" height="146.360" fill="#ffffff"/>
           <image x="119.400…" y="569.227" width="273.253" height="142.040" href="data:image/png;…"/>
```

같은 문서 안에서 흰 사각형 유무가 가시성과 1:1 로 갈린다.

| rhwp 쪽 | 문서 쪽 | `<image>` | 앞선 흰 `<rect>` | studio |
|---|---|---|---|---|
| 5 | 3 | 2 | 2 | 가려짐 |
| 6 | 4 | 2 | 2 | 가려짐 |
| 8 | 6 | 2 | 0 | 보임 |
| 9 | 7 | 1 | 0 | 보임 |
| 13 | 11 | 3 | 3 | 가려짐 |
| 14 | 12 | 3 | 0 | 보임 |

## 고친 방법

판정을 Rust 한 곳에 두고 studio 가 그 값을 본다.

1. `FlowStaticOcclusion` (`src/paint/replay_order.rs`) — layer tree 를 paint 순서대로 훑으며
   불투명 flow 채우기의 bbox 를 쌓고, **앞선 채우기와 겹치는 flow 그림**을 만나면 참이 된다.
   채우기 판정은 `fill_color`/`pattern`/`gradient` 중 하나가 있고 `opacity >= 1.0` 인
   `Rectangle`/`Ellipse`/`Path` 다. 테두리만 있는 도형은 세지 않는다.
2. `get_page_overlay_images_native` 가 그 값을 `flowStaticOccluded` 로 낸다.
3. `PageRenderer.shouldSplitStaticFlow` 가 참이면 분리하지 않는다 — 종전 `'flow'` 단일 평면
   렌더로 떨어져 그림이 제 순서에 그려진다.

구형 WASM(필드 없음)에서는 종전대로 분리를 허용한다(`wrapper.flowStaticOccluded === true`).
좁은 질의를 못 쓰는 트리 폴백 경로(`collectLayerPlaneSummary`)도 같은 규칙을 쓴다.

## 만지지 않은 경로

- `web_canvas.rs` 의 `LayerFilter::FlowDynamic`/`FlowStatic` 술어 — op 단위 무상태 판정이라
  "그림 밑" 이라는 순서 개념을 담을 수 없다. 분리 여부 게이트로 막는 쪽이 실패 안전하다.
- 렌더 산출 자체(SVG/PDF/`flow` 필터) — 원래 정상이라 손대지 않았다.

## 시험 명령

```bash
cargo test --test regression_suite_029 issue_5763       # 신규 계약 3건, ok
cargo test -p rhwp --lib paint::replay_order            # 기존 8건, ok
cargo test --test regression_suite_020 issue_938        # overlay JSON 계약 3건, ok
cd rhwp-studio && node --test tests/*.test.ts           # 1002 passed / 1 skipped / 0 failed
node scripts/rust-unit-test-tiers.mjs --check           # 4225 tests (증가 없음)
```

신규 계약 테스트는 `tests/cases/issue_5763_flow_static_occlusion.rs` 에 둔다 — source-side
`#[cfg(test)]` 총량 증가 금지 정책(`rust-unit-test-tiers.mjs`) 때문에 `src/` 안에 두지 않았다.

- `issue_5763_opaque_flow_fill_under_image_blocks_static_split`
- `issue_5763_non_overlapping_or_unfilled_shapes_keep_static_split`
- `issue_5763_fill_after_image_or_on_other_plane_keeps_static_split`
- `rhwp-studio/tests/render-backend.test.ts` — 요약 필드·분리 게이트·폴백 규칙 6개 앵커

## 시각 근거 (headless studio, 같은 문서)

`wasm-pack build --target web --out-dir pkg --no-opt` 로 이 브랜치의 WASM 을 만들고
studio dev 서버에 붙여 `samples/` 에 임시로 둔 같은 문서를 열었다.

```
overlay summary  page 5  flowImageCount=2  flowStaticOccluded=true
                 page 6  flowImageCount=2  flowStaticOccluded=true
                 page 13 flowImageCount=3  flowStaticOccluded=true
                 page 1·8·14              flowStaticOccluded=false   (분리 유지)
DOM <img> layer  page 5·6 → 0장 (분리 안 함 → canvas 가 제 순서로 그림)
```

수정 전 빈 흰 상자였던 문서 3·4·11쪽의 그림이 모두 보인다.

## fmt 게이트

```bash
cargo fmt --all -- --check      # exit 0
cargo clippy --all-targets -- -D warnings   # exit 0
```

`tests/generated/` 는 파생 산출물이라 fresh worktree 에 없다. 없으면 `cargo fmt --all` 이
"file does not exist" 로 exit 1 이 되므로 `node scripts/rust-test-suite-manifest.mjs --generate`
로 먼저 만든 뒤 게이트를 돌렸다. 이 파일들은 PR 에 포함하지 않는다.

## PR 메모

`gh pr create --base devel --body-file` · `closes #5763`
