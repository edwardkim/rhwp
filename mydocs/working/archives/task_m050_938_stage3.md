# Task M050 #938 Stage 3 완료보고서 — 워터마크 JPEG 투명 PNG 전처리 구현

## 단계 목표

`samples/복학원서.hwp` 중앙 워터마크의 근백색 JPEG 배경이 SVG/Studio/Canvas 렌더링에서 옅은 사각 영역으로 보이지 않도록, 워터마크 JPEG 전용 전처리를 구현한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `Cargo.toml` | WASM 빌드에서도 JPEG decoder가 포함되도록 `image` crate의 `jpeg` feature 명시 |
| `src/renderer/svg.rs` | 워터마크 JPEG의 외곽 연결 근백색 배경을 alpha 0 PNG로 변환하는 `watermark_jpeg_bytes_to_transparent_png_bytes()` 추가, SVG image emit 경로에 적용 |
| `src/document_core/queries/rendering.rs` | `get_page_overlay_images_native()` base64 생성 경로에서 같은 전처리 적용 |
| `src/renderer/web_canvas.rs` | WASM Canvas 직접 렌더 경로에서 워터마크 JPEG를 전처리한 뒤 draw 호출 |
| `tests/issue_938.rs` | SVG/overlay JSON이 워터마크를 transparent PNG로 내보내는지 확인하는 회귀 테스트 추가 |

## 구현 내용

### 1. 전처리 발동 조건

다음 조건을 모두 만족할 때만 JPEG를 투명 PNG로 변환한다.

- 이미지가 `image/jpeg`
- 그림 효과가 `RealPic`이 아님
- `brightness` 또는 `contrast` 값이 0이 아님
- JPEG 외곽의 `RGB >= 245` 근백색 비율이 85% 이상
- 전체 픽셀의 `RGB >= 245` 근백색 비율이 20% 이상

이 조건은 Stage 1에서 확인한 중앙 워터마크 특성에 맞춘 방어 조건이다.

```text
복학원서 중앙 워터마크:
  format=image/jpeg
  size=728x729
  effect=GrayScale
  brightness=-50
  contrast=70
  border_near_white_245_ratio=0.9571
  all_near_white_245_ratio=0.4595
```

### 2. 투명 배경 산출 방식

JPEG 자체에는 alpha/color-key 정보가 없으므로, 이미지 외곽에서 시작하는 flood-fill로 배경을 판정한다.

- 외곽과 연결된 `RGB >= 245` 픽셀만 배경으로 간주
- 배경 픽셀의 alpha를 0으로 설정
- 엠블럼 내부의 흰색 계열 픽셀은 외곽과 연결되지 않으면 보존
- 변환 실패 또는 조건 불충족 시 기존 JPEG를 그대로 사용

이번 단계에서는 hard alpha 변환을 적용했다. anti-alias 경계 보정용 soft ramp는 Stage 4 시각 검증에서 필요성이 확인될 때만 추가한다.

### 3. 적용 경로

SVG 경로:

```text
WMF -> SVG
BMP -> PNG
PCX -> PNG
watermark JPEG -> transparent PNG
기타 -> 원본
```

Studio overlay JSON 경로:

```text
기존: watermark mime=image/jpeg
변경: watermark mime=image/png
```

Canvas 직접 경로:

```text
draw_image_with_fill_mode(data, ...)
-> draw_image_with_fill_mode(render_data.as_ref(), ...)
```

기존 Task #677의 grayscale, brightness/contrast, opacity 적용은 유지했다. 차이는 필터와 opacity가 더 이상 JPEG의 흰 사각 배경에 적용되지 않도록 PNG alpha를 먼저 만든다는 점이다.

## 회귀 테스트

`tests/issue_938.rs`를 추가했다.

- `issue_938_svg_watermark_is_transparent_png`
  - `samples/복학원서.hwp` 1페이지 SVG 렌더
  - 728x729 워터마크가 `image/png` data URI로 출력되는지 확인
  - PNG alpha min/max와 투명 픽셀 수 확인

- `issue_938_overlay_watermark_is_transparent_png`
  - `get_page_overlay_images_native(0)` JSON 확인
  - watermark overlay의 `mime == image/png` 확인
  - base64 PNG의 alpha min/max와 투명 픽셀 수 확인

## 확인 결과

```text
cargo test --release --test issue_938 --no-run
결과: 성공
```

전용 테스트 바이너리 컴파일까지 확인했다. 실제 테스트 실행, 기존 회귀 테스트, SVG 산출물 비교, 시각 검증은 Stage 4에서 수행한다.

## 남은 검증

Stage 4에서 다음을 수행한다.

- `cargo test --release --test issue_938`
- `cargo test --release --test issue_514`
- `cargo test --release --test issue_516`
- `cargo test --release --test svg_snapshot`
- `./target/release/rhwp export-svg samples/복학원서.hwp -o output/debug/task938/stage4`
- 출력 SVG/PNG alpha 및 시각 차이 확인

## 다음 단계

작업지시자 승인 후 Stage 4 테스트 및 회귀 검증을 진행한다.
