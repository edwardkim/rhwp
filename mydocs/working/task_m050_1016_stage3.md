# Task M050 #1016 Stage 3 구현 완료보고서

## 1. 요약

`PaintOp::Image` 생성 단계에서 image visual payload를 resolve하도록 공통 경로를 추가했다.

이번 구현은 원본 `ImageNode` / parser / document IR을 변경하지 않고, `PageLayerTree`로 낮춰지는 paint op에만 resolved payload를 싣는다. 따라서 원본 문서 속성은 유지하면서 native Skia, browser CanvasKit, Studio overlay, transition renderer가 같은 baked watermark bytes를 사용할 수 있다.

## 2. 구현 내용

### 2.1 공통 resolver 추가

추가 파일:

```text
src/renderer/image_resolver.rs
```

주요 API:

```rust
resolve_image_payload(image: &ImageNode) -> Option<ResolvedImagePayload>
image_node_with_resolved_payload(image, resolved) -> ImageNode
```

처리 범위:

| 입력 | resolved 결과 |
|------|---------------|
| BMP | PNG 변환 payload |
| PCX | PNG 변환 payload |
| 워터마크성 JPEG | 한컴 참고 톤 baked PNG payload |
| 기타 이미지 | `None`, 원본 사용 |

워터마크 JPEG 조건은 #976과 동일하게 유지했다.

```text
mime == image/jpeg
effect != RealPic
brightness != 0 || contrast != 0
near-white border / pixel ratio gate 통과
```

### 2.2 `PaintOp::Image` resolved payload 추가

변경 파일:

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

`LayerBuilder`가 `RenderNodeType::Image`를 `PaintOp::Image`로 낮출 때 resolver를 호출하도록 연결했다.

### 2.3 PageLayerTree JSON 반영

변경 파일:

```text
src/paint/json.rs
src/paint/schema.rs
```

동작:

- `resolved`가 있으면 JSON `mime` / `base64`는 resolved payload를 우선 사용한다.
- `ResolvedImageKind::BakedWatermark`이면 `bakedWatermark:true`를 출력한다.
- 원본 `effect`, `brightness`, `contrast`는 진단/문서 속성 의미로 유지한다.
- schema minor version을 `12 -> 13`으로 올렸다.

### 2.4 renderer 연결

변경 파일:

```text
src/renderer/skia/renderer.rs
src/renderer/canvas.rs
src/renderer/web_canvas.rs
src/renderer/svg_layer.rs
src/renderer/svg.rs
```

동작:

- native Skia는 `resolved` bytes를 우선 decode한다.
- `resolved.suppress_effects == true`이면 Skia draw 경로에서 `ImageEffect::RealPic`으로 처리해 이중 보정을 막는다.
- WebCanvas / SVG layer transition path는 임시 `ImageNode` clone에 resolved bytes를 반영한다.
- legacy SVG helper 구현은 `image_resolver.rs`로 이동하고, `svg.rs`는 기존 public-in-crate helper 이름을 re-export해 기존 호출부와 테스트 호환성을 유지한다.

### 2.5 Studio 타입/overlay 연결

변경 파일:

```text
src/document_core/queries/rendering.rs
rhwp-studio/src/core/types.ts
```

동작:

- overlay JSON은 더 이상 JPEG 워터마크를 별도로 재판정해 bake하지 않고 `PaintOp::Image.resolved`를 사용한다.
- resolved baked watermark이면 `bakedWatermark:true`를 유지한다.
- `LayerImageOp` 타입에 `bakedWatermark?: boolean`을 추가했다.

## 3. 테스트 추가

변경 파일:

```text
tests/issue_938.rs
```

추가 테스트:

```text
issue_938_layer_tree_watermark_is_resolved_hancom_baked_png
```

검증 내용:

- `samples/복학원서.hwp` 1페이지 PageLayerTree JSON의 watermark image op가 `mime=image/png`인지 확인
- `bakedWatermark=true` 확인
- 같은 watermark op가 `mime=image/jpeg`로 노출되지 않음을 확인
- baked PNG의 크기, alpha, gray tone 통계가 #976 SVG/overlay 기준과 같은 범위인지 확인

## 4. 구현 중 수행한 sanity 검증

아래 검증은 Stage 3 구현 중 컴파일 오류와 직접 회귀를 막기 위해 수행했다. Stage 4에서는 이 결과를 기준으로 필요한 추가 검증과 최종 기록을 진행한다.

```text
cargo check
cargo fmt --check
cargo test --test issue_938
cargo test --test issue_516
cargo test --test issue_514
cargo test --test svg_snapshot
npm --prefix rhwp-studio test
cargo check --features native-skia
cargo test --features native-skia --no-run
```

결과:

| 명령 | 결과 | 비고 |
|------|------|------|
| `cargo check` | 통과 | 기본 feature |
| `cargo fmt --check` | 통과 | `cargo fmt` 적용 후 재확인 |
| `cargo test --test issue_938` | 통과 | 3 passed |
| `cargo test --test issue_516` | 통과 | 8 passed |
| `cargo test --test issue_514` | 통과 | 3 passed |
| `cargo test --test svg_snapshot` | 통과 | 8 passed |
| `npm --prefix rhwp-studio test` | 통과 | 25 passed |
| `cargo check --features native-skia` | 통과 | 최초 sandbox DNS 실패 후 승인된 네트워크 실행으로 확인 |
| `cargo test --features native-skia --no-run` | 통과 | 기존 warning 6건만 존재 |

`npm --prefix rhwp-studio run build`는 이 worktree에 `node_modules`가 없어 `tsc: command not found`로 실행되지 않았다. 이는 코드 실패가 아니라 로컬 의존성 설치 부재로 분류한다.

## 5. 남은 Stage 4 검증 후보

- PageLayerTree JSON payload를 별도 산출물로 남겨 expected field를 수동 확인한다.
- 가능하면 native Skia PNG export 산출물을 만들어 #976 기준 SVG/overlay와 시각 비교한다.
- browser CanvasKit은 PageLayerTree JSON의 resolved `base64`를 그대로 consume하는 경로라, Studio build 환경이 준비되면 type/build 검증까지 수행한다.
- 최종 보고서에서 `npm run build` 미수행 사유와 native-skia feature 검증 결과를 명시한다.

## 6. 다음 단계

작업지시자 승인 후 Stage 4 검증 보고서를 작성한다. 이슈 close는 최종 보고 및 별도 승인 전에는 수행하지 않는다.
