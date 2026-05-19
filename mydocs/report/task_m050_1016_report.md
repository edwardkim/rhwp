# Task M050 #1016 최종 보고서

## 1. 요약

#1016은 #976의 워터마크 JPEG baked PNG 처리를 renderer별 개별 패치가 아니라 `PaintOp::Image` 생성 단계의 resolved visual payload로 일반화하는 작업이다.

최종 구현은 원본 `Document` IR / parser / `PageRenderTree` 의미를 바꾸지 않고, `PageLayerTree`로 낮춰지는 paint op에만 resolved payload를 싣는다. 이를 통해 PageLayerTree JSON, native Skia, browser CanvasKit direct renderer, Studio overlay가 같은 resolved image bytes와 `bakedWatermark` 의미를 공유할 수 있게 했다.

작업 중 native Skia PNG 산출물의 시각 판정에서 `BehindText` z-order 문제가 별도로 드러났다. 이 문제는 image payload가 아니라 PageLayerTree replay compositor 정책 문제이므로 #1017로 분리했다.

## 2. 이슈 / 브랜치

| 항목 | 값 |
|------|----|
| Issue | #1016 — `PaintOp::Image` 생성 단계에서 워터마크 JPEG baked image를 resolved visual payload로 일반화 |
| 후속 Issue | #1017 — `PageLayerTree replay에서 BehindText/InFrontOfText z-order 합성 정책 일반화` |
| Supersedes | #992 |
| Follows | #976 |
| Refs | #938 |
| Branch | `local/task1016` |
| Worktree | `/private/tmp/rhwp-task1016` |
| Base | `upstream/devel` `9190dea8` |

## 3. 구현 결과

### 3.1 공통 resolver

추가:

```text
src/renderer/image_resolver.rs
```

제공 기능:

- BMP → PNG resolved payload
- PCX → PNG resolved payload
- 워터마크성 JPEG → 한컴 참고 톤 baked PNG resolved payload
- resolved payload를 임시 `ImageNode`에 반영하는 bridge helper

워터마크 JPEG 판정은 #976과 동일한 좁은 조건을 유지했다.

```text
mime == image/jpeg
effect != RealPic
brightness != 0 || contrast != 0
near-white border / pixel ratio gate 통과
```

### 3.2 `PaintOp::Image` 확장

변경:

```text
src/paint/paint_op.rs
src/paint/mod.rs
src/paint/builder.rs
```

추가 타입:

```rust
ResolvedImageKind::{FormatConverted, BakedWatermark}
ResolvedImagePayload { data, mime, kind, suppress_effects }
```

`LayerBuilder`는 `RenderNodeType::Image`를 `PaintOp::Image`로 낮출 때 resolver를 호출한다.

### 3.3 PageLayerTree JSON

변경:

```text
src/paint/json.rs
src/paint/schema.rs
```

동작:

- `resolved`가 있으면 JSON `mime` / `base64`는 resolved payload를 우선 사용한다.
- baked watermark이면 `bakedWatermark:true`를 출력한다.
- 원본 `effect`, `brightness`, `contrast`, `watermark` metadata는 유지한다.
- schema minor version은 `12 -> 13`으로 올렸다.

### 3.4 renderer 연결

변경:

```text
src/renderer/skia/renderer.rs
src/renderer/canvas.rs
src/renderer/web_canvas.rs
src/renderer/svg_layer.rs
src/renderer/svg.rs
src/renderer/mod.rs
```

동작:

- native Skia는 `PaintOp::Image.resolved` bytes를 우선 decode한다.
- `resolved.suppress_effects`가 true이면 native Skia image draw에서 `ImageEffect::RealPic`으로 처리해 이중 보정을 막는다.
- WebCanvas / SVG layer transition path는 임시 `ImageNode` clone에 resolved bytes를 반영한다.
- legacy SVG helper 구현은 `image_resolver.rs`로 이동하고, 기존 `svg.rs` helper 이름은 re-export로 유지했다.

### 3.5 Studio 타입 / overlay

변경:

```text
src/document_core/queries/rendering.rs
rhwp-studio/src/core/types.ts
```

동작:

- overlay JSON은 JPEG 워터마크를 별도 재판정하지 않고 `PaintOp::Image.resolved`를 사용한다.
- resolved baked watermark이면 `bakedWatermark:true`를 유지한다.
- `LayerImageOp` 타입에 `bakedWatermark?: boolean`을 추가했다.

## 4. 테스트 / 검증

추가 테스트:

```text
tests/issue_938.rs::issue_938_layer_tree_watermark_is_resolved_hancom_baked_png
```

확인한 핵심 계약:

- `samples/복학원서.hwp` 1페이지 중앙 워터마크 image op가 `mime=image/png`
- `bakedWatermark=true`
- watermark image op가 더 이상 `mime=image/jpeg`로 노출되지 않음
- baked PNG 크기 / alpha / gray tone 통계가 #976 SVG/overlay 기준 범위

실행 결과:

| 명령 | 결과 |
|------|------|
| `cargo check` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_938` | 통과, 3 passed |
| `cargo test --test issue_516` | 통과, 8 passed |
| `cargo test --test issue_514` | 통과, 3 passed |
| `cargo test --test svg_snapshot` | 통과, 8 passed |
| `npm --prefix rhwp-studio test` | 통과, 25 passed |
| `cargo check --features native-skia` | 통과 |
| `cargo test --features native-skia --no-run` | 통과 |
| `cargo test --features native-skia skia --lib` | 통과, 30 passed |
| `cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1016-stage4` | PNG 생성 성공 |

PageLayerTree 진단:

```text
image ops: 2, wrap=behindText: 2, mime png: 2, mime jpg: 0
```

native Skia PNG 산출:

```text
output/task1016-stage4/복학원서.png
PNG image data, 794 x 1123, 8-bit/color RGBA, non-interlaced
size: 175080 bytes
```

## 5. 제한 사항 / scope correction

작업 중 `output/task1016-stage4/복학원서.png`를 확인하면서, native Skia PNG export에서 중앙 워터마크가 글 내용 위에 보이고 opaque 흰 사각 영역도 노출되는 문제가 드러났다.

원인은 #1016 구현 실패가 아니라 범위 차이다.

- #976의 baked watermark PNG는 투명 PNG가 아니라 opaque PNG이다.
- 사각형 제거 효과는 `background -> behindText image -> flow text` 순서로 합성될 때 성립한다.
- Studio Canvas2D 기본 경로는 #516의 HTML Hybrid / overlay 방식으로 이 순서를 구현한다.
- native Skia와 browser CanvasKit direct renderer는 현재 PageLayerTree leaf 순서를 그대로 replay하므로 `wrap=behindText` z-order 의미를 별도로 재구성하지 않는다.

따라서 #1016의 완료 조건은 "renderer들이 같은 resolved image payload를 공유한다"는 contract로 좁혔다. `BehindText` / `InFrontOfText` replay z-order 일반화는 #1017로 분리했다.

GitHub 반영:

- #1016 본문 완료 조건 조정
- #1016 scope update comment 추가
- #1017 신규 이슈 생성

관련 근거:

- Discussion #529: 단기 HTML Hybrid, 장기 WebGPU 결정
- #516: `LayerFilter`, `renderPageToCanvasFiltered("flow"/"behind"/"front"/"all")`, Studio overlay 인프라
- #536: CanvasKit direct replay / no Canvas2D overlay fallback 방향
- #364 / #456 / #498: PageLayerTree generation, Canvas replay 전환, visual diff pipeline을 작은 contract 단위로 분리한 흐름

## 6. 미수행 / 환경 제한

`npm --prefix rhwp-studio run build`는 이 worktree에 `node_modules`가 없어 실행하지 못했다.

```text
tsc: command not found
```

`npm --prefix rhwp-studio test`는 통과했으므로, TypeScript production build는 의존성 설치가 있는 환경에서 추가 확인이 필요하다.

native Skia 관련 일부 명령은 rust-skia binary / crates.io 다운로드가 필요해 sandbox network에서 실패했고, 승인된 네트워크 실행으로 검증했다.

## 7. 결론

#1016의 핵심 목표인 `PaintOp::Image` 생성 단계 resolved baked watermark 일반화는 구현과 테스트를 완료했다.

이 작업으로 PageLayerTree JSON과 renderer 입력 payload에서 중앙 워터마크가 더 이상 원본 JPEG로 노출되지 않고, resolved PNG + `bakedWatermark` 의미를 공유한다.

다만 native Skia / browser CanvasKit direct renderer의 최종 시각 정합은 #1017의 `BehindText` / `InFrontOfText` z-order replay policy가 필요하다.

이슈 close는 작업지시자 승인 후에만 수행한다.
