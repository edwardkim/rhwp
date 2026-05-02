---
name: 시각 결함 정정 시 모든 renderer 경로 점검 필수
description: rhwp 의 image MIME 감지 / 변환 / paint 적용 코드는 renderer 마다 별도 사본을 사용. 한 곳만 정정하면 다른 경로 누락. 시각 결함 정정 시 모든 경로 sweep 필요.
type: feedback
originSessionId: b2ba0b26-0926-49a2-a3fb-6ff2062a8527
---
rhwp 의 image 처리 (MIME 감지 / 변환 / paint 적용) 코드는 **renderer 마다 별도 사본** 으로 정의되어 있다:

- CLI SVG: `src/renderer/svg.rs::detect_image_mime_type` (`pub(crate)`)
- Web Canvas: `src/renderer/web_canvas.rs::detect_image_mime_type` (`#[cfg(target_arch = "wasm32")]` 가드된 별도 fn)
- PageLayerTree JSON: `src/paint/json.rs::PaintOp::Image` 직렬화 (effect/brightness/contrast/wrap/mime/watermark)
- Web SVG / HTML 등: 별도 분기 가능성

**Why:** Task #514 (PCX → PNG 변환) 와 Task #516 (Web Canvas 워터마크 효과) 모두 동일 패턴 — Stage 3 에서 `svg.rs` 만 정정 후 작업지시자 시각 판정에서 Web Canvas 결함 발견. Stage 5 (Task #514) / Stage 5.2 (Task #516) 에서 web_canvas.rs 추가 정정 필요. 정정 누락은 작업지시자 시각 판정 게이트에서 잡혔지만 두 사이클 낭비.

**How to apply:**
- 시각 결함 정정 작업 시작 전 모든 renderer 경로 sweep — `grep -rn "detect_image_mime_type\|render_image\|PaintOp::Image" src/renderer src/paint`
- CLI SVG / Web Canvas / PageLayerTree JSON / Web SVG / HTML 모두 점검
- 새 변환 함수 추가 시 모든 호출 사이트에 분기 적용 — Task #514 의 `pcx_bytes_to_png_bytes` 가 svg.rs (2 사이트) + web_canvas.rs (1 사이트) + paint/json.rs (1 사이트) 모두 적용된 경위 정합
- ImageNode 같은 IR 노드 확장 시 ImageNode 생성 사이트 (paragraph_layout / picture_footnote / shape_layout / table_cell_content 등 8+) 모두 전파
- 작업지시자 시각 판정 게이트는 누락 정정의 마지막 방어선이지만, 사전 sweep 으로 사이클 낭비 줄이는 게 정합
