# Task M050 #1016 구현 계획서 — PaintOp::Image resolved baked watermark 일반화

## 1. 목적

#976에서 renderer별로 흩어진 JPEG 워터마크 baked PNG 처리를 `PaintOp::Image` 생성 단계의 visual payload로 일반화한다.

이번 구현의 핵심은 원본 `Document` IR이나 parser 결과를 바꾸지 않고, `PageRenderTree`에서 `PageLayerTree`로 낮추는 paint 단계에서만 resolved image payload를 싣는 것이다. 이렇게 하면 native Skia, browser CanvasKit, Studio overlay, PageLayerTree JSON이 같은 image bytes와 같은 후처리 생략 의미를 공유할 수 있다.

## 2. Stage 1 결론

Stage 1 보고서:

```text
mydocs/working/task_m050_1016_stage1.md
```

확인된 상태:

| 경로 | 현재 상태 |
|------|-----------|
| legacy SVG | 워터마크 JPEG를 baked PNG로 emit |
| WebCanvas legacy/render tree | SVG helper를 직접 호출해 baked PNG 사용 |
| Studio overlay JSON | `mime=image/png`, `bakedWatermark=true` |
| PageLayerTree JSON | 중앙 워터마크가 아직 `mime=image/jpeg` |
| native Skia | `PaintOp::Image.image.data` 원본 JPEG를 그대로 draw |
| browser CanvasKit | PageLayerTree JSON의 원본 JPEG base64를 그대로 decode |

진단 테스트 결과:

```text
image ops: 2, wrap=behindText: 2, mime png: 1, mime jpg: 1
```

따라서 문제는 native Skia 단독 누락이 아니라 `PaintOp::Image`가 resolved visual payload를 표현하지 않는 구조에 있다.

## 3. 설계 결정

### 3.1 선택안

`PaintOp::Image`에 paint 단계 전용 resolved payload를 추가한다.

예상 형태:

```rust
pub enum PaintOp {
    Image {
        bbox: BoundingBox,
        image: ImageNode,
        resolved: Option<Box<ResolvedImagePayload>>,
    },
    // ...
}
```

예상 payload:

```rust
pub struct ResolvedImagePayload {
    pub data: Vec<u8>,
    pub mime: &'static str,
    pub kind: ResolvedImageKind,
    pub suppress_effects: bool,
}

pub enum ResolvedImageKind {
    FormatConverted,
    BakedWatermark,
}
```

`resolved=None`은 원본 `ImageNode.data`를 그대로 사용한다는 의미다.

### 3.2 선택 이유

- `Document` / parser / `PageRenderTree` 원본 의미를 바꾸지 않는다.
- resolved image는 `PageLayerTree` replay 단계의 visual projection임을 타입상 분리한다.
- native Skia와 browser CanvasKit이 renderer 내부에서 워터마크 여부를 다시 추론하지 않는다.
- Studio overlay JSON도 `PaintOp::Image`의 resolved payload만 읽으면 된다.
- `ImageNode`에 resolved 필드를 추가하는 방식보다 semantic render tree와 paint layer의 책임이 덜 섞인다.

### 3.3 비선택안

`ImageNode`에 resolved 필드를 추가하는 방식은 변경량이 작지만, `ImageNode`가 render tree 의미와 paint visual projection 의미를 동시에 갖게 된다. 이번 이슈의 목표가 `PaintOp::Image` 생성 단계 일반화이므로 우선순위를 낮춘다.

공통 resolver 함수만 만들고 각 renderer에서 계속 호출하는 방식은 #992의 renderer별 패치와 본질적으로 비슷해 제외한다.

## 4. 구현 범위

### 4.1 공통 image resolver 추가

새 모듈 후보:

```text
src/renderer/image_resolver.rs
```

책임:

- MIME 판정
- BMP → PNG 변환
- PCX → PNG 변환
- 워터마크성 JPEG → 한컴 참고 톤 baked PNG 변환
- resolved payload 생성

기존 `src/renderer/svg.rs`의 다음 helper를 이동하거나 wrapper로 전환한다.

```text
detect_image_mime_type()
bmp_bytes_to_png_bytes()
pcx_bytes_to_png_bytes()
watermark_jpeg_bytes_to_hancom_baked_png_bytes()
```

공개 API 후보:

```rust
pub(crate) fn resolve_image_payload(image: &ImageNode) -> Option<ResolvedImagePayload>;
pub(crate) fn detect_image_mime_type(data: &[u8]) -> &'static str;
```

워터마크 bake 조건은 #976과 동일하게 유지한다.

```text
mime == image/jpeg
image.effect != RealPic
image.brightness != 0 || image.contrast != 0
near-white border / pixel ratio gate 통과
```

### 4.2 PaintOp 확장

변경 파일:

```text
src/paint/paint_op.rs
src/paint/builder.rs
```

작업:

- `ResolvedImagePayload`, `ResolvedImageKind` 정의
- `PaintOp::Image`에 `resolved` 필드 추가
- `LayerBuilder`의 `RenderNodeType::Image` 처리에서 `resolve_image_payload(image)` 호출
- 기존 `PaintOp::Image` match arm은 대부분 `..` 패턴으로 갱신
- 테스트용 직접 생성 코드는 `resolved: None` 명시

### 4.3 PageLayerTree JSON 정리

변경 파일:

```text
src/paint/json.rs
src/paint/schema.rs
```

작업:

- image op JSON의 `mime` / `base64`는 `resolved`가 있으면 resolved payload를 우선 사용
- `ResolvedImageKind::BakedWatermark`이면 `bakedWatermark:true` 출력
- `resolved.suppress_effects == true`이면 JSON에는 원본 `effect`, `brightness`, `contrast`를 유지하되, 소비자는 `bakedWatermark`로 후처리 생략 가능
- `schemaMinorVersion`을 `12 → 13`으로 올리고 schema contract test 갱신

원본 `effect/brightness/contrast`를 유지하는 이유:

- 문서 속성/진단 정보로는 여전히 워터마크 속성이 필요하다.
- 후처리 생략 여부는 `bakedWatermark` / resolved payload kind가 담당한다.

### 4.4 Studio overlay JSON 정리

변경 파일:

```text
src/document_core/queries/rendering.rs
rhwp-studio/src/view/page-renderer.ts
```

작업:

- `get_page_overlay_images_native()`의 `write_overlay_image()`가 별도 bake 판정을 하지 않도록 변경
- `PaintOp::Image { image, resolved, .. }`에서 resolved payload를 읽어 `mime`, `base64`, `bakedWatermark`를 작성
- `page-renderer.ts`의 기존 `bakedWatermark` 처리 로직은 유지

기대 결과:

```text
overlay JSON과 PageLayerTree JSON이 같은 resolved image payload를 사용
```

### 4.5 native Skia 경로 연결

변경 파일:

```text
src/renderer/skia/renderer.rs
```

작업:

- `PaintOp::Image` replay 시 `resolved`가 있으면 resolved bytes를 우선 사용
- `resolved.suppress_effects`가 true이면 `ImageEffect::RealPic`으로 draw
- 일반 PCX/BMP 변환 payload는 원본 `image.effect`를 유지

주의:

- native Skia image path는 현재 brightness/contrast를 적용하지 않는다.
- 이번 구현은 baked watermark에 이중 grayscale filter가 적용되지 않는 것을 우선 보장한다.

### 4.6 WebCanvas / SVG layer renderer 연결

변경 파일:

```text
src/renderer/web_canvas.rs
src/renderer/svg_layer.rs
```

작업:

- layer tree를 다시 `RenderNodeType::Image`로 조립하는 경로에서 `resolved` payload를 잃지 않도록 처리
- `resolved`가 있으면 임시 `ImageNode` clone의 `data`를 resolved bytes로 대체
- `suppress_effects`가 true이면 임시 clone의 `effect=RealPic`, `brightness=0`, `contrast=0`으로 후처리 중복을 막음

이 처리는 transition renderer의 bridge 성격 때문에 필요하다. 장기적으로는 SVG/WebCanvas도 `PaintOp::Image`를 직접 replay하는 방향이 더 좋지만, 이번 이슈의 범위는 기존 구조 안에서 의미 손실을 막는 것으로 제한한다.

### 4.7 legacy SVG 경로 정리

변경 파일:

```text
src/renderer/svg.rs
```

작업:

- 기존 #976 동작은 유지
- helper 구현은 `image_resolver.rs`로 이동하고 `svg.rs`는 공통 resolver/helper를 호출
- `render_image_node()`의 기존 wrapper 구조는 큰 폭으로 바꾸지 않는다

범위가 커지면 legacy SVG의 완전 통합은 후속 이슈로 분리한다. 단, helper의 원본 위치를 `svg.rs`에서 제거하는 것은 이번 이슈에 포함한다.

### 4.8 browser CanvasKit 타입 갱신

변경 파일:

```text
rhwp-studio/src/core/types.ts
```

작업:

- `LayerImageOp`에 `bakedWatermark?: boolean` 추가
- CanvasKit renderer는 기본적으로 PageLayerTree JSON의 `base64`를 그리므로 별도 bake 로직을 추가하지 않는다.

## 5. 테스트 계획

### 5.1 Rust 회귀 테스트

추가/갱신 후보:

```text
tests/issue_938.rs
tests/issue_516.rs
tests/issue_514.rs
src/paint/json.rs tests
src/paint/schema.rs tests
```

필수 신규 테스트:

1. PageLayerTree JSON의 중앙 워터마크가 `mime=image/png`, `bakedWatermark=true`인지 확인
2. PageLayerTree JSON에 중앙 워터마크 `image/jpeg`가 남지 않는지 확인
3. `bakedWatermark` PNG tone stats가 기존 overlay/SVG 기준과 같은지 확인
4. 일반 학교 로고 PNG에는 `bakedWatermark`가 붙지 않는지 확인

기존 테스트 기대 변화:

```text
issue_516_diag_count_image_ops:
기존: mime png: 1, mime jpg: 1
변경 후 예상: mime png: 2, mime jpg: 0
```

### 5.2 native Skia 검증

가능하면 feature-gated 테스트를 추가한다.

```text
cargo test --release --features native-skia --test issue_938 -- --nocapture
```

또는 최소 검증:

```text
cargo build --release --features native-skia
target/release/rhwp export-png samples/복학원서.hwp -p 0 -o output/poc/task1016/
```

단, native Skia 빌드는 skia-safe 의존성 때문에 시간이 길 수 있으므로 Stage 4에서 실행 가능성을 다시 판단한다.

### 5.3 Studio / CanvasKit 검증

명령 후보:

```text
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

CanvasKit renderer는 PageLayerTree JSON을 그대로 소비하므로 TypeScript type check와 build 통과를 확인한다.

### 5.4 전체 후보 명령

```text
cargo fmt --check
cargo test --test issue_938
cargo test --test issue_514
cargo test --test issue_516
cargo test --test svg_snapshot
cargo test --lib
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

## 6. 예상 변경 파일

Rust:

```text
src/renderer/mod.rs
src/renderer/image_resolver.rs
src/renderer/svg.rs
src/renderer/web_canvas.rs
src/renderer/svg_layer.rs
src/renderer/skia/renderer.rs
src/paint/paint_op.rs
src/paint/builder.rs
src/paint/json.rs
src/paint/schema.rs
src/document_core/queries/rendering.rs
tests/issue_938.rs
tests/issue_514.rs
tests/issue_516.rs
```

Studio:

```text
rhwp-studio/src/core/types.ts
```

문서:

```text
mydocs/working/task_m050_1016_stage3.md
mydocs/working/task_m050_1016_stage4.md
mydocs/report/task_m050_1016_report.md
mydocs/orders/20260520.md
```

## 7. 위험 및 완화

| 위험 | 대응 |
|------|------|
| `PaintOp::Image` 필드 추가로 match arm 누락 | 컴파일 에러를 활용하고 모든 pattern을 `..` 또는 resolved 처리로 명시 |
| JSON schema 소비자 호환성 | additive 필드만 추가, 기존 `mime/base64/effect/brightness/contrast` 유지 |
| transition renderer에서 resolved payload 손실 | `web_canvas.rs`, `svg_layer.rs`에서 임시 `ImageNode` 대체 처리 |
| baked PNG에 effect가 다시 적용됨 | `suppress_effects` 플래그로 Skia/WebCanvas/SVG layer 처리 |
| 일반 JPEG 오탐 | #976의 워터마크 조건과 near-white gate 유지 |
| helper 이동 중 SVG 회귀 | 기존 `issue_938_svg_watermark_is_hancom_baked_png` 유지 |

## 8. 승인 후 구현 순서

1. `image_resolver.rs` 추가 및 helper 이동
2. `PaintOp::Image` resolved payload 필드 추가
3. `LayerBuilder`에서 resolver 호출
4. PageLayerTree JSON / schema minor 갱신
5. overlay JSON에서 resolved payload 사용
6. Skia / WebCanvas layer / SVG layer 연결
7. Studio `LayerImageOp` 타입 갱신
8. 회귀 테스트 추가 및 기존 테스트 갱신
9. 검증 실행
10. Stage 3/4 보고서 작성

## 9. 승인 조건

이 구현 계획서 승인 후에만 소스 변경을 시작한다.
