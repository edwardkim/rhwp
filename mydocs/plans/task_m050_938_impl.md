# Task M050 #938 구현 계획서 — 워터마크 JPEG 근백색 배경 보정

## 전제

- 수행 계획서: `mydocs/plans/task_m050_938.md`
- Stage 1 완료보고서: `mydocs/working/task_m050_938_stage1.md`
- 대상 이슈: [#938](https://github.com/edwardkim/rhwp/issues/938)

Stage 1에서 중앙 워터마크의 직접 원인을 확인했다.

- 중앙 워터마크 BinData는 `image/jpeg`, `728×729`, 알파 없음
- 외곽 95.71%, 전체 45.95%가 `RGB >= 245` 근백색
- SVG는 JPEG 전체에 `grayscale` + brightness/contrast + `opacity=0.17` 적용
- rhwp-studio overlay는 JPEG 전체에 CSS filter + `mix-blend-mode:multiply` 적용
- `raw_picture_extra`에는 배경 투명 처리에 쓸 별도 alpha/색상키 값 없음

따라서 최초 구현은 **워터마크 JPEG 전용 근백색 배경 알파 변환**으로 제한했다.

2026-05-18 추가 분석에서 한컴 정답 PDF는 alpha/smask가 없는 opaque JPEG 워터마크를 사용하며, 흰 배경을 투명화한 뒤 filter/opacity/multiply를 다시 적용하는 방식은 정답지와 중간 톤이 달라지는 것을 확인했다.

최종 구현 방향은 다음으로 보정한다.

- 워터마크 JPEG를 한컴 정답지 픽셀 분포에 가까운 opaque PNG로 선보정한다.
- 선보정된 워터마크에는 SVG/CSS/Canvas filter, opacity, multiply를 중복 적용하지 않는다.
- 좌표 보정은 sample 전용 offset 하드코딩 없이 별도 anchor/page origin 분석 후 진행한다.

## 구현 방향

`ImageAttr::is_watermark()`가 true이고 원본 이미지가 JPEG이며, 이미지 외곽이 충분히 근백색인 경우에만 JPEG를 RGBA PNG로 재인코딩한다. 근백색 배경 픽셀은 alpha 0 또는 부드러운 alpha ramp로 변환한다.

이 방식의 의도:

- 일반 사진 JPEG에는 적용하지 않는다.
- 워터마크 엠블럼 내부 색상/명암은 기존 Task #677의 필터와 opacity를 유지한다.
- SVG와 rhwp-studio overlay가 같은 전처리 결과를 사용한다.
- PCX 학교 로고의 기존 흰색→투명 변환은 그대로 유지한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/svg.rs` | 워터마크 JPEG → 투명 PNG 변환 헬퍼 추가, SVG image emit 경로에 적용 |
| `src/document_core/queries/rendering.rs` | `get_page_overlay_images_native()`의 overlay base64 생성 경로에 같은 변환 적용 |
| `src/renderer/web_canvas.rs` | WASM Canvas 직접 이미지 렌더 경로에도 같은 변환 적용 |
| `tests/issue_938.rs` | 복학원서 워터마크 투명 PNG 변환 회귀 테스트 추가 |
| `mydocs/working/task_m050_938_stage3.md` | 구현 완료보고서 |
| `mydocs/working/task_m050_938_stage4.md` | 검증 완료보고서 |

필요 시 `src/renderer/svg.rs`에 이미 있는 이미지 변환 헬퍼를 재사용한다. 현 코드가 `document_core`와 `web_canvas`에서 `svg::pcx_bytes_to_png_bytes()` 등을 이미 호출하고 있으므로, 이번 단계에서는 새 모듈 분리보다 기존 패턴을 따른다.

## 세부 구현

### 1. 워터마크 배경 투명화 헬퍼 추가

추가 후보 함수:

```rust
pub(crate) fn watermark_jpeg_bytes_to_transparent_png_bytes(data: &[u8]) -> Option<Vec<u8>>
```

동작:

1. `image::load_from_memory_with_format(data, ImageFormat::Jpeg)`로 디코딩
2. RGBA로 변환
3. 외곽 픽셀 근백색 비율 측정
4. 조건을 만족하지 않으면 `None`
5. 조건을 만족하면 근백색 픽셀 alpha 변환 후 PNG 인코딩

발동 조건:

```text
border_near_white_ratio >= 0.85
all_near_white_ratio >= 0.20
```

Stage 1 측정값:

```text
border_near_white_ratio = 0.9571
all_near_white_ratio = 0.4595
```

픽셀 판정 초안:

```text
near-white: min(r,g,b) >= 245 && max(r,g,b)-min(r,g,b) <= 18
full transparent: min(r,g,b) >= 250 && spread <= 12
soft ramp: 235 <= min(r,g,b) < 250 && spread <= 18
```

soft ramp는 JPEG 압축 노이즈와 엠블럼 경계 anti-aliasing의 딱딱한 테두리를 줄이기 위한 것이다. Stage 3 구현 중 샘플 이미지로 수치 확인 후 최종 값을 코드 주석에 남긴다.

### 2. SVG 경로 적용

`src/renderer/svg.rs::render_image_node()`의 기존 변환 체인에 워터마크 JPEG 분기를 추가한다.

현재 순서:

```text
WMF → SVG
BMP → PNG
PCX → PNG
기타 → 원본
```

변경 후:

```text
WMF → SVG
BMP → PNG
PCX → PNG
watermark JPEG → transparent PNG
기타 → 원본
```

주의:

- SVG filter와 `opacity=0.17` 래핑은 유지한다.
- 알파가 생긴 PNG를 filter/opacity 그룹 안에 넣어도 alpha 채널은 유지되어야 한다.
- `fill_mode`, crop, tile 경로는 기존 `render_data`를 사용하므로 함께 적용된다.

### 3. rhwp-studio overlay JSON 경로 적용

`src/document_core/queries/rendering.rs::get_page_overlay_images_native()`에서 overlay 이미지 base64를 만들 때 같은 변환을 적용한다.

목표 JSON 변화:

```json
{
  "mime": "image/png",
  "effect": "grayScale",
  "brightness": -50,
  "contrast": 70,
  "watermark": { "preset": "custom" }
}
```

현재는 워터마크 overlay가 `mime=image/jpeg`로 내려간다. 변경 후에는 `mime=image/png`이고 PNG 내부 alpha가 투명 배경을 보존해야 한다.

`rhwp-studio/src/view/page-renderer.ts`는 이미 `img.mime`, `base64`, `filter`, `mixBlendMode`를 사용하므로 TypeScript 변경은 기본적으로 필요 없다. 다만 시각 검증에서 CSS filter가 alpha PNG에 의도대로 동작하는지 확인한다.

### 4. WASM Canvas 직접 경로 적용

`src/renderer/web_canvas.rs`에는 overlay가 아닌 직접 Canvas 렌더 경로가 남아 있다. #938 대표 경로는 overlay지만, 이미지 렌더러 경로 분리 회귀를 피하기 위해 여기에도 같은 전처리를 적용한다.

구현 후보:

- `draw_image_with_fill_mode()` 호출 전에 `is_watermark_image`와 원본 MIME을 판정해 `Cow<[u8]>`로 전처리
- 또는 `draw_image()`에 watermark 여부를 넘기지 않고 상위 `RenderNodeType::Image` 분기에서 전처리된 slice를 넘김

범위는 `target_arch = "wasm32"` 코드에 한정한다.

### 5. 회귀 테스트 추가

`tests/issue_938.rs` 신규.

테스트 항목:

1. `issue_938_svg_watermark_is_transparent_png`
   - `samples/복학원서.hwp` 1페이지 SVG 렌더
   - 페이지의 image data URI가 모두 PNG가 되었는지 확인
   - `728×729` 워터마크 PNG를 찾아 corner alpha가 0인지 확인

2. `issue_938_overlay_watermark_is_transparent_png`
   - `get_page_overlay_images_native(0)` JSON 확인
   - watermark overlay의 `mime`이 `image/png`인지 확인
   - base64 디코딩 후 alpha < 255 픽셀이 있는지 확인

3. helper 단위 테스트
   - 흰 외곽을 가진 JPEG는 PNG 변환됨
   - 근백색 외곽 조건이 부족한 JPEG는 변환하지 않음

기존 테스트 유지:

```bash
cargo test --release --test issue_514
cargo test --release --test issue_516
cargo test --release --test issue_938
cargo test --release --test svg_snapshot
```

필요 시:

```bash
cargo test --release --lib
cargo check --target wasm32-unknown-unknown --release --lib
```

### 6. 시각 검증

생성 자료:

```bash
./target/release/rhwp export-svg samples/복학원서.hwp -o output/debug/task938/stage4
```

확인 기준:

- 중앙 워터마크 주변의 옅은 사각 영역 제거
- 엠블럼 자체는 너무 흐려지거나 진해지지 않음
- 좌상단 학교 로고 PCX 투명 배경 유지
- 접수증/본문 레이아웃은 Task #677 승인 상태 유지

최종 시각 판정은 작업지시자 승인으로 확정한다.

## 구현 단계

### Stage 3 — 소스 구현

- 워터마크 JPEG → 투명 PNG 헬퍼 추가
- SVG 경로 적용
- overlay JSON 경로 적용
- WASM Canvas 직접 경로 적용
- `mydocs/working/task_m050_938_stage3.md` 작성

### Stage 4 — 테스트 및 회귀 검증

- `tests/issue_938.rs` 추가
- 대상 테스트 실행
- SVG/PDF 비교용 산출물 생성
- `mydocs/working/task_m050_938_stage4.md` 작성

### Stage 5 — 시각 판정 및 최종 보고

- 작업지시자 시각 판정
- 필요 시 threshold 보정
- 최종 보고서 `mydocs/report/task_m050_938_report.md` 작성
- `mydocs/orders/20260517.md` 상태 갱신

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| 워터마크 내부의 밝은 디테일이 투명화됨 | 외곽/전체 근백색 비율로 이미지 단위 gate 후, 픽셀 alpha ramp는 좁은 근백색 범위로 제한 |
| 일반 JPEG 사진 배경이 오탐 처리됨 | `ImageAttr::is_watermark()` + JPEG + 외곽 근백색 조건을 모두 만족할 때만 변환 |
| SVG와 Studio 결과가 달라짐 | SVG emit과 overlay JSON이 같은 helper를 사용하도록 적용 |
| PCX 학교 로고 회귀 | `issue_514` 유지, PCX 변환 분기보다 워터마크 JPEG 분기를 뒤에 둠 |
| WASM 빌드 회귀 | `cargo check --target wasm32-unknown-unknown --release --lib`로 확인 |

## 승인 게이트

본 구현 계획서 승인 후에만 소스 구현(Stage 3)을 시작한다.
