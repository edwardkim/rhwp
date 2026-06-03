# Task M050 #1016 Stage 1 완료 보고서 — image visual payload 경로 진단

## 1. 목적

#1016의 구현 전 단계로, #976의 JPEG 워터마크 baked PNG 처리가 현재 어느 경로에 적용되어 있고 `PageLayerTree` / `PaintOp::Image` / native Skia / browser CanvasKit 경로에는 어떤 의미로 전달되는지 확인했다.

이번 단계에서는 소스 구현을 하지 않았다.

## 2. 작업 환경

```text
branch: local/task1016
worktree: /private/tmp/rhwp-task1016
base: upstream/devel 9190dea8
target: samples/복학원서.hwp page 1
```

`GIT_LFS_SKIP_SMUDGE=1`로 worktree를 생성했기 때문에 `pdf-large/` 일부 파일은 LFS pointer 상태다. 이번 Stage 1 대상인 `samples/복학원서.hwp`와 `pdf/복학원서-2022.pdf`는 정상 파일로 확인했다.

## 3. 현재 경로 요약

### 3.1 legacy SVG 경로

`src/renderer/svg.rs`의 `render_image_node()`는 이미지별로 직접 MIME을 판정하고 워터마크 JPEG이면 PNG로 bake한다.

```text
src/renderer/svg.rs:1259-1289
```

현재 조건:

```text
effect != RealPic
brightness != 0 || contrast != 0
mime == image/jpeg
watermark_jpeg_bytes_to_hancom_baked_png_bytes(data) 성공
```

baked 성공 시:

```text
render_mime = image/png
baked_watermark = true
effect filter 생략
brightness/contrast filter 생략
opacity=0.17 생략
```

핵심 bake helper는 아직 SVG renderer 파일에 있다.

```text
src/renderer/svg.rs:2898-2980
```

### 3.2 WebCanvas legacy render tree 경로

`src/renderer/web_canvas.rs`의 `RenderNodeType::Image` 처리도 SVG helper를 직접 호출한다.

```text
src/renderer/web_canvas.rs:561-599
```

baked 성공 시 `ctx.filter`와 `global_alpha=0.17`을 생략한다. 즉 WebCanvas는 현재 `PageLayerTree`의 `PaintOp::Image` 의미를 읽어서 처리하는 것이 아니라, render tree로 되돌린 `ImageNode`에서 다시 판정한다.

### 3.3 Studio overlay JSON 경로

`DocumentCore::get_page_overlay_images_native()` 안의 `write_overlay_image()`도 별도로 MIME 판정과 bake를 수행한다.

```text
src/document_core/queries/rendering.rs:467-540
```

이 경로는 baked 성공 시 overlay JSON에 다음을 싣는다.

```json
"mime": "image/png",
"bakedWatermark": true
```

Studio overlay renderer는 `bakedWatermark`가 true이면 CSS filter, mix-blend-mode, opacity를 생략한다.

```text
rhwp-studio/src/view/page-renderer.ts:230-250
```

### 3.4 PageLayerTree JSON 경로

`PaintOp::Image` JSON 직렬화는 PCX/BMP 변환만 수행하고 JPEG 워터마크 bake는 하지 않는다.

```text
src/paint/json.rs:623-695
```

현재 이 경로는 다음은 출력한다.

```text
mime
base64
effect
brightness
contrast
watermark.preset
wrap
```

하지만 다음은 출력하지 않는다.

```text
bakedWatermark
resolved PNG payload
후처리 생략 의미
```

### 3.5 PaintOp 생성 경로

`LayerBuilder`는 `RenderNodeType::Image`를 그대로 clone해서 `PaintOp::Image`로 낮춘다.

```text
src/paint/builder.rs:86-89
```

`ImageNode`는 원본 `data`, `effect`, `brightness`, `contrast`, `text_wrap` 등을 갖고 있지만, resolved bytes / resolved MIME / baked 여부를 표현하는 필드는 없다.

```text
src/renderer/render_tree.rs:711-748
```

즉 현재는 `PaintOp::Image` 생성 단계에서 visual image payload를 resolve하지 않는다.

### 3.6 native Skia 경로

native Skia renderer는 `PaintOp::Image.image.data`를 그대로 `draw_image()`에 전달한다.

```text
src/renderer/skia/renderer.rs:776-788
```

`draw_image_bytes()`는 `ImageEffect`만 color filter로 처리한다.

```text
src/renderer/skia/image_conv.rs:70-101
```

brightness/contrast와 #976 baked watermark 의미는 현재 native Skia image replay 경로의 입력 계약에 없다. 따라서 PageLayerTree가 원본 JPEG payload를 들고 있으면 native Skia도 원본 JPEG를 받는다.

### 3.7 browser CanvasKit 경로

browser CanvasKit renderer는 `LayerImageOp.base64`를 decode해서 `MakeImageFromEncoded()`로 그린다.

```text
rhwp-studio/src/view/canvaskit-renderer.ts:369-386
```

TypeScript `LayerImageOp`에는 `mime`, `base64`, `effect`, `brightness`, `contrast`, `wrap`까지만 있고 `bakedWatermark`가 없다.

```text
rhwp-studio/src/core/types.ts:865-875
```

즉 CanvasKit은 현재 PageLayerTree JSON에 실린 원본 image payload를 그대로 받는다.

## 4. 실행 검증

### 4.1 PageLayerTree image op MIME 분포

명령:

```bash
cargo test --test issue_516 issue_516_diag_count_image_ops -- --nocapture
```

결과:

```text
image ops: 2, wrap=behindText: 2, mime png: 1, mime jpg: 1
test issue_516_diag_count_image_ops ... ok
```

해석:

- 페이지 1에는 image op가 2개 있다.
- 학교 로고는 PNG로 노출된다.
- 중앙 워터마크는 PageLayerTree JSON에서 아직 JPEG로 노출된다.
- 두 그림 모두 `wrap=behindText`다.

### 4.2 PageLayerTree image op 위치

명령:

```bash
cargo test --test issue_516 issue_516_diag_image_op_locations -- --nocapture
```

결과 핵심:

```text
image op #0 ... "mime":"image/png"
image op #1 ... "mime":"image/jpeg"
```

두 번째 image op의 bbox는 중앙 워터마크 위치다.

```text
x=137.707, y=270.240, width=495.040, height=495.733
```

### 4.3 legacy SVG baked PNG 유지

명령:

```bash
cargo test --test issue_938 issue_938_svg_watermark_is_hancom_baked_png -- --nocapture
```

결과:

```text
test issue_938_svg_watermark_is_hancom_baked_png ... ok
```

해석:

- legacy SVG 경로는 #976 이후 중앙 워터마크를 baked PNG로 emit한다.
- 이 경로는 현재 정상 회귀 guard가 있다.

### 4.4 Studio overlay baked PNG 유지

명령:

```bash
cargo test --test issue_938 issue_938_overlay_watermark_is_hancom_baked_png -- --nocapture
```

결과:

```text
test issue_938_overlay_watermark_is_hancom_baked_png ... ok
```

해석:

- Studio overlay JSON은 현재 `mime=image/png`, `bakedWatermark=true`를 갖는다.
- PageLayerTree JSON과 overlay JSON의 image payload 의미가 다르다.

## 5. 진단 결론

현재 #976 처리는 renderer/consumer별 후단에 흩어져 있다.

| 경로 | 현재 워터마크 bake 여부 | 비고 |
|------|--------------------------|------|
| legacy SVG | 적용됨 | `src/renderer/svg.rs` 내부 판정 |
| WebCanvas legacy/render tree | 적용됨 | SVG helper 직접 호출 |
| Studio overlay JSON | 적용됨 | `get_page_overlay_images_native()` 안에서 재판정 |
| PageLayerTree JSON | 미적용 | JPEG 원본 payload 유지 |
| native Skia | 미적용 | `PaintOp::Image.image.data` 그대로 draw |
| browser CanvasKit | 미적용 | PageLayerTree JSON base64 그대로 decode |

따라서 #1016의 핵심 문제는 native Skia만의 누락이 아니라, `PaintOp::Image`가 아직 “resolved visual image payload”를 표현하지 않는다는 점이다.

## 6. Stage 2 구현 후보

구현 계획서에서 다음 후보를 비교해야 한다.

### 후보 A — `ImageNode`에 resolved payload 필드 추가

- 장점: 기존 `PaintOp::Image { image: ImageNode }` 구조를 크게 바꾸지 않는다.
- 장점: Skia, CanvasKit JSON, overlay JSON이 같은 `ImageNode` 필드를 읽기 쉽다.
- 위험: `ImageNode`가 render tree 의미와 layer tree visual projection 의미를 동시에 갖게 된다.

### 후보 B — `PaintOp::Image` payload를 확장

- 장점: resolved image는 명확히 paint/layer 단계 의미가 된다.
- 장점: `Document` IR과 `PageRenderTree` 원본 의미를 건드리지 않는다.
- 위험: `PaintOp::Image` enum과 JSON 직렬화, renderer match arm의 변경 범위가 커진다.

### 후보 C — 공통 resolver 함수만 만들고 각 경로에서 호출

- 장점: 가장 작은 변경으로 중복 helper 위치를 정리할 수 있다.
- 위험: #1016의 본질인 “PaintOp 생성 단계 resolved payload 일반화”에는 부족하다.
- 위험: renderer별 재판정 구조가 남는다.

Stage 1 기준 추천은 후보 B를 중심으로 하되, 구현 부담이 과하면 후보 A와 절충하는 방식이다. 단, 후보 C 단독은 #992 수준의 renderer별 patch에 가깝기 때문에 권장하지 않는다.

## 7. Stage 2에서 확정할 사항

1. 공통 resolver 모듈 위치
   - 후보: `src/renderer/image_resolver.rs`, `src/paint/image_resolver.rs`, `src/paint/image_payload.rs`
2. resolved payload 필드 구조
   - 최소 필드: `bytes`, `mime`, `baked_watermark`
   - 검토 필드: 원본 `effect/brightness/contrast` 유지 여부, 후처리 정책 enum
3. JSON schema minor bump 여부
   - 현재 `schemaMinorVersion=12`
   - additive 필드라도 `bakedWatermark` / resolved payload 의미 추가면 bump가 합리적이다.
4. 후처리 생략 계약
   - baked watermark는 renderer가 effect/brightness/contrast/opacity/mix-blend를 중복 적용하지 않아야 한다.
5. legacy SVG 연결 여부
   - 기존 테스트를 유지하면서 공통 resolver로 옮길 수 있으면 같은 작업에 포함한다.
   - 구조 위험이 크면 legacy SVG는 기존 동작 유지 후 후속 이슈로 분리한다.

## 8. 검증 상태

통과:

```text
cargo test --test issue_516 issue_516_diag_count_image_ops -- --nocapture
cargo test --test issue_516 issue_516_diag_image_op_locations -- --nocapture
cargo test --test issue_938 issue_938_svg_watermark_is_hancom_baked_png -- --nocapture
cargo test --test issue_938 issue_938_overlay_watermark_is_hancom_baked_png -- --nocapture
```

비고:

- 첫 `cargo test` 실행은 sandbox DNS 제한으로 실패했고, 사용자 승인 경로로 네트워크 허용 후 의존성 다운로드와 테스트를 완료했다.
- native Skia feature 빌드/렌더는 Stage 1에서 실행하지 않았다. 현재 결론은 code path와 PageLayerTree JSON 진단에 근거한다.

## 9. 다음 단계

작업지시자 승인 후 `mydocs/plans/task_m050_1016_impl.md` 구현 계획서를 작성한다. 구현 계획서에서는 resolved payload 구조와 변경 파일 범위, 테스트 목록을 확정한다.
